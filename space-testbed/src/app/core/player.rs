use space_engine::core::camera::Camera;

use crate::app::utils::physics::PhysicsContext;

#[derive(Debug)]
pub struct Player {
    pub camera: Camera,
    pub moving: bool,
    pub physics: PhysicsContext,
}
