use ash::vk;
use thiserror::Error;

use crate::vulkan::{
    physical_device::{get_required_device_extensions, get_required_device_features},
    queue::QueueFamilyIndices,
};

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
        let graphics_index = queue_family_indices.graphics_queue_family_index.unwrap();
        let present_index = queue_family_indices.present_queue_family_index.unwrap();

        let mut queue_create_infos = Vec::new();
        let queue_priority = [1.0f32];

        if graphics_index == present_index {
            queue_create_infos.push(
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(graphics_index)
                    .queue_priorities(&queue_priority),
            );
        } else {
            queue_create_infos.push(
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(graphics_index)
                    .queue_priorities(&queue_priority),
            );
            queue_create_infos.push(
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(present_index)
                    .queue_priorities(&queue_priority),
            );
        }

        let required_device_features = get_required_device_features();
        let raw_required_device_extensions: Vec<*const i8> = get_required_device_extensions()
            .iter()
            .map(|s| s.as_ptr())
            .collect();

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&required_device_features)
            .enabled_extension_names(&raw_required_device_extensions);

        let logical_device = unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .map_err(|e| LogicalDeviceManagerError::CreateDeviceFailed(e.to_string()))?
        };

        Ok(Self { logical_device })
    }
}
