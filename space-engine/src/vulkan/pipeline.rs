use anyhow::Result;
use ash::{Device, vk};

pub fn create_graphics_pipeline(
    device: &Device,
    vertex_shader: vk::ShaderModule,
    fragment_shader: vk::ShaderModule,
    extent: vk::Extent2D,
    format: vk::Format,
    depth_format: vk::Format,
    depth_enabled: bool,
    pipeline_layout: vk::PipelineLayout,
    pipeline_cache: vk::PipelineCache,
    topology: vk::PrimitiveTopology,
    enable_blend: bool,
) -> Result<vk::Pipeline> {
    let entry_point = c"main";

    let graphics_pipeline = unsafe {
        device.create_graphics_pipelines(
            pipeline_cache,
            &[vk::GraphicsPipelineCreateInfo::default()
                .stages(&[
                    vk::PipelineShaderStageCreateInfo::default()
                        .stage(vk::ShaderStageFlags::VERTEX)
                        .module(vertex_shader)
                        .name(entry_point),
                    vk::PipelineShaderStageCreateInfo::default()
                        .stage(vk::ShaderStageFlags::FRAGMENT)
                        .module(fragment_shader)
                        .name(entry_point),
                ])
                .vertex_input_state(&vk::PipelineVertexInputStateCreateInfo::default())
                .input_assembly_state(
                    &vk::PipelineInputAssemblyStateCreateInfo::default().topology(topology),
                )
                .viewport_state(
                    &vk::PipelineViewportStateCreateInfo::default()
                        .viewports(&[vk::Viewport::default()
                            .width(extent.width as f32)
                            .height(extent.height as f32)
                            .min_depth(0.0)
                            .max_depth(1.0)])
                        .scissors(&[vk::Rect2D {
                            offset: vk::Offset2D { x: 0, y: 0 },
                            extent,
                        }]),
                )
                .rasterization_state(
                    &vk::PipelineRasterizationStateCreateInfo::default()
                        .polygon_mode(vk::PolygonMode::FILL)
                        .cull_mode(vk::CullModeFlags::NONE)
                        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                        .line_width(1.0),
                )
                .multisample_state(
                    &vk::PipelineMultisampleStateCreateInfo::default()
                        .rasterization_samples(vk::SampleCountFlags::TYPE_1),
                )
                .depth_stencil_state(
                    &vk::PipelineDepthStencilStateCreateInfo::default()
                        .depth_test_enable(depth_enabled)
                        .depth_write_enable(depth_enabled)
                        .depth_compare_op(vk::CompareOp::LESS)
                        .depth_bounds_test_enable(false)
                        .stencil_test_enable(false),
                )
                .color_blend_state(
                    &vk::PipelineColorBlendStateCreateInfo::default().attachments(&[
                        vk::PipelineColorBlendAttachmentState::default()
                            .color_write_mask(vk::ColorComponentFlags::RGBA)
                            .blend_enable(enable_blend)
                            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                            .color_blend_op(vk::BlendOp::ADD)
                            .src_alpha_blend_factor(vk::BlendFactor::ONE)
                            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                            .alpha_blend_op(vk::BlendOp::ADD),
                    ]),
                )
                .dynamic_state(
                    &vk::PipelineDynamicStateCreateInfo::default()
                        .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]),
                )
                .layout(pipeline_layout)
                .push_next(
                    &mut vk::PipelineRenderingCreateInfoKHR::default()
                        .color_attachment_formats(&[format])
                        .depth_attachment_format(depth_format)
                        .stencil_attachment_format(vk::Format::UNDEFINED),
                )],
            None,
        )
    }
    .unwrap()
    .into_iter()
    .next()
    .unwrap();

    Ok(graphics_pipeline)
}
