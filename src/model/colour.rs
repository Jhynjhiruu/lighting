use citro3d::math::FVec4;

#[derive(Debug, Clone, Copy)]
pub struct Colour([u8; 4]);

impl Colour {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([r, g, b, a])
    }

    pub fn r(&self) -> u8 {
        self.0[0]
    }

    pub fn g(&self) -> u8 {
        self.0[1]
    }

    pub fn b(&self) -> u8 {
        self.0[2]
    }

    pub fn a(&self) -> u8 {
        self.0[3]
    }
}

impl From<&Colour> for FVec4 {
    fn from(val: &Colour) -> Self {
        let [r, g, b, a] = val.0;
        let (r, g, b, a) = (
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        );
        FVec4::new(r, g, b, a)
    }
}

impl From<&Colour> for citro3d::material::Color {
    fn from(value: &Colour) -> Self {
        let col: FVec4 = value.into();
        Self {
            r: col.x(),
            g: col.y(),
            b: col.z(),
        }
    }
}
