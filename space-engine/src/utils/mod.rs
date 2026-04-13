pub mod image_utils;
pub mod math;

#[derive(Default, Clone, Copy, Debug)]
pub struct Point2D {
    pub x: i32,
    pub y: i32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Rotation3D {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct ColorRGBA {
    pub r: u32,
    pub g: u32,
    pub b: u32,
    pub a: u32,
}
impl ColorRGBA {
    pub fn change_by(&mut self, dr: u32, dg: u32, db: u32, da: u32) {
        self.r = (self.r + dr).clamp(0, 255);
        self.g = (self.g + dg).clamp(0, 255);
        self.b = (self.b + db).clamp(0, 255);
        self.a = (self.a + da).clamp(0, 255);
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}
