use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

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
impl Point3D {
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
