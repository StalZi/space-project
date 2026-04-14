use enum_dispatch::enum_dispatch;
use space_engine::core::camera::Camera;
use space_engine::logger::{LogLevel, Logger};
use space_engine::render::context::RenderingContext;
use space_engine::render::scene::objects::Cube;
use space_engine::render::ui::objects::UIObject;
use space_engine::utils::Point3D;

use crate::app::core::player::Player;
use crate::app::debug::rand_cubes::generate_random_cubes;
use crate::app::input::keyboard_handler::{KeyboardCommand, KeyboardHandler};
use crate::app::input::mouse_handler::MouseCommand;
use crate::app::utils::physics::PhysicsContext;

#[enum_dispatch(EngineState)]
#[derive(Debug)]
pub enum GameState {
    MenuState,
    WorldState,
}

pub enum StateTransition {
    None,                       // Ничего не делать
    SwitchTo(GameState),        // Сменить на другое состояние
    Push(Box<dyn EngineState>), // Добавить в стек (пауза)
    Pop,                        // Убрать из стека
    Quit,                       // Выйти из игры
}

#[enum_dispatch]
pub trait EngineState {
    fn ui_capacity(&self) -> usize {
        0
    }
    fn cube_capacity(&self) -> usize {
        0
    }

    fn get_ui(&self) -> Option<&Vec<UIObject>> {
        None
    }
    fn get_cubes(&self) -> Option<&[Cube]> {
        None
    }
    fn get_camera(&self) -> Option<&Camera> {
        None
    }
    fn get_context(&'_ self) -> RenderingContext<'_> {
        //sorted_by_z is an option
        let sorted_by_z: Option<Vec<UIObject>> = self.get_ui().map(|ui_objects| {
            let mut sorted = ui_objects.clone();
            sorted.sort_by(|a, b| a.position.z.partial_cmp(&b.position.z).unwrap());
            sorted
        });
        RenderingContext {
            cubes: self.get_cubes(),
            ui_objects: sorted_by_z,
            camera: self.get_camera(),
        }
    }

    fn update(&mut self, _elapsed: f32, _keyboard_handler: &KeyboardHandler) {}

    fn handle_keyboard_command(&mut self, _command: &KeyboardCommand) -> StateTransition {
        StateTransition::None
    }

    fn handle_mouse_command(&mut self, _command: &MouseCommand) -> StateTransition {
        StateTransition::None
    }
}

#[derive(Debug)]
pub struct MenuState {
    pub ui_objects: Vec<UIObject>,
    hovered_index: Option<usize>,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            ui_objects: vec![
                UIObject::default()
                    .position(0.0, 0.0, 0.0)
                    .size(200, 200)
                    .bg_color(255, 0, 0, 100),
                UIObject::default()
                    .position(200.0, 0.0, 0.0)
                    .size(200, 200)
                    .bg_color(0, 255, 0, 100),
                UIObject::default()
                    .position(0.0, 200.0, 0.0)
                    .size(200, 200)
                    .bg_color(0, 0, 255, 100),
                UIObject::default()
                    .position(200.0, 200.0, 0.0)
                    .size(200, 200)
                    .bg_color(255, 255, 255, 100),
            ],
            hovered_index: None,
        }
    }
}

impl EngineState for MenuState {
    fn ui_capacity(&self) -> usize {
        self.ui_objects.len()
    }

    fn get_ui(&self) -> Option<&Vec<UIObject>> {
        Some(&self.ui_objects)
    }

    fn handle_keyboard_command(&mut self, command: &KeyboardCommand) -> StateTransition {
        match command {
            KeyboardCommand::SwitchToWorld => {
                StateTransition::SwitchTo(GameState::from(WorldState::new()))
            }
            _ => StateTransition::None,
        }
    }

    fn handle_mouse_command(&mut self, command: &MouseCommand) -> StateTransition {
        match command {
            MouseCommand::HoverUIObject(index) => {
                if let Some(hovered_index) = self.hovered_index {
                    if hovered_index == *index {
                        return StateTransition::None;
                    } else {
                        self.handle_mouse_command(&MouseCommand::UnhoverUIObject);
                    }
                }
                self.ui_objects[*index].hover(true);
                self.hovered_index = Some(*index);
            }
            MouseCommand::UnhoverUIObject => {
                if let Some(hovered_index) = self.hovered_index {
                    self.ui_objects[hovered_index].hover(false);
                    self.hovered_index = None;
                }
            }
            MouseCommand::PressUIObject(index) => {
                if *index == 0 {
                    return StateTransition::SwitchTo(GameState::from(WorldState::new()));
                } else {
                    self.ui_objects[*index].bg_color.change_by(10, 10, 10, 0);
                }
            }
            _ => {}
        };
        StateTransition::None
    }
}

#[derive(Debug)]
pub struct WorldState {
    player: Player,
    ui_objects: Vec<UIObject>,
    cube_objects: Vec<Cube>,
    logger: &'static Logger,
}

impl WorldState {
    pub fn new() -> Self {
        let logger = Logger::get_logger();
        Self {
            logger,
            player: Player {
                camera: Camera::default(),
                moving: false,
                physics: PhysicsContext {
                    mass: 3.0,
                    force: Point3D::default(),
                    acceleration: Point3D::default(),
                    velocity: Point3D::default(),
                    g: 9.81,
                    kinetic_friction_coefficient: 0.0007,
                    static_friction_coefficient: 0.005,
                    stop_threshold: 0.001,
                    master_speed_coefficient: 1.0,
                },
            },
            ui_objects: vec![
                UIObject::default()
                    .position(200.0, 500.0, 0.0)
                    .size(400, 100)
                    .bg_color(100, 100, 100, 100),
            ],

            cube_objects: generate_random_cubes(1000),
        }
    }
}

impl EngineState for WorldState {
    fn ui_capacity(&self) -> usize {
        self.ui_objects.len()
    }
    fn cube_capacity(&self) -> usize {
        self.cube_objects.len()
    }
    fn get_ui(&self) -> Option<&Vec<UIObject>> {
        Some(&self.ui_objects)
    }
    fn get_cubes(&self) -> Option<&[Cube]> {
        Some(&self.cube_objects)
    }
    fn get_camera(&self) -> Option<&Camera> {
        Some(&self.player.camera)
    }

    fn update(&mut self, dt: f32, keyboard_handler: &KeyboardHandler) {
        keyboard_handler.handle_movement(&mut self.player); // self.player.physics.force updated

        let friction = if self.player.moving {
            self.player.physics.kinetic_friction_coefficient
                * self.player.physics.mass
                * self.player.physics.g
        } else {
            self.player.physics.static_friction_coefficient
                * self.player.physics.mass
                * self.player.physics.g
        };

        self.player.physics.acceleration = self.player.physics.force / self.player.physics.mass;
        self.player.physics.velocity += self.player.physics.acceleration * dt;
        self.player
            .physics
            .velocity
            .bring_closer_to_zero_by((self.player.physics.velocity * friction).abs());

        //
        //
        // if !self.player.physics.force.close_to_zero_by(self.player.physics.stop_threshold) {
        //     let friction = self.player.physics.kinetic_friction_coefficient * self.player.physics.mass * self.player.physics.g;
        //     self.player.physics.acceleration = (self.player.physics.force - friction) / self.player.physics.mass;
        // }

        if self
            .player
            .physics
            .velocity
            .close_to_zero_by(self.player.physics.stop_threshold)
        {
            self.player.physics.velocity = Point3D::default();
            self.player.moving = false;
        } else {
            self.player.moving = true;
        }
        let delta_move =
            self.player.physics.velocity * self.player.physics.master_speed_coefficient;
        self.player.camera.change_position(delta_move);
    }

    fn handle_keyboard_command(&mut self, command: &KeyboardCommand) -> StateTransition {
        match command {
            KeyboardCommand::SwitchToMenu => {
                return StateTransition::SwitchTo(GameState::from(MenuState::new()));
            }
            KeyboardCommand::MoveCamera(delta) => {
                self.player.camera.change_position(*delta);
            }
            _ => {}
        };
        StateTransition::None
    }

    fn handle_mouse_command(&mut self, command: &MouseCommand) -> StateTransition {
        if let MouseCommand::RotateCamera(delta) = command {
            self.player.camera.change_rotation(*delta);
        };
        StateTransition::None
    }
}
