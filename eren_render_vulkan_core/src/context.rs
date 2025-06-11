use eren_window::window::WindowSize;
use winit::window::Window;

use crate::vulkan::{
    instance::VulkanInstanceManager, physical_device::PhysicalDeviceManager,
    surface::SurfaceManager,
};

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
    pub fn new(draw_frame: F) -> Self {
        let entry = unsafe { ash::Entry::load().expect("Failed to load entry") };
        Self {
            draw_frame,
            entry,
            instance_manager: None,
            surface_manager: None,
            physical_device_manager: None,
        }
    }

    pub fn init(&mut self, window: &Window) {
        let instance_manager = VulkanInstanceManager::new(&self.entry, window);
        let surface_manager = SurfaceManager::new(&self.entry, &instance_manager, window);
        let physical_device_manager = PhysicalDeviceManager::new(&instance_manager);

        self.instance_manager = Some(instance_manager);
        self.surface_manager = Some(surface_manager);
        self.physical_device_manager = Some(physical_device_manager);
    }

    pub fn resize(&mut self, window_size: WindowSize) {}

    pub fn destroy(&mut self) {
        self.instance_manager = None;
        self.surface_manager = None;
        self.physical_device_manager = None;
    }

    pub fn redraw(&mut self) {}
}
