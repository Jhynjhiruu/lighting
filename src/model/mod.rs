use citro3d::Instance;
use glam::{Mat4, Quat, Vec3};

use crate::asset_server::{retrieve_asset, AssetKey};
use crate::Uniforms;
use vert_attr::VertAttrBuilder;

pub mod colour;
pub mod material;
pub mod shape;
pub mod texture;

use shape::Shape;

#[derive(Debug)]
pub struct Model<T: VertAttrBuilder> {
    pub pos: Vec3,
    pub rot: Vec3,
    shapes: Vec<AssetKey<Shape<T>>>,
}

impl<T: VertAttrBuilder + 'static> Model<T> {
    pub fn new(pos: Vec3, rot: Vec3, shapes: Vec<AssetKey<Shape<T>>>) -> Self {
        Self { pos, rot, shapes }
    }

    pub fn draw(&self, gpu: &mut Instance, uniforms: &Uniforms) {
        let scale = Vec3::new(1.0, 1.0, 1.0);

        let rotation = Quat::from_rotation_x(-self.rot.y)
            * Quat::from_rotation_y(self.rot.x)
            * Quat::from_rotation_z(self.rot.z);

        let transform =
            Mat4::from_scale_rotation_translation(scale, -rotation, -self.pos).inverse();

        gpu.bind_vertex_uniform(uniforms.model_matrix, transform);

        for shape in &self.shapes {
            let shape = retrieve_asset(shape);
            shape.draw(gpu, uniforms);
        }
    }
}
