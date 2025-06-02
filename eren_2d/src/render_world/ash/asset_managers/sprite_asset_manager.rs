use std::{collections::HashMap, hash::Hash, path::Path};

use ash::{Device, vk};
use glam::Vec2;
use image::{DynamicImage, EncodableLayout, GenericImageView};

use eren_core::render_world::ash::buffer::{MemoryLocation, create_buffer};

pub struct SpriteGpuResource {
    pub size: Vec2,
    pub descriptor_set: vk::DescriptorSet,
    pub image: vk::Image,
    pub image_memory: vk::DeviceMemory,
    pub image_view: vk::ImageView,
}

impl SpriteGpuResource {
    fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_image_view(self.image_view, None);
            device.destroy_image(self.image, None);
            device.free_memory(self.image_memory, None);
        }
    }
}

pub struct AshSpriteAssetManager<SA> {
    device: Option<Device>,
    phys_mem_props: Option<vk::PhysicalDeviceMemoryProperties>,
    graphics_queue: Option<vk::Queue>,
    command_pool: Option<vk::CommandPool>,

    descriptor_set_layout: Option<vk::DescriptorSetLayout>,
    descriptor_pool: Option<vk::DescriptorPool>,
    sampler: Option<vk::Sampler>,

    loading_assets: Vec<SA>,
    loaded_images: HashMap<SA, DynamicImage>,

    gpu_resources: HashMap<SA, SpriteGpuResource>,
}

impl<SA: Eq + Hash + Clone> AshSpriteAssetManager<SA> {
    pub fn new() -> Self {
        Self {
            device: None,
            phys_mem_props: None,
            graphics_queue: None,
            command_pool: None,
            descriptor_set_layout: None,
            descriptor_pool: None,
            sampler: None,

            loading_assets: Vec::new(),
            loaded_images: HashMap::new(),

            gpu_resources: HashMap::new(),
        }
    }

    /// Returns the descriptor‑set layout required by the renderer (set = 1).
    pub fn descriptor_set_layout(&self) -> Option<vk::DescriptorSetLayout> {
        self.descriptor_set_layout
    }

    /// Initialise GPU‑side objects once the Vulkan device is ready.
    ///
    /// * `phys_mem_props` – returned from `instance.get_physical_device_memory_properties`.
    /// * `graphics_queue` / `command_pool` – used for the one‑shot upload + layout transition.
    pub fn on_gpu_resources_ready(
        &mut self,
        device: Device,
        phys_mem_props: vk::PhysicalDeviceMemoryProperties,
        graphics_queue: vk::Queue,
        command_pool: vk::CommandPool,
        max_sprites: u32,
    ) {
        debug_assert!(self.device.is_none(), "GPU resources already initialised");

        // Create one sampler (nearest‑neighbour, clamp‑to‑edge) shared by every sprite.
        let sampler = unsafe {
            device
                .create_sampler(
                    &vk::SamplerCreateInfo::default()
                        .mag_filter(vk::Filter::NEAREST)
                        .min_filter(vk::Filter::NEAREST)
                        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE),
                    None,
                )
                .expect("Failed to create sprite sampler")
        };

        // Combined‑image‑sampler binding at binding = 0 (set = 1)
        let bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)];

        let set_layout_create = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&set_layout_create, None)
                .expect("Failed to create sprite descriptor‑set layout")
        };

        // Descriptor pool able to allocate up to `max_sprites` sets.
        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: max_sprites,
        }];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(max_sprites)
            .pool_sizes(&pool_sizes);
        let descriptor_pool = unsafe {
            device
                .create_descriptor_pool(&pool_info, None)
                .expect("Failed to create sprite descriptor pool")
        };

        self.device = Some(device);
        self.phys_mem_props = Some(phys_mem_props);
        self.graphics_queue = Some(graphics_queue);
        self.command_pool = Some(command_pool);
        self.descriptor_set_layout = Some(descriptor_set_layout);
        self.descriptor_pool = Some(descriptor_pool);
        self.sampler = Some(sampler);

        // Upload any sprites that were requested *before* the GPU became ready.
        let to_upload: Vec<(SA, DynamicImage)> = self
            .loaded_images
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        for (asset, img) in to_upload {
            self.create_gpu_resource(asset, &img);
        }
    }

    /// Tears down GPU objects (e.g. on device loss / swap‑chain recreation with new device)
    pub fn on_gpu_resources_lost(&mut self) {
        if let Some(device) = &self.device {
            unsafe {
                // Destroy per‑sprite resources
                for (_, res) in self.gpu_resources.drain() {
                    res.destroy(device);
                }
                // Layout, pool, sampler
                if let Some(pool) = self.descriptor_pool.take() {
                    device.destroy_descriptor_pool(pool, None);
                }
                if let Some(layout) = self.descriptor_set_layout.take() {
                    device.destroy_descriptor_set_layout(layout, None);
                }
                if let Some(sampler) = self.sampler.take() {
                    device.destroy_sampler(sampler, None);
                }
            }
        }
        self.device = None;
        self.phys_mem_props = None;
        self.graphics_queue = None;
        self.command_pool = None;
    }

    /// Loads an image from disk (or elsewhere) *and* immediately uploads it if the GPU is ready.
    pub fn load_sprite<P: AsRef<Path>>(&mut self, asset: SA, path: P) {
        let image = image::open(path).expect("Failed to load sprite");
        self.loaded_images.insert(asset.clone(), image.clone());
        self.create_gpu_resource(asset, &image);
    }

    /// Returns GPU resource for a sprite (if it has already been uploaded).
    pub fn get_gpu_resource(&self, asset: &SA) -> Option<&SpriteGpuResource> {
        self.gpu_resources.get(asset)
    }

    fn create_gpu_resource(&mut self, asset: SA, image: &DynamicImage) {
        let (device, phys_mem_props, queue, cmd_pool, set_layout, sampler, pool) = match (
            &self.device,
            &self.phys_mem_props,
            &self.graphics_queue,
            &self.command_pool,
            &self.descriptor_set_layout,
            &self.sampler,
            &self.descriptor_pool,
        ) {
            (Some(d), Some(p), Some(q), Some(cp), Some(l), Some(s), Some(pool)) => {
                (d, p, q, cp, l, s, pool)
            }
            _ => {
                // GPU not ready yet – we'll upload later
                return;
            }
        };

        let rgba8 = image.to_rgba8();
        let (width, height) = image.dimensions();
        let extent = vk::Extent3D {
            width,
            height,
            depth: 1,
        };

        let img_ci = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_SRGB)
            .extent(extent)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image_handle = unsafe {
            device
                .create_image(&img_ci, None)
                .expect("Failed to create sprite image")
        };

        let mem_requirements = unsafe { device.get_image_memory_requirements(image_handle) };
        let mem_type_index = find_memory_type(
            mem_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            phys_mem_props,
        )
        .expect("Unable to find device‑local memory type for sprite texture");

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type_index);
        let image_memory = unsafe { device.allocate_memory(&alloc_info, None) }
            .expect("Failed to allocate sprite image memory");
        unsafe {
            device
                .bind_image_memory(image_handle, image_memory, 0)
                .expect("Failed to bind sprite image memory");
        }

        let staging = create_buffer(
            device,
            phys_mem_props,
            Some(rgba8.as_bytes()),
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        );

        // One‑time command buffer
        let cmd_buf_alloc = vk::CommandBufferAllocateInfo::default()
            .command_pool(*cmd_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let cmd_buf = unsafe { device.allocate_command_buffers(&cmd_buf_alloc) }.expect(
            "Failed to alloc
            upload CB",
        )[0];

        let cmd_begin = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe { device.begin_command_buffer(cmd_buf, &cmd_begin) }.unwrap();

        // Transition: UNDEFINED -> TRANSFER_DST_OPTIMAL
        let barrier_0 = vk::ImageMemoryBarrier::default()
            .image(image_handle)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1),
            )
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
        unsafe {
            device.cmd_pipeline_barrier(
                cmd_buf,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier_0],
            );
        }

        // Copy buffer->image
        let copy = vk::BufferImageCopy::default()
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .layer_count(1),
            )
            .image_extent(extent);
        unsafe {
            device.cmd_copy_buffer_to_image(
                cmd_buf,
                staging.buffer,
                image_handle,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[copy],
            )
        };

        // Transition: TRANSFER_DST_OPTIMAL -> SHADER_READ_ONLY_OPTIMAL
        let barrier_1 = vk::ImageMemoryBarrier::default()
            .image(image_handle)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1),
            )
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ);
        unsafe {
            device.cmd_pipeline_barrier(
                cmd_buf,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier_1],
            );
        }

        unsafe { device.end_command_buffer(cmd_buf) }.unwrap();

        let binding = [cmd_buf];
        let submit_info = vk::SubmitInfo::default().command_buffers(&binding);
        unsafe { device.queue_submit(*queue, &[submit_info], vk::Fence::null()) }
            .expect("Failed to submit sprite upload");
        unsafe { device.queue_wait_idle(*queue).unwrap() };

        // Clean up staging buffer and command buffer
        staging.destroy(device);
        unsafe { device.free_command_buffers(*cmd_pool, &[cmd_buf]) };

        // View
        let view_ci = vk::ImageViewCreateInfo::default()
            .image(image_handle)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_SRGB)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .level_count(1)
                    .layer_count(1),
            );
        let image_view = unsafe { device.create_image_view(&view_ci, None) }
            .expect("Failed to create sprite view");

        // Allocate & update descriptor set
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(*pool)
            .set_layouts(std::slice::from_ref(set_layout));
        let descriptor_set = unsafe { device.allocate_descriptor_sets(&alloc_info) }
            .expect("Failed to alloc sprite set")[0];

        let image_info = vk::DescriptorImageInfo::default()
            .sampler(*sampler)
            .image_view(image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        let write = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&image_info));
        unsafe { device.update_descriptor_sets(&[write], &[]) };

        // Store resource for user access
        self.gpu_resources.insert(
            asset,
            SpriteGpuResource {
                size: Vec2::new(width as f32, height as f32),
                descriptor_set,
                image: image_handle,
                image_memory,
                image_view,
            },
        );
    }
}

impl<SA> Drop for AshSpriteAssetManager<SA> {
    fn drop(&mut self) {
        if let Some(device) = &self.device {
            unsafe {
                for (_, res) in self.gpu_resources.drain() {
                    res.destroy(device);
                }
                if let Some(pool) = self.descriptor_pool.take() {
                    device.destroy_descriptor_pool(pool, None);
                }
                if let Some(layout) = self.descriptor_set_layout.take() {
                    device.destroy_descriptor_set_layout(layout, None);
                }
                if let Some(sampler) = self.sampler.take() {
                    device.destroy_sampler(sampler, None);
                }
            }
        }
    }
}

fn find_memory_type(
    type_bits: u32,
    reqs: vk::MemoryPropertyFlags,
    props: &vk::PhysicalDeviceMemoryProperties,
) -> Option<u32> {
    for i in 0..props.memory_type_count {
        if (type_bits & (1 << i)) != 0
            && props.memory_types[i as usize].property_flags.contains(reqs)
        {
            return Some(i);
        }
    }
    None
}
