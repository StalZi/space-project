use crate::utils::{ColorRGBA, Point3D, Rotation3D};

#[derive(Default, Debug)]
pub struct Cube {
    pub position: Point3D,
    pub rotation: Rotation3D,
    pub size: Point3D,
    pub color: ColorRGBA,
}

impl Cube {
    pub fn position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = Point3D { x, y, z };
        self
    }
    pub fn rotation(mut self, pitch: f32, yaw: f32, roll: f32) -> Self {
        self.rotation = Rotation3D { pitch, yaw, roll };
        self
    }
    pub fn size(mut self, x: f32, y: f32, z: f32) -> Self {
        self.size = Point3D { x, y, z };
        self
    }
    pub fn color(mut self, r: u32, g: u32, b: u32, a: u32) -> Self {
        self.color = ColorRGBA { r, g, b, a };
        self
    }
}
