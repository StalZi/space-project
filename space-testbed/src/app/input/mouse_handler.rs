use space_engine::utils::{Point3D, Rotation3D, Size2D};
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, WindowEvent};

use crate::app::state::{GameState, MenuState};

pub enum MouseCommand {
    None,
    RotateCamera(Rotation3D),
    HoverUIObject(usize),
    UnhoverUIObject,
    PressUIObject(usize),
}

pub struct MouseHandler {
    current_position: Option<PhysicalPosition<f64>>,
}

impl MouseHandler {
    pub fn new() -> Self {
        Self {
            current_position: None,
        }
    }

    fn in_area(
        &self,
        position: &PhysicalPosition<f64>,
        object_pos: Point3D,
        object_size: Size2D,
    ) -> bool {
        (position.x > object_pos.x as f64)
            && (position.x < (object_pos.x + object_size.width as f32) as f64)
            && (position.y > object_pos.y as f64)
            && (position.y < (object_pos.y + object_size.height as f32) as f64)
    }

    pub fn handle_mouse_event(&mut self, event: &WindowEvent, state: &GameState) -> MouseCommand {
        match state {
            GameState::MenuState(state) => self.handle_menu_mouse_event(event, state),
            GameState::WorldState(_) => MouseCommand::None,
        }
    }

    pub fn handle_motion_event(
        &mut self,
        delta: (f64, f64),
        mouse_sensitivity: f32,
        state: &GameState,
    ) -> MouseCommand {
        match state {
            GameState::MenuState(_) => MouseCommand::None,
            GameState::WorldState(_) => MouseCommand::RotateCamera(Rotation3D {
                pitch: delta.1 as f32 * mouse_sensitivity,
                yaw: delta.0 as f32 * mouse_sensitivity,
                roll: 0.0,
            }),
        }
    }

    pub fn handle_menu_mouse_event(
        &mut self,
        event: &WindowEvent,
        state: &MenuState,
    ) -> MouseCommand {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.current_position = Some(*position);
                for (index, object) in state.ui_objects.iter().enumerate() {
                    if self.in_area(position, object.position, object.size) {
                        return MouseCommand::HoverUIObject(index);
                    }
                }
                return MouseCommand::UnhoverUIObject;
            }
            WindowEvent::CursorLeft { .. } => {
                return MouseCommand::UnhoverUIObject;
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } => {
                if let Some(current_position) = self.current_position {
                    for (index, object) in state.ui_objects.iter().enumerate() {
                        if self.in_area(&current_position, object.position, object.size) {
                            return MouseCommand::PressUIObject(index);
                        }
                    }
                }
            }
            _ => {}
        };
        MouseCommand::None
    }
}
