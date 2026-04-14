use std::collections::{HashMap, HashSet};

use space_engine::logger::{LogLevel, Logger};
use space_engine::utils::Point3D;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::app::core::player::Player;

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
                repeat: false,
                ..
            }
            | KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::KeyD),
                state,
                repeat: false,
                ..
            }
            | KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::Space),
                state,
                repeat: false,
                ..
            }
            | KeyEvent {
                physical_key: PhysicalKey::Code(KeyCode::ShiftLeft),
                state,
                repeat: false,
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

    pub fn handle_movement(&self, player: &mut Player) {
        player.physics.force = Point3D::default();
        if self.keys_pressed.is_empty() {
            return;
        }
        for key in self.keys_pressed.iter() {
            match key {
                PhysicalKey::Code(KeyCode::KeyW) => {
                    player.physics.force.x += player.camera.rotation.yaw.to_radians().sin();
                    player.physics.force.z -= player.camera.rotation.yaw.to_radians().cos();
                }
                PhysicalKey::Code(KeyCode::KeyS) => {
                    player.physics.force.x -= player.camera.rotation.yaw.to_radians().sin();
                    player.physics.force.z += player.camera.rotation.yaw.to_radians().cos();
                }
                PhysicalKey::Code(KeyCode::KeyA) => {
                    player.physics.force.x -= player.camera.rotation.yaw.to_radians().cos();
                    player.physics.force.z -= player.camera.rotation.yaw.to_radians().sin();
                }
                PhysicalKey::Code(KeyCode::KeyD) => {
                    player.physics.force.x += player.camera.rotation.yaw.to_radians().cos();
                    player.physics.force.z += player.camera.rotation.yaw.to_radians().sin();
                }
                PhysicalKey::Code(KeyCode::Space) => {
                    player.physics.force.y += 1.0;
                }
                PhysicalKey::Code(KeyCode::ShiftLeft) => {
                    player.physics.force.y -= 1.0;
                }
                _ => {}
            }
        }
    }
}
