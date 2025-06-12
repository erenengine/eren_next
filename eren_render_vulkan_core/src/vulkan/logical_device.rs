use ash::vk;
use thiserror::Error;

use crate::vulkan::queue::QueueFamilyIndices;

#[derive(Debug, Error)]
pub enum LogicalDeviceManagerError {
    #[error("Failed to create device: {0}")]
    CreateDeviceFailed(String),
}

pub struct LogicalDeviceManager {
    logical_device: ash::Device,
}

impl LogicalDeviceManager {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        queue_family_indices: &QueueFamilyIndices,
    ) -> Result<Self, LogicalDeviceManagerError> {
        let priorities = [1.0f32];
        let queue_infos = [
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(
                    queue_family_indices
                        .graphics_queue_family_index
                        .expect("No graphics queue family index"),
                )
                .queue_priorities(&priorities),
            vk::DeviceQueueCreateInfo::default()
                .queue_family_index(
                    queue_family_indices
                        .present_queue_family_index
                        .expect("No present queue family index"),
                )
                .queue_priorities(&priorities),
        ];

        let device_create_info = vk::DeviceCreateInfo::default().queue_create_infos(&queue_infos);

        let logical_device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .map_err(|e| LogicalDeviceManagerError::CreateDeviceFailed(e.to_string()))?
        };

        Ok(Self { logical_device })
    }
}
