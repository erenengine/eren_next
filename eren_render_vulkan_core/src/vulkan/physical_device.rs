//TODO: 개선 필요

use std::ffi::CStr;

use ash::{khr, vk};

use crate::vulkan::{instance::VulkanInstanceManager, surface::SurfaceManager};

struct QueueFamilyIndices {
    found_graphics_queue_family_index: Option<u32>,
    found_presentation_queue_family_index: Option<u32>,
}

impl QueueFamilyIndices {
    fn found(&self) -> bool {
        self.found_graphics_queue_family_index.is_some()
            && self.found_presentation_queue_family_index.is_some()
    }
}

struct SwapChainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

pub struct PhysicalDeviceManager {}

impl PhysicalDeviceManager {
    pub fn new(instance_manager: &VulkanInstanceManager, surface_manager: &SurfaceManager) -> Self {
        let physical_devices = unsafe {
            instance_manager
                .instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate physical devices")
        };

        let (physical_device, queue_family_indices) = physical_devices
            .into_iter()
            .find_map(|physical_device| {
                let queue_family_indices = Self::find_queue_families(
                    &instance_manager.instance,
                    &surface_manager.surface_loader,
                    surface_manager.surface,
                    physical_device,
                );
                let extensions_supported = Self::check_device_extension_support(
                    &instance_manager.instance,
                    physical_device,
                );
                let swapchain_adequate = if extensions_supported {
                    let swapchain_support = Self::query_swapchain_support(
                        &surface_manager.surface_loader,
                        surface_manager.surface,
                        physical_device,
                    );
                    !swapchain_support.formats.is_empty()
                        && !swapchain_support.present_modes.is_empty()
                } else {
                    false
                };

                // Check for features
                let features = unsafe {
                    instance_manager
                        .instance
                        .get_physical_device_features(physical_device)
                };
                let supported_features = features.shader_clip_distance == vk::TRUE;

                if queue_family_indices.found()
                    && extensions_supported
                    && swapchain_adequate
                    && supported_features
                {
                    Some((physical_device, queue_family_indices))
                } else {
                    None
                }
            })
            .expect("Failed to find a suitable GPU");

        Self {}
    }

    fn find_queue_families(
        instance: &ash::Instance,
        surface_loader: &khr::surface::Instance,
        surface: vk::SurfaceKHR,
        device: vk::PhysicalDevice,
    ) -> QueueFamilyIndices {
        let mut indices = QueueFamilyIndices {
            found_graphics_queue_family_index: None,
            found_presentation_queue_family_index: None,
        };
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(device) };

        for (i, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.found_graphics_queue_family_index = Some(i as u32);
            }
            let present_support = unsafe {
                surface_loader.get_physical_device_surface_support(device, i as u32, surface)
            }
            .unwrap_or(false);
            if present_support {
                indices.found_presentation_queue_family_index = Some(i as u32);
            }
            if indices.found() {
                break;
            }
        }
        indices
    }

    fn get_required_device_extensions() -> Vec<&'static CStr> {
        let mut extensions = vec![khr::swapchain::NAME];
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extensions.push(ash::khr::portability_subset::NAME);
        }
        extensions
    }

    fn check_device_extension_support(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
    ) -> bool {
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

    fn query_swapchain_support(
        surface_loader: &khr::surface::Instance,
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
}
