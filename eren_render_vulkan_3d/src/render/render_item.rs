use std::sync::Arc;

use ash::vk;

pub struct Mesh {
    pub vertex_buffer: vk::Buffer,
    pub vertex_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_memory: vk::DeviceMemory,
    pub index_count: u32,
}

pub struct Material {
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set: vk::DescriptorSet,
}

pub struct RenderItem {
    pub mesh: Arc<Mesh>,
    pub material: Arc<Material>,
    pub transform: glam::Mat4,
}
