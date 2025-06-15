use ash::vk;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("Failed to find suitable memory type")]
    FindSuitableMemoryTypeFailed,

    #[error("Failed to create image: {0}")]
    CreateImageFailed(String),

    #[error("Failed to allocate memory: {0}")]
    AllocateMemoryFailed(String),

    #[error("Failed to bind memory to image: {0}")]
    BindMemoryToImageFailed(String),

    #[error("Failed to create buffer: {0}")]
    CreateBufferFailed(String),

    #[error("Failed to bind memory to buffer: {0}")]
    BindMemoryToBufferFailed(String),
}

pub fn find_memory_type(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
) -> Result<u32, MemoryError> {
    let mem_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };

    for i in 0..mem_properties.memory_type_count {
        if (type_filter & (1 << i)) != 0
            && mem_properties.memory_types[i as usize]
                .property_flags
                .contains(properties)
        {
            return Ok(i);
        }
    }

    Err(MemoryError::FindSuitableMemoryTypeFailed)
}

pub fn create_image_with_memory(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    image_info: &vk::ImageCreateInfo,
    memory_flags: vk::MemoryPropertyFlags,
) -> Result<(vk::Image, vk::DeviceMemory), MemoryError> {
    let image = unsafe {
        device
            .create_image(image_info, None)
            .map_err(|e| MemoryError::CreateImageFailed(e.to_string()))?
    };

    let mem_requirements = unsafe { device.get_image_memory_requirements(image) };

    let mem_type_index = find_memory_type(
        instance,
        physical_device,
        mem_requirements.memory_type_bits,
        memory_flags,
    )?;

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(mem_requirements.size)
        .memory_type_index(mem_type_index);

    let memory = unsafe {
        device
            .allocate_memory(&alloc_info, None)
            .map_err(|e| MemoryError::AllocateMemoryFailed(e.to_string()))?
    };

    unsafe {
        device
            .bind_image_memory(image, memory, 0)
            .map_err(|e| MemoryError::BindMemoryToImageFailed(e.to_string()))?;
    }

    Ok((image, memory))
}

pub fn create_buffer_with_memory(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    memory_properties: vk::MemoryPropertyFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory), MemoryError> {
    let buffer_info = vk::BufferCreateInfo::default()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_info, None) }
        .map_err(|e| MemoryError::CreateBufferFailed(e.to_string()))?;

    let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let mem_type_index = find_memory_type(
        instance,
        physical_device,
        mem_requirements.memory_type_bits,
        memory_properties,
    )?;

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(mem_requirements.size)
        .memory_type_index(mem_type_index);

    let memory = unsafe { device.allocate_memory(&alloc_info, None) }
        .map_err(|e| MemoryError::AllocateMemoryFailed(e.to_string()))?;

    unsafe {
        device
            .bind_buffer_memory(buffer, memory, 0)
            .map_err(|e| MemoryError::BindMemoryToBufferFailed(e.to_string()))?;
    }

    Ok((buffer, memory))
}
