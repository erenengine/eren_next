use ash::{khr, vk};
use thiserror::Error;
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

#[derive(Debug, Error)]
pub enum SurfaceManagerError {
    #[error("Failed to create surface: {0}")]
    CreateSurfaceFailed(String),
}

pub struct SurfaceManager {
    pub surface_loader: khr::surface::Instance,
    pub surface: vk::SurfaceKHR,
}

impl SurfaceManager {
    pub fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &Window,
    ) -> Result<Self, SurfaceManagerError> {
        let surface_loader = khr::surface::Instance::new(entry, instance);

        let surface = unsafe {
            ash_window::create_surface(
                entry,
                instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .map_err(|e| SurfaceManagerError::CreateSurfaceFailed(e.to_string()))?
        };

        Ok(Self {
            surface_loader,
            surface,
        })
    }
}

impl Drop for SurfaceManager {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}
