use ash::{khr::surface, vk};
use thiserror::Error;

use crate::vulkan::{
    queue::{QueueFamilyIndices, find_queue_family_indices},
    swapchain::{SwapchainSupportDetails, SwapchainSupportError, get_swapchain_support_details},
};

#[derive(Debug, Error)]
pub enum PhysicalDeviceManagerError {
    #[error("Failed to enumerate physical devices: {0}")]
    EnumeratePhysicalDevicesFailed(String),

    #[error("Swapchain support query failed: {0}")]
    SwapchainSupportQueryFailed(#[from] SwapchainSupportError),

    #[error("No suitable physical device found")]
    NoSuitablePhysicalDevice,
}

pub struct PhysicalDeviceManager {
    pub queue_family_indices: QueueFamilyIndices,
    pub swapchain_support_details: SwapchainSupportDetails,
    pub physical_device: vk::PhysicalDevice,
}

impl PhysicalDeviceManager {
    pub fn new(
        instance: &ash::Instance,
        surface_loader: &surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> Result<Self, PhysicalDeviceManagerError> {
        let physical_devices = unsafe {
            instance.enumerate_physical_devices().map_err(|e| {
                PhysicalDeviceManagerError::EnumeratePhysicalDevicesFailed(e.to_string())
            })?
        };

        for physical_device in physical_devices {
            if !has_required_device_features(instance, physical_device)
                || !has_required_device_extensions(instance, physical_device)
            {
                continue;
            }

            let queue_family_indices =
                find_queue_family_indices(instance, surface_loader, surface, physical_device);
            if !queue_family_indices.is_complete() {
                continue;
            }

            let swapchain_support_details =
                get_swapchain_support_details(surface_loader, surface, physical_device)?;
            if swapchain_support_details.formats.is_empty()
                || swapchain_support_details.present_modes.is_empty()
            {
                continue;
            }

            return Ok(Self {
                queue_family_indices,
                swapchain_support_details,
                physical_device,
            });
        }

        Err(PhysicalDeviceManagerError::NoSuitablePhysicalDevice)
    }
}

pub fn get_required_device_features() -> vk::PhysicalDeviceFeatures {
    vk::PhysicalDeviceFeatures::default().shader_clip_distance(true)
}

fn has_required_device_features(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> bool {
    let features = unsafe { instance.get_physical_device_features(physical_device) };

    if features.shader_clip_distance != vk::TRUE {
        return false;
    }

    true
}

pub fn get_required_device_extensions() -> Vec<&'static std::ffi::CStr> {
    let mut required_extensions = vec![ash::khr::swapchain::NAME];

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        required_extensions.push(ash::khr::portability_subset::NAME);
    }

    required_extensions
}

fn has_required_device_extensions(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> bool {
    let extensions = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device)
            .unwrap_or_else(|_| Vec::new())
    };

    let required_extensions = get_required_device_extensions();

    for required_ext_name_cstr in required_extensions.iter() {
        let required_ext_name =
            unsafe { std::ffi::CStr::from_ptr(required_ext_name_cstr.as_ptr()) };

        let found = extensions.iter().any(|ext| {
            let available_ext_name =
                unsafe { std::ffi::CStr::from_ptr(ext.extension_name.as_ptr()) };

            available_ext_name == required_ext_name
        });

        if !found {
            return false;
        }
    }

    true
}
