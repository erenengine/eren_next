use ash::{Device, vk};
use eren_core::render_world::ash::buffer::{
    MemoryLocation, create_buffer_with_size, find_memory_type_index,
};
use glam::Vec2;
use image::{DynamicImage, EncodableLayout, GenericImageView};
use std::{collections::HashMap, hash::Hash, path::Path};

pub struct SpriteGpuResource {
    pub size: Vec2,
    pub descriptor_set: vk::DescriptorSet,
    image: vk::Image, // Made private, manage through methods if needed
    image_memory: vk::DeviceMemory,
    image_view: vk::ImageView,
}

impl SpriteGpuResource {
    // Internal destroy, called by AshSpriteAssetManager's Drop or on_gpu_resources_lost
    fn destroy(&self, device: &Device) {
        unsafe {
            device.destroy_image_view(self.image_view, None);
            device.destroy_image(self.image, None);
            device.free_memory(self.image_memory, None);
            // Descriptor sets are freed from the pool, not individually here typically
        }
    }
}

pub struct AshSpriteAssetManager<SA>
where
    SA: Eq + Hash + Clone,
{
    device: Option<Device>, // Store as Option<Arc<Device>> if shared or just Device if exclusively owned logic
    phys_mem_props: Option<vk::PhysicalDeviceMemoryProperties>,
    graphics_queue: Option<vk::Queue>,
    command_pool: Option<vk::CommandPool>,

    descriptor_set_layout: Option<vk::DescriptorSetLayout>,
    descriptor_pool: Option<vk::DescriptorPool>,
    sampler: Option<vk::Sampler>,

    // loading_assets: Vec<SA>, // Not used in provided logic
    loaded_images_cache: HashMap<SA, DynamicImage>, // Keep images in CPU memory if needed for re-upload
    gpu_resources: HashMap<SA, SpriteGpuResource>,
    max_sprites_capacity: u32,
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
            loaded_images_cache: HashMap::new(),
            gpu_resources: HashMap::new(),
            max_sprites_capacity: 0,
        }
    }

    pub fn descriptor_set_layout(&self) -> Option<vk::DescriptorSetLayout> {
        self.descriptor_set_layout
    }

    pub fn on_gpu_resources_ready(
        &mut self,
        device: Device, // Take ownership or Arc
        phys_mem_props: vk::PhysicalDeviceMemoryProperties,
        graphics_queue: vk::Queue,
        command_pool: vk::CommandPool,
        max_sprites: u32,
    ) {
        // debug_assert!(self.device.is_none(), "GPU resources already initialised for AssetManager");
        if self.device.is_some() {
            // This might be a re-initialization after device loss.
            // Call on_gpu_resources_lost first to clean up old state.
            self.on_gpu_resources_lost_internal(false); // Don't nullify device yet
        }

        self.max_sprites_capacity = max_sprites;

        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::NEAREST)
            .min_filter(vk::Filter::NEAREST)
            .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .border_color(vk::BorderColor::FLOAT_TRANSPARENT_BLACK) // Or OPAQUE_BLACK
            .unnormalized_coordinates(false); // Usually false for texture sampling
        // Add mipmapping setup if using mipmaps
        let sampler = unsafe { device.create_sampler(&sampler_info, None) }
            .expect("Failed to create sprite sampler");

        let bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0) // Binding for combined image sampler
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)];
        let set_layout_create = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        let descriptor_set_layout =
            unsafe { device.create_descriptor_set_layout(&set_layout_create, None) }
                .expect("Failed to create sprite descriptor set layout");

        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: max_sprites, // Max number of sprites (descriptor sets)
        }];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(max_sprites)
            .pool_sizes(&pool_sizes)
            .flags(vk::DescriptorPoolCreateFlags::empty()); // Or FREE_DESCRIPTOR_SET_BIT if sets are individually freed
        let descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None) }
            .expect("Failed to create sprite descriptor pool");

        self.device = Some(device);
        self.phys_mem_props = Some(phys_mem_props);
        self.graphics_queue = Some(graphics_queue);
        self.command_pool = Some(command_pool);
        self.descriptor_set_layout = Some(descriptor_set_layout);
        self.descriptor_pool = Some(descriptor_pool);
        self.sampler = Some(sampler);

        // Re-upload any sprites that were loaded before GPU was ready or after device loss
        let to_upload: Vec<(SA, DynamicImage)> = self
            .loaded_images_cache
            .iter()
            .map(|(k, v)| (k.clone(), v.clone())) // Clone to avoid borrowing issues
            .collect();
        for (asset_id, img) in to_upload {
            if !self.gpu_resources.contains_key(&asset_id) {
                // Only upload if not already a GPU resource
                self.upload_image_to_gpu(asset_id, &img);
            }
        }
    }

    /// Internal cleanup logic, `keep_device_ref` is true if called from Drop
    fn on_gpu_resources_lost_internal(&mut self, keep_device_ref: bool) {
        if let Some(device_ref) = &self.device {
            // Use existing device ref for cleanup
            unsafe {
                for (_, res) in self.gpu_resources.drain() {
                    res.destroy(device_ref);
                }
                if let Some(pool) = self.descriptor_pool.take() {
                    device_ref.destroy_descriptor_pool(pool, None);
                }
                if let Some(layout) = self.descriptor_set_layout.take() {
                    device_ref.destroy_descriptor_set_layout(layout, None);
                }
                if let Some(sampler_val) = self.sampler.take() {
                    // Renamed to avoid conflict
                    device_ref.destroy_sampler(sampler_val, None);
                }
            }
        }
        if !keep_device_ref {
            self.device = None; // Nullify if device is truly lost
        }
        // Keep phys_mem_props, graphics_queue, command_pool if device is kept (e.g. for reinit)
        // Or clear them if device is truly lost.
        if !keep_device_ref {
            self.phys_mem_props = None;
            self.graphics_queue = None;
            self.command_pool = None;
        }
    }

    pub fn on_gpu_resources_lost(&mut self) {
        self.on_gpu_resources_lost_internal(false);
    }

    pub fn load_sprite<P: AsRef<Path>>(&mut self, asset_id: SA, path: P) {
        // Avoid reloading if already cached (CPU or GPU)
        if self.loaded_images_cache.contains_key(&asset_id)
            || self.gpu_resources.contains_key(&asset_id)
        {
            // Potentially log a warning or handle as an update if behavior is desired
            return;
        }

        let image = image::open(path).expect("Failed to load sprite image");
        // If GPU is ready, upload immediately. Otherwise, it will be uploaded when on_gpu_resources_ready is called.
        if self.device.is_some() {
            self.upload_image_to_gpu(asset_id.clone(), &image);
        }
        self.loaded_images_cache.insert(asset_id, image); // Always cache on CPU
    }

    pub fn get_gpu_resource(&self, asset_id: &SA) -> Option<&SpriteGpuResource> {
        self.gpu_resources.get(asset_id)
    }

    fn upload_image_to_gpu(&mut self, asset_id: SA, image_data: &DynamicImage) {
        let (device, phys_mem_props, queue, cmd_pool, set_layout_val, sampler_val, desc_pool_val) =
            match (
                self.device.as_ref(),
                self.phys_mem_props.as_ref(),
                self.graphics_queue,
                self.command_pool,
                self.descriptor_set_layout,
                self.sampler,
                self.descriptor_pool,
            ) {
                (Some(d), Some(pmp), Some(q), Some(cp), Some(sl), Some(s), Some(dp)) => {
                    (d, pmp, q, cp, sl, s, dp)
                }
                _ => {
                    // GPU resources not fully initialized, defer upload.
                    // This case should be handled by on_gpu_resources_ready re-uploading.
                    return;
                }
            };

        let rgba8_image = image_data.to_rgba8();
        let (width, height) = image_data.dimensions();
        let image_extent = vk::Extent3D {
            width,
            height,
            depth: 1,
        };
        let image_pixel_data = rgba8_image.as_bytes();
        let image_data_size = image_pixel_data.len() as vk::DeviceSize;

        // --- Create Image ---
        let image_create_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_SRGB) // Assuming sRGB format for sprites
            .extent(image_extent)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE) // No concurrent access needed typically
            .initial_layout(vk::ImageLayout::UNDEFINED);
        let image_handle = unsafe { device.create_image(&image_create_info, None) }
            .expect("Failed to create sprite image handle");

        let mem_requirements = unsafe { device.get_image_memory_requirements(image_handle) };
        let mem_type_index = find_memory_type_index(
            &mem_requirements,
            phys_mem_props,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .expect("Unable to find device-local memory type for sprite image");

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type_index);
        let image_memory = unsafe { device.allocate_memory(&alloc_info, None) }
            .expect("Failed to allocate sprite image memory");
        unsafe { device.bind_image_memory(image_handle, image_memory, 0) }
            .expect("Failed to bind sprite image memory");

        // --- Staging Buffer and Upload ---
        let staging_buffer = create_buffer_with_size(
            device,
            phys_mem_props,
            image_data_size,
            Some(image_pixel_data), // Pass data directly
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu, // Host visible for data copy
        );

        // --- Command Buffer for Transfer and Layout Transition ---
        let cmd_buf_alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(cmd_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let cmd_buf = unsafe { device.allocate_command_buffers(&cmd_buf_alloc_info) }
            .expect("Failed to allocate command buffer for sprite upload")[0];

        let cmd_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            device
                .begin_command_buffer(cmd_buf, &cmd_begin_info)
                .unwrap();
        }

        // Transition: UNDEFINED -> TRANSFER_DST_OPTIMAL
        let barrier_to_transfer_dst = vk::ImageMemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::empty()) // No prior access
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED) // No ownership transfer
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image_handle)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });
        unsafe {
            device.cmd_pipeline_barrier(
                cmd_buf,
                vk::PipelineStageFlags::TOP_OF_PIPE, // Before any writes
                vk::PipelineStageFlags::TRANSFER,    // Before transfer operations
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier_to_transfer_dst],
            );
        }

        // Copy buffer to image
        let buffer_image_copy_region = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0) // Tightly packed
            .buffer_image_height(0) // Tightly packed
            .image_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(image_extent);
        unsafe {
            device.cmd_copy_buffer_to_image(
                cmd_buf,
                staging_buffer.buffer,
                image_handle,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[buffer_image_copy_region],
            );
        }

        // Transition: TRANSFER_DST_OPTIMAL -> SHADER_READ_ONLY_OPTIMAL
        let barrier_to_shader_read = vk::ImageMemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image_handle)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });
        unsafe {
            device.cmd_pipeline_barrier(
                cmd_buf,
                vk::PipelineStageFlags::TRANSFER, // After transfer operations
                vk::PipelineStageFlags::FRAGMENT_SHADER, // Before shader reads
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier_to_shader_read],
            );
        }

        unsafe {
            device.end_command_buffer(cmd_buf).unwrap();
        }

        // Submit and wait
        let submit_info = vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&cmd_buf));
        unsafe {
            device
                .queue_submit(queue, &[submit_info], vk::Fence::null())
                .expect("Failed to submit sprite image upload commands");
            device
                .queue_wait_idle(queue)
                .expect("Queue wait idle failed after sprite upload");
        }

        // Cleanup temporary resources
        staging_buffer.destroy(device);
        unsafe {
            device.free_command_buffers(cmd_pool, &[cmd_buf]);
        }

        // --- Create Image View ---
        let view_create_info = vk::ImageViewCreateInfo::default()
            .image(image_handle)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_SRGB) // Must match image format
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });
        let image_view = unsafe { device.create_image_view(&view_create_info, None) }
            .expect("Failed to create sprite image view");

        // --- Allocate and Update Descriptor Set ---
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(desc_pool_val)
            .set_layouts(std::slice::from_ref(&set_layout_val));
        let descriptor_set = unsafe { device.allocate_descriptor_sets(&alloc_info) }
            .expect("Failed to allocate sprite descriptor set")[0];

        let image_info_for_descriptor = vk::DescriptorImageInfo::default()
            .sampler(sampler_val)
            .image_view(image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        let write_descriptor_set = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0) // Matches layout binding
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&image_info_for_descriptor));
        unsafe {
            device.update_descriptor_sets(&[write_descriptor_set], &[]);
        }

        self.gpu_resources.insert(
            asset_id,
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

impl<SA: Eq + Hash + Clone> Drop for AshSpriteAssetManager<SA> {
    fn drop(&mut self) {
        // Pass true to keep device ref as it's being dropped from self.device
        self.on_gpu_resources_lost_internal(true);
        // self.device will be dropped automatically if it's owned (Option<Device>)
        // If it's an Arc<Device>, the Arc is dropped.
    }
}
