use citro3d::attrib::Format;

use vert_attr::VertAttrs;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl VertAttrs for Vec3 {
    const FORMAT: Format = Format::Float;
    const SIZE: u8 = 3;
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl VertAttrs for Vec2 {
    const FORMAT: Format = Format::Float;
    const SIZE: u8 = 2;
}
