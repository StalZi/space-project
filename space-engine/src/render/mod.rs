use crate::logger::{LogLevel, Logger};
use crate::resources::ResourceManager;

mod dynamic_rendering;
use dynamic_rendering::begin_rendering;

use crate::utils::RenderingContext;

pub mod ui;
use ui::UIRenderer;

pub mod scene;
use std::sync::Arc;

use anyhow::Result;
use ash::{Device, vk};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{AllocationScheme, Allocator};
use scene::SceneRenderer;

use crate::utils::image_utils::{EngineImage, EngineImageAttributes, create_image, destroy_image};
use crate::vulkan::VulkanContext;

fn create_render_target(
    device: &Device,
    resolution: (u32, u32),
    format: vk::Format,
    allocator: &mut Allocator,
) -> Result<EngineImage> {
    let render_target = create_image(
        device,
        "render target",
        allocator,
        EngineImageAttributes {
            extent: vk::Extent3D::default()
                .width(resolution.0)
                .height(resolution.1)
                .depth(1),
            format,
            usage_flags: vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST,
            memory_location: MemoryLocation::GpuOnly,
            is_linear: false,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            aspect_flags: vk::ImageAspectFlags::COLOR,
        },
    )?;

    Ok(render_target)
}

fn create_depth_target(
    device: &Device,
    resolution: (u32, u32),
    allocator: &mut Allocator,
) -> Result<EngineImage> {
    let depth_target = create_image(
        device,
        "depth target",
        allocator,
        EngineImageAttributes {
            extent: vk::Extent3D::default()
                .width(resolution.0)
                .height(resolution.1)
                .depth(1),
            format: vk::Format::D32_SFLOAT,
            usage_flags: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            memory_location: MemoryLocation::GpuOnly,
            is_linear: false,
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
            aspect_flags: vk::ImageAspectFlags::DEPTH,
        },
    )?;

    Ok(depth_target)
}

pub struct GameRenderer {
    allocator: Allocator,
    scene_renderer: SceneRenderer,
    ui_renderer: UIRenderer,
    pub render_targets: Vec<EngineImage>,
    pub depth_targets: Vec<EngineImage>,
    context: Arc<VulkanContext>,
    logger: &'static Logger,
}

impl GameRenderer {
    pub fn new(
        context: Arc<VulkanContext>,
        mut allocator: Allocator,
        resolution: (u32, u32),
        format: vk::Format,
        buffering: usize,
        ui_object_capacity: usize,
    ) -> Result<Self> {
        let logger = Logger::get_logger();

        let render_targets = (0..buffering)
            .map(|_| {
                create_render_target(&context.device.handle, resolution, format, &mut allocator)
            })
            .collect::<Result<Vec<_>>>()?;

        let depth_targets = (0..buffering)
            .map(|_| create_depth_target(&context.device.handle, resolution, &mut allocator))
            .collect::<Result<Vec<_>>>()?;

        logger.log("Creating scene renderer", LogLevel::Info);
        let scene_renderer = SceneRenderer::new(context.clone(), resolution, format)?;

        logger.log("Creating UI renderer", LogLevel::Info);
        let ui_renderer = UIRenderer::new(
            context.clone(),
            resolution,
            format,
            &mut allocator,
            ui_object_capacity,
        )?;

        Ok(Self {
            context,
            allocator,
            scene_renderer,
            ui_renderer,
            render_targets,
            depth_targets,
            logger,
        })
    }

    pub fn resize(&mut self, new_resolution: (u32, u32)) -> Result<()> {
        for (render_target, depth_target) in self
            .render_targets
            .iter_mut()
            .zip(self.depth_targets.iter_mut())
        {
            destroy_image(
                &self.context.device.handle,
                &mut self.allocator,
                render_target,
            )?;
            *render_target = create_render_target(
                &self.context.device.handle,
                new_resolution,
                render_target.attributes.format,
                &mut self.allocator,
            )?;

            destroy_image(
                &self.context.device.handle,
                &mut self.allocator,
                depth_target,
            )?;
            *depth_target = create_depth_target(
                &self.context.device.handle,
                new_resolution,
                &mut self.allocator,
            )?;
        }
        Ok(())
    }

    pub fn set_ui_capacity(&mut self, capacity: usize) -> Result<()> {
        self.ui_renderer
            .set_ui_capacity(capacity, &mut self.allocator)
    }

    pub fn render(
        &mut self,
        command_buffer: vk::CommandBuffer,
        clear_color: vk::ClearColorValue,
        render_target_index: usize,
        context: RenderingContext,
        resource_manager: &mut ResourceManager,
    ) -> Result<()> {
        let render_target = &mut self.render_targets[render_target_index];

        begin_rendering(
            &self.context.device.handle,
            command_buffer,
            render_target,
            Some(&self.depth_targets[render_target_index]),
            clear_color,
            vk::Rect2D::default().extent(vk::Extent2D {
                width: render_target.attributes.extent.width,
                height: render_target.attributes.extent.height,
            }),
        );
        if let (Some(meshes_path), Some(mesh_states)) = (context.meshes_path, context.mesh_states) {
            let meshes_info =
                resource_manager.get_or_init_meshes(meshes_path, &mut self.allocator)?;
            self.scene_renderer.draw(
                command_buffer,
                render_target,
                meshes_info,
                mesh_states,
                context.camera,
            )?;
        }

        if let Some(ui_objects) = &context.ui_objects {
            self.ui_renderer
                .draw(command_buffer, render_target, ui_objects)?;
        }

        unsafe { self.context.device.handle.cmd_end_rendering(command_buffer) };

        Ok(())
    }
}

impl Drop for GameRenderer {
    fn drop(&mut self) {
        unsafe {
            self.context.device.handle.device_wait_idle().unwrap();

            for render_target in self.render_targets.iter_mut() {
                destroy_image(
                    &self.context.device.handle,
                    &mut self.allocator,
                    render_target,
                )
                .unwrap();
            }

            for depth_target in self.depth_targets.iter_mut() {
                destroy_image(
                    &self.context.device.handle,
                    &mut self.allocator,
                    depth_target,
                )
                .unwrap();
            }
        }
    }
}
