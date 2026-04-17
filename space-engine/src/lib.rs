#![allow(dead_code)]
#![allow(clippy::too_many_arguments)]

pub mod logger;
use logger::{LogLevel, Logger};

mod vulkan;
use vulkan::VulkanContext;

mod window_renderer;
use window_renderer::WindowRenderer;

pub mod core;

mod render;

pub mod utils;
use utils::RenderingContext;

mod resources;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use ash::vk;
use resources::ResourceManager;
use winit::window::Window;

pub struct Engine {
    pub window_renderer: WindowRenderer,
    context: Arc<VulkanContext>,
    pub window: Arc<Window>,
    logger: &'static Logger,
    resource_manager: ResourceManager,
}

impl Engine {
    pub fn new(window: Arc<Window>, ui_buffer_capacity: usize) -> Result<Self> {
        let logger = Logger::get_logger();

        logger.log("====== Creating engine context ======", LogLevel::Info);
        let start = Instant::now();
        let context = Arc::new(VulkanContext::new(window.clone())?);
        let duration = start.elapsed();
        logger.log(
            format!(
                "====== Engine context created successfully in {}s ======",
                duration.as_secs_f32()
            ),
            LogLevel::Success,
        );

        logger.log(
            "====== Initializing the window renderer ======",
            LogLevel::Info,
        );
        let start = Instant::now();
        let window_renderer = WindowRenderer::new(
            context.clone(),
            window.clone(),
            3,
            vk::Format::R16G16B16A16_SFLOAT,
            vk::ClearColorValue {
                float32: [0.1, 0.0, 0.0, 1.0],
            },
            ui_buffer_capacity,
        )?;
        let duration = start.elapsed();
        logger.log(
            format!(
                "====== Window renderer Initialized successfully in {}s ======",
                duration.as_secs_f32()
            ),
            LogLevel::Success,
        );

        let resource_manager = ResourceManager::new(context.clone())?;

        Ok(Self {
            resource_manager,
            window_renderer,
            window,
            context,
            logger,
        })
    }

    pub fn request_redraw(&mut self, rendering_context: RenderingContext) {
        self.window_renderer
            .render(rendering_context, &mut self.resource_manager)
            .expect("Failed to draw the renderer");
    }
}
