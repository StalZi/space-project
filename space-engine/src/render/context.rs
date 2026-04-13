use crate::core::camera::Camera;
use crate::render::scene::objects::Cube;
use crate::render::ui::objects::UIObject;

pub struct RenderingContext<'a> {
    pub camera: Option<&'a Camera>,
    pub cubes: Option<&'a [Cube]>,
    pub ui_objects: Option<Vec<UIObject>>,
}
