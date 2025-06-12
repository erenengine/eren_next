use ash::vk;

struct DrawRequest<MA> {
    pub matrix: Mat4,
    pub alpha: f32,
    pub model_asset_id: MA,
}

pub struct DrawCall {
    pub mesh: Arc<Mesh>,
    pub material: Arc<Material>,
    pub transform: Mat4,
    pub alpha: f32,
    pub render_pass_tag: RenderPassTag,
}

pub struct Renderer {
    device: ash::Device,
    command_pool: vk::CommandPool,
    graphics_queue: vk::Queue,
    command_buffer: vk::CommandBuffer,

    //shadow_pass: ShadowPass,
    geometry_pass: GeometryPass,
    //post_process_pass: PostProcessPass,
}

impl Renderer {
    pub fn new(device: ash::Device, ...) -> Self {
        // 각 패스 초기화
        let shadow_pass = ShadowPass::new(&device, ...);
        let geometry_pass = GeometryPass::new(&device, ...);
        let post_process_pass = PostProcessPass::new(&device, ...);

        // 커맨드 버퍼 할당
        let command_buffer = allocate_command_buffer(&device, command_pool);

        Renderer {
            device,
            command_pool,
            graphics_queue,
            command_buffer,
            //shadow_pass,
            geometry_pass,
            //post_process_pass,
        }
    }

    pub fn render_frame(&mut self) {
        // 1. Begin Command Buffer
        let begin_info = vk::CommandBufferBeginInfo::builder();
        unsafe {
            self.device
                .begin_command_buffer(self.command_buffer, &begin_info)
                .expect("begin command buffer failed");
        }

        // 2. Shadow pass: generate depth map from light POV
        //self.shadow_pass.record(&self.device, self.command_buffer);

        // 3. Geometry pass: draw actual scene with lighting & shadows
        self.geometry_pass.record(&self.device, self.command_buffer);

        // 4. Post-process pass: tone mapping, gamma correction
        //self.post_process_pass.record(&self.device, self.command_buffer);

        // 5. End and submit
        unsafe {
            self.device
                .end_command_buffer(self.command_buffer)
                .expect("end command buffer failed");

            let submit_info = vk::SubmitInfo::builder()
                .command_buffers(&[self.command_buffer]);

            self.device
                .queue_submit(self.graphics_queue, &[submit_info.build()], vk::Fence::null())
                .expect("submit failed");

            self.device
                .queue_wait_idle(self.graphics_queue)
                .expect("queue wait failed");
        }
    }
}
