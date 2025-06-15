use std::sync::Arc;

use ash::vk;
use eren_window::window::WindowSize;
use thiserror::Error;
use winit::window::Window;

use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    renderer::{FrameContext, Renderer},
    vulkan::{
        device::{DeviceManager, DeviceManagerError},
        instance::{VulkanInstanceManager, VulkanInstanceManagerError},
        physical_device::{PhysicalDeviceManager, PhysicalDeviceManagerError},
        surface::{SurfaceManager, SurfaceManagerError},
        swapchain::{SwapchainManager, SwapchainManagerError},
    },
};

#[derive(Debug, Error)]
pub enum GraphicsContextError {
    #[error("Failed to load entry: {0}")]
    LoadEntry(#[from] ash::LoadingError),

    #[error("Failed to create instance: {0}")]
    CreateInstanceFailed(#[from] VulkanInstanceManagerError),

    #[error("Failed to create surface: {0}")]
    CreateSurfaceFailed(#[from] SurfaceManagerError),

    #[error("Failed to create physical device: {0}")]
    CreatePhysicalDeviceFailed(#[from] PhysicalDeviceManagerError),

    #[error("Failed to create logical device: {0}")]
    CreateLogicalDeviceFailed(#[from] DeviceManagerError),

    #[error("Failed to create swapchain: {0}")]
    CreateSwapchainFailed(#[from] SwapchainManagerError),

    #[error("Failed to create semaphores: {0}")]
    CreateSemaphoresFailed(String),

    #[error("Failed to create fences: {0}")]
    CreateFencesFailed(String),

    #[error("Failed to create swapchain image views: {0}")]
    CreateSwapchainImageViewsFailed(String),

    #[error("Failed to create command pool: {0}")]
    CreateCommandPoolFailed(String),

    #[error("Failed to create command buffers: {0}")]
    CreateCommandBuffersFailed(String),

    #[error("Failed to acquire next image: {0}")]
    AcquireNextImageFailed(String),

    #[error("Failed to wait for fences: {0}")]
    WaitForFencesFailed(String),

    #[error("Failed to reset command buffer: {0}")]
    ResetCommandBufferFailed(String),

    #[error("Failed to begin command buffer: {0}")]
    BeginCommandBufferFailed(String),

    #[error("Failed to end command buffer: {0}")]
    EndCommandBufferFailed(String),

    #[error("Failed to reset fences: {0}")]
    ResetFencesFailed(String),

    #[error("Failed to queue submit: {0}")]
    QueueSubmitFailed(String),

    #[error("Failed to queue present: {0}")]
    QueuePresentFailed(String),

    #[error("Failed to wait for device idle: {0}")]
    DeviceWaitIdleFailed(String),
}

pub struct GraphicsContext {
    entry: ash::Entry,

    window: Option<Arc<Window>>,
    pub instance_manager: Option<VulkanInstanceManager>,
    surface_manager: Option<SurfaceManager>,
    pub physical_device_manager: Option<PhysicalDeviceManager>,
    pub device_manager: Option<DeviceManager>,

    pub swapchain_manager: Option<SwapchainManager>,
    pub swapchain_image_views: Vec<vk::ImageView>,

    command_pool: Option<vk::CommandPool>,
    command_buffers: Vec<vk::CommandBuffer>,

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    frame_completion_fences: Vec<vk::Fence>,
    image_in_flight_fences: Vec<vk::Fence>,

    current_frame: usize,
    swapchain_needs_recreation: bool,
}

impl GraphicsContext {
    pub fn new() -> Result<Self, GraphicsContextError> {
        let entry = unsafe { ash::Entry::load()? };
        Ok(Self {
            entry,

            window: None,
            instance_manager: None,
            surface_manager: None,
            physical_device_manager: None,
            device_manager: None,
            swapchain_manager: None,

            swapchain_image_views: Vec::new(),
            command_pool: None,
            command_buffers: Vec::new(),

            image_available_semaphores: Vec::new(),
            render_finished_semaphores: Vec::new(),
            frame_completion_fences: Vec::new(),
            image_in_flight_fences: Vec::new(),

            current_frame: 0,
            swapchain_needs_recreation: false,
        })
    }

    pub fn init(&mut self, window: Arc<Window>) -> Result<(), GraphicsContextError> {
        let instance_manager = VulkanInstanceManager::new(&self.entry, window.clone())?;
        let surface_manager =
            SurfaceManager::new(&self.entry, &instance_manager.instance, window.clone())?;
        let physical_device_manager = PhysicalDeviceManager::new(
            &instance_manager.instance,
            &surface_manager.surface_loader,
            surface_manager.surface,
        )?;
        let device_manager = DeviceManager::new(
            &instance_manager.instance,
            physical_device_manager.physical_device,
            &physical_device_manager.queue_family_indices,
        )?;

        let command_pool = unsafe {
            device_manager
                .device
                .create_command_pool(
                    &vk::CommandPoolCreateInfo::default()
                        .queue_family_index(
                            physical_device_manager
                                .queue_family_indices
                                .graphics_queue_family_index
                                .expect("Graphics queue family index not found"),
                        )
                        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER), // Or TRANSIENT if re-recorded often
                    None,
                )
                .map_err(|e| GraphicsContextError::CreateCommandPoolFailed(e.to_string()))?
        };

        self.command_buffers = unsafe {
            device_manager
                .device
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::default()
                        .command_pool(command_pool)
                        .level(vk::CommandBufferLevel::PRIMARY)
                        .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32),
                )
                .map_err(|e| GraphicsContextError::CreateCommandBuffersFailed(e.to_string()))?
        };

        let semaphore_info = vk::SemaphoreCreateInfo::default();

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            self.image_available_semaphores.push(unsafe {
                device_manager
                    .device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| GraphicsContextError::CreateSemaphoresFailed(e.to_string()))?
            });

            self.frame_completion_fences.push(unsafe {
                device_manager
                    .device
                    .create_fence(
                        &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                        None,
                    )
                    .map_err(|e| GraphicsContextError::CreateFencesFailed(e.to_string()))?
            });
        }

        self.window = Some(window);
        self.instance_manager = Some(instance_manager);
        self.surface_manager = Some(surface_manager);
        self.physical_device_manager = Some(physical_device_manager);
        self.device_manager = Some(device_manager);
        self.command_pool = Some(command_pool);

        self.create_swapchain()?;

        for _ in 0..self.swapchain_manager.as_ref().unwrap().amount_of_images {
            self.render_finished_semaphores.push(unsafe {
                self.device_manager
                    .as_ref()
                    .unwrap()
                    .device
                    .create_semaphore(&semaphore_info, None)
                    .map_err(|e| GraphicsContextError::CreateSemaphoresFailed(e.to_string()))?
            });

            self.image_in_flight_fences.push(vk::Fence::null());
        }

        Ok(())
    }

    fn create_swapchain(&mut self) -> Result<(), GraphicsContextError> {
        if let (
            Some(window),
            Some(instance_manager),
            Some(surface_manager),
            Some(physical_device_manager),
            Some(device_manager),
        ) = (
            &self.window,
            &self.instance_manager,
            &self.surface_manager,
            &self.physical_device_manager,
            &self.device_manager,
        ) {
            let swapchain_manager = SwapchainManager::new(
                window,
                &instance_manager.instance,
                &surface_manager.surface_loader,
                surface_manager.surface,
                physical_device_manager.physical_device,
                &physical_device_manager.queue_family_indices,
                &device_manager.device,
            )?;

            for &image in &swapchain_manager.swapchain_images {
                let create_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(swapchain_manager.preferred_surface_format)
                    .components(vk::ComponentMapping::default())
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });

                let image_view = unsafe {
                    device_manager
                        .device
                        .create_image_view(&create_info, None)
                        .map_err(|e| {
                            GraphicsContextError::CreateSwapchainImageViewsFailed(e.to_string())
                        })?
                };

                self.swapchain_image_views.push(image_view);
            }

            self.swapchain_manager = Some(swapchain_manager);
        }

        Ok(())
    }

    pub fn resize(&mut self, _window_size: WindowSize) {
        self.swapchain_needs_recreation = true;
    }

    pub fn destroy(&mut self) {
        unsafe {
            if let Some(device_manager) = &self.device_manager {
                device_manager
                    .device
                    .device_wait_idle()
                    .expect("Failed to wait for device idle");

                for &image_view in &self.swapchain_image_views {
                    device_manager.device.destroy_image_view(image_view, None);
                }

                for &semaphore in self.image_available_semaphores.iter() {
                    device_manager.device.destroy_semaphore(semaphore, None);
                }

                for &semaphore in self.render_finished_semaphores.iter() {
                    device_manager.device.destroy_semaphore(semaphore, None);
                }

                for &fence in self.frame_completion_fences.iter() {
                    device_manager.device.destroy_fence(fence, None);
                }

                // image_in_flight_fences contains copies of frame_completion_fences, no need to double destroy.

                if let Some(command_pool) = self.command_pool {
                    device_manager
                        .device
                        .destroy_command_pool(command_pool, None);
                }
            }
        }

        self.swapchain_manager = None;
        self.device_manager = None;
        self.physical_device_manager = None;
        self.surface_manager = None;
        self.instance_manager = None;
    }

    fn recreate_swapchain(&mut self) -> Result<(), GraphicsContextError> {
        if let Some(device_manager) = &self.device_manager {
            unsafe {
                device_manager
                    .device
                    .device_wait_idle()
                    .map_err(|e| GraphicsContextError::DeviceWaitIdleFailed(e.to_string()))?;

                for &image_view in &self.swapchain_image_views {
                    device_manager.device.destroy_image_view(image_view, None);
                }
            };

            self.swapchain_image_views.clear();
            self.swapchain_manager = None;

            self.create_swapchain()?;
        }
        Ok(())
    }

    pub fn redraw<R: Renderer<RI>, RI>(
        &mut self,
        renderer: &R,
        render_items: &[RI],
    ) -> Result<bool, GraphicsContextError> {
        let mut renderer_needs_recreation = false;

        if let (Some(device_manager), Some(swapchain_manager)) =
            (&self.device_manager, &self.swapchain_manager)
        {
            unsafe {
                device_manager
                    .device
                    .wait_for_fences(
                        &[self.frame_completion_fences[self.current_frame]],
                        true,
                        std::u64::MAX,
                    )
                    .map_err(|e| GraphicsContextError::WaitForFencesFailed(e.to_string()))?
            };

            let (image_index, _) = unsafe {
                swapchain_manager
                    .swapchain_loader
                    .acquire_next_image(
                        swapchain_manager.swapchain,
                        u64::MAX,
                        self.image_available_semaphores[self.current_frame],
                        vk::Fence::null(), // Not using a fence here
                    )
                    .map_err(|e| GraphicsContextError::AcquireNextImageFailed(e.to_string()))?
            };

            // Check if a previous frame is using this image (i.e. there is its fence to wait on)
            if self.image_in_flight_fences[image_index as usize] != vk::Fence::null() {
                unsafe {
                    device_manager
                        .device
                        .wait_for_fences(
                            &[self.image_in_flight_fences[image_index as usize]],
                            true,
                            std::u64::MAX,
                        )
                        .map_err(|e| GraphicsContextError::WaitForFencesFailed(e.to_string()))?
                };
            }

            // Mark the image as now being in use by this frame
            self.image_in_flight_fences[image_index as usize] =
                self.frame_completion_fences[self.current_frame];

            let command_buffer = self.command_buffers[self.current_frame];

            unsafe {
                device_manager
                    .device
                    .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                    .map_err(|e| GraphicsContextError::ResetCommandBufferFailed(e.to_string()))?;
            }

            unsafe {
                device_manager
                    .device
                    .begin_command_buffer(command_buffer, &vk::CommandBufferBeginInfo::default())
                    .map_err(|e| GraphicsContextError::BeginCommandBufferFailed(e.to_string()))?;
            }

            renderer.render(
                &FrameContext {
                    command_buffer,
                    image_index: image_index as usize,
                },
                render_items,
            );

            unsafe {
                device_manager
                    .device
                    .end_command_buffer(command_buffer)
                    .map_err(|e| GraphicsContextError::EndCommandBufferFailed(e.to_string()))?;
            }

            let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
            let signal_semaphores = [self.render_finished_semaphores[image_index as usize]];
            let wait_dst_stage_mask = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

            let submit_infos = [vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_dst_stage_mask)
                .command_buffers(std::slice::from_ref(&command_buffer))
                .signal_semaphores(&signal_semaphores)];

            unsafe {
                device_manager
                    .device
                    .reset_fences(&[self.frame_completion_fences[self.current_frame]])
                    .map_err(|e| GraphicsContextError::ResetFencesFailed(e.to_string()))?;

                device_manager
                    .device
                    .queue_submit(
                        device_manager.graphics_queue,
                        &submit_infos,
                        self.frame_completion_fences[self.current_frame],
                    )
                    .map_err(|e| GraphicsContextError::QueueSubmitFailed(e.to_string()))?;
            }

            let swapchains = [swapchain_manager.swapchain];
            let indices = [image_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&indices);

            let present_result = unsafe {
                swapchain_manager
                    .swapchain_loader
                    .queue_present(device_manager.present_queue, &present_info)
            };

            match present_result {
                Ok(is_suboptimal) if is_suboptimal => self.swapchain_needs_recreation = true,
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => self.swapchain_needs_recreation = true,
                Err(e) => return Err(GraphicsContextError::QueuePresentFailed(e.to_string())),
                _ => {}
            }

            if self.swapchain_needs_recreation {
                self.swapchain_needs_recreation = false;
                self.recreate_swapchain()?;
                renderer_needs_recreation = true;
            }

            self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        }

        Ok(renderer_needs_recreation)
    }
}

impl Drop for GraphicsContext {
    fn drop(&mut self) {
        self.destroy();
    }
}
