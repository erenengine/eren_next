use std::os::raw::c_void;

use ash::vk;
use eren_window::window::WindowSize;
use winit::window::Window;

#[derive(Debug)]
pub struct FrameContext {}

pub struct GraphicsContext<F>
where
    F: Fn(&FrameContext),
{
    draw_frame: F,
    instance: ash::Instance,
    debug_utils: ash::ext::debug_utils::Instance,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
}

impl<F> GraphicsContext<F>
where
    F: Fn(&FrameContext),
{
    pub fn new(appname: &'static str, draw_frame: F) -> Self {
        let entry = unsafe { ash::Entry::load().expect("Failed to load entry") };

        let enginename = std::ffi::CString::new("ErenEngine").unwrap();
        let appname = std::ffi::CString::new(appname).unwrap();
        let app_info = vk::ApplicationInfo {
            p_application_name: appname.as_ptr(),
            p_engine_name: enginename.as_ptr(),
            engine_version: vk::make_api_version(0, 1, 0, 0),
            application_version: vk::make_api_version(0, 1, 0, 0),
            api_version: vk::API_VERSION_1_3,
            ..Default::default()
        };

        let layer_names: Vec<std::ffi::CString> =
            vec![std::ffi::CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
        let layer_name_pointers: Vec<*const i8> = layer_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let mut extension_name_pointers: Vec<*const i8> =
            vec![ash::ext::debug_utils::NAME.as_ptr()];
        let mut instance_create_flags = vk::InstanceCreateFlags::empty();

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            use ash::khr::{get_physical_device_properties2, portability_enumeration};

            extension_name_pointers.push(portability_enumeration::NAME.as_ptr());
            extension_name_pointers.push(get_physical_device_properties2::NAME.as_ptr());
            instance_create_flags |= vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
        }

        let debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            pfn_user_callback: Some(vulkan_debug_utils_callback),
            ..Default::default()
        };

        let instance_create_info = vk::InstanceCreateInfo {
            p_next: &debug_create_info as *const vk::DebugUtilsMessengerCreateInfoEXT
                as *const c_void,
            p_application_info: &app_info,
            pp_enabled_layer_names: layer_name_pointers.as_ptr(),
            enabled_layer_count: layer_name_pointers.len() as u32,
            pp_enabled_extension_names: extension_name_pointers.as_ptr(),
            enabled_extension_count: extension_name_pointers.len() as u32,
            flags: instance_create_flags,
            ..Default::default()
        };

        let instance = unsafe {
            entry
                .create_instance(&instance_create_info, None)
                .expect("Failed to create instance")
        };

        let debug_utils = ash::ext::debug_utils::Instance::new(&entry, &instance);
        let debug_utils_messenger = unsafe {
            debug_utils
                .create_debug_utils_messenger(&debug_create_info, None)
                .expect("Failed to create debug utils messenger")
        };

        Self {
            draw_frame,
            instance,
            debug_utils,
            debug_utils_messenger,
        }
    }

    pub fn init(&mut self, window: &Window) {}

    pub fn resize(&mut self, window_size: WindowSize) {}

    pub fn destroy(&mut self) {}

    pub fn redraw(&mut self) {}
}

impl<F> Drop for GraphicsContext<F>
where
    F: Fn(&FrameContext),
{
    fn drop(&mut self) {
        unsafe {
            self.debug_utils
                .destroy_debug_utils_messenger(self.debug_utils_messenger, None);
            self.instance.destroy_instance(None)
        };
    }
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = unsafe { std::ffi::CStr::from_ptr((*p_callback_data).p_message) };
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();
    println!("[Debug][{}][{}] {:?}", severity, ty, message);
    vk::FALSE
}
