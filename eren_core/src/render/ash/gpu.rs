use ash::{
    Device, Entry, Instance,
    ext::debug_utils,
    khr::{surface, swapchain},
    vk,
};
use std::{
    borrow::Cow,
    error::Error,
    ffi::{CStr, CString, c_char},
    ops::Drop,
};
use winit::{
    dpi::LogicalSize,
    event_loop::ActiveEventLoop,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

// Re-use the helper functions from the example
#[macro_export]
macro_rules! offset_of {
    ($base:path, $field:ident) => {{
        #[allow(unused_unsafe)]
        unsafe {
            let b: $base = std::mem::zeroed();
            std::ptr::addr_of!(b.$field) as isize - std::ptr::addr_of!(b) as isize
        }
    }};
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = unsafe { &*p_callback_data };
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        unsafe { CStr::from_ptr(callback_data.p_message_id_name) }.to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        unsafe { CStr::from_ptr(callback_data.p_message) }.to_string_lossy()
    };

    // Filter out noisy messages if desired
    // if message_id_name.contains("VUID-vkCmdDispatch-None-08612") { // Example filter
    //     return vk::FALSE;
    // }

    println!(
        "{:?}:\n{:?} [{}({})]: {}\n",
        message_severity, message_type, message_id_name, message_id_number, message,
    );

    vk::FALSE
}

pub fn find_memorytype_index(
    memory_req: &vk::MemoryRequirements,
    memory_prop: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    memory_prop.memory_types[..memory_prop.memory_type_count as _]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags.contains(flags)
        })
        .map(|(index, _memory_type)| index as u32)
}

// Simplified one-shot command buffer for internal use
fn submit_one_time_commands<F>(
    device: &Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
    action: F,
) -> Result<(), vk::Result>
where
    F: FnOnce(vk::CommandBuffer),
{
    unsafe {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = device.allocate_command_buffers(&alloc_info)?[0];

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        device.begin_command_buffer(command_buffer, &begin_info)?;

        action(command_buffer);

        device.end_command_buffer(command_buffer)?;

        let submit_info =
            vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&command_buffer));

        let fence = device.create_fence(&vk::FenceCreateInfo::default(), None)?;
        device.queue_submit(queue, std::slice::from_ref(&submit_info), fence)?;
        device.wait_for_fences(std::slice::from_ref(&fence), true, u64::MAX)?;

        device.destroy_fence(fence, None);
        device.free_command_buffers(command_pool, std::slice::from_ref(&command_buffer));
    }
    Ok(())
}

pub struct VulkanState {
    // Core Vulkan
    pub entry: Entry,
    pub instance: Option<Instance>,
    pub physical_device: Option<vk::PhysicalDevice>,
    pub device_memory_properties: Option<vk::PhysicalDeviceMemoryProperties>,
    pub queue_family_index: Option<u32>,
    pub device: Option<Device>,
    pub queue: Option<vk::Queue>,

    // Window and Surface
    // We make VulkanState own the window, as surface and swapchain depend on it.
    pub window: Option<Window>,
    pub surface_loader: Option<surface::Instance>,
    pub surface: Option<vk::SurfaceKHR>,
    pub surface_format: Option<vk::SurfaceFormatKHR>,
    pub surface_resolution: Option<vk::Extent2D>,
    pub present_mode: Option<vk::PresentModeKHR>,

    // Swapchain
    pub swapchain_loader: Option<swapchain::Device>,
    pub swapchain: Option<vk::SwapchainKHR>,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,

    // Depth Buffer
    pub depth_format: vk::Format, // Fixed for simplicity, can be configurable
    pub depth_image: Option<vk::Image>,
    pub depth_image_view: Option<vk::ImageView>,
    pub depth_image_memory: Option<vk::DeviceMemory>,

    // Command Management
    pub command_pool: Option<vk::CommandPool>,

    // Debug
    pub debug_utils_loader: Option<debug_utils::Instance>,
    pub debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    enable_validation_layers: bool,

    // State
    pub gpu_initialized: bool,
}

impl VulkanState {
    pub fn new(enable_validation_layers: bool) -> Result<Self, Box<dyn Error>> {
        let entry = unsafe { Entry::load()? };
        Ok(Self {
            entry,
            instance: None,
            physical_device: None,
            device_memory_properties: None,
            queue_family_index: None,
            device: None,
            queue: None,
            window: None,
            surface_loader: None,
            surface: None,
            surface_format: None,
            surface_resolution: None,
            present_mode: None,
            swapchain_loader: None,
            swapchain: None,
            swapchain_images: Vec::new(),
            swapchain_image_views: Vec::new(),
            depth_format: vk::Format::D32_SFLOAT, // Or D16_UNORM, D24_UNORM_S8_UINT
            depth_image: None,
            depth_image_view: None,
            depth_image_memory: None,
            command_pool: None,
            debug_utils_loader: None,
            debug_messenger: None,
            enable_validation_layers,
            gpu_initialized: false,
        })
    }

    pub fn create_window_if_needed(
        &mut self,
        event_loop: &ActiveEventLoop,
        title: &str,
        width: u32,
        height: u32,
    ) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title(title.to_string())
                .with_inner_size(LogicalSize::new(width, height));
            let window = event_loop.create_window(attrs).unwrap();
            self.window = Some(window);
        }
    }

    pub fn ensure_initialized(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.gpu_initialized {
            self.init_gpu()?;
            self.gpu_initialized = true;
        }
        Ok(())
    }

    fn init_gpu(&mut self) -> Result<(), Box<dyn Error>> {
        unsafe {
            let window = self
                .window
                .as_ref()
                .ok_or("Window not created before GPU init")?;

            // 1. Instance Creation
            let app_name = CString::new("VulkanApp")?;
            let engine_name = CString::new("NoEngine")?;
            let app_info = vk::ApplicationInfo::default()
                .application_name(&app_name)
                .application_version(vk::make_api_version(0, 1, 0, 0))
                .engine_name(&engine_name)
                .engine_version(vk::make_api_version(0, 1, 0, 0))
                .api_version(vk::API_VERSION_1_2); // Request 1.2 for simplicity, can be 1.0

            let mut extension_names_raw =
                ash_window::enumerate_required_extensions(window.display_handle()?.as_raw())?
                    .to_vec();

            let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
            let mut layers_names_raw: Vec<*const c_char> = Vec::new();
            if self.enable_validation_layers {
                layers_names_raw.push(layer_names[0].as_ptr());
                extension_names_raw.push(debug_utils::NAME.as_ptr());
            }

            #[cfg(any(target_os = "macos", target_os = "ios"))]
            {
                extension_names_raw.push(ash::khr::portability_enumeration::NAME.as_ptr());
                extension_names_raw.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
            }

            let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
                vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
            } else {
                vk::InstanceCreateFlags::empty()
            };

            let mut instance_create_info = vk::InstanceCreateInfo::default()
                .application_info(&app_info)
                .enabled_extension_names(&extension_names_raw)
                .flags(create_flags);
            if self.enable_validation_layers {
                instance_create_info = instance_create_info.enabled_layer_names(&layers_names_raw);
            }

            let instance = self.entry.create_instance(&instance_create_info, None)?;
            self.instance = Some(instance);
            let instance_ref = self.instance.as_ref().unwrap();

            // 2. Debug Messenger
            if self.enable_validation_layers {
                let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                    .message_severity(
                        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                            | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING, // | vk::DebugUtilsMessageSeverityFlagsEXT::INFO // Often too verbose
                                                                              // | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    )
                    .message_type(
                        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                    )
                    .pfn_user_callback(Some(vulkan_debug_callback));

                let debug_utils_loader = debug_utils::Instance::new(&self.entry, instance_ref);
                let debug_messenger =
                    debug_utils_loader.create_debug_utils_messenger(&debug_info, None)?;
                self.debug_utils_loader = Some(debug_utils_loader);
                self.debug_messenger = Some(debug_messenger);
            }

            // 3. Surface Creation
            let surface = ash_window::create_surface(
                &self.entry,
                instance_ref,
                window.display_handle()?.as_raw(),
                window.window_handle()?.as_raw(),
                None,
            )?;
            let surface_loader = surface::Instance::new(&self.entry, instance_ref);
            self.surface = Some(surface);
            self.surface_loader = Some(surface_loader);
            let surface_ref = self.surface.unwrap();
            let surface_loader_ref = self.surface_loader.as_ref().unwrap();

            // 4. Physical Device Selection
            let pdevices = instance_ref.enumerate_physical_devices()?;
            let (pdevice, queue_family_index) = pdevices
                .into_iter()
                .find_map(|pdevice| {
                    instance_ref
                        .get_physical_device_queue_family_properties(pdevice)
                        .iter()
                        .enumerate()
                        .find_map(|(index, info)| {
                            let supports_graphics =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                            let supports_surface = surface_loader_ref
                                .get_physical_device_surface_support(
                                    pdevice,
                                    index as u32,
                                    surface_ref,
                                )
                                .unwrap_or(false);
                            if supports_graphics && supports_surface {
                                // Check for required extensions like swapchain
                                let extensions = instance_ref
                                    .enumerate_device_extension_properties(pdevice)
                                    .unwrap_or_default();
                                let has_swapchain_ext = extensions.iter().any(|ext| {
                                    CStr::from_ptr(ext.extension_name.as_ptr()) == swapchain::NAME
                                });
                                if has_swapchain_ext {
                                    Some((pdevice, index as u32))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                })
                .ok_or("Couldn't find suitable physical device.")?;
            self.physical_device = Some(pdevice);
            self.queue_family_index = Some(queue_family_index);
            self.device_memory_properties =
                Some(instance_ref.get_physical_device_memory_properties(pdevice));

            // 5. Logical Device Creation
            let device_extension_names_raw = [
                swapchain::NAME.as_ptr(),
                #[cfg(any(target_os = "macos", target_os = "ios"))]
                ash::khr::portability_subset::NAME.as_ptr(),
                // Add other extensions if needed, e.g., for ray tracing, mesh shaders, etc.
                // external_memory_win32::NAME.as_ptr(), // Example if needed
            ];

            let features = vk::PhysicalDeviceFeatures::default() // Enable features as needed
                .shader_clip_distance(true); // Example from ash_examples
            // .sampler_anisotropy(true) // if you use anisotropic filtering

            // You might need vk::PhysicalDeviceVulkan12Features etc. for newer features
            // let mut features12 = vk::PhysicalDeviceVulkan12Features::default().timeline_semaphore(true);
            // let device_create_info = vk::DeviceCreateInfo::default().push_next(&mut features12) ...

            let priorities = [1.0];
            let queue_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities);

            let device_create_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features);

            let device = instance_ref.create_device(pdevice, &device_create_info, None)?;
            let queue = device.get_device_queue(queue_family_index, 0);
            self.device = Some(device);
            self.queue = Some(queue);
            let device_ref = self.device.as_ref().unwrap();

            // 6. Swapchain Creation (Initial)
            let swapchain_loader = swapchain::Device::new(instance_ref, device_ref);
            self.swapchain_loader = Some(swapchain_loader);

            // 7. Command Pool
            let pool_create_info = vk::CommandPoolCreateInfo::default()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER) // Allow resetting individual command buffers
                .queue_family_index(queue_family_index);
            let command_pool = device_ref.create_command_pool(&pool_create_info, None)?;
            self.command_pool = Some(command_pool);
        }
        self.recreate_swapchain_and_depth();
        Ok(())
    }

    fn cleanup_swapchain_and_depth(&mut self) {
        if let Some(device) = self.device.as_ref() {
            unsafe {
                device
                    .wait_for_fences(&[], true, u64::MAX)
                    .unwrap_or_default(); // Ensure no operations are pending

                if let Some(view) = self.depth_image_view.take() {
                    device.destroy_image_view(view, None);
                }
                if let Some(image) = self.depth_image.take() {
                    device.destroy_image(image, None);
                }
                if let Some(memory) = self.depth_image_memory.take() {
                    device.free_memory(memory, None);
                }

                for view in self.swapchain_image_views.drain(..) {
                    device.destroy_image_view(view, None);
                }
                if let Some(swapchain) = self.swapchain.take() {
                    if let Some(loader) = self.swapchain_loader.as_ref() {
                        loader.destroy_swapchain(swapchain, None);
                    }
                }
            }
        }
        self.swapchain_images.clear();
    }

    fn recreate_swapchain_and_depth(&mut self) -> Result<(), Box<dyn Error>> {
        let pdevice = self.physical_device.ok_or("Physical device not set")?;
        let device = self.device.as_ref().ok_or("Device not set")?;
        let surface = self.surface.ok_or("Surface not set")?;
        let surface_loader = self
            .surface_loader
            .as_ref()
            .ok_or("Surface loader not set")?;
        let swapchain_loader = self
            .swapchain_loader
            .as_ref()
            .ok_or("Swapchain loader not set")?;
        let window = self.window.as_ref().ok_or("Window not set")?;

        unsafe {
            device.device_wait_idle()?; // Wait for current operations to complete

            // Query surface capabilities
            let surface_caps =
                surface_loader.get_physical_device_surface_capabilities(pdevice, surface)?;
            let formats = surface_loader.get_physical_device_surface_formats(pdevice, surface)?;
            let present_modes =
                surface_loader.get_physical_device_surface_present_modes(pdevice, surface)?;

            // Choose surface format
            let surface_format = formats
                .iter()
                .find(|f| {
                    f.format == vk::Format::B8G8R8A8_SRGB // Prefer sRGB for color
                        && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                })
                .unwrap_or(&formats[0]) // Fallback to the first available
                .clone();
            self.surface_format = Some(surface_format);

            // Choose present mode
            let present_mode = present_modes
                .iter()
                .cloned()
                .find(|&mode| mode == vk::PresentModeKHR::MAILBOX) // Prefer Mailbox for low latency
                .unwrap_or(vk::PresentModeKHR::FIFO); // FIFO is always available
            self.present_mode = Some(present_mode);

            // Determine surface resolution
            let current_extent = if surface_caps.current_extent.width != u32::MAX {
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
            self.surface_resolution = Some(current_extent);

            let mut desired_image_count = surface_caps.min_image_count + 1;
            if surface_caps.max_image_count > 0
                && desired_image_count > surface_caps.max_image_count
            {
                desired_image_count = surface_caps.max_image_count;
            }

            let pre_transform = if surface_caps
                .supported_transforms
                .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
            {
                vk::SurfaceTransformFlagsKHR::IDENTITY
            } else {
                surface_caps.current_transform
            };

            let swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
                .surface(surface)
                .min_image_count(desired_image_count)
                .image_format(surface_format.format)
                .image_color_space(surface_format.color_space)
                .image_extent(current_extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT) // Could also be TRANSFER_DST for blitting
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(pre_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(present_mode)
                .clipped(true)
                .old_swapchain(vk::SwapchainKHR::null()); // No old swapchain first time

            let swapchain = swapchain_loader.create_swapchain(&swapchain_create_info, None)?;
            self.swapchain = Some(swapchain);

            self.swapchain_images = swapchain_loader.get_swapchain_images(swapchain)?;
            self.swapchain_image_views = self
                .swapchain_images
                .iter()
                .map(|&image| {
                    let view_info = vk::ImageViewCreateInfo::default()
                        .image(image)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(surface_format.format)
                        .subresource_range(
                            vk::ImageSubresourceRange::default()
                                .aspect_mask(vk::ImageAspectFlags::COLOR)
                                .base_mip_level(0)
                                .level_count(1)
                                .base_array_layer(0)
                                .layer_count(1),
                        );
                    device.create_image_view(&view_info, None)
                })
                .collect::<Result<Vec<_>, _>>()?;

            // Create Depth Buffer
            let depth_format = self.depth_format;
            let depth_image_info = vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .format(depth_format)
                .extent(current_extent.into())
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);

            let depth_image = device.create_image(&depth_image_info, None)?;
            let mem_req = device.get_image_memory_requirements(depth_image);
            let mem_props = self
                .device_memory_properties
                .as_ref()
                .ok_or("Device memory properties not set")?;
            let mem_type_index =
                find_memorytype_index(&mem_req, mem_props, vk::MemoryPropertyFlags::DEVICE_LOCAL)
                    .ok_or("Failed to find suitable memory type for depth image")?;

            let alloc_info = vk::MemoryAllocateInfo::default()
                .allocation_size(mem_req.size)
                .memory_type_index(mem_type_index);
            let depth_image_memory = device.allocate_memory(&alloc_info, None)?;
            device.bind_image_memory(depth_image, depth_image_memory, 0)?;

            let depth_image_view_info = vk::ImageViewCreateInfo::default()
                .image(depth_image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(depth_format)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::DEPTH)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                );
            let depth_image_view = device.create_image_view(&depth_image_view_info, None)?;

            self.depth_image = Some(depth_image);
            self.depth_image_memory = Some(depth_image_memory);
            self.depth_image_view = Some(depth_image_view);

            // Transition depth image layout
            let command_pool = self.command_pool.ok_or("Command pool not set")?;
            let queue = self.queue.ok_or("Queue not set")?;
            submit_one_time_commands(device, command_pool, queue, |cmd_buffer| {
                let barrier = vk::ImageMemoryBarrier::default()
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .src_access_mask(vk::AccessFlags::empty())
                    .dst_access_mask(
                        vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                            | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                    )
                    .image(depth_image)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::DEPTH)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1),
                    );
                device.cmd_pipeline_barrier(
                    cmd_buffer,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS, // Or LATE_FRAGMENT_TESTS
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[barrier],
                );
            })?;
        }
        self.cleanup_swapchain_and_depth(); // Clean up old resources
        Ok(())
    }

    pub fn resize_surface(&mut self, _width: u32, _height: u32) -> Result<(), Box<dyn Error>> {
        // Width and height parameters are noted, but actual size comes from window/surface capabilities.
        // This function effectively recreates the swapchain.
        if !self.gpu_initialized {
            return Ok(());
        } // Nothing to resize if not initialized
        self.recreate_swapchain_and_depth()
    }

    pub fn release(&mut self) {
        if let Some(device) = self.device.take() {
            unsafe {
                device.device_wait_idle().unwrap_or_default();
                self.cleanup_swapchain_and_depth(); // Handles swapchain and depth

                if let Some(pool) = self.command_pool.take() {
                    device.destroy_command_pool(pool, None);
                }
                // Swapchain loader is implicitly handled by device destruction, or could be manually dropped
                // self.swapchain_loader.take();
                device.destroy_device(None);
            }
        }

        if let Some(instance) = self.instance.take() {
            unsafe {
                if let Some(surface) = self.surface.take() {
                    if let Some(loader) = self.surface_loader.take() {
                        loader.destroy_surface(surface, None);
                    }
                }
                if let Some(messenger) = self.debug_messenger.take() {
                    if let Some(loader) = self.debug_utils_loader.take() {
                        loader.destroy_debug_utils_messenger(messenger, None);
                    }
                }
                instance.destroy_instance(None);
            }
        }

        self.window = None; // Window is dropped here if owned
        self.gpu_initialized = false;
        // Entry doesn't need explicit cleanup
    }
}

impl Drop for VulkanState {
    fn drop(&mut self) {
        self.release();
    }
}
