use ash::vk;

pub struct GeometryPass {
    pub render_pass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub framebuffer: vk::Framebuffer,
}

impl GeometryPass {
    pub fn new(device: &ash::Device, color_format: vk::Format, depth_format: vk::Format) -> Self {
        // 1. Create render pass (color + depth)
        // 2. Create pipeline (vertex + fragment)
        // 3. Create framebuffer from swapchain image & depth view

        GeometryPass {
            render_pass,
            pipeline,
            pipeline_layout,
            framebuffer,
        }
    }

    pub fn record(&self, device: &ash::Device, cmd: vk::CommandBuffer) {
        let clear_colors = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.1, 0.1, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
        ];

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
            .clear_values(&clear_colors);

        unsafe {
            device.cmd_begin_render_pass(cmd, &render_pass_info, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            // bind mesh, descriptor sets, etc.
            device.cmd_end_render_pass(cmd);
        }
    }
}
