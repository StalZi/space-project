use std::collections::{HashMap, HashSet};

use space_engine::logger::{LogLevel, Logger};
use space_engine::utils::Point3D;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub enum KeyboardCommand {
    None,
    ToggleFullscreen,
    SwitchToMenu,
    SwitchToWorld,
    MoveCamera(Point3D),
}

#[derive(Debug)]
pub enum Keys {
    Forward,
    Backward,
    Left,
    Right,
    Escape,
    Enter,
}

#[derive(Debug)]
pub struct KeyboardHandler {
    logger: &'static Logger,
    keys_pressed: HashSet<PhysicalKey>,
    pub key_map: HashMap<KeyCode, Keys>,
}

impl KeyboardHandler {
    pub fn new() -> Self {
        let key_map = HashMap::from([
            (KeyCode::Escape, Keys::Escape),
            (KeyCode::Enter, Keys::Enter),
            (KeyCode::KeyW, Keys::Forward),
            (KeyCode::KeyS, Keys::Backward),
            (KeyCode::KeyA, Keys::Left),
            (KeyCode::KeyD, Keys::Right),
        ]);
        let logger = Logger::get_logger();
        let keys_pressed = HashSet::new();
        Self {
            key_map,
            logger,
            keys_pressed,
        }
    }

    pub fn register_key(&mut self, event: &KeyEvent) -> KeyboardCommand {
        self.logger.log(
            format!(
                "Key {:?} {}",
                event.physical_key,
                if event.state == ElementState::Pressed {
                    "pressed"
                } else {
                    "released"
                }
            ),
            LogLevel::Verbose,
        );
        match event {
            KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::Escape),
                state: ElementState::Pressed,
                ..
            } => return KeyboardCommand::SwitchToMenu,
            KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::Enter),
                state: ElementState::Pressed,
                ..
            } => return KeyboardCommand::SwitchToWorld,
            KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::F11),
                state: ElementState::Pressed,
                ..
            } => return KeyboardCommand::ToggleFullscreen,
            KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::KeyW),
                state,
                ..
            }
            | KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::KeyS),
                state,
                ..
            }
            | KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::KeyA),
                state,
                ..
            }
            | KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::KeyD),
                state,
                ..
            }
            | KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::Space),
                state,
                ..
            }
            | KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::ShiftLeft),
                state,
                ..
            } => {
                if let ElementState::Pressed = state {
                    self.keys_pressed.insert(event.physical_key);
                } else {
                    self.keys_pressed.remove(&event.physical_key);
                }
            }
            _ => {}
        };
        KeyboardCommand::None
    }

    pub fn handle_keys(&self, yaw: f32, delta_move: &mut Point3D) {
        for key in self.keys_pressed.iter() {
            match key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    delta_move.x += yaw.to_radians().sin() * 0.1;
                    delta_move.z -= yaw.to_radians().cos() * 0.1;
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    delta_move.x -= yaw.to_radians().sin() * 0.1;
                    delta_move.z += yaw.to_radians().cos() * 0.1;
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    delta_move.x -= yaw.to_radians().cos() * 0.1;
                    delta_move.z -= yaw.to_radians().sin() * 0.1;
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    delta_move.x += yaw.to_radians().cos() * 0.1;
                    delta_move.z += yaw.to_radians().sin() * 0.1;
                }
                PhysicalKey::Code(KeyCode::Space) => {
                    delta_move.y += 0.1;
                }
                PhysicalKey::Code(KeyCode::ShiftLeft) => {
                    delta_move.y -= 0.1;
                }
                _ => {}
            }
        }
    }
}
