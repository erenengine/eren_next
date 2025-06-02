use ash::{Device, vk};
use std::ffi::c_void;

#[derive(Clone, Copy)]
pub enum MemoryLocation {
    GpuOnly,
    CpuToGpu, // Mapped, HostVisible + HostCoherent
    GpuToCpu, // Mapped, HostVisible + HostCached (needs flush/invalidate)
}

pub struct BufferResource {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub size: vk::DeviceSize,
    pub mapped_ptr: Option<*mut c_void>, // For persistently mapped buffers
}

impl BufferResource {
    pub fn destroy(&self, device: &Device) {
        unsafe {
            if self.mapped_ptr.is_some() {
                device.unmap_memory(self.memory);
            }
            device.destroy_buffer(self.buffer, None);
            device.free_memory(self.memory, None);
        }
    }
}

pub fn find_memory_type_index(
    memory_req: &vk::MemoryRequirements,
    mem_props: &vk::PhysicalDeviceMemoryProperties,
    flags: vk::MemoryPropertyFlags,
) -> Option<u32> {
    mem_props.memory_types[..mem_props.memory_type_count as usize]
        .iter()
        .enumerate()
        .find(|(index, memory_type)| {
            (1 << index) & memory_req.memory_type_bits != 0
                && memory_type.property_flags.contains(flags)
        })
        .map(|(index, _memory_type)| index as u32)
}

pub fn create_buffer_with_size<T: Copy>(
    device: &Device,
    mem_props: &vk::PhysicalDeviceMemoryProperties,
    buffer_size: vk::DeviceSize,
    initial_data: Option<&[T]>, // Data to initialize the buffer with
    usage: vk::BufferUsageFlags,
    location: MemoryLocation,
) -> BufferResource {
    let buffer_info = vk::BufferCreateInfo::default()
        .size(buffer_size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let buffer = unsafe {
        device
            .create_buffer(&buffer_info, None)
            .expect("Failed to create buffer")
    };
    let mem_req = unsafe { device.get_buffer_memory_requirements(buffer) };

    let (mem_flags, map_permanently) = match location {
        MemoryLocation::GpuOnly => (vk::MemoryPropertyFlags::DEVICE_LOCAL, false),
        MemoryLocation::CpuToGpu => (
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            true, // Typically persistently mapped for dynamic updates
        ),
        MemoryLocation::GpuToCpu => (
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_CACHED,
            true, // Typically persistently mapped for reading
        ),
    };

    let mem_type_index = find_memory_type_index(&mem_req, mem_props, mem_flags)
        .expect("Failed to find suitable memory type for buffer");

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(mem_req.size)
        .memory_type_index(mem_type_index);
    let memory = unsafe {
        device
            .allocate_memory(&alloc_info, None)
            .expect("Failed to allocate buffer memory")
    };
    unsafe {
        device
            .bind_buffer_memory(buffer, memory, 0)
            .expect("Failed to bind buffer memory")
    };

    let mut mapped_ptr = None;

    if let Some(data_slice) = initial_data {
        let data_actual_size = (std::mem::size_of::<T>() * data_slice.len()) as vk::DeviceSize;
        assert!(
            data_actual_size <= buffer_size,
            "Initial data size exceeds buffer capacity"
        );

        let ptr = unsafe {
            device
                .map_memory(memory, 0, data_actual_size, vk::MemoryMapFlags::empty())
                .unwrap()
        };
        unsafe {
            std::ptr::copy_nonoverlapping(
                data_slice.as_ptr() as *const c_void,
                ptr,
                data_actual_size as usize,
            );
        }
        if map_permanently {
            mapped_ptr = Some(ptr);
        } else {
            unsafe {
                device.unmap_memory(memory);
            }
        }
    } else if map_permanently {
        mapped_ptr = Some(unsafe {
            device
                .map_memory(memory, 0, buffer_size, vk::MemoryMapFlags::empty())
                .unwrap()
        });
    }

    BufferResource {
        buffer,
        memory,
        size: buffer_size,
        mapped_ptr,
    }
}
