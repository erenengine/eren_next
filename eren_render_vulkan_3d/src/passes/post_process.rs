use ash::vk;

pub struct PostProcessPass {
    pub render_pass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub framebuffer: vk::Framebuffer,
}

impl PostProcessPass {
    pub fn new(device: &ash::Device, color_format: vk::Format) -> Self {
        // 1. Render pass with only color attachment (swapchain)
        // 2. Pipeline using fullscreen triangle or quad
        // 3. Framebuffer: swapchain image view

        PostProcessPass {
            render_pass,
            pipeline,
            pipeline_layout,
            framebuffer,
        }
    }

    pub fn record(&self, device: &ash::Device, cmd: vk::CommandBuffer) {
        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: 800,
                    height: 600,
                },
            })
            .clear_values(&[clear_color]);

        unsafe {
            device.cmd_begin_render_pass(cmd, &render_pass_info, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            // bind tone-mapping texture from geometry pass
            device.cmd_draw(cmd, 3, 1, 0, 0); // full-screen triangle
            device.cmd_end_render_pass(cmd);
        }
    }
}
