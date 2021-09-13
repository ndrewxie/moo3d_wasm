pub mod material;
pub use material::Material;

use crate::camera::Camera;
use crate::rendering::{CubeFace, Renderer};

const BLOCK_BUNDLE_SIZE: usize = 32;

#[derive(Clone, Copy)]
pub enum Shape {
    Block,
    VPlank,
    HPlank,
    Mini, // 1/4th the size of normal blocks
}
#[derive(Clone, Copy)]
pub struct BlockData {
    pub shape: Shape,
    pub material: Material,
}

pub enum Block {
    Full(BlockData),
    Multiple(Vec<BlockData>),
}

pub struct BlockBundle {
    blocks: [Block; BLOCK_BUNDLE_SIZE * BLOCK_BUNDLE_SIZE * BLOCK_BUNDLE_SIZE],
}

pub struct World {
    camera: Camera,
    bundles: Vec<BlockBundle>,
    world_bundle_size: usize,
    world_bundle_squared: usize,
    offset_x: usize,
    offset_y: usize,
    offset_z: usize,
    location_x: usize, // relative to offset bundle
    location_y: usize,
    location_z: usize,
}

impl BlockData {
    pub fn new(shape: Shape, material: Material) -> Self {
        Self { shape, material }
    }
}

impl BlockBundle {
    pub fn get(&self, x: usize, y: usize, z: usize) -> &Block {
        &self.blocks[BLOCK_BUNDLE_SIZE * BLOCK_BUNDLE_SIZE * z + BLOCK_BUNDLE_SIZE * y + x]
    }
    pub fn get_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        &mut self.blocks[BLOCK_BUNDLE_SIZE * BLOCK_BUNDLE_SIZE * z + BLOCK_BUNDLE_SIZE * y + x]
    }
}

impl World {
    pub fn get_bundle(
        &self,
        bundle_x: usize,
        bundle_y: usize,
        bundle_z: usize,
    ) -> Option<&BlockBundle> {
        if bundle_x < self.offset_x || bundle_y < self.offset_y || bundle_z < self.offset_z {
            return None;
        }

        let x_coord = bundle_x - self.offset_x;
        let y_coord = bundle_y - self.offset_y;
        let z_coord = bundle_z - self.offset_z;

        if x_coord >= self.world_bundle_size
            || y_coord >= self.world_bundle_size
            || z_coord >= self.world_bundle_size
        {
            return None;
        }

        unsafe {
            Some(self.bundles.get_unchecked(
                z_coord * self.world_bundle_squared + y_coord * self.world_bundle_size + x_coord,
            ))
        }
    }
    pub fn get_bundle_mut(
        &mut self,
        bundle_x: usize,
        bundle_y: usize,
        bundle_z: usize,
    ) -> Option<&mut BlockBundle> {
        if bundle_x < self.offset_x || bundle_y < self.offset_y || bundle_z < self.offset_z {
            return None;
        }

        let x_coord = bundle_x - self.offset_x;
        let y_coord = bundle_y - self.offset_y;
        let z_coord = bundle_z - self.offset_z;

        if x_coord >= self.world_bundle_size
            || y_coord >= self.world_bundle_size
            || z_coord >= self.world_bundle_size
        {
            return None;
        }

        unsafe {
            Some(self.bundles.get_unchecked_mut(
                z_coord * self.world_bundle_squared + y_coord * self.world_bundle_size + x_coord,
            ))
        }
    }
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        let bundle = self.get_bundle(
            x / BLOCK_BUNDLE_SIZE,
            y / BLOCK_BUNDLE_SIZE,
            z / BLOCK_BUNDLE_SIZE,
        )?;
        Some(bundle.get(
            x % BLOCK_BUNDLE_SIZE,
            y % BLOCK_BUNDLE_SIZE,
            z % BLOCK_BUNDLE_SIZE,
        ))
    }
    pub fn get_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut Block> {
        let bundle = self.get_bundle_mut(
            x / BLOCK_BUNDLE_SIZE,
            y / BLOCK_BUNDLE_SIZE,
            z / BLOCK_BUNDLE_SIZE,
        )?;
        Some(bundle.get_mut(
            x % BLOCK_BUNDLE_SIZE,
            y % BLOCK_BUNDLE_SIZE,
            z % BLOCK_BUNDLE_SIZE,
        ))
    }
    fn full_faces_list(
        bundle: &BlockBundle,
        x: usize,
        y: usize,
        z: usize,
    ) -> [Option<CubeFace>; 6] {
        let mut to_return = [None; 6];
        let mut j: usize = 0;

        if x < 1 || !bundle.get(x - 1, y, z).is_full() {
            to_return[j] = Some(CubeFace::MinusX);
            j += 1;
        }
        if x < BLOCK_BUNDLE_SIZE - 1 || !bundle.get(x + 1, y, z).is_full() {
            to_return[j] = Some(CubeFace::PlusX);
            j += 1;
        }

        if y < 1 || !bundle.get(x, y - 1, z).is_full() {
            to_return[j] = Some(CubeFace::MinusY);
            j += 1;
        }
        if y < BLOCK_BUNDLE_SIZE - 1 || !bundle.get(x, y + 1, z).is_full() {
            to_return[j] = Some(CubeFace::PlusY);
            j += 1;
        }

        if z < 1 || !bundle.get(x, y, z - 1).is_full() {
            to_return[j] = Some(CubeFace::MinusZ);
            j += 1;
        }
        if z < BLOCK_BUNDLE_SIZE - 1 || !bundle.get(x, y, z + 1).is_full() {
            to_return[j] = Some(CubeFace::PlusZ);
            j += 1;
        }
        to_return
    }
    pub fn draw_block(
        camera: &mut Camera,
        bundle: &BlockBundle,
        dx: usize,
        dy: usize,
        dz: usize,
        renderer: &mut Renderer,
    ) {
        let block = bundle.get(dx, dy, dz);
        match block {
            Block::Full(block_data) => {
                let full_faces = Self::full_faces_list(bundle, dx, dy, dz);
                for element in full_faces {
                    if let Some(face) = element {
                        /*
                        renderer.draw_cubeface(
                            &Point3D::from_euc_coords(),

                        );
                        */
                    } else {
                        break;
                    }
                }
            }
            Block::Multiple(block_data_vec) => {}
        }
    }
    pub fn draw_bundle(&self, u: usize, v: usize, w: usize, renderer: &mut Renderer) {}
}

impl Block {
    pub fn is_full(&self) -> bool {
        match self {
            Self::Full(_) => true,
            _ => false,
        }
    }
}
