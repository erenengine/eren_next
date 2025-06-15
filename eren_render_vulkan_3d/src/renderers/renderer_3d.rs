use std::sync::Arc;

use ash::vk;
use eren_render_vulkan_core::renderer::{FrameContext, Renderer};
use thiserror::Error;

use crate::passes::test_pass::{TestPass, TestPassError};

#[derive(Debug, Error)]
pub enum Renderer3DError {
    #[error("Failed to create test pass: {0}")]
    TestPassCreationFailed(#[from] TestPassError),
}

pub struct Renderer3D {
    test_pass: TestPass,
}

impl Renderer3D {
    pub fn new(
        device: Arc<ash::Device>,
        swapchain_image_views: &Vec<vk::ImageView>,
        surface_format: vk::Format,
        image_extent: vk::Extent2D,
    ) -> Result<Self, Renderer3DError> {
        let test_pass = TestPass::new(device, swapchain_image_views, surface_format, image_extent)?;
        Ok(Self { test_pass })
    }
}

impl Renderer for Renderer3D {
    fn render(&self, frame_context: &FrameContext) {
        self.test_pass.record(frame_context);
    }
}
