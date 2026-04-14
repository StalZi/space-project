use crate::utils::math::wrap_angle;
use crate::utils::{Point3D, Rotation3D};

#[derive(Debug, Default)]
pub struct Camera {
    pub position: Point3D,
    pub rotation: Rotation3D,
}

impl Camera {
    pub fn new(position: Point3D, rotation: Rotation3D) -> Self {
        Self {
            position,
            rotation,
        }
    }

    pub fn change_position(&mut self, delta: Point3D) {
        self.position.x += delta.x;
        self.position.y += delta.y;
        self.position.z += delta.z;
    }

    pub fn change_rotation(&mut self, delta: Rotation3D) {
        self.rotation.pitch = wrap_angle(self.rotation.pitch + delta.pitch);
        self.rotation.yaw = wrap_angle(self.rotation.yaw + delta.yaw);
        self.rotation.roll = wrap_angle(self.rotation.roll + delta.roll);
    }
}
