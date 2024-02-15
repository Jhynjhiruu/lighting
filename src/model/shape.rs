use citro3d::{
    attrib,
    buffer::{self, Primitive},
    Instance,
};
use ctru::linear::LinearAllocator;
use vert_attr::VertAttrBuilder;

use crate::{
    asset_server::{retrieve_asset, AssetKey},
    Uniforms,
};

use super::material::Material;

#[derive(Debug)]
pub struct Shape<T: VertAttrBuilder> {
    mat: AssetKey<Material>,
    prim_type: Primitive,
    verts: Vec<T, LinearAllocator>,
    attr_info: attrib::Info,
}

impl<T: VertAttrBuilder> Shape<T> {
    pub fn new(mat: AssetKey<Material>, prim_type: Primitive, verts: Vec<T>) -> Self {
        let mut vertex_buffer = Vec::with_capacity_in(verts.len(), LinearAllocator);
        vertex_buffer.extend(verts);

        let attr_info = T::vert_attrs();

        Self {
            mat,
            prim_type,
            verts: vertex_buffer,
            attr_info,
        }
    }

    pub fn draw(&self, gpu: &mut Instance, uniforms: &Uniforms) {
        let mat = retrieve_asset(&self.mat);
        let tex = mat.get_texture();
        mat.set_light_env(gpu, uniforms);

        let stage0 = citro3d::texenv::Stage::new(0).unwrap();
        let stage1 = citro3d::texenv::Stage::new(1).unwrap();

        if let Some(t) = tex {
            t.bind(0);

            /*if mat.use_vertex_colours() {
                gpu.texenv(stage0)
                    .src(
                        citro3d::texenv::Mode::BOTH,
                        citro3d::texenv::Source::Texture0,
                        Some(citro3d::texenv::Source::PrimaryColor),
                        None,
                    )
                    .func(
                        citro3d::texenv::Mode::BOTH,
                        citro3d::texenv::CombineFunc::Add,
                    );
            } else {*/
            gpu.texenv(stage0)
                .src(
                    citro3d::texenv::Mode::BOTH,
                    citro3d::texenv::Source::FragmentPrimaryColor,
                    Some(citro3d::texenv::Source::FragmentSecondaryColor),
                    None,
                )
                .func(
                    citro3d::texenv::Mode::BOTH,
                    citro3d::texenv::CombineFunc::Add,
                );

            gpu.texenv(stage1)
                .src(
                    citro3d::texenv::Mode::BOTH,
                    citro3d::texenv::Source::Previous,
                    Some(citro3d::texenv::Source::Texture0),
                    None,
                )
                .func(
                    citro3d::texenv::Mode::BOTH,
                    citro3d::texenv::CombineFunc::Modulate,
                );
            //}
        } else {
            let env = gpu.texenv(stage0);
            env.reset();

            env.src(
                citro3d::texenv::Mode::BOTH,
                citro3d::texenv::Source::FragmentPrimaryColor,
                Some(citro3d::texenv::Source::FragmentSecondaryColor),
                None,
            )
            .func(
                citro3d::texenv::Mode::BOTH,
                citro3d::texenv::CombineFunc::Add,
            );
        }

        let mut buf_info = buffer::Info::new();
        let buf_vtos = buf_info
            .add(&self.verts, &self.attr_info)
            .expect("failed to bind verts");

        gpu.set_attr_info(&self.attr_info);
        gpu.draw_arrays(self.prim_type, buf_vtos);
    }
}
