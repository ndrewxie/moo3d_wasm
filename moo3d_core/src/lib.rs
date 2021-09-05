#![allow(dead_code)]

pub mod rendering;

use rendering::rendermath::{Point3D, RenderMatrices};

#[repr(C)]
pub struct GameState {
    last_frame: usize,
    renderer: rendering::Renderer,
}
impl GameState {
    pub fn new(width: usize, height: usize, texture_array: &[u8]) -> Self {
        Self {
            renderer: rendering::Renderer::new(
                width,
                height,
                120.0 * std::f32::consts::PI / 180.0,
                texture_array,
            ),
            last_frame: 0,
        }
    }
    pub fn get_pixels(&self) -> &[u8] {
        self.renderer.get_pixels()
    }
    pub fn get_mut_pixels(&mut self) -> &mut [u8] {
        self.renderer.get_mut_pixels()
    }
    pub fn render_cubeplane(&mut self, _curr_time: usize) {
        let center_x = self.renderer.width / 2;
        let center_y = self.renderer.height / 2;
        let near = self.renderer.get_near() as isize;
        let scale = self.renderer.scale as isize;
        self.renderer.clear();

        for i in -15..15 {
            for j in -20..15 {
                self.renderer.draw_cuboid(
                    &Point3D::from_euc_coords(
                        center_x as isize + scale * i,
                        center_y as isize + 2 * scale,
                        5 * near + scale * j,
                    ),
                    &(0.0, 0.0, 0.0),
                    &[scale as f32, scale as f32, scale as f32],
                    if i % 2 == 0 { 1 } else { 0 },
                );
            }
        }
    }
    pub fn render_faceplane(&mut self, _curr_time: usize) {
        let center_x = self.renderer.width / 2;
        let center_y = self.renderer.height / 2;
        let near = self.renderer.get_near() as isize;
        let scale = self.renderer.scale as isize;
        self.renderer.clear();

        for i in -15..15 {
            for j in -20..15 {
                self.renderer.draw_cubeface(
                    &Point3D::from_euc_coords(
                        center_x as isize + scale * i,
                        center_y as isize + 2 * scale,
                        5 * near + scale * j,
                    ),
                    5,
                    &[scale as f32 * 0.5, scale as f32 * 0.5, scale as f32 * 0.5],
                    &RenderMatrices::identity(),
                    if i % 2 == 0 { 1 } else { 0 },
                );
            }
        }
    }
    pub fn render_spinningcube(&mut self, curr_time: usize) {
        let center_x = self.renderer.width / 2;
        let center_y = self.renderer.height / 2;
        let near = self.renderer.get_near() as isize;
        self.renderer.clear();

        let angle = (curr_time / 50) as f32 * std::f32::consts::PI / 180.0;

        self.renderer.draw_cuboid(
            &Point3D::from_euc_coords(center_x as isize, center_y as isize, 5 * near),
            &(angle, 0.0, angle),
            &[near as f32, near as f32, near as f32],
            0,
        );
    }
    pub fn render(&mut self, curr_time: usize) {
        //self.render_spinningcube(curr_time);
        //self.render_cubeplane(curr_time);
        self.render_faceplane(curr_time);
    }
    pub fn translate_camera(&mut self, trans_x: isize, trans_y: isize, trans_z: isize) {
        self.renderer.camera.translate(trans_x, trans_y, trans_z);
    }
    pub fn rotate_camera(&mut self, d_rotation: f32, d_inclination: f32) {
        self.renderer
            .camera
            .translate_look(d_rotation, d_inclination);
    }
}
