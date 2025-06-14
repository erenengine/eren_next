use std::sync::Arc;

use ash::vk;
use eren_render_vulkan_core::renderer::{FrameContext, Renderer};
use thiserror::Error;

use crate::passes::test_pass::{TestPass, TestPassError};

#[derive(Debug, Error)]
pub enum TestRendererError {
    #[error("Failed to create test pass: {0}")]
    TestPassCreationFailed(#[from] TestPassError),
}

pub struct TestRenderer {
    test_pass: TestPass,
}

impl TestRenderer {
    pub fn new(
        logical_device: Arc<ash::Device>,
        swapchain_image_views: &Vec<vk::ImageView>,
        surface_format: vk::Format,
        image_extent: vk::Extent2D,
    ) -> Result<Self, TestRendererError> {
        let test_pass = TestPass::new(
            logical_device,
            swapchain_image_views,
            surface_format,
            image_extent,
        )?;
        Ok(Self { test_pass })
    }
}

impl Renderer for TestRenderer {
    fn render(&self, frame_context: &FrameContext) {
        self.test_pass.record(frame_context);
    }
}
