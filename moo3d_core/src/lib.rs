#![allow(dead_code)]

pub mod rendering;

use rendering::gfx;
use rendering::gfx::{Color, Texture};
use rendering::rendermath::{Matrix, Point3D, Vector};

#[repr(C)]
pub struct GameState {
    last_frame: usize,
    renderer: rendering::Renderer,
    textures: Vec<Texture>,
}
impl GameState {
    pub fn new(width: usize, height: usize, texture_array: &[u8]) -> Self {
        let texture_slice_len = (4 * (gfx::TEXTURE_LEN + 1)) as usize;

        let mut textures: Vec<Texture> = Vec::new();

        assert_eq!(texture_array.len() % texture_slice_len, 0);

        let mut acc: Vec<Color> = Vec::new();
        for pixel_indx in (0..texture_array.len()).step_by(4) {
            acc.push(Color::new(
                texture_array[pixel_indx],
                texture_array[pixel_indx + 1],
                texture_array[pixel_indx + 2],
                texture_array[pixel_indx + 3],
            ));
            if acc.len() as isize == gfx::TEXTURE_LEN + 1 {
                textures.push(Texture::new(acc));
                acc = Vec::new();
            }
        }

        Self {
            renderer: rendering::Renderer::new(width, height, 120.0 * std::f32::consts::PI / 180.0),
            last_frame: 0,
            textures,
        }
    }
    pub fn get_pixels(&self) -> &[u8] {
        self.renderer.get_pixels()
    }
    pub fn get_mut_pixels(&mut self) -> &mut [u8] {
        self.renderer.get_mut_pixels()
    }
    pub fn render(&mut self, curr_time: usize) {
        let angle = (curr_time / 50) as f32 * std::f32::consts::PI / 180.0;

        let center_x = self.renderer.width / 2;
        let center_y = self.renderer.height / 2;

        let near = self.renderer.get_near() as isize;

        self.renderer.clear();

        for i in -15..15 {
            for j in -5..15 {
                self.renderer.draw_cuboid(
                    &Point3D::from_euc_coords(
                        center_x as isize + near * i,
                        center_y as isize + 2 * near,
                        5 * near + near * j,
                    ),
                    &(0.0, 0.0, 0.0),
                    &[near as f32, near as f32, near as f32],
                    if i % 2 == 0 {
                        &self.textures[1]
                    } else {
                        &self.textures[0]
                    },
                );
            }
        }
        /*
        self.renderer.draw_cuboid(
            &Point3D::from_euc_coords(center_x as isize, center_y as isize, 5 * near),
            &(angle, 0.0, angle),
            &[near as f32, near as f32, near as f32],
            &self.textures[0],
        );
        */
    }
    pub fn translate_camera(&mut self, trans_x: isize, trans_y: isize, trans_z: isize) {
        self.renderer.camera.translate(trans_x, trans_y, trans_z);
    }
    pub fn translate_camera_look(&mut self, trans_x: f32, trans_y: f32, trans_z: f32) {
        self.renderer
            .camera
            .translate_look(trans_x, trans_y, trans_z);
    }
}
