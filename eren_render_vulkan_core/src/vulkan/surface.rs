use ash::{khr, vk};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

use crate::vulkan::instance::VulkanInstanceManager;

pub struct SurfaceManager {
    pub surface_loader: khr::surface::Instance,
    pub surface: vk::SurfaceKHR,
}

impl SurfaceManager {
    pub fn new(
        entry: &ash::Entry,
        instance_manager: &VulkanInstanceManager,
        window: &Window,
    ) -> Self {
        let surface_loader = khr::surface::Instance::new(entry, &instance_manager.instance);
        let surface = unsafe {
            ash_window::create_surface(
                entry,
                &instance_manager.instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .expect("Failed to create window surface")
        };
        Self {
            surface_loader,
            surface,
        }
    }
}

impl Drop for SurfaceManager {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}
