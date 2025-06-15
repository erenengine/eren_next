use ash::vk;
use eren_render_vulkan_core::renderer::{FrameContext, Renderer};
use thiserror::Error;

use crate::{
    passes::{
        final_pass::{FinalPass, FinalPassError},
        test_pass::{TestPass, TestPassError},
    },
    render::render_item::RenderItem,
};

#[derive(Debug, Error)]
pub enum TestRendererError {
    #[error("Failed to create test pass: {0}")]
    TestPassCreationFailed(#[from] TestPassError),

    #[error("Failed to create sampler: {0}")]
    SamplerCreationFailed(String),

    #[error("Failed to create descriptor pool: {0}")]
    DescriptorPoolCreationFailed(String),

    #[error("Failed to create descriptor set layout: {0}")]
    DescriptorSetLayoutCreationFailed(String),

    #[error("Failed to create final pass: {0}")]
    FinalPassCreationFailed(#[from] FinalPassError),
}

pub struct TestRenderer {
    device: ash::Device,

    test_pass: TestPass,

    sampler: vk::Sampler,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_sets: Vec<vk::DescriptorSet>,

    final_pass: FinalPass,
}

impl TestRenderer {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: ash::Device,
        swapchain_image_views: &Vec<vk::ImageView>,
        surface_format: vk::Format,
        image_extent: vk::Extent2D,
    ) -> Result<Self, TestRendererError> {
        let test_pass = TestPass::new(instance, physical_device, device.clone(), image_extent)?;

        let sampler_create_info = vk::SamplerCreateInfo::default();
        let sampler = unsafe {
            device
                .create_sampler(&sampler_create_info, None)
                .map_err(|e| TestRendererError::SamplerCreationFailed(e.to_string()))?
        };

        let descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let descriptor_set_layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(std::slice::from_ref(&descriptor_set_layout_binding));

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&descriptor_set_layout_info, None)
                .map_err(|e| TestRendererError::DescriptorSetLayoutCreationFailed(e.to_string()))?
        };

        let pool_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: swapchain_image_views.len() as u32,
        };

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(std::slice::from_ref(&pool_size))
            .max_sets(swapchain_image_views.len() as u32)
            .flags(vk::DescriptorPoolCreateFlags::empty());

        let descriptor_pool = unsafe {
            device
                .create_descriptor_pool(&pool_info, None)
                .map_err(|e| TestRendererError::DescriptorPoolCreationFailed(e.to_string()))?
        };

        let descriptor_sets: Vec<vk::DescriptorSet> = swapchain_image_views
            .iter()
            .map(|_| {
                test_pass.create_descriptor_set(descriptor_pool, descriptor_set_layout, sampler)
            })
            .collect::<Result<_, _>>()?;

        let final_pass = FinalPass::new(
            device.clone(),
            swapchain_image_views,
            surface_format,
            image_extent,
            descriptor_set_layout,
        )?;

        Ok(Self {
            device,
            test_pass,
            sampler,
            descriptor_pool,
            descriptor_set_layout,
            descriptor_sets,
            final_pass,
        })
    }
}

impl Renderer<RenderItem> for TestRenderer {
    fn render(&self, frame_context: &FrameContext, _render_items: &[RenderItem]) {
        self.test_pass.record(frame_context);

        self.final_pass.record(
            frame_context,
            self.descriptor_sets[frame_context.image_index],
        );
    }
}

impl Drop for TestRenderer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait for device idle");

            self.device.destroy_sampler(self.sampler, None);
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}
