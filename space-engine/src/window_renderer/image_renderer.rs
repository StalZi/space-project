use std::{path::Path, sync::Arc};

use anyhow::Result;

use ash::vk;

use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::AllocationScheme;

use crate::context::{EngineImage, EngineImageAttrubutes, ImageLayoutState, VulkanContext};
use crate::window_renderer::renderer::Renderer;

pub struct ImageRenderer {
    context: Arc<VulkanContext>,
    renderer: Renderer,

    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,

    clear_color: vk::ClearColorValue,
    fence: vk::Fence,

    image: EngineImage,
}

impl ImageRenderer {
    pub fn new(
        context: Arc<VulkanContext>,
        format: vk::Format,
        resolution: (u32, u32),
        clear_color: vk::ClearColorValue,
    ) -> Result<Self> {
        unsafe {
            let command_pool = context.logical_device.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .queue_family_index(context.queue_families.graphics.index),
                None,
            )?;
            let command_buffer = context.logical_device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(command_pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(1),
            )?[0];

            let mut renderer = Renderer::new(context.clone(), resolution, format, 1)?;

            let fence = context
                .logical_device
                .create_fence(&vk::FenceCreateInfo::default(), None)?;

            let image = context.create_image(
                "image",
                &mut renderer.allocator,
                EngineImageAttrubutes {
                    format,
                    extent: vk::Extent3D {
                        width: resolution.0,
                        height: resolution.1,
                        depth: 1,
                    },
                    usage_flags: vk::ImageUsageFlags::TRANSFER_DST,
                    memory_location: MemoryLocation::GpuToCpu,
                    is_linear: true,
                    allocation_scheme: AllocationScheme::GpuAllocatorManaged,
                },
            )?;

            Ok(Self {
                fence,
                context,
                renderer,
                command_pool,
                command_buffer,
                clear_color,
                image,
            })
        }
    }

    pub fn render(&mut self) -> Result<()> {
        unsafe {
            self.context
                .logical_device
                .reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty())?;

            let command_buffer_begin_info = vk::CommandBufferBeginInfo::default();
            self.context
                .logical_device
                .begin_command_buffer(self.command_buffer, &command_buffer_begin_info)?;

            self.renderer.render(
                self.command_buffer,
                self.clear_color,
                0, // render target index
            )?;

            // Transition the image to TRANSFER_DST_OPTIMAL
            self.context.transition_image_layout(
                self.command_buffer,
                self.image.handle,
                ImageLayoutState::UNDEFINED,
                ImageLayoutState::TRANSFER_DST,
            );

            // Transition the render target image to transfer src layout
            self.context.transition_image_layout(
                self.command_buffer,
                self.renderer.render_targets[0].handle,
                self.renderer.render_targets[0].layout,
                ImageLayoutState::TRANSFER_SRC,
            );

            // Copy the render target image to the destination image
            self.context.blit_image(
                self.command_buffer,
                self.renderer.render_targets[0].handle,
                self.image.handle,
                self.image.attributes.extent,
                self.image.attributes.extent,
            );

            self.context
                .logical_device
                .end_command_buffer(self.command_buffer)?;

            self.context.logical_device.queue_submit2(
                self.context.queues[self.context.queue_families.graphics.index as usize],
                &[vk::SubmitInfo2KHR::default().command_buffer_infos(&[
                    vk::CommandBufferSubmitInfoKHR::default()
                        .command_buffer(self.command_buffer)
                        .device_mask(1),
                ])],
                self.fence,
            )?;

            self.context
                .logical_device
                .wait_for_fences(&[self.fence], true, u64::MAX)?;
            self.context.logical_device.reset_fences(&[self.fence])?;

            let data = self
                .image
                .allocation
                .as_ref()
                .unwrap()
                .mapped_slice()
                .unwrap();

            image::save_buffer(
                Path::new("output.png"),
                data,
                self.image.attributes.extent.width,
                self.image.attributes.extent.height,
                image::ColorType::Rgba8,
            )?;
        }

        Ok(())
    }
}
