use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SwapchainSupportError {
    #[error("Failed to enumerate swapchain support: {0}")]
    EnumerateSwapchainSupportFailed(String),

    #[error("Failed to enumerate swapchain formats: {0}")]
    EnumerateSwapchainFormatsFailed(String),

    #[error("Failed to enumerate swapchain present modes: {0}")]
    EnumerateSwapchainPresentModesFailed(String),
}

pub struct SwapchainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub fn get_swapchain_support_details(
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    device: vk::PhysicalDevice,
) -> Result<SwapchainSupportDetails, SwapchainSupportError> {
    Ok(unsafe {
        SwapchainSupportDetails {
            capabilities: surface_loader
                .get_physical_device_surface_capabilities(device, surface)
                .map_err(|e| {
                    SwapchainSupportError::EnumerateSwapchainSupportFailed(e.to_string())
                })?,
            formats: surface_loader
                .get_physical_device_surface_formats(device, surface)
                .map_err(|e| {
                    SwapchainSupportError::EnumerateSwapchainFormatsFailed(e.to_string())
                })?,
            present_modes: surface_loader
                .get_physical_device_surface_present_modes(device, surface)
                .map_err(|e| {
                    SwapchainSupportError::EnumerateSwapchainPresentModesFailed(e.to_string())
                })?,
        }
    })
}
