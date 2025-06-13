use ash::vk;
use eren_window::window::WindowSize;
use thiserror::Error;
use winit::window::Window;

use crate::{
    constants::MAX_FRAMES_IN_FLIGHT,
    vulkan::{
        instance::{VulkanInstanceManager, VulkanInstanceManagerError},
        logical_device::{LogicalDeviceManager, LogicalDeviceManagerError},
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
    CreateLogicalDeviceFailed(#[from] LogicalDeviceManagerError),

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
}

#[derive(Debug, Error)]
pub enum RedrawError {
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
}

#[derive(Debug)]
pub struct FrameContext {
    command_buffer: vk::CommandBuffer,
    swapchain_image_view: vk::ImageView,
}

pub struct GraphicsContext<F>
where
    F: Fn(&FrameContext),
{
    draw_frame: F,
    entry: ash::Entry,

    instance_manager: Option<VulkanInstanceManager>,
    surface_manager: Option<SurfaceManager>,
    physical_device_manager: Option<PhysicalDeviceManager>,
    logical_device_manager: Option<LogicalDeviceManager>,
    swapchain_manager: Option<SwapchainManager>,

    swapchain_image_views: Vec<vk::ImageView>,
    command_pool: Option<vk::CommandPool>,
    command_buffers: Vec<vk::CommandBuffer>,

    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    frame_completion_fences: Vec<vk::Fence>,
    image_in_flight_fences: Vec<vk::Fence>,

    current_frame: usize,
}

impl<F> GraphicsContext<F>
where
    F: Fn(&FrameContext),
{
    pub fn new(draw_frame: F) -> Result<Self, GraphicsContextError> {
        let entry = unsafe { ash::Entry::load()? };
        Ok(Self {
            draw_frame,
            entry,

            instance_manager: None,
            surface_manager: None,
            physical_device_manager: None,
            logical_device_manager: None,
            swapchain_manager: None,

            swapchain_image_views: Vec::new(),
            command_pool: None,
            command_buffers: Vec::new(),

            image_available_semaphores: Vec::new(),
            render_finished_semaphores: Vec::new(),
            frame_completion_fences: Vec::new(),
            image_in_flight_fences: Vec::new(),

            current_frame: 0,
        })
    }

    pub fn init(&mut self, window: &Window) -> Result<(), GraphicsContextError> {
        let instance_manager = VulkanInstanceManager::new(&self.entry, window)?;
        let surface_manager = SurfaceManager::new(&self.entry, &instance_manager.instance, window)?;
        let physical_device_manager = PhysicalDeviceManager::new(
            &instance_manager.instance,
            &surface_manager.surface_loader,
            surface_manager.surface,
        )?;
        let logical_device_manager = LogicalDeviceManager::new(
            &instance_manager.instance,
            physical_device_manager.physical_device,
            &physical_device_manager.queue_family_indices,
        )?;
        let swapchain_manager = SwapchainManager::new(
            window,
            &instance_manager.instance,
            surface_manager.surface,
            &physical_device_manager.queue_family_indices,
            &physical_device_manager.swapchain_support_details,
            &logical_device_manager.logical_device,
        )?;

        for image in &swapchain_manager.swapchain_images {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(*image)
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
                logical_device_manager
                    .logical_device
                    .create_image_view(&create_info, None)
                    .map_err(|e| {
                        GraphicsContextError::CreateSwapchainImageViewsFailed(e.to_string())
                    })?
            };

            self.swapchain_image_views.push(image_view);
        }

        let command_pool = unsafe {
            logical_device_manager
                .logical_device
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
            logical_device_manager
                .logical_device
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::default()
                        .command_pool(command_pool)
                        .level(vk::CommandBufferLevel::PRIMARY)
                        .command_buffer_count(MAX_FRAMES_IN_FLIGHT as u32),
                )
                .map_err(|e| GraphicsContextError::CreateCommandBuffersFailed(e.to_string()))?
        };

        let semaphore_create_info = vk::SemaphoreCreateInfo::default();

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            self.image_available_semaphores.push(unsafe {
                logical_device_manager
                    .logical_device
                    .create_semaphore(&semaphore_create_info, None)
                    .map_err(|e| GraphicsContextError::CreateSemaphoresFailed(e.to_string()))?
            });

            self.render_finished_semaphores.push(unsafe {
                logical_device_manager
                    .logical_device
                    .create_semaphore(&semaphore_create_info, None)
                    .map_err(|e| GraphicsContextError::CreateSemaphoresFailed(e.to_string()))?
            });

            self.frame_completion_fences.push(unsafe {
                logical_device_manager
                    .logical_device
                    .create_fence(
                        &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                        None,
                    )
                    .map_err(|e| GraphicsContextError::CreateFencesFailed(e.to_string()))?
            });
        }

        // image_usage_fences needs to be sized to swapchain image count, not MAX_FRAMES_IN_FLIGHT
        // and initialized to vk::Fence::null()
        self.image_in_flight_fences = vec![vk::Fence::null(); swapchain_manager.amount_of_images];

        self.instance_manager = Some(instance_manager);
        self.surface_manager = Some(surface_manager);
        self.physical_device_manager = Some(physical_device_manager);
        self.logical_device_manager = Some(logical_device_manager);
        self.swapchain_manager = Some(swapchain_manager);

        self.command_pool = Some(command_pool);

        Ok(())
    }

    pub fn resize(&mut self, window_size: WindowSize) {
        //TODO:
        println!("Resizing not implemented");
    }

    pub fn destroy(&mut self) {
        self.instance_manager = None;
        self.surface_manager = None;
        self.physical_device_manager = None;
        self.logical_device_manager = None;
        self.swapchain_manager = None;
    }

    pub fn redraw(&mut self) -> Result<(), RedrawError> {
        if let (Some(logical_device_manager), Some(swapchain_manager)) =
            (&self.logical_device_manager, &self.swapchain_manager)
        {
            let (image_index, _) = unsafe {
                swapchain_manager
                    .swapchain_loader
                    .acquire_next_image(
                        swapchain_manager.swapchain,
                        u64::MAX,
                        self.image_available_semaphores[self.current_frame],
                        vk::Fence::null(), // Not using a fence here
                    )
                    .map_err(|e| RedrawError::AcquireNextImageFailed(e.to_string()))?
            };

            unsafe {
                logical_device_manager
                    .logical_device
                    .wait_for_fences(
                        &[self.frame_completion_fences[self.current_frame]],
                        true,
                        std::u64::MAX,
                    )
                    .map_err(|e| RedrawError::WaitForFencesFailed(e.to_string()))?
            };

            // Check if a previous frame is using this image (i.e. there is its fence to wait on)
            if self.image_in_flight_fences[image_index as usize] != vk::Fence::null() {
                unsafe {
                    logical_device_manager
                        .logical_device
                        .wait_for_fences(
                            &[self.image_in_flight_fences[image_index as usize]],
                            true,
                            std::u64::MAX,
                        )
                        .map_err(|e| RedrawError::WaitForFencesFailed(e.to_string()))?
                };
            }

            // Mark the image as now being in use by this frame
            self.image_in_flight_fences[image_index as usize] =
                self.frame_completion_fences[self.current_frame];

            let command_buffer = self.command_buffers[self.current_frame];

            unsafe {
                logical_device_manager
                    .logical_device
                    .reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
                    .map_err(|e| RedrawError::ResetCommandBufferFailed(e.to_string()))?;
            }

            unsafe {
                logical_device_manager
                    .logical_device
                    .begin_command_buffer(command_buffer, &vk::CommandBufferBeginInfo::default())
                    .map_err(|e| RedrawError::BeginCommandBufferFailed(e.to_string()))?;
            }

            (self.draw_frame)(&FrameContext {
                command_buffer,
                swapchain_image_view: self.swapchain_image_views[image_index as usize],
            });

            unsafe {
                logical_device_manager
                    .logical_device
                    .end_command_buffer(command_buffer)
                    .map_err(|e| RedrawError::EndCommandBufferFailed(e.to_string()))?;
            }

            let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
            let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
            let wait_dst_stage_mask = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

            let submit_infos = [vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_dst_stage_mask)
                .command_buffers(std::slice::from_ref(&command_buffer))
                .signal_semaphores(&signal_semaphores)];

            unsafe {
                logical_device_manager
                    .logical_device
                    .reset_fences(&[self.frame_completion_fences[self.current_frame]])
                    .map_err(|e| RedrawError::ResetFencesFailed(e.to_string()))?;
            }

            unsafe {
                logical_device_manager
                    .logical_device
                    .queue_submit(
                        logical_device_manager.graphics_queue,
                        &submit_infos,
                        self.frame_completion_fences[self.current_frame],
                    )
                    .map_err(|e| RedrawError::QueueSubmitFailed(e.to_string()))?;
            }

            let swapchains = [swapchain_manager.swapchain];
            let indices = [image_index];
            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&indices);

            unsafe {
                swapchain_manager
                    .swapchain_loader
                    .queue_present(logical_device_manager.present_queue, &present_info)
                    .map_err(|e| RedrawError::QueuePresentFailed(e.to_string()))?;
            }

            self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        }

        Ok(())
    }
}

impl<F> Drop for GraphicsContext<F>
where
    F: Fn(&FrameContext),
{
    fn drop(&mut self) {
        if let Some(logical_device_manager) = &self.logical_device_manager {
            for image_view in &self.swapchain_image_views {
                unsafe {
                    logical_device_manager
                        .logical_device
                        .destroy_image_view(*image_view, None);
                }
            }

            unsafe {
                logical_device_manager
                    .logical_device
                    .device_wait_idle()
                    .expect("Failed to wait for device idle")
            };

            for &semaphore in self.image_available_semaphores.iter() {
                unsafe {
                    logical_device_manager
                        .logical_device
                        .destroy_semaphore(semaphore, None)
                };
            }

            for &semaphore in self.render_finished_semaphores.iter() {
                unsafe {
                    logical_device_manager
                        .logical_device
                        .destroy_semaphore(semaphore, None)
                };
            }

            for &fence in self.frame_completion_fences.iter() {
                unsafe {
                    logical_device_manager
                        .logical_device
                        .destroy_fence(fence, None)
                };
            }

            // image_in_flight_fences contains copies of frame_completion_fences, no need to double destroy.

            if let Some(command_pool) = self.command_pool {
                unsafe {
                    logical_device_manager
                        .logical_device
                        .destroy_command_pool(command_pool, None)
                };
            }
        }
    }
}
