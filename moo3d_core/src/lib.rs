#![allow(dead_code)]

pub mod rendering;

use rendering::gfx::{Color, Texture};
use rendering::rendermath::{Matrix, Point3D, Vector};

#[repr(C)]
pub struct GameState {
    last_frame: usize,
    renderer: rendering::Renderer,
    textures: Vec<Texture>
}
impl GameState {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            renderer: rendering::Renderer::new(width, height, 120.0 * std::f32::consts::PI / 180.0),
            last_frame: 0,
            textures: vec![Texture::checkerboard()]
        }
    }
    pub fn get_pixels(&self) -> &[u8] {
        self.renderer.get_pixels()
    }
    pub fn get_mut_pixels(&mut self) -> &mut [u8] {
        self.renderer.get_mut_pixels()
    }
    pub fn render(&mut self, curr_time: usize) {
        let center_x = self.renderer.width / 2;
        let center_y = self.renderer.height / 2;

        let near = self.renderer.get_near() as isize;

        self.renderer.clear();
        for i in -5..5 {
            for j in -3..3 {
                self.renderer.draw_cuboid(
                    &Point3D::from_euc_coords(
                        center_x as isize + near * i,
                        center_y as isize + near * j,
                        5 * near,
                    ),
                    &(0.0, 0.0, 0.0),
                    &(near as usize, near as usize, near as usize),
                    &self.textures[0],
                );
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cuboid() {
        let mut gs_manager = GameState::new(1918, 959);
        gs_manager.renderer.camera.translate(0, 0, 2300);
        for j in 0..300 {
            gs_manager.render(j);
        }
    }
}
