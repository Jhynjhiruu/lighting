use citro3d::light::{LightLutId, LutData, LutInput};
use citro3d::{material, Instance};

use crate::asset_server::{retrieve_asset, AssetKey};
use crate::Uniforms;

use super::colour::Colour;
use super::texture::GPUTexture;

#[derive(Debug)]
pub struct Material {
    texture: Option<AssetKey<GPUTexture>>,
    ambient: Option<AssetKey<Colour>>,
    diffuse: Option<AssetKey<Colour>>,
    specular0: Option<AssetKey<Colour>>,
    specular1: Option<AssetKey<Colour>>,
    emission: Option<AssetKey<Colour>>,
}

impl Material {
    pub fn new(
        texture: Option<AssetKey<GPUTexture>>,
        ambient: Option<AssetKey<Colour>>,
        diffuse: Option<AssetKey<Colour>>,
        specular0: Option<AssetKey<Colour>>,
        specular1: Option<AssetKey<Colour>>,
        emission: Option<AssetKey<Colour>>,
    ) -> Self {
        Self {
            texture,
            ambient,
            diffuse,
            specular0,
            specular1,
            emission,
        }
    }

    pub fn get_texture(&self) -> Option<&GPUTexture> {
        if let Some(key) = &self.texture {
            Some(retrieve_asset(key))
        } else {
            None
        }
    }

    pub fn set_light_env(&self, gpu: &mut Instance, _uniforms: &Uniforms) {
        let to_material_colour = |col: &AssetKey<Colour>| retrieve_asset(col).into();

        let mat = material::Material {
            ambient: self.ambient.as_ref().map(to_material_colour),
            diffuse: self.diffuse.as_ref().map(to_material_colour),
            specular0: self.specular0.as_ref().map(to_material_colour),
            specular1: self.specular1.as_ref().map(to_material_colour),
            emission: self.emission.as_ref().map(to_material_colour),
        };

        let light_env = gpu.light_env_mut();

        light_env.connect_lut(LightLutId::D0, LutInput::LightNormal, LutData::phong(30.0));
        light_env.set_material(mat);
    }
}
