use ash::vk;
use eren_render_vulkan_core::renderer::{FrameContext, Renderer};
use thiserror::Error;

use crate::{
    passes::shadow_pass::{ShadowPass, ShadowPassError},
    render::render_item::RenderItem,
};

#[derive(Debug, Error)]
pub enum Renderer3DError {
    #[error("Failed to create shadow pass: {0}")]
    ShadowPassCreationFailed(#[from] ShadowPassError),
}

pub struct Renderer3D {
    shadow_pass: ShadowPass,
}

impl Renderer3D {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: ash::Device,
        image_extent: vk::Extent2D,
    ) -> Result<Self, Renderer3DError> {
        let shadow_pass = ShadowPass::new(instance, physical_device, device, image_extent)?;
        Ok(Self { shadow_pass })
    }
}

impl Renderer<RenderItem> for Renderer3D {
    fn render(&self, frame_context: &FrameContext, render_items: &[RenderItem]) {
        self.shadow_pass.record(frame_context, render_items);
    }
}
