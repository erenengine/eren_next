use ash::vk;
use eren_window::window::WindowSize;
use thiserror::Error;
use winit::window::Window;

use crate::vulkan::{
    instance::{VulkanInstanceManager, VulkanInstanceManagerError},
    logical_device::{LogicalDeviceManager, LogicalDeviceManagerError},
    physical_device::{PhysicalDeviceManager, PhysicalDeviceManagerError},
    surface::{SurfaceManager, SurfaceManagerError},
    swapchain::{SwapchainManager, SwapchainManagerError},
};

#[derive(Debug, Error)]
pub enum GraphicsContextError {
    #[error("Failed to load entry: {0}")]
    LoadEntry(#[from] ash::LoadingError),

    #[error("Failed to create instance: {0}")]
    CreateInstanceFailed(#[from] VulkanInstanceManagerError),

    #[error("Failed to create surface: {0}")]
    CreateSurfaceFailed(#[from] SurfaceManagerError),

    #[error("Failed to create physical device: {0}")]
    CreatePhysicalDeviceFailed(#[from] PhysicalDeviceManagerError),

    #[error("Failed to create logical device: {0}")]
    CreateLogicalDeviceFailed(#[from] LogicalDeviceManagerError),

    #[error("Failed to create swapchain: {0}")]
    CreateSwapchainFailed(#[from] SwapchainManagerError),
}

#[derive(Debug)]
pub struct FrameContext {
    command_buffer: vk::CommandBuffer,
    framebuffer: vk::Framebuffer,
}

pub struct GraphicsContext<F>
where
    F: Fn(&FrameContext),
{
    draw_frame: F,
    entry: ash::Entry,

    instance_manager: Option<VulkanInstanceManager>,
    surface_manager: Option<SurfaceManager>,
    physical_device_manager: Option<PhysicalDeviceManager>,
    logical_device_manager: Option<LogicalDeviceManager>,
    swapchain_manager: Option<SwapchainManager>,
}

impl<F> GraphicsContext<F>
where
    F: Fn(&FrameContext),
{
    pub fn new(draw_frame: F) -> Result<Self, GraphicsContextError> {
        let entry = unsafe { ash::Entry::load()? };
        Ok(Self {
            draw_frame,
            entry,
            instance_manager: None,
            surface_manager: None,
            physical_device_manager: None,
            logical_device_manager: None,
            swapchain_manager: None,
        })
    }

    pub fn init(&mut self, window: &Window) -> Result<(), GraphicsContextError> {
        let instance_manager = VulkanInstanceManager::new(&self.entry, window)?;
        let surface_manager = SurfaceManager::new(&self.entry, &instance_manager.instance, window)?;
        let physical_device_manager = PhysicalDeviceManager::new(
            &instance_manager.instance,
            &surface_manager.surface_loader,
            surface_manager.surface,
        )?;
        let logical_device_manager = LogicalDeviceManager::new(
            &instance_manager.instance,
            physical_device_manager.physical_device,
            &physical_device_manager.queue_family_indices,
        )?;
        let swapchain_manager = SwapchainManager::new(
            window,
            &instance_manager.instance,
            surface_manager.surface,
            &physical_device_manager.queue_family_indices,
            &physical_device_manager.swapchain_support_details,
            &logical_device_manager.logical_device,
        )?;

        self.instance_manager = Some(instance_manager);
        self.surface_manager = Some(surface_manager);
        self.physical_device_manager = Some(physical_device_manager);
        self.logical_device_manager = Some(logical_device_manager);
        self.swapchain_manager = Some(swapchain_manager);

        Ok(())
    }

    pub fn resize(&mut self, window_size: WindowSize) {
        //TODO:
        println!("Resizing not implemented");
    }

    pub fn destroy(&mut self) {
        self.instance_manager = None;
        self.surface_manager = None;
        self.physical_device_manager = None;
        self.logical_device_manager = None;
        self.swapchain_manager = None;
    }

    pub fn redraw(&mut self) {
        //TODO:
        //println!("Redraw not implemented");
    }
}
