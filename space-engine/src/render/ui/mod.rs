use crate::logger::{LogLevel, Logger};

pub mod objects;
use std::sync::Arc;

use anyhow::Result;
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use objects::UIObject;

use crate::resources::shader::load_shader_module;
use crate::utils::image_utils::EngineImage;
use crate::vulkan::VulkanContext;
use crate::vulkan::buffer::{create_buffer, ensure_buffer_capacity, required_buffer_size};
use crate::vulkan::descriptor::{
    allocate_descriptor_set, create_descriptor_pool, create_descriptor_set_layout,
    write_storage_buffer_descriptor,
};
use crate::vulkan::pipeline::create_graphics_pipeline;

const SHADERS_DIR: &str = "res/shaders/ui/compiled";

#[repr(C)]
#[derive(Copy, Clone)]
struct UIObjectData {
    position_size: [f32; 4], // x, y, width, height
    color: [f32; 4],         // r, g, b, a
}

pub struct UIRenderer {
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
    buffer: vk::Buffer,
    buffer_allocation: gpu_allocator::vulkan::Allocation,
    buffer_size: vk::DeviceSize,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    context: Arc<VulkanContext>,
    logger: &'static Logger,
}

impl UIRenderer {
    pub fn new(
        context: Arc<VulkanContext>,
        resolution: (u32, u32),
        format: vk::Format,
        allocator: &mut Allocator,
        initial_ui_object_capacity: usize,
    ) -> Result<Self> {
        let logger = Logger::get_logger();

        logger.log("Loading ui shaders", LogLevel::Info);
        let vertex_shader = load_shader_module(&context.device.handle, SHADERS_DIR, "vert.spv")
            .unwrap_or_else(|_| {
                logger.log(
                    format!(
                        "Failed to load vertex shader for UI renderer from {}/vert.spv.",
                        SHADERS_DIR
                    ),
                    LogLevel::Error,
                );
                panic!(
                    "Failed to load vertex shader for UI renderer from {}",
                    SHADERS_DIR
                )
            });
        let fragment_shader = load_shader_module(&context.device.handle, SHADERS_DIR, "frag.spv")
            .unwrap_or_else(|_| {
                logger.log(
                    format!(
                        "Failed to load fragment shader for UI renderer from {}/frag.spv.",
                        SHADERS_DIR
                    ),
                    LogLevel::Error,
                );
                panic!(
                    "Failed to load fragment shader for UI renderer from {}",
                    SHADERS_DIR
                )
            });

        // Descriptor set layout for storage buffer
        let descriptor_set_layout_bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)];
        let descriptor_set_layout =
            create_descriptor_set_layout(&context.device.handle, &descriptor_set_layout_bindings)?;

        // Pipeline layout with push constants and descriptor set
        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(8); // vec2 viewport_size

        let pipeline_layout = unsafe {
            context.device.handle.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .set_layouts(&[descriptor_set_layout])
                    .push_constant_ranges(&[push_constant_range]),
                None,
            )
        }?;

        let pipeline = create_graphics_pipeline(
            &context.device.handle,
            vertex_shader,
            fragment_shader,
            vk::Extent2D {
                width: resolution.0,
                height: resolution.1,
            },
            format,
            None,
            pipeline_layout,
            Default::default(),
            vk::PrimitiveTopology::TRIANGLE_STRIP,
            true,
        )?;

        logger.log("Destroying ui shader modules", LogLevel::Info);
        unsafe {
            context
                .device
                .handle
                .destroy_shader_module(vertex_shader, None);
            context
                .device
                .handle
                .destroy_shader_module(fragment_shader, None);
        };

        // Descriptor pool
        let descriptor_pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)];
        let descriptor_pool =
            create_descriptor_pool(&context.device.handle, &descriptor_pool_sizes, 1)?;

        // Allocate descriptor set
        let descriptor_set = allocate_descriptor_set(
            &context.device.handle,
            descriptor_pool,
            &[descriptor_set_layout],
        )?;

        // Create buffer for UI objects data with the specified initial capacity.
        let buffer_size = required_buffer_size::<UIObjectData>(initial_ui_object_capacity);
        let (buffer, allocation) = create_buffer(
            &context.device.handle,
            allocator,
            buffer_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            gpu_allocator::MemoryLocation::CpuToGpu,
            "UI Objects Buffer",
        )?;

        // Update descriptor set
        write_storage_buffer_descriptor(
            &context.device.handle,
            descriptor_set,
            0,
            buffer,
            0,
            vk::WHOLE_SIZE,
        );

        Ok(Self {
            context,
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
            descriptor_pool,
            descriptor_set,
            buffer,
            buffer_allocation: allocation,
            buffer_size,
            logger,
        })
    }

    pub fn set_ui_capacity(&mut self, capacity: usize, allocator: &mut Allocator) -> Result<()> {
        ensure_buffer_capacity::<UIObjectData>(
            &self.context.device.handle,
            allocator,
            &mut self.buffer,
            &mut self.buffer_allocation,
            &mut self.buffer_size,
            capacity,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
            gpu_allocator::MemoryLocation::CpuToGpu,
            "UI Objects Buffer",
        )?;

        write_storage_buffer_descriptor(
            &self.context.device.handle,
            self.descriptor_set,
            0,
            self.buffer,
            0,
            vk::WHOLE_SIZE,
        );

        Ok(())
    }

    pub fn draw(
        &mut self,
        command_buffer: vk::CommandBuffer,
        render_target: &EngineImage,
        ui_objects: &[UIObject],
    ) -> Result<()> {
        let data: Vec<UIObjectData> = ui_objects
            .iter()
            .map(|obj| UIObjectData {
                position_size: [
                    obj.position.x,
                    obj.position.y,
                    obj.size.width as f32,
                    obj.size.height as f32,
                ],
                color: [
                    obj.bg_color.r as f32 / 255.0,
                    obj.bg_color.g as f32 / 255.0,
                    obj.bg_color.b as f32 / 255.0,
                    obj.bg_color.a as f32 / 255.0,
                ],
            })
            .collect();

        let mapped = self.buffer_allocation.mapped_ptr().unwrap().as_ptr() as *mut UIObjectData;
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), mapped, data.len());
        }

        unsafe {
            self.context.device.handle.cmd_set_viewport(
                command_buffer,
                0,
                &[vk::Viewport::default()
                    .width(render_target.attributes.extent.width as f32)
                    .height(render_target.attributes.extent.height as f32)],
            );

            self.context.device.handle.cmd_set_scissor(
                command_buffer,
                0,
                &[vk::Rect2D::default().extent(vk::Extent2D {
                    width: render_target.attributes.extent.width,
                    height: render_target.attributes.extent.height,
                })],
            );

            self.context.device.handle.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            self.context.device.handle.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.descriptor_set],
                &[],
            );

            let viewport_size = [
                render_target.attributes.extent.width as f32,
                render_target.attributes.extent.height as f32,
            ];
            let push_constants = [
                viewport_size[0].to_le_bytes(),
                viewport_size[1].to_le_bytes(),
            ]
            .concat();

            self.context.device.handle.cmd_push_constants(
                command_buffer,
                self.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                &push_constants,
            );

            self.context.device.handle.cmd_draw(
                command_buffer,
                4,                       // vertex count
                ui_objects.len() as u32, // instance count
                0,                       // first vertex
                0,                       // first instance
            );
        }

        Ok(())
    }
}

impl Drop for UIRenderer {
    fn drop(&mut self) {
        unsafe {
            self.context
                .device
                .handle
                .destroy_pipeline(self.pipeline, None);
            self.context
                .device
                .handle
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.context
                .device
                .handle
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.context
                .device
                .handle
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.context.device.handle.destroy_buffer(self.buffer, None);
        }
        // Allocation is dropped automatically
    }
}
