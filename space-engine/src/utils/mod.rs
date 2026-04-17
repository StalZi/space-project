use std::collections::HashMap;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::core::camera::Camera;

pub mod image_utils;
pub mod math;

#[derive(Default, Clone, Copy, Debug)]
pub struct Point2D {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct PointUV {
    pub u: f32,
    pub v: f32,
}
impl PointUV {
    pub fn from_arr(arr: [f32; 2]) -> Self {
        Self {
            u: arr[0],
            v: arr[1],
        }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Point3D {
    pub fn from_arr(arr: [f32; 3]) -> Self {
        Self {
            x: arr[0],
            y: arr[1],
            z: arr[2],
        }
    }
    pub fn close_to_zero_by(&self, by: f32) -> bool {
        self.x.abs() < by && self.y.abs() < by && self.z.abs() < by
    }
    pub fn bring_closer_to_zero_by(&mut self, by: Point3D) {
        if self.x > 0.0 {
            self.x = (self.x - by.x).max(0.0);
        } else {
            self.x = (self.x + by.x).min(0.0);
        }
        if self.y > 0.0 {
            self.y = (self.y - by.y).max(0.0);
        } else {
            self.y = (self.y + by.y).min(0.0);
        }
        if self.z > 0.0 {
            self.z = (self.z - by.z).max(0.0);
        } else {
            self.z = (self.z + by.z).min(0.0);
        }
    }
    pub fn some(&self) -> bool {
        self.x != 0.0 && self.y != 0.0 && self.z != 0.0
    }
    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }
}

impl Neg for Point3D {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Sub<Point3D> for Point3D {
    type Output = Self;
    fn sub(self, point: Point3D) -> Self::Output {
        Self {
            x: self.x - point.x,
            y: self.y - point.y,
            z: self.z - point.z,
        }
    }
}
impl SubAssign<Point3D> for Point3D {
    fn sub_assign(&mut self, point: Point3D) {
        self.x -= point.x;
        self.y -= point.y;
        self.z -= point.z;
    }
}

impl Sub<f32> for Point3D {
    type Output = Self;
    fn sub(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs,
        }
    }
}
impl SubAssign<f32> for Point3D {
    fn sub_assign(&mut self, rhs: f32) {
        self.x -= rhs;
        self.y -= rhs;
        self.z -= rhs;
    }
}

impl Add<Point3D> for Point3D {
    type Output = Self;
    fn add(self, point: Point3D) -> Self::Output {
        Self {
            x: self.x + point.x,
            y: self.y + point.y,
            z: self.z + point.z,
        }
    }
}
impl AddAssign<Point3D> for Point3D {
    fn add_assign(&mut self, point: Point3D) {
        self.x += point.x;
        self.y += point.y;
        self.z += point.z;
    }
}

impl Add<f32> for Point3D {
    type Output = Self;
    fn add(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
        }
    }
}
impl AddAssign<f32> for Point3D {
    fn add_assign(&mut self, rhs: f32) {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
    }
}

impl Mul<Point3D> for Point3D {
    type Output = Self;
    fn mul(self, point: Point3D) -> Self::Output {
        Self {
            x: self.x * point.x,
            y: self.y * point.y,
            z: self.z * point.z,
        }
    }
}
impl MulAssign<Point3D> for Point3D {
    fn mul_assign(&mut self, point: Point3D) {
        self.x *= point.x;
        self.y *= point.y;
        self.z *= point.z;
    }
}

impl Mul<f32> for Point3D {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}
impl MulAssign<f32> for Point3D {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<f32> for Point3D {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}
impl DivAssign<f32> for Point3D {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Rotation3D {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct ColorRGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorRGBA {
    pub fn from_arr(arr: [f32; 4]) -> Self {
        Self {
            r: arr[0],
            g: arr[1],
            b: arr[2],
            a: arr[3],
        }
    }
    pub fn change_by(&mut self, dr: f32, dg: f32, db: f32, da: f32) {
        self.r = (self.r + dr).clamp(0.0, 255.0);
        self.g = (self.g + dg).clamp(0.0, 255.0);
        self.b = (self.b + db).clamp(0.0, 255.0);
        self.a = (self.a + da).clamp(0.0, 255.0);
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Size2D {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Size3D {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}
impl Default for Size3D {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            depth: 1.0,
        }
    }
}
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
    pub fn color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.color = ColorRGBA { r, g, b, a };
        self
    }
}
#[derive(Default, Debug, Clone)]
pub struct UIObject {
    pub position: Point3D,
    pub size: Size2D,
    pub bg_color: ColorRGBA,
}

impl UIObject {
    pub fn position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = Point3D { x, y, z };
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = Size2D { width, height };
        self
    }

    pub fn bg_color(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.bg_color = ColorRGBA { r, g, b, a };
        self
    }

    pub fn hover(&mut self, is_hovered: bool) {
        if is_hovered {
            self.size = Size2D {
                width: self.size.width + 20,
                height: self.size.height + 20,
            };
            self.position = Point3D {
                x: self.position.x - 10.0,
                y: self.position.y - 10.0,
                z: self.position.z + 1.0,
            };
        } else {
            self.size = Size2D {
                width: self.size.width - 20,
                height: self.size.height - 20,
            };
            self.position = Point3D {
                x: self.position.x + 10.0,
                y: self.position.y + 10.0,
                z: self.position.z - 1.0,
            };
        }
    }
}

pub struct RenderingContext<'a> {
    pub camera: Option<&'a Camera>,
    pub cubes: Option<&'a [Cube]>,
    pub ui_objects: Option<Vec<UIObject>>,
    pub meshes_path: Option<&'a String>,
    pub mesh_states: Option<&'a HashMap<String, Vec<MeshState>>>,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct MeshState {
    pub position: Point3D,
    pub rotation: Rotation3D,
    pub size: Size3D,
}
impl MeshState {
    pub fn position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = Point3D { x, y, z };
        self
    }
    pub fn rotation(mut self, pitch: f32, yaw: f32, roll: f32) -> Self {
        self.rotation = Rotation3D { pitch, yaw, roll };
        self
    }
    pub fn size(mut self, width: f32, height: f32, depth: f32) -> Self {
        self.size = Size3D {
            width,
            height,
            depth,
        };
        self
    }
}
