use ash::vk;

pub struct QueueFamilyIndices {
    graphics_queue_family_index: Option<u32>,
    present_queue_family_index: Option<u32>,
}

impl QueueFamilyIndices {
    fn is_complete(&self) -> bool {
        self.graphics_queue_family_index.is_some() && self.present_queue_family_index.is_some()
    }
}

pub fn find_queue_family_indices(
    instance: &ash::Instance,
    surface_loader: &ash::khr::surface::Instance,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> QueueFamilyIndices {
    let mut indices = QueueFamilyIndices {
        graphics_queue_family_index: None,
        present_queue_family_index: None,
    };

    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    for (i, queue_family) in queue_families.iter().enumerate() {
        if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            indices.graphics_queue_family_index = Some(i as u32);
        }

        let present_support = unsafe {
            surface_loader
                .get_physical_device_surface_support(physical_device, i as u32, surface)
                .unwrap_or(false)
        };

        if present_support {
            indices.present_queue_family_index = Some(i as u32);
        }

        if indices.is_complete() {
            break;
        }
    }

    indices
}
