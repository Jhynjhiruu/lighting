use citro3d::texture::{Tex, TexParams};

pub struct Texture {
    width: u16,
    height: u16,
    data: Vec<u8>,
}

impl std::fmt::Debug for Texture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Texture")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl Texture {
    pub fn new(width: u16, height: u16, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }
}

#[derive(Debug)]
pub struct GPUTexture {
    tex: Tex,
}

impl From<&Texture> for GPUTexture {
    fn from(value: &Texture) -> Self {
        let t = Tex::new(TexParams::new_2d(value.width, value.height)).unwrap();
        t.upload(&value.data);
        Self { tex: t }
    }
}

impl GPUTexture {
    pub fn bind(&self, unit_id: i32) {
        self.tex.bind(unit_id)
    }
}
