use std::sync::Arc;

use anyhow::Result;
use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use gpu_allocator::{AllocationSizes, AllocatorDebugSettings};
use winit::window::Window;

use crate::render::GameRenderer;
use crate::render::context::RenderingContext;
use crate::utils::image_utils::{ImageLayoutState, blit_image, transition_image_layout};
use crate::vulkan::VulkanContext;
use crate::vulkan::swapchain::EngineSwapchain;

pub struct WindowRenderer {
    in_flight_frames_count: usize,
    frame_index: usize,
    context: Arc<VulkanContext>,
    clear_color: vk::ClearColorValue,
    pub renderer: GameRenderer,
    swapchain: EngineSwapchain,
    pub window: Arc<Window>,
}

impl WindowRenderer {
    pub fn new(
        context: Arc<VulkanContext>,
        window: Arc<Window>,
        in_flight_frames_count: usize,
        format: vk::Format,
        clear_color: vk::ClearColorValue,
        initial_ui_capacity: usize,
    ) -> Result<Self> {
        let mut swapchain = EngineSwapchain::new(context.clone(), window.clone())
            .expect("Failed to create swapchain for renderer");
        swapchain
            .resize(window.inner_size())
            .expect("Failed to fill the swapchain");

        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: context.instance.clone(),
            device: context.device.handle.clone(),
            physical_device: context.phys_device.handle,
            buffer_device_address: true,
            debug_settings: AllocatorDebugSettings::default(),
            allocation_sizes: AllocationSizes::default(),
        })?;

        let renderer = GameRenderer::new(
            context.clone(),
            allocator,
            (swapchain.extent.width, swapchain.extent.height),
            format,
            in_flight_frames_count,
            initial_ui_capacity,
        )?;

        Ok(Self {
            renderer,
            frame_index: 0,
            window,
            swapchain,
            context,
            in_flight_frames_count,
            clear_color,
        })
    }

    pub fn resize(&mut self) -> Result<()> {
        self.swapchain.is_dirty = true;
        Ok(())
    }

    pub fn render(&mut self, rendering_context: &RenderingContext) -> Result<()> {
        unsafe {
            self.context.device.handle.wait_for_fences(
                &[self.swapchain.images[self.frame_index].in_flight_fence],
                true,
                u64::MAX,
            )?;
            if self.swapchain.is_dirty {
                self.context.device.handle.device_wait_idle()?;
                self.swapchain.resize(self.window.inner_size())?;
                if self.swapchain.extent.width == 0 || self.swapchain.extent.height == 0 {
                    return Ok(());
                }
                self.renderer
                    .resize((self.swapchain.extent.width, self.swapchain.extent.height))?;
            };

            if self.swapchain.extent.width == 0 || self.swapchain.extent.height == 0 {
                return Ok(());
            }

            let image_index = match self.swapchain.acquire_next_image(
                self.swapchain.images[self.frame_index].image_available_semaphore,
            ) {
                Ok(index) => index,
                Err(_) => {
                    self.swapchain.is_dirty = true;
                    return Ok(());
                }
            };

            let command_buffer = &self.swapchain.images[self.frame_index].command_buffer;

            self.context
                .device
                .handle
                .reset_fences(&[self.swapchain.images[self.frame_index].in_flight_fence])?;

            self.context
                .device
                .handle
                .reset_command_buffer(*command_buffer, vk::CommandBufferResetFlags::empty())?;
            self.context.device.handle.begin_command_buffer(
                *command_buffer,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )?;

            self.renderer.render(
                *command_buffer,
                self.clear_color,
                self.frame_index,
                rendering_context,
            )?;

            let render_target = &self.renderer.render_targets[self.frame_index];

            // Transition the swapchain image to transfer dst layout
            transition_image_layout(
                &self.context.device.handle,
                *command_buffer,
                self.swapchain.images[image_index as usize].handle,
                ImageLayoutState::UNDEFINED,
                ImageLayoutState::TRANSFER_DST,
            );

            // Transition the render target image to transfer src layout
            transition_image_layout(
                &self.context.device.handle,
                *command_buffer,
                render_target.handle,
                ImageLayoutState::RENDERABLE,
                ImageLayoutState::TRANSFER_SRC,
            );

            // Copy the render target image to the swapchain image
            blit_image(
                &self.context.device.handle,
                *command_buffer,
                render_target.handle,
                self.swapchain.images[image_index as usize].handle,
                render_target.attributes.extent,
                vk::Extent3D::default()
                    .width(self.swapchain.extent.width)
                    .height(self.swapchain.extent.height)
                    .depth(1),
            );

            // Transition the swapchain image to present layout
            transition_image_layout(
                &self.context.device.handle,
                self.swapchain.images[self.frame_index].command_buffer,
                self.swapchain.images[image_index as usize].handle,
                ImageLayoutState::TRANSFER_DST,
                ImageLayoutState::PRESENT,
            );

            self.context
                .device
                .handle
                .end_command_buffer(*command_buffer)?;
            // submit the command buffer
            self.context.device.handle.queue_submit2(
                self.context.device.queues[self.context.device.qfs.graphics.index as usize],
                &[vk::SubmitInfo2KHR::default()
                    .command_buffer_infos(&[vk::CommandBufferSubmitInfoKHR::default()
                        .command_buffer(*command_buffer)
                        .device_mask(1)])
                    .wait_semaphore_infos(&[vk::SemaphoreSubmitInfo::default()
                        .semaphore(
                            self.swapchain.images[self.frame_index].image_available_semaphore,
                        )
                        .stage_mask(vk::PipelineStageFlags2KHR::COLOR_ATTACHMENT_OUTPUT)])
                    .signal_semaphore_infos(&[vk::SemaphoreSubmitInfo::default()
                        .semaphore(
                            self.swapchain.images[image_index as usize].render_finished_semaphore,
                        )
                        .stage_mask(vk::PipelineStageFlags2KHR::COLOR_ATTACHMENT_OUTPUT)])],
                self.swapchain.images[self.frame_index].in_flight_fence,
            )?;

            self.swapchain.present(
                image_index,
                self.swapchain.images[image_index as usize].render_finished_semaphore,
            )?;

            self.frame_index = (self.frame_index + 1) % self.in_flight_frames_count;
            Ok(())
        }
    }
}
