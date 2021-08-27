#![allow(dead_code)]

pub mod rendering;

use rendering::rendermath::{Point3D, Vector, Matrix};
use rendering::gfx::{Color, Texture};

#[repr(C)]
pub struct GameState {
    last_frame: usize,
    renderer: rendering::Renderer,
}
impl GameState {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            renderer: rendering::Renderer::new(width, height, 120.0 * std::f32::consts::PI / 180.0),
            last_frame: 0,
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

        self.renderer.clear(&Color::new(0, 0, 0, 255));
        self.renderer.draw_cuboid(
            &Point3D::from_euc_coords(center_x as isize, center_y as isize, 5 * near),
            &(
                std::f32::consts::PI / 3.0,
                std::f32::consts::PI / 4.0, 
                0.0,
            ),
            &(near as usize, near as usize, near as usize)
        );
        self.renderer.write_triangle(
            &(center_x/3, center_y/4),
            &(center_x/4, center_y/5),
            &(center_x/5, center_y/3),
            &Color::new(255, 255, 255, 255),
        );
    }
    pub fn translate_camera(&mut self, trans_x: isize, trans_y: isize, trans_z: isize) {
        self.renderer.camera.translate(trans_x, trans_y, trans_z);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cuboid() {
        let mut gs_manager = GameState::new(1918, 959);
        gs_manager.render(5);
    }
}