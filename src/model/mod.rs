use citro3d::math::Matrix4;
use citro3d::Instance;

use crate::asset_server::{retrieve_asset, AssetKey};
use crate::{Uniforms, Vec3};
use vert_attr::VertAttrBuilder;

pub mod colour;
pub mod material;
pub mod shape;
//pub mod texture;

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
        let Vec3 { x, y, z } = self.pos;

        let mut transform = Matrix4::identity();

        transform.scale(1.0, 1.0, 1.0);

        transform.translate(x, y, z);

        transform.rotate_x(-self.rot.y);
        transform.rotate_y(self.rot.x);
        transform.rotate_z(self.rot.z);

        gpu.bind_vertex_uniform(uniforms.model_matrix, &transform);

        for shape in &self.shapes {
            let shape = retrieve_asset(shape);
            shape.draw(gpu, uniforms);
        }
    }
}
