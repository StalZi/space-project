use std::mem::size_of;
use std::sync::Arc;

use anyhow::Result;

use crate::core::camera::Camera;
use crate::logger::{LogLevel, Logger};
use crate::utils::math::IDENTITY4;

pub mod objects;
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use objects::Cube;

use crate::resources::shader::load_shader_module;
use crate::utils::image_utils::EngineImage;
use crate::vulkan::VulkanContext;
use crate::vulkan::pipeline::create_graphics_pipeline;

const SHADERS_DIR: &str = "res/shaders/scene/compiled";

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct ScenePushConstants {
    mvp: [[f32; 4]; 4],
    color: [f32; 4],
}

pub struct SceneRenderer {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    context: Arc<VulkanContext>,
    logger: &'static Logger,
}

impl SceneRenderer {
    pub fn new(
        context: Arc<VulkanContext>,
        resolution: (u32, u32),
        format: vk::Format,
        _allocator: &mut Allocator,
    ) -> Result<Self> {
        let logger = Logger::get_logger();

        logger.log("Loading scene shaders", LogLevel::Info);
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

        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(size_of::<ScenePushConstants>() as u32);

        let pipeline_layout = unsafe {
            context.device.handle.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
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
            Some(vk::Format::D32_SFLOAT),
            pipeline_layout,
            Default::default(),
            vk::PrimitiveTopology::TRIANGLE_LIST,
            false,
        )?;

        logger.log("Destroying scene shader modules", LogLevel::Info);
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

        Ok(Self {
            context,
            pipeline,
            pipeline_layout,
            logger,
        })
    }

    pub fn draw(
        &self,
        command_buffer: vk::CommandBuffer,
        render_target: &EngineImage,
        cube_objects: &[Cube],
        camera: Option<&Camera>,
    ) -> Result<()> {
        let aspect = render_target.attributes.extent.width as f32
            / render_target.attributes.extent.height as f32;
        let projection = perspective(90.0_f32.to_radians(), aspect, 0.1, 1000.0);

        let view = if let Some(camera) = camera {
            let translate_camera =
                translate(-camera.position.x, camera.position.y, -camera.position.z);
            let rx = rotate_x((camera.rotation.pitch).to_radians());
            let ry = rotate_y((-camera.rotation.yaw).to_radians());
            let rz = rotate_z((-camera.rotation.roll).to_radians());
            let rotation_camera = mul_mat4(rx, mul_mat4(ry, rz));
            mul_mat4(rotation_camera, translate_camera)
        } else {
            IDENTITY4
        };

        unsafe {
            self.context.device.handle.cmd_set_viewport(
                command_buffer,
                0,
                &[vk::Viewport::default()
                    .width(render_target.attributes.extent.width as f32)
                    .height(render_target.attributes.extent.height as f32)
                    .min_depth(0.0)
                    .max_depth(1.0)],
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

            for cube in cube_objects {
                let model = build_model_matrix(cube);
                let mvp = mul_mat4(projection, mul_mat4(view, model));
                let push_constants = ScenePushConstants {
                    mvp,
                    color: [
                        cube.color.r as f32 / 255.0,
                        cube.color.g as f32 / 255.0,
                        cube.color.b as f32 / 255.0,
                        cube.color.a as f32 / 255.0,
                    ],
                };

                self.context.device.handle.cmd_push_constants(
                    command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    std::slice::from_raw_parts(
                        (&push_constants as *const ScenePushConstants) as *const u8,
                        size_of::<ScenePushConstants>(),
                    ),
                );

                self.context
                    .device
                    .handle
                    .cmd_draw(command_buffer, 36, 1, 0, 0);
            }
        }
        Ok(())
    }
}

fn build_model_matrix(cube: &Cube) -> [[f32; 4]; 4] {
    let scale_matrix = scale(cube.size.x, cube.size.y, cube.size.z);
    let rx = rotate_x(cube.rotation.yaw.to_radians());
    let ry = rotate_y(cube.rotation.pitch.to_radians());
    let rz = rotate_z(cube.rotation.roll.to_radians());
    let rotation = mul_mat4(rz, mul_mat4(ry, rx));
    let translation = translate(cube.position.x, cube.position.y, cube.position.z);
    mul_mat4(translation, mul_mat4(rotation, scale_matrix))
}

fn translate(x: f32, y: f32, z: f32) -> [[f32; 4]; 4] {
    let mut m = IDENTITY4;
    m[0][3] = x;
    m[1][3] = y;
    m[2][3] = z;
    m
}

fn scale(x: f32, y: f32, z: f32) -> [[f32; 4]; 4] {
    let mut m = IDENTITY4;
    m[0][0] = x;
    m[1][1] = y;
    m[2][2] = z;
    m
}

fn rotate_x(angle: f32) -> [[f32; 4]; 4] {
    let mut m = IDENTITY4;
    let c = angle.cos();
    let s = angle.sin();
    m[1][1] = c;
    m[1][2] = s;
    m[2][1] = -s;
    m[2][2] = c;
    m
}

fn rotate_y(angle: f32) -> [[f32; 4]; 4] {
    let mut m = IDENTITY4;
    let c = angle.cos();
    let s = angle.sin();
    m[0][0] = c;
    m[0][2] = -s;
    m[2][0] = s;
    m[2][2] = c;
    m
}

fn rotate_z(angle: f32) -> [[f32; 4]; 4] {
    let mut m = IDENTITY4;
    let c = angle.cos();
    let s = angle.sin();
    m[0][0] = c;
    m[0][1] = s;
    m[1][0] = -s;
    m[1][1] = c;
    m
}

fn mul_mat4(a: [[f32; 4]; 4], b: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0; 4]; 4];
    for row in 0..4 {
        for col in 0..4 {
            for i in 0..4 {
                result[row][col] += a[row][i] * b[i][col];
            }
        }
    }
    result
}

fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let f = 1.0 / (fov_y / 2.0).tan();
    let mut m = [[0.0; 4]; 4];
    m[0][0] = f / aspect;
    m[1][1] = f;
    m[2][2] = far / (near - far);
    m[2][3] = (far * near) / (near - far);
    m[3][2] = -1.0;
    m
}

impl Drop for SceneRenderer {
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
        }
    }
}
