use ash::vk;
use eren_render_vulkan_core::renderer::FrameContext;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShadowPassError {
    #[error("Failed to create depth image: {0}")]
    CreateDepthImageFailed(String),
}

pub struct ShadowPass {}

impl ShadowPass {
    pub fn new(
        device: &ash::Device,
        extent: vk::Extent2D,
        depth_format: vk::Format,
    ) -> Result<Self, ShadowPassError> {
        let depth_format = vk::Format::D32_SFLOAT;

        Ok(Self {})
    }

    pub fn record(&self, frame_context: &FrameContext) {}
}
