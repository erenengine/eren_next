use ash::vk;
use eren_render_vulkan_core::renderer::{FrameContext, Renderer};
use thiserror::Error;

use crate::{
    passes::{
        final_pass::{FinalPass, FinalPassError},
        geometry_pass::{CameraUBO, GeometryPass, GeometryPassError},
        shadow_pass::{LightVP, ShadowPass, ShadowPassError},
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

        let proj = glam::Mat4::perspective_rh(
            45_f32.to_radians(),
            image_extent.width as f32 / image_extent.height as f32,
            0.1,
            100.0,
        );
        let view = glam::Mat4::look_at_rh(
            glam::Vec3::new(3.0, 3.0, 3.0), // eye
            glam::Vec3::ZERO,               // at
            glam::Vec3::Y,                  // up
        );
        let view_proj = proj * view;

        let light_view = glam::Mat4::look_at_rh(
            glam::Vec3::new(4.0, 5.0, 2.0),
            glam::Vec3::ZERO,
            glam::Vec3::Y,
        );
        let light_proj = glam::Mat4::orthographic_rh(-6.0, 6.0, -6.0, 6.0, 0.1, 20.0);
        let light_vp = light_proj * light_view;

        shadow_pass.upload_light_vp_buffer(&LightVP {
            light_view_proj: light_vp,
        })?;

        geometry_pass.upload_camera_buffer(&CameraUBO {
            view_proj,
            light_view_proj: light_vp,
            light_dir: glam::Vec3::new(
                -light_view.z_axis.x,
                -light_view.z_axis.y,
                -light_view.z_axis.z,
            ),
            _pad: 0.0,
        })?;

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
