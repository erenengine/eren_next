use ash::vk;

pub enum MemoryLocation {
    GpuOnly,
    CpuToGpu,
}

pub struct BufferResource {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: vk::DeviceSize,
}

impl BufferResource {
    pub fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.destroy_buffer(self.buffer, None);
            device.free_memory(self.memory, None);
        }
    }
}

pub fn create_buffer<T: bytemuck::Pod>(
    device: &ash::Device,
    phys: &vk::PhysicalDeviceMemoryProperties,
    contents: Option<&[T]>,
    usage: vk::BufferUsageFlags,
    location: MemoryLocation,
) -> BufferResource {
    let byte_len =
        (contents.map(|c| c.len()).unwrap_or(1) * std::mem::size_of::<T>()) as vk::DeviceSize;

    let buffer_create_info = vk::BufferCreateInfo::default()
        .usage(usage)
        .size(byte_len)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_create_info, None) }.unwrap();

    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let mem_type_index = (0..phys.memory_type_count)
        .find(|&i| {
            (requirements.memory_type_bits & (1 << i)) != 0
                && phys.memory_types[i as usize]
                    .property_flags
                    .contains(match location {
                        MemoryLocation::GpuOnly => vk::MemoryPropertyFlags::DEVICE_LOCAL,
                        MemoryLocation::CpuToGpu => {
                            vk::MemoryPropertyFlags::HOST_VISIBLE
                                | vk::MemoryPropertyFlags::HOST_COHERENT
                        }
                    })
        })
        .expect("No suitable memory type found!");

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(requirements.size)
        .memory_type_index(mem_type_index as _);

    let memory = unsafe { device.allocate_memory(&alloc_info, None) }.unwrap();
    unsafe { device.bind_buffer_memory(buffer, memory, 0) }.unwrap();

    if let (Some(data), MemoryLocation::CpuToGpu) = (contents, location) {
        unsafe {
            let ptr = device
                .map_memory(memory, 0, byte_len, vk::MemoryMapFlags::empty())
                .unwrap();
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const std::ffi::c_void,
                ptr,
                byte_len as usize,
            );
            device.unmap_memory(memory);
        }
    }

    BufferResource {
        buffer,
        memory,
        size: byte_len,
    }
}
