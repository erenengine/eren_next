use ash::vk;
use winit::dpi::PhysicalSize;

pub trait AshEngine {
    fn on_gpu_resources_ready(
        &mut self,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: ash::Device,
        graphics_queue: vk::Queue,
        command_pool: vk::CommandPool,
        swapchain_format: vk::Format,
        window_size: PhysicalSize<u32>,
        scale_factor: f64,
        max_sprites: u32,
    );

    fn on_gpu_resources_lost(&mut self);

    fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, scale_factor: f64);

    fn update(
        &mut self,
        command_buffer: vk::CommandBuffer,
        frame_buffer: vk::Framebuffer,
        render_area: vk::Rect2D,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
    );
}
