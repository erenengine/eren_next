pub struct ShadowPass {
    pub framebuffer: vk::Framebuffer,
    pub render_pass: vk::RenderPass,
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    pub depth_image: vk::Image,
    pub depth_view: vk::ImageView,
}

impl ShadowPass {
    pub fn new(device: &ash::Device, extent: vk::Extent2D, depth_format: vk::Format) -> Self {
        // 1. Create depth image & view
        // 2. Create render pass (depth-only)
        // 3. Create framebuffer
        // 4. Create pipeline with vertex shader only (no fragment needed)

        // 예제에서는 생략된 초기화 생략

        ShadowPass {
            framebuffer,
            render_pass,
            pipeline,
            pipeline_layout,
            depth_image,
            depth_view,
        }
    }

    pub fn record(&self, device: &ash::Device, cmd: vk::CommandBuffer) {
        let clear = vk::ClearDepthStencilValue {
            depth: 1.0,
            stencil: 0,
        };

        let render_pass_begin = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: 1024,
                    height: 1024,
                },
            })
            .clear_values(&[vk::ClearValue {
                depth_stencil: clear,
            }]);

        unsafe {
            device.cmd_begin_render_pass(cmd, &render_pass_begin, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            // bind & draw your scene from light’s POV
            device.cmd_end_render_pass(cmd);
        }
    }
}
