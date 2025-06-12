use eren_window::window::WindowSize;
use thiserror::Error;
use winit::window::Window;

use crate::vulkan::{
    instance::{VulkanInstanceManager, VulkanInstanceManagerError},
    physical_device::{PhysicalDeviceManager, PhysicalDeviceManagerError},
    surface::{SurfaceManager, SurfaceManagerError},
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
}

#[derive(Debug)]
pub struct FrameContext {}

pub struct GraphicsContext<F>
where
    F: Fn(&FrameContext),
{
    draw_frame: F,
    entry: ash::Entry,

    instance_manager: Option<VulkanInstanceManager>,
    surface_manager: Option<SurfaceManager>,
    physical_device_manager: Option<PhysicalDeviceManager>,
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
        })
    }

    pub fn init(&mut self, window: &Window) -> Result<(), GraphicsContextError> {
        let instance_manager = VulkanInstanceManager::new(&self.entry, window)?;
        let surface_manager = SurfaceManager::new(&self.entry, &instance_manager, window)?;
        let physical_device_manager =
            PhysicalDeviceManager::new(&instance_manager, &surface_manager)?;

        self.instance_manager = Some(instance_manager);
        self.surface_manager = Some(surface_manager);
        self.physical_device_manager = Some(physical_device_manager);

        Ok(())
    }

    pub fn resize(&mut self, window_size: WindowSize) {}

    pub fn destroy(&mut self) {
        self.instance_manager = None;
        self.surface_manager = None;
        self.physical_device_manager = None;
    }

    pub fn redraw(&mut self) {}
}
