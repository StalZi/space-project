use crate::utils::{ColorRGBA, Point3D, Size};

#[derive(Default, Debug, Clone)]
pub struct UIObject {
    pub position: Point3D,
    pub size: Size,
    pub bg_color: ColorRGBA,
}

impl UIObject {
    pub fn position(mut self, x: f32, y: f32, z: f32) -> Self {
        self.position = Point3D { x, y, z };
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = Size { width, height };
        self
    }

    pub fn bg_color(mut self, r: u32, g: u32, b: u32, a: u32) -> Self {
        self.bg_color = ColorRGBA { r, g, b, a };
        self
    }

    pub fn hover(&mut self, is_hovered: bool) {
        if is_hovered {
            self.size = Size {
                width: self.size.width + 20,
                height: self.size.height + 20,
            };
            self.position = Point3D {
                x: self.position.x - 10.0,
                y: self.position.y - 10.0,
                z: self.position.z + 1.0,
            };
        } else {
            self.size = Size {
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
