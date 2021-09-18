#[derive(Clone)]
pub struct MaterialData {
    pub is_transparent: bool,
    pub is_solid: bool,
    // Face ordering is the same as defined in
    // CubeFace
    pub texture: Option<[u16; 6]>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Material {
    Empty,
    Dirt,
    Grass,
}
impl Material {
    pub const fn data(self) -> MaterialData {
        match self {
            Self::Empty => MaterialData {
                is_transparent: true,
                is_solid: false,
                texture: None,
            },
            Self::Dirt => MaterialData {
                is_transparent: false,
                is_solid: true,
                texture: Some([1, 1, 1, 1, 1, 1]),
            },
            Self::Grass => MaterialData {
                is_transparent: false,
                is_solid: true,
                texture: Some([0, 0, 0, 0, 0, 0]),
            },
        }
    }
}
