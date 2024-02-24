use citro3d::light::{BumpMode, LightLut, LightLutId, LutInput};
use citro3d::{material, Instance};

use crate::asset_server::{retrieve_asset, AssetKey};
use crate::Uniforms;

use super::colour::Colour;
use super::texture::GPUTexture;

#[derive(Debug)]
pub struct Material {
    texture: Option<AssetKey<GPUTexture>>,
    normal: Option<AssetKey<GPUTexture>>,
    ambient: Option<AssetKey<Colour>>,
    diffuse: Option<AssetKey<Colour>>,
    specular0: Option<AssetKey<Colour>>,
    specular1: Option<AssetKey<Colour>>,
    emission: Option<AssetKey<Colour>>,
    shininess: Option<f32>,
}

impl Material {
    pub fn new(
        texture: Option<AssetKey<GPUTexture>>,
        normal: Option<AssetKey<GPUTexture>>,
        ambient: Option<AssetKey<Colour>>,
        diffuse: Option<AssetKey<Colour>>,
        specular0: Option<AssetKey<Colour>>,
        specular1: Option<AssetKey<Colour>>,
        emission: Option<AssetKey<Colour>>,
        shininess: Option<f32>,
    ) -> Self {
        Self {
            texture,
            normal,
            ambient,
            diffuse,
            specular0,
            specular1,
            emission,
            shininess,
        }
    }

    pub fn get_texture(&self) -> Option<&GPUTexture> {
        if let Some(key) = &self.texture {
            Some(retrieve_asset(key))
        } else {
            None
        }
    }

    pub fn get_normal(&self) -> Option<&GPUTexture> {
        if let Some(key) = &self.normal {
            Some(retrieve_asset(key))
        } else {
            None
        }
    }

    pub fn set_light_env(&self, gpu: &mut Instance, _uniforms: &Uniforms, use_normal: bool) {
        let to_material_colour = |col: &AssetKey<Colour>| retrieve_asset(col).into();

        let mat = material::Material {
            ambient: self.ambient.as_ref().map(to_material_colour),
            diffuse: self.diffuse.as_ref().map(to_material_colour),
            specular0: self.specular0.as_ref().map(to_material_colour),
            specular1: self.specular1.as_ref().map(to_material_colour),
            emission: self.emission.as_ref().map(to_material_colour),
        };

        let mut light_env = gpu.light_env_mut();

        light_env.as_mut().connect_lut(
            LightLutId::D0,
            LutInput::NormalView,
            LightLut::from_fn(|i| i.powf(self.shininess.unwrap_or(30.0)), false),
        );
        light_env.as_mut().set_material(mat);
        if use_normal {
            light_env.as_mut().set_normal_map(BumpMode::AsBump, 1);
        } else {
            light_env.as_mut().set_normal_map(BumpMode::None, 0);
        }
    }
}
