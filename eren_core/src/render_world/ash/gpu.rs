use std::sync::Arc;

use super::engine::AshEngine;
use crate::render_world::common::gpu::GpuResourceManager;
use ash::{Entry, vk};
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};
use winit::{dpi::PhysicalSize, window::Window};

use ash::khr::{get_physical_device_properties2, portability_enumeration, surface, swapchain};

const MAX_SPRITES: u32 = 2048;
const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct AshGpuResourceManager {
    engine: Box<dyn AshEngine>,
    entry: Entry,

    device: Option<ash::Device>,
    swapchain_device: Option<swapchain::Device>,
    swapchain_khr: Option<vk::SwapchainKHR>,
    swapchain_extent: Option<vk::Extent2D>,
    graphics_queue: Option<vk::Queue>,
    present_queue: Option<vk::Queue>,

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,

    current_frame: usize,
}

impl AshGpuResourceManager {
    pub fn new(engine: Box<dyn AshEngine>) -> Self {
        let entry = unsafe { Entry::load().expect("Failed to load entry point") };
        Self {
            engine,
            entry,

            device: None,
            swapchain_device: None,
            swapchain_khr: None,
            swapchain_extent: None,
            graphics_queue: None,
            present_queue: None,

            image_available_semaphores: Vec::new(),
            render_finished_semaphores: Vec::new(),
            in_flight_fences: Vec::new(),
            current_frame: 0,
        }
    }
}

impl GpuResourceManager for AshGpuResourceManager {
    fn on_window_ready(&mut self, window: Arc<Window>) {
        let mut extension_names =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .unwrap()
                .to_vec();

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extension_names.push(portability_enumeration::NAME.as_ptr());
            extension_names.push(get_physical_device_properties2::NAME.as_ptr());
        }

        let application_info = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);

        let create_flags = vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&extension_names)
            .flags(create_flags);

        let instance = unsafe { self.entry.create_instance(&create_info, None) }
            .expect("Failed to create Vulkan instance");

        let surface = unsafe {
            ash_window::create_surface(
                &self.entry,
                &instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
        }
        .unwrap();

        let surface_loader = surface::Instance::new(&self.entry, &instance);

        let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() };

        let (physical_device, graphics_q_index, present_q_index) = physical_devices
            .into_iter()
            .find_map(|pd| {
                let mut graphics_q_index = None;
                let mut present_q_index = None;

                for (index, qf) in
                    unsafe { instance.get_physical_device_queue_family_properties(pd) }
                        .iter()
                        .enumerate()
                {
                    if qf.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                        graphics_q_index = Some(index as u32);
                    }
                    let supports_present = unsafe {
                        surface_loader.get_physical_device_surface_support(
                            pd,
                            index as u32,
                            surface,
                        )
                    }
                    .unwrap_or(false);
                    if supports_present {
                        present_q_index = Some(index as u32);
                    }
                }

                if let (Some(g), Some(p)) = (graphics_q_index, present_q_index) {
                    Some((pd, g, p))
                } else {
                    None
                }
            })
            .unwrap();

        let priorities = [1.0];
        let queue_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_q_index)
            .queue_priorities(&priorities);

        let device_extension_names = [
            swapchain::NAME.as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ash::khr::portability_subset::NAME.as_ptr(),
        ];

        let features = vk::PhysicalDeviceFeatures::default().shader_clip_distance(true);

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names)
            .enabled_features(&features);

        let device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .unwrap()
        };

        let graphics_queue = unsafe { device.get_device_queue(graphics_q_index, 0) };

        let pool_create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(graphics_q_index);

        let command_pool = unsafe { device.create_command_pool(&pool_create_info, None).unwrap() };

        let formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .unwrap()
        };

        let window_size = window.inner_size();
        let scale_factor = window.scale_factor();

        let swapchain_loader = swapchain::Device::new(&instance, &device);

        let surface_caps = unsafe {
            surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .unwrap()
        };

        let mut desired_image_count = surface_caps.min_image_count + 1;
        if surface_caps.max_image_count > 0 && desired_image_count > surface_caps.max_image_count {
            desired_image_count = surface_caps.max_image_count;
        }

        let surface_format = formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB // Prefer sRGB for color
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&formats[0]) // Fallback to the first available
            .clone();

        let swapchain_extent = if surface_caps.current_extent.width != u32::MAX {
            surface_caps.current_extent
        } else {
            let size = window.inner_size();
            vk::Extent2D {
                width: size.width.clamp(
                    surface_caps.min_image_extent.width,
                    surface_caps.max_image_extent.width,
                ),
                height: size.height.clamp(
                    surface_caps.min_image_extent.height,
                    surface_caps.max_image_extent.height,
                ),
            }
        };

        let pre_transform = if surface_caps
            .supported_transforms
            .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
        {
            vk::SurfaceTransformFlagsKHR::IDENTITY
        } else {
            surface_caps.current_transform
        };

        let present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .unwrap()
        };

        let present_mode = present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX) // Prefer Mailbox for low latency
            .unwrap_or(vk::PresentModeKHR::FIFO); // FIFO is always available

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(desired_image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(swapchain_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT) // Could also be TRANSFER_DST for blitting
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(pre_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null()); // No old swapchain first time

        let swapchain_khr = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .unwrap()
        };

        let present_queue = unsafe { device.get_device_queue(present_q_index, 0) };

        self.device = Some(device.clone());
        self.swapchain_device = Some(swapchain_loader);
        self.swapchain_khr = Some(swapchain_khr);
        self.swapchain_extent = Some(swapchain_extent);
        self.graphics_queue = Some(graphics_queue);
        self.present_queue = Some(present_queue);

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let image_available =
                unsafe { device.create_semaphore(&semaphore_info, None) }.unwrap();
            let render_finished =
                unsafe { device.create_semaphore(&semaphore_info, None) }.unwrap();
            let in_flight = unsafe { device.create_fence(&fence_info, None) }.unwrap();

            self.image_available_semaphores.push(image_available);
            self.render_finished_semaphores.push(render_finished);
            self.in_flight_fences.push(in_flight);
        }

        self.engine.on_gpu_resources_ready(
            &instance,
            physical_device,
            device,
            graphics_queue,
            command_pool,
            surface_format.format,
            window_size,
            scale_factor,
            MAX_SPRITES,
        );
    }

    fn on_window_lost(&mut self) {
        self.engine.on_gpu_resources_lost();
    }

    fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, window_scale_factor: f64) {
        self.engine
            .on_window_resized(window_size, window_scale_factor);
    }

    fn update(&mut self) {
        if let (
            Some(device),
            Some(swapchain_loader),
            Some(swapchain_khr),
            Some(swapchain_extent),
            Some(graphics_queue),
            Some(present_queue),
        ) = (
            &self.device,
            &self.swapchain_device,
            &self.swapchain_khr,
            &self.swapchain_extent,
            &self.graphics_queue,
            &self.present_queue,
        ) {
            let (image_index, _is_suboptimal) = unsafe {
                swapchain_loader.acquire_next_image(
                    *swapchain_khr,
                    u64::MAX,
                    self.image_available_semaphores[self.current_frame],
                    vk::Fence::null(),
                )
            }
            .expect("Failed to acquire next swapchain image");

            let command_buffer = self.command_buffers[image_index as usize];

            let render_area = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: *swapchain_extent,
            };

            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: (*swapchain_extent).width as f32,
                height: (*swapchain_extent).height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };

            let scissor = render_area;

            self.engine.update(
                command_buffer,
                self.frame_buffers[image_index as usize],
                render_area,
                viewport,
                scissor,
            );

            unsafe {
                device.end_command_buffer(command_buffer).unwrap();
            }

            // Submit
            let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
            let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

            let command_buffers = [command_buffer];
            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);

            let in_flight_fence = self.in_flight_fences[self.current_frame];

            unsafe {
                device
                    .queue_submit(*graphics_queue, &[submit_info], in_flight_fence)
                    .unwrap();
            }

            let swapchains = [*swapchain_khr];
            let image_indices = [image_index];

            // Present
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            unsafe {
                swapchain_loader
                    .queue_present(*present_queue, &present_info)
                    .unwrap()
            };

            self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        }
    }
}
