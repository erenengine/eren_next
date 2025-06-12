use ash::vk;
use thiserror::Error;
use winit::{raw_window_handle::HasDisplayHandle, window::Window};

#[derive(Debug, Error)]
pub enum VulkanInstanceManagerError {
    #[error("Failed to enumerate required extensions: {0}")]
    ExtensionEnumerationFailed(String),

    #[error("Failed to create instance: {0}")]
    CreateInstanceFailed(String),

    #[error("Failed to create debug utils messenger: {0}")]
    CreateDebugUtilsMessengerFailed(String),
}

pub struct VulkanInstanceManager {
    pub instance: ash::Instance,
    debug_utils_loader: ash::ext::debug_utils::Instance,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
}

impl VulkanInstanceManager {
    pub fn new(entry: &ash::Entry, window: &Window) -> Result<Self, VulkanInstanceManagerError> {
        let app_name = std::ffi::CString::new(window.title()).unwrap();
        let engine_name = std::ffi::CString::new("ErenEngine").unwrap();

        let app_info = vk::ApplicationInfo::default()
            .api_version(vk::API_VERSION_1_3)
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0));

        let layer_names: Vec<std::ffi::CString> =
            vec![std::ffi::CString::new("VK_LAYER_KHRONOS_validation").unwrap()];

        let layer_name_pointers: Vec<*const i8> = layer_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let mut extension_name_pointers: Vec<*const i8> =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .map_err(|e| VulkanInstanceManagerError::ExtensionEnumerationFailed(e.to_string()))?
                .to_vec();

        extension_name_pointers.push(ash::ext::debug_utils::NAME.as_ptr());

        let mut instance_create_flags = vk::InstanceCreateFlags::empty();

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            use ash::khr::{get_physical_device_properties2, portability_enumeration};

            extension_name_pointers.push(portability_enumeration::NAME.as_ptr());
            extension_name_pointers.push(get_physical_device_properties2::NAME.as_ptr());
            instance_create_flags |= vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
        }

        let mut debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    //| vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            )
            .pfn_user_callback(Some(vulkan_debug_utils_callback));

        let instance_create_info = vk::InstanceCreateInfo::default()
            .push_next(&mut debug_create_info)
            .application_info(&app_info)
            .enabled_layer_names(&layer_name_pointers)
            .enabled_extension_names(&extension_name_pointers)
            .flags(instance_create_flags);

        let instance = unsafe {
            entry
                .create_instance(&instance_create_info, None)
                .map_err(|e| VulkanInstanceManagerError::CreateInstanceFailed(e.to_string()))?
        };

        let debug_utils_loader = ash::ext::debug_utils::Instance::new(&entry, &instance);
        let debug_utils_messenger = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_create_info, None)
                .map_err(|e| {
                    VulkanInstanceManagerError::CreateDebugUtilsMessengerFailed(e.to_string())
                })?
        };

        Ok(Self {
            instance,
            debug_utils_loader,
            debug_utils_messenger,
        })
    }
}

impl Drop for VulkanInstanceManager {
    fn drop(&mut self) {
        unsafe {
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_utils_messenger, None);
            self.instance.destroy_instance(None);
        }
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
