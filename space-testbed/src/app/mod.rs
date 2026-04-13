use std::sync::Arc;
use std::time::{Duration, Instant};

mod debug;
mod input;

use anyhow::Result;
use space_engine::Engine;
use space_engine::logger::{LogLevel, Logger};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowId};

use crate::app::input::keyboard_handler::{KeyboardCommand, KeyboardHandler};
use crate::app::input::mouse_handler::MouseHandler;
use crate::app::state::{EngineState, GameState, MenuState, StateTransition};

mod state;

pub const FPS_CAP: Option<u32> = None;
pub const TPS: u32 = 100;

pub struct App {
    state: GameState,
    engine: Option<Engine>,
    logger: &'static Logger,
    next_redraw_time: Instant,
    keyboard_handler: KeyboardHandler,
    mouse_handler: MouseHandler,
    next_update_time: Instant,
}

impl App {
    pub fn new() -> Self {
        let logger = Logger::get_logger();

        let keyboard_handler = KeyboardHandler::new();
        let mouse_handler = MouseHandler::new();

        let state = GameState::from(MenuState::new());

        Self {
            keyboard_handler,
            mouse_handler,
            state,
            engine: None,
            logger,
            next_redraw_time: Instant::now(),
            next_update_time: Instant::now(),
        }
    }

    fn apply_transition(&mut self, transition: StateTransition) -> Result<()> {
        if let Some(engine) = &mut self.engine {
            match transition {
                StateTransition::None => {}
                StateTransition::SwitchTo(new_state) => {
                    if matches!(&new_state, GameState::WorldState(_)) {
                        self.logger.log("Switching to world state", LogLevel::Info);
                        engine.window.set_cursor_visible(false);
                        engine.window.set_cursor_grab(CursorGrabMode::Confined)?;
                    } else {
                        self.logger
                            .log("Switching from world state", LogLevel::Info);
                        engine.window.set_cursor_visible(true);
                        engine.window.set_cursor_grab(CursorGrabMode::None)?;
                    }
                    self.state = new_state;
                    let capacity = self.state.ui_capacity();
                    engine.window_renderer.renderer.set_ui_capacity(capacity)?;
                }
                StateTransition::Push(_) => todo!(),
                StateTransition::Pop => todo!(),
                StateTransition::Quit => todo!(),
            }
        }

        Ok(())
    }
    fn handle_command(&mut self, command: &KeyboardCommand) {
        if matches!(command, KeyboardCommand::ToggleFullscreen)
            && let Some(engine) = &mut self.engine
        {
            if engine.window.fullscreen().is_some() {
                engine.window.set_fullscreen(None);
            } else {
                engine.window.set_fullscreen(Some(Fullscreen::Borderless(
                    engine.window.current_monitor(),
                )));
            }
        }
    }

    fn update(&mut self) {
        self.state.update(&self.keyboard_handler);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.logger.log("Application resumed", LogLevel::Info);

        let window_attributes = Window::default_attributes().with_title("Space Game");

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.engine = Some(Engine::new(window, self.state.ui_capacity()).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        if let Some(engine) = &mut self.engine {
            match event {
                WindowEvent::CloseRequested => {
                    self.logger.log("Closing window", LogLevel::Info);
                    event_loop.exit();
                }
                WindowEvent::RedrawRequested => {
                    engine
                        .window_renderer
                        .render(&self.state.get_context())
                        .expect("Failed to draw the renderer");
                }
                WindowEvent::Resized(_) => {
                    engine.window_renderer.resize().unwrap();
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    engine.window_renderer.resize().unwrap();
                }
                WindowEvent::KeyboardInput {
                    event: key_event, ..
                } => {
                    let command = self.keyboard_handler.register_key(&key_event);
                    self.handle_command(&command);
                    let transition = self.state.handle_keyboard_command(&command);
                    self.apply_transition(transition)
                        .expect("Failed to apply transition");
                }
                WindowEvent::CursorMoved { .. } | WindowEvent::CursorLeft { .. } | WindowEvent::MouseInput { .. } => {
                    let command = self.mouse_handler.handle_mouse_event(&event, &self.state);
                    let transition = self.state.handle_mouse_command(&command);
                    self.apply_transition(transition)
                        .expect("Failed to apply transition");
                }
                _ => {}
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event {
            let command = self.mouse_handler.handle_motion_event(delta, &self.state);
            let transition = self.state.handle_mouse_command(&command);
            self.apply_transition(transition)
                .expect("Failed to apply transition");
            //self.logger.log(format!("Camera {:?}", self.state.get_camera().unwrap_or(&(Camera::default()))), LogLevel::Verbose);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if Instant::now() >= self.next_update_time {
            self.update();
            self.next_update_time += Duration::from_secs_f32(1.0 / TPS as f32);
        }

        if let Some(fps_cap) = FPS_CAP {
            let target_frame = Duration::from_secs_f32(1.0 / fps_cap as f32);
            let now = Instant::now();

            if now >= self.next_redraw_time {
                if let Some(engine) = self.engine.as_mut() {
                    engine.request_redraw();
                }
                while self.next_redraw_time <= now {
                    self.next_redraw_time += target_frame;
                }
            }

            event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_redraw_time));
        } else {
            if let Some(engine) = self.engine.as_mut() {
                engine.request_redraw();
            }
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // Drop the engine when the app is suspended to free up resources
        self.engine = None;
    }
}
