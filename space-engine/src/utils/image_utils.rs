use anyhow::Result;
use ash::{Device, vk};
use gpu_allocator::{
    MemoryLocation,
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator},
};

pub struct EngineImageAttributes {
    pub memory_location: MemoryLocation,
    pub is_linear: bool,
    pub allocation_scheme: AllocationScheme,
    pub extent: vk::Extent3D,
    pub format: vk::Format,
    pub usage_flags: vk::ImageUsageFlags,
    pub aspect_flags: vk::ImageAspectFlags,
}

pub struct EngineImage {
    pub handle: vk::Image,
    pub allocation: Option<Allocation>,
    pub view: vk::ImageView,
    pub attributes: EngineImageAttributes,
}

#[derive(Copy, Clone)]
pub struct ImageLayoutState {
    pub access_mask: vk::AccessFlags2,
    pub layout: vk::ImageLayout,
    pub stage_mask: vk::PipelineStageFlags2,
    pub queue_family_index: u32,
}

impl ImageLayoutState {
    // Undefined State
    pub const UNDEFINED: Self = Self {
        access_mask: vk::AccessFlags2::NONE,
        layout: vk::ImageLayout::UNDEFINED,
        stage_mask: vk::PipelineStageFlags2::ALL_COMMANDS,
        queue_family_index: vk::QUEUE_FAMILY_IGNORED,
    };

    // Renderable State
    pub const RENDERABLE: Self = Self {
        access_mask: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        queue_family_index: vk::QUEUE_FAMILY_IGNORED,
    };

    // Depth renderable state
    pub const DEPTH_RENDERABLE: Self = Self {
        access_mask: vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        stage_mask: vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS,
        queue_family_index: vk::QUEUE_FAMILY_IGNORED,
    };

    // Present State
    pub const PRESENT: Self = Self {
        access_mask: vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
        layout: vk::ImageLayout::PRESENT_SRC_KHR,
        stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        queue_family_index: vk::QUEUE_FAMILY_IGNORED,
    };

    pub const TRANSFER_SRC: Self = Self {
        access_mask: vk::AccessFlags2::TRANSFER_READ,
        layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        stage_mask: vk::PipelineStageFlags2::TRANSFER,
        queue_family_index: vk::QUEUE_FAMILY_IGNORED,
    };

    pub const TRANSFER_DST: Self = Self {
        access_mask: vk::AccessFlags2::TRANSFER_WRITE,
        layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        stage_mask: vk::PipelineStageFlags2::TRANSFER,
        queue_family_index: vk::QUEUE_FAMILY_IGNORED,
    };
}

pub fn create_image(
    device: &Device,
    name: &str,
    allocator: &mut Allocator,
    attributes: EngineImageAttributes,
) -> Result<EngineImage> {
    let image = unsafe {
        device.create_image(
            &vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .format(attributes.format)
                .extent(attributes.extent)
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(attributes.usage_flags)
                .sharing_mode(vk::SharingMode::EXCLUSIVE),
            None,
        )?
    };
    let requirements = unsafe { device.get_image_memory_requirements(image) };

    let allocation = allocator.allocate(&AllocationCreateDesc {
        name,
        requirements,
        location: MemoryLocation::GpuOnly,
        linear: attributes.is_linear,
        allocation_scheme: attributes.allocation_scheme,
    })?;

    (unsafe { device.bind_image_memory(image, allocation.memory(), allocation.offset()) })?;

    let view = create_image_view(
        device,
        image,
        attributes.format,
        attributes.aspect_flags,
    )?;

    Ok(EngineImage {
        handle: image,
        allocation: Some(allocation),
        view,
        attributes,
    })
}

pub fn create_image_view(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
    aspect_flags: vk::ImageAspectFlags,
) -> Result<vk::ImageView> {
    let create_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(
            vk::ImageSubresourceRange::default()
                .aspect_mask(aspect_flags)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1),
        );

    let image_view = unsafe {
        device
            .create_image_view(&create_info, None)
            .expect("Failed to create image view")
    };

    Ok(image_view)
}

pub fn transition_image_layout(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    image: vk::Image,
    old_layout: ImageLayoutState,
    new_layout: ImageLayoutState,
) {
    unsafe {
        let aspect_mask = if new_layout.layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        {
            vk::ImageAspectFlags::DEPTH
        } else {
            vk::ImageAspectFlags::COLOR
        };

        device.cmd_pipeline_barrier2(
            command_buffer,
            &vk::DependencyInfo::default().image_memory_barriers(&[
                vk::ImageMemoryBarrier2::default()
                    .src_stage_mask(old_layout.stage_mask)
                    .dst_stage_mask(new_layout.stage_mask)
                    .src_access_mask(old_layout.access_mask)
                    .dst_access_mask(new_layout.access_mask)
                    .old_layout(old_layout.layout)
                    .new_layout(new_layout.layout)
                    .src_queue_family_index(old_layout.queue_family_index)
                    .dst_queue_family_index(new_layout.queue_family_index)
                    .image(image)
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(aspect_mask)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1),
                    ),
            ]),
        );
    }
}

pub fn blit_image(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    src_image: vk::Image,
    dst_image: vk::Image,
    src_extent: vk::Extent3D,
    dst_extent: vk::Extent3D,
) {
    let subresource = vk::ImageSubresourceLayers::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_array_layer(0)
        .layer_count(1);

    unsafe {
        device.cmd_blit_image(
            command_buffer,
            src_image,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            dst_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[vk::ImageBlit::default()
                .src_subresource(subresource)
                .dst_subresource(subresource)
                .src_offsets([
                    vk::Offset3D::default(),
                    vk::Offset3D {
                        x: src_extent.width as i32,
                        y: src_extent.height as i32,
                        z: src_extent.depth as i32,
                    },
                ])
                .dst_offsets([
                    vk::Offset3D::default(),
                    vk::Offset3D {
                        x: dst_extent.width as i32,
                        y: dst_extent.height as i32,
                        z: dst_extent.depth as i32,
                    },
                ])],
            vk::Filter::NEAREST,
        );
    }
}

pub fn destroy_image(
    device: &Device,
    allocator: &mut Allocator,
    image: &mut EngineImage,
) -> Result<()> {
    unsafe {
        device.destroy_image_view(image.view, None);
        if let Some(allocation) = image.allocation.take() {
            allocator.free(allocation)?;
        }
        device.destroy_image(image.handle, None);
    }
    Ok(())
}
