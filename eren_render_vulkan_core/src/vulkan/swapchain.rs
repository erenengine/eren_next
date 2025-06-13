use ash::{khr::swapchain, vk};
use thiserror::Error;
use winit::window::Window;

use crate::vulkan::queue::QueueFamilyIndices;

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

#[derive(Debug, Error)]
pub enum SwapchainManagerError {
    #[error("Failed to create swapchain: {0}")]
    CreateSwapchainFailed(String),

    #[error("Failed to get swapchain images: {0}")]
    GetSwapchainImagesFailed(String),
}

pub struct SwapchainManager {
    pub swapchain_loader: swapchain::Device,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub amount_of_images: usize,
    pub preferred_surface_format: vk::Format,
    pub image_extent: vk::Extent2D,
}

impl SwapchainManager {
    pub fn new(
        window: &Window,
        instance: &ash::Instance,
        surface: vk::SurfaceKHR,
        queue_family_indices: &QueueFamilyIndices,
        support_details: &SwapchainSupportDetails,
        logical_device: &ash::Device,
    ) -> Result<Self, SwapchainManagerError> {
        let mut min_image_count = support_details.capabilities.min_image_count + 1;
        if support_details.capabilities.max_image_count > 0
            && min_image_count > support_details.capabilities.max_image_count
        {
            min_image_count = support_details.capabilities.max_image_count;
        }

        let surface_format = select_preferred_surface_format(&support_details.formats);
        let image_extent = determine_swapchain_extent(&window, &support_details.capabilities);
        let present_mode = select_preferred_present_mode(&support_details.present_modes);

        let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(min_image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(image_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(support_details.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let indices = [
            queue_family_indices.graphics_queue_family_index.unwrap(),
            queue_family_indices.present_queue_family_index.unwrap(),
        ];
        if queue_family_indices.graphics_queue_family_index.is_some()
            && queue_family_indices.present_queue_family_index.is_some()
            && queue_family_indices.graphics_queue_family_index
                != queue_family_indices.present_queue_family_index
        {
            swapchain_create_info = swapchain_create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&indices);
        } else {
            swapchain_create_info =
                swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        }

        let swapchain_loader = swapchain::Device::new(instance, logical_device);

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_create_info, None)
                .map_err(|e| SwapchainManagerError::CreateSwapchainFailed(e.to_string()))
        }?;

        let swapchain_images = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .map_err(|e| SwapchainManagerError::GetSwapchainImagesFailed(e.to_string()))?
        };

        let amount_of_images = swapchain_images.len();

        Ok(Self {
            swapchain_loader,
            swapchain,
            swapchain_images,
            amount_of_images,
            preferred_surface_format: surface_format.format,
            image_extent,
        })
    }
}

impl Drop for SwapchainManager {
    fn drop(&mut self) {
        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
        }
    }
}

fn select_preferred_surface_format(formats: &[vk::SurfaceFormatKHR]) -> &vk::SurfaceFormatKHR {
    formats
        .iter()
        .find(|f| {
            f.format == vk::Format::B8G8R8A8_SRGB
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or_else(|| &formats[0])
}

fn determine_swapchain_extent(
    window: &Window,
    capabilities: &vk::SurfaceCapabilitiesKHR,
) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        return capabilities.current_extent;
    }

    let window_size = window.inner_size();
    let width = window_size.width.clamp(
        capabilities.min_image_extent.width,
        capabilities.max_image_extent.width,
    );
    let height = window_size.height.clamp(
        capabilities.min_image_extent.height,
        capabilities.max_image_extent.height,
    );
    vk::Extent2D { width, height }
}

fn select_preferred_present_mode(
    available_present_modes: &[vk::PresentModeKHR],
) -> vk::PresentModeKHR {
    // Prefer MAILBOX mode for lower latency and less tearing
    if available_present_modes.contains(&vk::PresentModeKHR::MAILBOX) {
        vk::PresentModeKHR::MAILBOX
    } else {
        // FIFO is guaranteed to be available on all platforms
        vk::PresentModeKHR::FIFO
    }
}
