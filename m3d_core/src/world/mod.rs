pub mod material;

pub use material::Material;

use crate::camera::{Camera, CameraCache, UNITS_PER_BLOCK};
use crate::rendering::{CubeFace, Renderer};
use crate::rendermath::{Point3D, RenderMatrices, Vector};

const BLOCK_BUNDLE_SIZE: usize = 16;

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Block,
    VPlank,
    HPlank,
    Mini, // 1/4th the size of normal blocks
}
#[derive(Clone, Copy, Debug)]
pub struct BlockData {
    pub shape: Shape,
    pub material: Material,
}
#[derive(Debug)]
pub enum Block {
    Full(BlockData),
    Multiple(Vec<BlockData>),
}
#[derive(Debug)]
pub struct BlockBundle {
    blocks: Vec<Block>,
}

pub struct World {
    pub camera: Camera,
    pub data: WorldData,
}
pub struct WorldData {
    pub bundles: Vec<BlockBundle>,
    pub world_bundle_size: usize,
    pub world_bundle_squared: usize,
    pub offset_x: usize,
    pub offset_y: usize,
    pub offset_z: usize,
}

impl World {
    pub fn new(camera: Camera, world_bundle_size: usize) -> Self {
        let offset_x = camera.data.position.x_coord() as usize / BLOCK_BUNDLE_SIZE;
        let offset_y = camera.data.position.y_coord() as usize / BLOCK_BUNDLE_SIZE;
        let offset_z = camera.data.position.z_coord() as usize / BLOCK_BUNDLE_SIZE;

        let num_bundles = world_bundle_size * world_bundle_size * world_bundle_size;
        let mut bundles: Vec<BlockBundle> = Vec::with_capacity(num_bundles);
        for _ in 0..num_bundles {
            bundles.push(BlockBundle::new());
        }
        Self {
            camera,
            data: WorldData {
                bundles,
                world_bundle_size,
                world_bundle_squared: world_bundle_size * world_bundle_size,
                offset_x,
                offset_y,
                offset_z,
            },
        }
    }
    fn full_faces_list(
        bundle: &BlockBundle,
        x: usize,
        y: usize,
        z: usize,
    ) -> [Option<CubeFace>; 6] {
        let mut to_return = [None; 6];
        let mut j: usize = 0;

        if x < 1 || !bundle.get(x - 1, y, z).is_occluder() {
            to_return[j] = Some(CubeFace::MinusX);
            j += 1;
        }
        if x + 1 >= BLOCK_BUNDLE_SIZE || !bundle.get(x + 1, y, z).is_occluder() {
            to_return[j] = Some(CubeFace::PlusX);
            j += 1;
        }

        if y < 1 || !bundle.get(x, y - 1, z).is_occluder() {
            to_return[j] = Some(CubeFace::MinusY);
            j += 1;
        }
        if y + 1 >= BLOCK_BUNDLE_SIZE || !bundle.get(x, y + 1, z).is_occluder() {
            to_return[j] = Some(CubeFace::PlusY);
            j += 1;
        }

        if z < 1 || !bundle.get(x, y, z - 1).is_occluder() {
            to_return[j] = Some(CubeFace::MinusZ);
            j += 1;
        }
        if z + 1 >= BLOCK_BUNDLE_SIZE || !bundle.get(x, y, z + 1).is_occluder() {
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
        base_x: f32,
        base_y: f32,
        base_z: f32,
        renderer: &mut Renderer,
    ) {
        let block = bundle.get(dx, dy, dz);
        if let Block::Full(block_data) = block {
            let material_data = block_data.material.data();
            if material_data.is_transparent {
                return;
            }

            let full_faces = Self::full_faces_list(bundle, dx, dy, dz);
            if full_faces[0].is_none() {
                return;
            }

            let halfsides = block_data.shape.halfsides();
            let center = Point3D::from_euc_coords_float(
                (dx * UNITS_PER_BLOCK) as f32 + halfsides[0] + base_x,
                (dy * UNITS_PER_BLOCK) as f32 + halfsides[1] + base_y,
                (dz * UNITS_PER_BLOCK) as f32 + halfsides[2] + base_z,
            );
            let transform = RenderMatrices::identity().matrix_mul(&CameraCache::reverse_frustum(
                &mut camera.cache.reverse_frustum,
                &camera.data,
            ));

            for element in full_faces {
                if let Some(face) = element {
                    renderer.draw_cubeface(camera, &center, face, &halfsides, &transform, 0);
                } else {
                    break;
                }
            }
        }
    }
    pub fn draw_bundle(
        world_data: &WorldData,
        camera: &mut Camera,
        u: usize,
        v: usize,
        w: usize,
        renderer: &mut Renderer,
    ) {
        let bundle_size = (BLOCK_BUNDLE_SIZE * UNITS_PER_BLOCK) as f32;

        let x = (u + world_data.offset_x) as f32 * bundle_size;
        let y = (v + world_data.offset_y) as f32 * bundle_size;
        let z = (w + world_data.offset_z) as f32 * bundle_size;

        if Camera::in_frustum(
            &Point3D::from_euc_coords_float(x, y, z),
            &mut camera.cache,
            &camera.data,
        )
        .is_some()
            || Camera::in_frustum(
                &Point3D::from_euc_coords_float(bundle_size + x, y, z),
                &mut camera.cache,
                &camera.data,
            )
            .is_some()
            || Camera::in_frustum(
                &Point3D::from_euc_coords_float(bundle_size + x, bundle_size + y, z),
                &mut camera.cache,
                &camera.data,
            )
            .is_some()
            || Camera::in_frustum(
                &Point3D::from_euc_coords_float(bundle_size + x, y, bundle_size + z),
                &mut camera.cache,
                &camera.data,
            )
            .is_some()
            || Camera::in_frustum(
                &Point3D::from_euc_coords_float(x, bundle_size + y, z),
                &mut camera.cache,
                &camera.data,
            )
            .is_some()
            || Camera::in_frustum(
                &Point3D::from_euc_coords_float(bundle_size + x, bundle_size + y, z),
                &mut camera.cache,
                &camera.data,
            )
            .is_some()
            || Camera::in_frustum(
                &Point3D::from_euc_coords_float(x, bundle_size + y, bundle_size + z),
                &mut camera.cache,
                &camera.data,
            )
            .is_some()
            || Camera::in_frustum(
                &Point3D::from_euc_coords_float(bundle_size + x, bundle_size + y, bundle_size + z),
                &mut camera.cache,
                &camera.data,
            )
            .is_some()
        {
            if let Some(bundle) = world_data.get_bundle(
                u + world_data.offset_x,
                v + world_data.offset_y,
                w + world_data.offset_z,
            ) {
                for indx in 0..BLOCK_BUNDLE_SIZE {
                    for indy in 0..BLOCK_BUNDLE_SIZE {
                        for indz in 0..BLOCK_BUNDLE_SIZE {
                            Self::draw_block(camera, &bundle, indx, indy, indz, x, y, z, renderer);
                        }
                    }
                }
            }
        }
    }
    pub fn draw_all(world_data: &WorldData, camera: &mut Camera, renderer: &mut Renderer) {
        for indx in 0..world_data.world_bundle_size {
            for indy in 0..world_data.world_bundle_size {
                for indz in 0..world_data.world_bundle_size {
                    Self::draw_bundle(world_data, camera, indx, indy, indz, renderer);
                }
            }
        }
    }
}
impl WorldData {
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
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<(&Block, &BlockBundle)> {
        let bundle = self.get_bundle(
            x / BLOCK_BUNDLE_SIZE,
            y / BLOCK_BUNDLE_SIZE,
            z / BLOCK_BUNDLE_SIZE,
        )?;
        let block = bundle.get(
            x % BLOCK_BUNDLE_SIZE,
            y % BLOCK_BUNDLE_SIZE,
            z % BLOCK_BUNDLE_SIZE,
        );
        Some((block, bundle))
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
}

impl Block {
    pub fn is_full(&self) -> bool {
        match self {
            Self::Full(_) => true,
            _ => false,
        }
    }
    pub fn is_occluder(&self) -> bool {
        match self {
            Self::Full(block_data) => !block_data.material.data().is_transparent,
            _ => false,
        }
    }
    pub fn new() -> Self {
        Self::Full(BlockData {
            shape: Shape::Block,
            material: Material::Empty,
        })
    }
}

impl Shape {
    pub fn halfsides(&self) -> [f32; 3] {
        match self {
            Self::Block => [2.0, 2.0, 2.0],
            Self::VPlank => [2.0, 2.0, 1.0],
            Self::HPlank => [2.0, 1.0, 2.0],
            Self::Mini => [0.5, 0.5, 0.5],
        }
    }
}

impl BlockData {
    pub fn new(shape: Shape, material: Material) -> Self {
        Self { shape, material }
    }
}

impl BlockBundle {
    pub fn new() -> Self {
        let num_blocks = BLOCK_BUNDLE_SIZE * BLOCK_BUNDLE_SIZE * BLOCK_BUNDLE_SIZE;
        let mut blocks: Vec<Block> = Vec::with_capacity(num_blocks);
        for _ in 0..num_blocks {
            blocks.push(Block::new());
        }
        Self { blocks }
    }
    pub fn get(&self, x: usize, y: usize, z: usize) -> &Block {
        &self.blocks[BLOCK_BUNDLE_SIZE * BLOCK_BUNDLE_SIZE * z + BLOCK_BUNDLE_SIZE * y + x]
    }
    pub fn get_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Block {
        &mut self.blocks[BLOCK_BUNDLE_SIZE * BLOCK_BUNDLE_SIZE * z + BLOCK_BUNDLE_SIZE * y + x]
    }
}
