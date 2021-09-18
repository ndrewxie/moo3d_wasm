#![allow(dead_code)]

mod etc;
pub mod player;
pub mod rendering;
pub mod world;

pub use etc::camera;
pub use etc::rendermath;

use rendering::gfx::{Color, FarLight, Light, NearLight};
use rendermath::{Point3D, RenderMatrices, Vector};
use world::{Block, BlockData, Material, Shape, World};

pub struct GameState {
    last_frame: usize,
    renderer: rendering::Renderer,
    world: World,
}
impl GameState {
    pub fn new(width: usize, height: usize, texture_array: &[u8]) -> Self {
        let game_camera = camera::Camera::new(
            Point3D::from_euc_coords(0, 0, 0),
            (0.0, 0.0),
            std::f32::consts::PI * (135.0 / 180.0),
            100,
            width,
            height,
        );
        let mut world = World::new(game_camera, 10);
        for indx in 0..150 {
            for indy in 0..150 {
                *world.data.get_mut(indx, indy, 5).unwrap() = Block::Full(BlockData {
                    shape: Shape::Block,
                    material: if indx % 2 == 0 {
                        Material::Dirt
                    } else {
                        Material::Grass
                    },
                });
            }
        }
        world.data.lights = vec![
            Light::Near(NearLight::new(
                Color::new(255, 0, 0, 255),
                3500,
                Point3D::from_euc_coords_float(5.0, 5.0, -10.0),
            )),
            Light::Near(NearLight::new(
                Color::new(0, 255, 0, 255),
                3000,
                Point3D::from_euc_coords_float(50.0, 50.0, -10.0),
            )),
            Light::Far(FarLight::new(
                Color::new(255, 255, 255, 255),
                120,
                Vector::with_data(vec![0.0, 1.0, 0.0]),
            )),
        ];
        Self {
            renderer: rendering::Renderer::new(&world.camera, width, height, texture_array),
            world,
            last_frame: 0,
        }
    }
    pub fn get_pixels(&self) -> &[u8] {
        self.renderer.get_pixels()
    }
    pub fn get_mut_pixels(&mut self) -> &mut [u8] {
        self.renderer.get_mut_pixels()
    }
    pub fn render_world(&mut self, _curr_time: usize) {
        self.renderer.clear();
        World::draw_all(&self.world.data, &mut self.world.camera, &mut self.renderer);
    }
    pub fn render(&mut self, curr_time: usize) {
        self.render_world(curr_time);
    }
    pub fn translate_camera(&mut self, trans_x: isize, trans_y: isize, trans_z: isize) {
        self.world.camera.translate(trans_x, trans_y, trans_z);
    }
    pub fn rotate_camera(&mut self, d_rotation: f32, d_inclination: f32) {
        self.world.camera.translate_look(d_rotation, d_inclination);
    }
}
