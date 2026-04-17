use ash::{Device, vk};

use crate::utils::image_utils::{EngineImage, ImageLayoutState, transition_image_layout};

pub fn begin_rendering(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    render_target: &EngineImage,
    depth_target: Option<&EngineImage>,
    clear_color: vk::ClearColorValue,
    render_area: vk::Rect2D,
) {
    unsafe {
        transition_image_layout(
            device,
            command_buffer,
            render_target.handle,
            ImageLayoutState::UNDEFINED,
            ImageLayoutState::RENDERABLE,
        );

        if let Some(depth_target) = depth_target {
            transition_image_layout(
                device,
                command_buffer,
                depth_target.handle,
                ImageLayoutState::UNDEFINED,
                ImageLayoutState::DEPTH_RENDERABLE,
            );
        }

        let color_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(render_target.view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .clear_value(vk::ClearValue { color: clear_color })
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE);

        let color_attachments = [color_attachment];
        let mut rendering_info = vk::RenderingInfo::default()
            .layer_count(1)
            .color_attachments(&color_attachments)
            .render_area(render_area);

        let depth_attachment_storage;
        if let Some(depth_target) = depth_target {
            depth_attachment_storage = Some(
                vk::RenderingAttachmentInfo::default()
                    .image_view(depth_target.view)
                    .image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .clear_value(vk::ClearValue {
                        depth_stencil: vk::ClearDepthStencilValue {
                            depth: 1.0,
                            stencil: 0,
                        },
                    })
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::STORE),
            );

            rendering_info =
                rendering_info.depth_attachment(depth_attachment_storage.as_ref().unwrap());
        }

        device.cmd_begin_rendering(command_buffer, &rendering_info)
    };
}
