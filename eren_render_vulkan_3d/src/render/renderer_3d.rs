use ash::vk;
use eren_render_vulkan_core::renderer::{FrameContext, Renderer};
use thiserror::Error;

use crate::{
    passes::{
        final_pass::{FinalPass, FinalPassError},
        geometry_pass::{GeometryPass, GeometryPassError},
        shadow_pass::{ShadowPass, ShadowPassError},
    },
    render::render_item::RenderItem,
};

#[derive(Debug, Error)]
pub enum Renderer3DError {
    #[error("Failed to create shadow pass: {0}")]
    ShadowPassCreationFailed(#[from] ShadowPassError),

    #[error("Failed to create geometry pass: {0}")]
    GeometryPassCreationFailed(#[from] GeometryPassError),

    #[error("Failed to create final pass: {0}")]
    FinalPassCreationFailed(#[from] FinalPassError),
}

pub struct Renderer3D {
    shadow_pass: ShadowPass,
    geometry_pass: GeometryPass,
    final_pass: FinalPass,
}

impl Renderer3D {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: ash::Device,
        swapchain_image_views: &Vec<vk::ImageView>,
        surface_format: vk::Format,
        image_extent: vk::Extent2D,
    ) -> Result<Self, Renderer3DError> {
        let shadow_pass = ShadowPass::new(instance, physical_device, device.clone(), image_extent)?;

        let geometry_pass = GeometryPass::new(
            instance,
            physical_device,
            device.clone(),
            image_extent,
            shadow_pass.depth_image_view,
        )?;

        let final_pass = FinalPass::new(
            device.clone(),
            swapchain_image_views,
            surface_format,
            image_extent,
            geometry_pass.color_image_view,
        )?;

        Ok(Self {
            shadow_pass,
            geometry_pass,
            final_pass,
        })
    }
}

impl Renderer<RenderItem> for Renderer3D {
    fn render(&self, frame_context: &FrameContext, render_items: &[RenderItem]) {
        self.shadow_pass.record(frame_context, render_items);
        self.geometry_pass.record(frame_context, render_items);
        self.final_pass.record(frame_context);
    }
}
