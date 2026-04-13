use enum_dispatch::enum_dispatch;
use space_engine::core::camera::Camera;
use space_engine::render::context::RenderingContext;
use space_engine::render::scene::objects::Cube;
use space_engine::render::ui::objects::UIObject;
use space_engine::utils::{Point3D, Rotation3D};

use crate::app::debug::rand_cubes::generate_random_cubes;
use crate::app::input::keyboard_handler::{KeyboardCommand, KeyboardHandler};
use crate::app::input::mouse_handler::MouseCommand;

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

    fn update(&mut self, _keyboard_handler: &KeyboardHandler) {}

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
    camera: Camera,
    ui_objects: Vec<UIObject>,
    cube_objects: Vec<Cube>,
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(
                Point3D {
                    x: 0.0,
                    y: 0.0,
                    z: 5.0,
                },
                Rotation3D {
                    pitch: 0.0,
                    yaw: 0.0,
                    roll: 0.0,
                },
            ),
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
        Some(&self.camera)
    }

    fn update(&mut self, keyboard_handler: &KeyboardHandler) {
        keyboard_handler.handle_keys(self.camera.rotation.yaw, &mut self.camera.delta_move);
        self.camera.change_position(self.camera.delta_move);
        self.camera.delta_move = Point3D::default();
    }

    fn handle_keyboard_command(&mut self, command: &KeyboardCommand) -> StateTransition {
        match command {
            KeyboardCommand::SwitchToMenu => {
                return StateTransition::SwitchTo(GameState::from(MenuState::new()));
            }
            KeyboardCommand::MoveCamera(delta) => {
                self.camera.change_position(*delta);
            }
            _ => {}
        };
        StateTransition::None
    }

    fn handle_mouse_command(&mut self, command: &MouseCommand) -> StateTransition {
        if let MouseCommand::RotateCamera(delta) = command {
            self.camera.change_rotation(*delta);
        };
        StateTransition::None
    }
}
