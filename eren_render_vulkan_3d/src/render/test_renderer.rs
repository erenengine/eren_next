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

    #[error("Failed to create final pass: {0}")]
    FinalPassCreationFailed(#[from] FinalPassError),
}

pub struct TestRenderer {
    test_pass: TestPass,
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

        let final_pass = FinalPass::new(
            device.clone(),
            swapchain_image_views,
            surface_format,
            image_extent,
            test_pass.color_image_view,
        )?;

        Ok(Self {
            test_pass,
            final_pass,
        })
    }
}

impl Renderer<RenderItem> for TestRenderer {
    fn render(&self, frame_context: &FrameContext, _render_items: &[RenderItem]) {
        self.test_pass.record(frame_context);
        self.final_pass.record(frame_context);
    }
}
