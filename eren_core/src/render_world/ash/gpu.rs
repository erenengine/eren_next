use std::{ffi::CString, sync::Arc};

use super::engine::AshEngine;
use crate::render_world::common::gpu::GpuResourceManager;
use ash::{Entry, Instance, vk};
use ash_window::create_surface;
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};
use winit::{dpi::PhysicalSize, window::Window};

use ash::khr::{get_physical_device_properties2, portability_enumeration, surface, swapchain};

pub struct AshGpuResourceManager {
    engine: Box<dyn AshEngine>,
    entry: Entry,
}

impl AshGpuResourceManager {
    pub fn new(engine: Box<dyn AshEngine>) -> Self {
        let entry = unsafe { Entry::load().expect("Failed to load entry point") };
        Self { engine, entry }
    }
}

impl GpuResourceManager for AshGpuResourceManager {
    fn on_window_ready(&mut self, window: Arc<Window>) {
        let mut extension_names =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .unwrap()
                .to_vec();

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extension_names.push(portability_enumeration::NAME.as_ptr());
            extension_names.push(get_physical_device_properties2::NAME.as_ptr());
        }

        let application_info = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);

        let create_flags = vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&extension_names)
            .flags(create_flags);

        let instance = unsafe { self.entry.create_instance(&create_info, None) }
            .expect("Failed to create Vulkan instance");

        let surface = unsafe {
            ash_window::create_surface(
                &self.entry,
                &instance,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
        }
        .unwrap();

        let surface_loader = surface::Instance::new(&self.entry, &instance);

        let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() };

        let (physical_device, graphics_q_index, present_q_index) = physical_devices
            .into_iter()
            .find_map(|pd| {
                let mut graphics_q_index = None;
                let mut present_q_index = None;

                for (index, qf) in
                    unsafe { instance.get_physical_device_queue_family_properties(pd) }
                        .iter()
                        .enumerate()
                {
                    if qf.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                        graphics_q_index = Some(index as u32);
                    }
                    let supports_present = unsafe {
                        surface_loader.get_physical_device_surface_support(
                            pd,
                            index as u32,
                            surface,
                        )
                    }
                    .unwrap_or(false);
                    if supports_present {
                        present_q_index = Some(index as u32);
                    }
                }

                if let (Some(g), Some(p)) = (graphics_q_index, present_q_index) {
                    Some((pd, g, p))
                } else {
                    None
                }
            })
            .unwrap();

        println!(
            "Physical device: {:?}, Graphics queue: {:?}, Present queue: {:?}",
            physical_device, graphics_q_index, present_q_index
        );

        /*self.engine.on_gpu_resources_ready(
            &instance,
            physical_device: vk::PhysicalDevice,
            device: ash::Device,
            graphics_queue: vk::Queue,
            command_pool: vk::CommandPool,
            swapchain_format: vk::Format,
            window_size: PhysicalSize<u32>,
            scale_factor: f64,
            max_sprites: u32,
        );*/
    }

    fn on_window_lost(&mut self) {
        self.engine.on_gpu_resources_lost();
    }

    fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, window_scale_factor: f64) {
        self.engine
            .on_window_resized(window_size, window_scale_factor);
    }

    fn update(&mut self) {
        //self.engine.update();
    }
}
