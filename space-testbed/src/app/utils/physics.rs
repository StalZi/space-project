use space_engine::utils::Point3D;


#[derive(Debug)]
pub struct PhysicsContext {
    pub mass: f32,
    pub force: Point3D,
    pub acceleration: Point3D,
    pub velocity: Point3D,
    pub g: f32,
    pub kinetic_friction_coefficient: f32,
    pub static_friction_coefficient: f32,
    pub master_speed_coefficient: f32,
    pub stop_threshold: f32,
}