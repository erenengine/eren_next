use std::ffi::CStr;
use std::sync::Arc;

use ash::{Entry, Instance as AshInstance, vk}; // Aliased AshInstance to avoid conflict
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};
use winit::{dpi::PhysicalSize, window::Window};

use ash::khr::{get_physical_device_properties2, portability_enumeration, surface, swapchain};

use crate::render_world::common::gpu::GpuResourceManager;

use super::engine::AshEngine;

const MAX_SPRITES: u32 = 2048; // This constant is also used by AshEngine2D
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

struct QueueFamilyIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
}

impl QueueFamilyIndices {
    fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

struct SwapChainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

pub struct AshGpuResourceManager {
    engine: Box<dyn AshEngine>,
    entry: Entry,
    instance: Option<AshInstance>,
    debug_utils_loader: Option<ash::ext::debug_utils::Instance>,
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,

    surface_loader: Option<surface::Instance>,
    surface_khr: Option<vk::SurfaceKHR>,

    physical_device: Option<vk::PhysicalDevice>,
    device: Option<ash::Device>,

    graphics_queue: Option<vk::Queue>,
    present_queue: Option<vk::Queue>,
    graphics_queue_family_index: u32,
    present_queue_family_index: u32,

    swapchain_loader: Option<swapchain::Device>,
    swapchain_khr: Option<vk::SwapchainKHR>,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_format: Option<vk::Format>,
    swapchain_extent: Option<vk::Extent2D>,
    swapchain_framebuffers: Vec<vk::Framebuffer>,

    render_pass: Option<vk::RenderPass>,
    command_pool: Option<vk::CommandPool>,
    command_buffers: Vec<vk::CommandBuffer>,

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: Vec<vk::Fence>, // To track fences for swapchain images

    current_frame: usize,
    framebuffer_resized: bool,   // Flag to signal swapchain recreation
    window: Option<Arc<Window>>, // Keep a reference to the window
}

impl AshGpuResourceManager {
    pub fn new(engine: Box<dyn AshEngine>) -> Self {
        let entry = unsafe { Entry::load().expect("Failed to load Vulkan entry point") };
        Self {
            engine,
            entry,
            instance: None,
            debug_utils_loader: None,
            debug_messenger: None,
            surface_loader: None,
            surface_khr: None,
            physical_device: None,
            device: None,
            graphics_queue: None,
            present_queue: None,
            graphics_queue_family_index: u32::MAX,
            present_queue_family_index: u32::MAX,
            swapchain_loader: None,
            swapchain_khr: None,
            swapchain_images: Vec::new(),
            swapchain_image_views: Vec::new(),
            swapchain_format: None,
            swapchain_extent: None,
            swapchain_framebuffers: Vec::new(),
            render_pass: None,
            command_pool: None,
            command_buffers: Vec::new(),
            image_available_semaphores: Vec::new(),
            render_finished_semaphores: Vec::new(),
            in_flight_fences: Vec::new(),
            images_in_flight: Vec::new(),
            current_frame: 0,
            framebuffer_resized: false,
            window: None,
        }
    }

    fn create_instance(&mut self, window: &Window) {
        let mut extension_names =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .expect("Failed to enumerate required extensions")
                .to_vec();

        let mut instance_create_flags = vk::InstanceCreateFlags::empty();

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extension_names.push(portability_enumeration::NAME.as_ptr());
            // This is an instance extension, not device.
            extension_names.push(get_physical_device_properties2::NAME.as_ptr());
            instance_create_flags |= vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
        }

        // Enable validation layers if in debug mode
        let layer_names =
            [unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") }];
        let layers_names_raw: Vec<*const i8> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        #[cfg(debug_assertions)]
        {
            extension_names.push(ash::ext::debug_utils::NAME.as_ptr());
        }

        let app_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"AshApp\0") };
        let engine_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"NoEngine\0") };
        let application_info = vk::ApplicationInfo::default()
            .application_name(app_name)
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(engine_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_3);

        let mut create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&extension_names)
            .flags(instance_create_flags);

        #[cfg(debug_assertions)]
        {
            create_info = create_info.enabled_layer_names(&layers_names_raw);
        }

        let instance = unsafe { self.entry.create_instance(&create_info, None) }
            .expect("Failed to create Vulkan instance");

        #[cfg(debug_assertions)]
        self.setup_debug_messenger(&instance);

        self.instance = Some(instance);
    }

    fn setup_debug_messenger(&mut self, instance: &AshInstance) {
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING, // | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                                                                      // | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        let loader = ash::ext::debug_utils::Instance::new(&self.entry, instance);
        let messenger = unsafe { loader.create_debug_utils_messenger(&debug_info, None) }
            .expect("Failed to create debug messenger");

        self.debug_utils_loader = Some(loader);
        self.debug_messenger = Some(messenger);
    }

    fn create_surface(&mut self, window: &Window) {
        let instance = self.instance.as_ref().expect("Instance not created");
        let surface = unsafe {
            ash_window::create_surface(
                &self.entry,
                instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
        }
        .expect("Failed to create window surface");

        self.surface_loader = Some(surface::Instance::new(&self.entry, instance));
        self.surface_khr = Some(surface);
    }

    fn pick_physical_device(&mut self) {
        let instance = self.instance.as_ref().expect("Instance not created");
        let surface_khr = self.surface_khr.expect("Surface not created");
        let surface_loader = self
            .surface_loader
            .as_ref()
            .expect("Surface loader not created");

        let physical_devices = unsafe { instance.enumerate_physical_devices() }
            .expect("Failed to enumerate physical devices");

        let (physical_device, indices) = physical_devices
            .into_iter()
            .find_map(|pd| {
                let indices = Self::find_queue_families(instance, surface_loader, surface_khr, pd);
                let extensions_supported = Self::check_device_extension_support(instance, pd);
                let swapchain_adequate = if extensions_supported {
                    let swapchain_support =
                        Self::query_swapchain_support(surface_loader, surface_khr, pd);
                    !swapchain_support.formats.is_empty()
                        && !swapchain_support.present_modes.is_empty()
                } else {
                    false
                };
                // Check for features
                let features = unsafe { instance.get_physical_device_features(pd) };
                let supported_features = features.shader_clip_distance == vk::TRUE;

                if indices.is_complete()
                    && extensions_supported
                    && swapchain_adequate
                    && supported_features
                {
                    Some((pd, indices))
                } else {
                    None
                }
            })
            .expect("Failed to find a suitable GPU");

        self.physical_device = Some(physical_device);
        self.graphics_queue_family_index = indices.graphics_family.unwrap();
        self.present_queue_family_index = indices.present_family.unwrap();
    }

    fn find_queue_families(
        instance: &AshInstance,
        surface_loader: &surface::Instance,
        surface: vk::SurfaceKHR,
        device: vk::PhysicalDevice,
    ) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        };
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(device) };

        for (i, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i as u32);
            }
            let present_support = unsafe {
                surface_loader.get_physical_device_surface_support(device, i as u32, surface)
            }
            .unwrap_or(false);
            if present_support {
                indices.present_family = Some(i as u32);
            }
            if indices.is_complete() {
                break;
            }
        }
        indices
    }

    fn get_required_device_extensions() -> Vec<&'static CStr> {
        let mut extensions = vec![swapchain::NAME];
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extensions.push(ash::khr::portability_subset::NAME);
        }
        extensions
    }

    fn check_device_extension_support(instance: &AshInstance, device: vk::PhysicalDevice) -> bool {
        let available_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(device)
                .unwrap_or_else(|_| Vec::new())
        };
        let required_extensions = Self::get_required_device_extensions();

        for required_ext_name_cstr in required_extensions.iter() {
            let required_ext_name = unsafe { CStr::from_ptr(required_ext_name_cstr.as_ptr()) };
            let found = available_extensions.iter().any(|ext| {
                let avail_ext_name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                avail_ext_name == required_ext_name
            });
            if !found {
                return false;
            }
        }
        true
    }

    fn create_logical_device(&mut self) {
        let instance = self.instance.as_ref().expect("Instance not created");
        let physical_device = self.physical_device.expect("Physical device not selected");

        let mut queue_create_infos = vec![];
        let mut unique_queue_families = std::collections::HashSet::new();
        unique_queue_families.insert(self.graphics_queue_family_index);
        unique_queue_families.insert(self.present_queue_family_index);

        let queue_priority = 1.0f32;
        for queue_family_index in unique_queue_families {
            let queue_create_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family_index)
                .queue_priorities(std::slice::from_ref(&queue_priority));
            queue_create_infos.push(queue_create_info);
        }

        let device_extensions_raw: Vec<*const i8> = Self::get_required_device_extensions()
            .iter()
            .map(|s| s.as_ptr())
            .collect();

        let features = vk::PhysicalDeviceFeatures::default().shader_clip_distance(true); // Example feature

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions_raw)
            .enabled_features(&features);

        let device = unsafe { instance.create_device(physical_device, &device_create_info, None) }
            .expect("Failed to create logical device");

        self.graphics_queue =
            Some(unsafe { device.get_device_queue(self.graphics_queue_family_index, 0) });
        self.present_queue =
            Some(unsafe { device.get_device_queue(self.present_queue_family_index, 0) });
        self.swapchain_loader = Some(swapchain::Device::new(instance, &device));
        self.device = Some(device);
    }

    fn query_swapchain_support(
        surface_loader: &surface::Instance,
        surface: vk::SurfaceKHR,
        device: vk::PhysicalDevice,
    ) -> SwapChainSupportDetails {
        unsafe {
            let capabilities = surface_loader
                .get_physical_device_surface_capabilities(device, surface)
                .expect("Failed to query surface capabilities");
            let formats = surface_loader
                .get_physical_device_surface_formats(device, surface)
                .expect("Failed to query surface formats");
            let present_modes = surface_loader
                .get_physical_device_surface_present_modes(device, surface)
                .expect("Failed to query surface present modes");
            SwapChainSupportDetails {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    fn choose_swap_surface_format(
        available_formats: &[vk::SurfaceFormatKHR],
    ) -> vk::SurfaceFormatKHR {
        available_formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or_else(|| &available_formats[0])
            .clone()
    }

    fn choose_swap_present_mode(
        available_present_modes: &[vk::PresentModeKHR],
    ) -> vk::PresentModeKHR {
        available_present_modes
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO) // Always available
    }

    fn choose_swap_extent(
        window: &Window,
        capabilities: &vk::SurfaceCapabilitiesKHR,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            let window_size = window.inner_size();
            vk::Extent2D {
                width: window_size.width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: window_size.height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    fn create_swapchain(&mut self) {
        let window = self.window.as_ref().expect("Window not available");
        let physical_device = self.physical_device.expect("Physical device not available");
        let surface_khr = self.surface_khr.expect("Surface not available");
        let surface_loader = self
            .surface_loader
            .as_ref()
            .expect("Surface loader not available");
        let device = self.device.as_ref().expect("Device not available");
        let swapchain_loader = self
            .swapchain_loader
            .as_ref()
            .expect("Swapchain loader not available");

        let swapchain_support =
            Self::query_swapchain_support(surface_loader, surface_khr, physical_device);
        let surface_format = Self::choose_swap_surface_format(&swapchain_support.formats);
        let present_mode = Self::choose_swap_present_mode(&swapchain_support.present_modes);
        let extent = Self::choose_swap_extent(window, &swapchain_support.capabilities);

        let mut image_count = swapchain_support.capabilities.min_image_count + 1;
        if swapchain_support.capabilities.max_image_count > 0
            && image_count > swapchain_support.capabilities.max_image_count
        {
            image_count = swapchain_support.capabilities.max_image_count;
        }

        let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface_khr)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT) // Add TRANSFER_DST if you blit to swapchain
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let queue_family_indices = [
            self.graphics_queue_family_index,
            self.present_queue_family_index,
        ];
        if self.graphics_queue_family_index != self.present_queue_family_index {
            swapchain_create_info = swapchain_create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices);
        } else {
            swapchain_create_info =
                swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        }

        let swapchain_khr =
            unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }
                .expect("Failed to create swapchain");

        self.swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain_khr) }
            .expect("Failed to get swapchain images");
        self.swapchain_format = Some(surface_format.format);
        self.swapchain_extent = Some(extent);
        self.swapchain_khr = Some(swapchain_khr);
    }

    fn create_image_views(&mut self) {
        let device = self.device.as_ref().expect("Device not available");
        self.swapchain_image_views = self
            .swapchain_images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(self.swapchain_format.expect("Swapchain format not set"))
                    .components(vk::ComponentMapping::default()) // RGBA order
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                unsafe { device.create_image_view(&create_info, None) }
                    .expect("Failed to create image view")
            })
            .collect();
    }

    fn create_render_pass(&mut self) {
        let device = self.device.as_ref().expect("Device not available");
        let color_attachment = vk::AttachmentDescription::default()
            .format(self.swapchain_format.expect("Swapchain format not set"))
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        // Example depth attachment (if needed)
        // let depth_attachment = vk::AttachmentDescription { ... };
        // let depth_attachment_ref = vk::AttachmentReference { attachment: 1, layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL };

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref));
        // .depth_stencil_attachment(if use_depth { &depth_attachment_ref } else { ptr::null() });

        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));

        self.render_pass = Some(
            unsafe { device.create_render_pass(&render_pass_info, None) }
                .expect("Failed to create render pass"),
        );
    }

    fn create_framebuffers(&mut self) {
        let device = self.device.as_ref().expect("Device not available");
        let render_pass = self.render_pass.expect("Render pass not created");
        let extent = self.swapchain_extent.expect("Swapchain extent not set");

        self.swapchain_framebuffers = self
            .swapchain_image_views
            .iter()
            .map(|&view| {
                let attachments = [view]; // Add depth view if using depth
                let framebuffer_info = vk::FramebufferCreateInfo::default()
                    .render_pass(render_pass)
                    .attachments(&attachments)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);
                unsafe { device.create_framebuffer(&framebuffer_info, None) }
                    .expect("Failed to create framebuffer")
            })
            .collect();
    }

    fn create_command_pool(&mut self) {
        let device = self.device.as_ref().expect("Device not available");
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER) // Or TRANSIENT if re-recorded often
            .queue_family_index(self.graphics_queue_family_index);
        self.command_pool = Some(
            unsafe { device.create_command_pool(&pool_info, None) }
                .expect("Failed to create command pool"),
        );
    }

    fn create_command_buffers(&mut self) {
        let device = self.device.as_ref().expect("Device not available");
        let command_pool = self.command_pool.expect("Command pool not created");
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32); // One CB per frame in flight
        self.command_buffers = unsafe { device.allocate_command_buffers(&alloc_info) }
            .expect("Failed to allocate command buffers");
    }

    fn create_sync_objects(&mut self) {
        let device = self.device.as_ref().expect("Device not available");
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED); // Create signaled for first frame

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            self.image_available_semaphores
                .push(unsafe { device.create_semaphore(&semaphore_info, None) }.unwrap());
            self.render_finished_semaphores
                .push(unsafe { device.create_semaphore(&semaphore_info, None) }.unwrap());
            self.in_flight_fences
                .push(unsafe { device.create_fence(&fence_info, None) }.unwrap());
        }
        // images_in_flight needs to be sized to swapchain image count, not MAX_FRAMES_IN_FLIGHT
        // and initialized to vk::Fence::null()
        self.images_in_flight = vec![vk::Fence::null(); self.swapchain_images.len()];
    }

    fn cleanup_swapchain(&mut self) {
        let device = self.device.as_ref().unwrap();
        unsafe {
            device
                .wait_for_fences(&self.in_flight_fences, true, u64::MAX)
                .expect("Failed to wait for fences during cleanup");

            for framebuffer in self.swapchain_framebuffers.drain(..) {
                device.destroy_framebuffer(framebuffer, None);
            }
            // Command buffers are from a pool associated with MAX_FRAMES_IN_FLIGHT, not swapchain directly
            // so they are not cleaned up here typically, unless the pool needs recreation due to queue family change.
            // self.command_buffers will be reused.

            if let Some(render_pass) = self.render_pass.take() {
                device.destroy_render_pass(render_pass, None);
            }
            for image_view in self.swapchain_image_views.drain(..) {
                device.destroy_image_view(image_view, None);
            }
            if let Some(swapchain_khr) = self.swapchain_khr.take() {
                self.swapchain_loader
                    .as_ref()
                    .unwrap()
                    .destroy_swapchain(swapchain_khr, None);
            }
        }
    }

    fn recreate_swapchain(&mut self) {
        // Wait for device to be idle if window is minimized (extent 0,0)
        let window = self
            .window
            .as_ref()
            .expect("Window not available for recreate");
        let mut size = window.inner_size();
        while size.width == 0 || size.height == 0 {
            size = window.inner_size();
            // This should integrate with the event loop properly, not block here.
            // For now, this is a placeholder for proper minimized handling.
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        if let Some(device) = &self.device {
            unsafe {
                device.device_wait_idle().expect("Device wait idle failed");
            }
        }

        self.cleanup_swapchain();
        self.create_swapchain();
        self.create_image_views();
        self.create_render_pass(); // Render pass might depend on new swapchain format
        self.create_framebuffers();
        // Command buffers might need to be re-recorded if render pass/framebuffers changed significantly.
        // Sync objects (semaphores/fences) are generally reusable.
        // images_in_flight needs to be resized and reset.
        self.images_in_flight = vec![vk::Fence::null(); self.swapchain_images.len()];

        let new_size = self.window.as_ref().unwrap().inner_size();
        let new_scale_factor = self.window.as_ref().unwrap().scale_factor();
        self.engine.on_window_resized(new_size, new_scale_factor);
        // Pass down new swapchain related info if engine needs to rebuild pipelines or framebuffers
        // For this design, engine's on_gpu_resources_ready might need to be callable for re-init
        // or a specific "on_swapchain_recreated" method on the engine.
        // For now, on_window_resized informs it of size change, pipelines should adapt or be generic.

        let new_framebuffers = self.swapchain_framebuffers.clone();
        self.engine.set_swapchain_framebuffers(new_framebuffers);
    }

    fn init_vulkan(&mut self, window: &Window) {
        self.create_instance(window);
        self.create_surface(window);
        self.pick_physical_device();
        self.create_logical_device();
        self.create_swapchain();
        self.create_image_views();
        self.create_render_pass();
        self.create_framebuffers();
        self.create_command_pool();
        self.create_command_buffers(); // For MAX_FRAMES_IN_FLIGHT
        self.create_sync_objects();

        let device = self.device.as_ref().unwrap().clone(); // Clone for AshEngine
        let instance = self.instance.as_ref().unwrap();
        let physical_device = self.physical_device.unwrap();
        let graphics_queue = self.graphics_queue.unwrap();
        let command_pool = self.command_pool.unwrap();
        let render_pass = self.render_pass.unwrap();
        let swapchain_format = self.swapchain_format.unwrap();

        let framebuffers_clone = self.swapchain_framebuffers.clone();

        // Notify engine
        self.engine.on_gpu_resources_ready(
            instance,
            physical_device,
            device,
            graphics_queue,
            command_pool,
            swapchain_format,
            render_pass,
            framebuffers_clone,
            window.inner_size(),
            window.scale_factor(),
            MAX_SPRITES,
            MAX_FRAMES_IN_FLIGHT,
        );
    }
}

impl GpuResourceManager for AshGpuResourceManager {
    fn on_window_ready(&mut self, window: Arc<Window>) {
        self.window = Some(window.clone());
        self.init_vulkan(&window);
    }

    fn on_window_lost(&mut self) {
        // This implies the surface is gone, Vulkan context might need full re-init
        // or just cleanup. Let engine know.
        if let Some(device) = &self.device {
            unsafe {
                device.device_wait_idle().expect("Device wait idle failed");
            }
        }
        self.engine.on_gpu_resources_lost();
        self.cleanup_swapchain(); // Clean more if device itself is lost

        // Destroy sync objects, command pool, device, surface, instance, etc.
        // This drop will handle it, but if this is not the end of the app,
        // selective cleanup is needed.
        // For simplicity, full cleanup on lost is often done by dropping AshGpuResourceManager.
    }

    fn on_window_resized(&mut self, _window_size: PhysicalSize<u32>, _window_scale_factor: f64) {
        // The actual size/scale comes from self.window. This is just a signal.
        self.framebuffer_resized = true;
    }

    fn update(&mut self) {
        if self.device.is_none() {
            return;
        } // Not initialized yet

        let device = self.device.as_ref().unwrap();
        let swapchain_loader = self.swapchain_loader.as_ref().unwrap();

        let wait_fences = [self.in_flight_fences[self.current_frame]];
        unsafe {
            device
                .wait_for_fences(&wait_fences, true, u64::MAX)
                .expect("Failed to wait for fence");
        }

        let acquire_result = unsafe {
            swapchain_loader.acquire_next_image(
                self.swapchain_khr.unwrap(),
                u64::MAX,
                self.image_available_semaphores[self.current_frame],
                vk::Fence::null(), // Not using a fence here
            )
        };

        let image_index = match acquire_result {
            Ok((image_idx, _is_suboptimal)) => image_idx,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                self.recreate_swapchain();
                return;
            }
            Err(e) => panic!("Failed to acquire swapchain image: {:?}", e),
        };

        // Check if a previous frame is using this image (i.e. there is its fence to wait on)
        if self.images_in_flight[image_index as usize] != vk::Fence::null() {
            unsafe {
                device
                    .wait_for_fences(
                        &[self.images_in_flight[image_index as usize]],
                        true,
                        u64::MAX,
                    )
                    .expect("Failed to wait for image in flight fence");
            }
        }
        // Mark the image as now being in use by this frame
        self.images_in_flight[image_index as usize] = self.in_flight_fences[self.current_frame];

        let command_buffer = self.command_buffers[self.current_frame];
        unsafe {
            // Resetting command buffer before recording
            device
                .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                .expect("Failed to reset command buffer");

            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            device
                .begin_command_buffer(command_buffer, &begin_info)
                .expect("Failed to begin command buffer");
        }

        // --- Engine Update ---
        // The engine is responsible for its own render pass begin/end within this command buffer
        // or AshGpuResourceManager can start the render pass here if it's common for all engine updates.
        // For now, assume engine handles its render pass with the provided command_buffer & image_index.
        self.engine
            .update(command_buffer, image_index, self.current_frame);
        // --- End Engine Update ---

        let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
        let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_infos = [vk::SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(std::slice::from_ref(&command_buffer))
            .signal_semaphores(&signal_semaphores)];

        unsafe {
            device
                .reset_fences(&[self.in_flight_fences[self.current_frame]])
                .expect("Failed to reset fence");
            device
                .queue_submit(
                    self.graphics_queue.unwrap(),
                    &submit_infos,
                    self.in_flight_fences[self.current_frame],
                )
                .expect("Failed to submit draw command buffer");
        }

        unsafe {
            device
                .end_command_buffer(command_buffer)
                .expect("Failed to end command buffer");
        }

        let swapchains = [self.swapchain_khr.unwrap()];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        let present_result =
            unsafe { swapchain_loader.queue_present(self.present_queue.unwrap(), &present_info) };

        let mut recreate = self.framebuffer_resized; // Check flag from on_window_resized
        match present_result {
            Ok(is_suboptimal) if is_suboptimal => recreate = true,
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => recreate = true,
            Err(e) => panic!("Failed to present swapchain image: {:?}", e),
            _ => {}
        }
        if recreate {
            self.framebuffer_resized = false; // Reset flag
            self.recreate_swapchain();
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

impl Drop for AshGpuResourceManager {
    fn drop(&mut self) {
        if let Some(device) = &self.device {
            unsafe {
                device
                    .device_wait_idle()
                    .expect("Device wait idle failed on drop");

                device
                    .wait_for_fences(&self.in_flight_fences, true, u64::MAX)
                    .expect("Failed to wait for fences during cleanup");

                for framebuffer in self.swapchain_framebuffers.drain(..) {
                    device.destroy_framebuffer(framebuffer, None);
                }
                // Command buffers are from a pool associated with MAX_FRAMES_IN_FLIGHT, not swapchain directly
                // so they are not cleaned up here typically, unless the pool needs recreation due to queue family change.
                // self.command_buffers will be reused.

                if let Some(render_pass) = self.render_pass.take() {
                    device.destroy_render_pass(render_pass, None);
                }
                for image_view in self.swapchain_image_views.drain(..) {
                    device.destroy_image_view(image_view, None);
                }
                if let Some(swapchain_khr) = self.swapchain_khr.take() {
                    self.swapchain_loader
                        .as_ref()
                        .unwrap()
                        .destroy_swapchain(swapchain_khr, None);
                }

                for &semaphore in self.image_available_semaphores.iter() {
                    device.destroy_semaphore(semaphore, None);
                }
                for &semaphore in self.render_finished_semaphores.iter() {
                    device.destroy_semaphore(semaphore, None);
                }
                for &fence in self.in_flight_fences.iter() {
                    device.destroy_fence(fence, None);
                }
                // images_in_flight contains copies of in_flight_fences, no need to double destroy.

                if let Some(pool) = self.command_pool.take() {
                    // Command buffers are freed when pool is destroyed
                    device.destroy_command_pool(pool, None);
                }

                // Device is destroyed after all its resources
                // self.device is an Option<ash::Device>
            }
        }
        // swapchain_loader is dropped automatically (it's just a struct with function pointers)

        if let Some(device) = self.device.take() {
            // Take ownership to drop
            unsafe {
                device.destroy_device(None);
            }
        }

        if let (Some(surface_loader), Some(surface_khr), Some(instance)) = (
            self.surface_loader.take(),
            self.surface_khr.take(),
            self.instance.as_ref(),
        ) {
            unsafe {
                surface_loader.destroy_surface(surface_khr, None);
            }
        }

        if let (Some(debug_utils_loader), Some(debug_messenger), Some(instance)) = (
            self.debug_utils_loader.take(),
            self.debug_messenger.take(),
            self.instance.as_ref(),
        ) {
            #[cfg(debug_assertions)]
            unsafe {
                debug_utils_loader.destroy_debug_utils_messenger(debug_messenger, None);
            }
        }

        if let Some(instance) = self.instance.take() {
            // Take ownership to drop
            unsafe {
                instance.destroy_instance(None);
            }
        }
    }
}

// --- Vulkan Debug Callback ---
unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;
    let message_id_name = if callback_data.p_message_id_name.is_null() {
        std::borrow::Cow::from("None")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };
    let message = if callback_data.p_message.is_null() {
        std::borrow::Cow::from("None")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity, message_type, message_id_name, message_id_number, message,
    );
    vk::FALSE // VK_FALSE means the call will not be aborted
}
