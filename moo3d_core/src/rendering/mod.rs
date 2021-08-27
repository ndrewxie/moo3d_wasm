use std::cmp;

pub mod gfx;
pub mod rendermath;

use gfx::{Color, Texture};
use rendermath::{Vector, Matrix, Point3D, RenderMatrices};

#[repr(C)]
pub struct Camera {
    pub position: Point3D,
    pub target: Point3D,
    pub near: isize,
    pub far: isize,
}

#[derive(Clone, Copy)]
pub enum Choice<A, B> {
    First(A),
    Second(B),
} 

#[repr(C)]
pub struct Renderer {
    pub width: usize,
    pub height: usize,
    pixels: Vec<u8>,
    z_buffer: Vec<f32>,
    pub camera: Camera,
}
impl Renderer {
    pub fn new(width: usize, height: usize, fov_horizontal: f32) -> Self {
        let near = (width as f32 / 2.0) / (fov_horizontal / 2.0).tan();
        let far = near * 200.0;
        Self {
            pixels: vec![0; 4 * width * height],
            z_buffer: vec![-1.0; width * height],
            width,
            height,
            camera: Camera::new(
                Point3D::from_euc_coords(width as isize/2, height as isize/2, 0),
                Point3D::from_euc_coords(width as isize/2, height as isize/2, 1),
                near as isize,
                far as isize,
            )
        }
    }
    pub fn clear(&mut self, color: &Color) {
        for y_coord in 0..self.height {
            for x_coord in 0..self.width {
                self.write_pixel(x_coord as isize, y_coord as isize, -1.0, color);
            }
        }
    }
    pub fn get_pixels(&self) -> &[u8] {
        &self.pixels
    }
    pub fn get_mut_pixels(&mut self) -> &mut [u8] {
        &mut self.pixels
    }
    pub fn get_near(&self) -> isize {
        self.camera.near
    }
    pub fn get_far(&self) -> isize {
        self.camera.far
    }
    #[inline(always)]
    pub fn write_pixel(&mut self, x: isize, y: isize, z: f32, color: &Color) {
        if x >= self.width as isize || x < 0 {
            return;
        }
        if y >= self.height as isize || y < 0 {
            return;
        }
        if z >= 0.0 && z < self.z_buffer[y as usize * self.width + x as usize] {
            return;
        }

        let offset = 4 * (y as usize * self.width + x as usize);
        self.pixels[offset] = color.r;
        self.pixels[offset+1] = color.g;
        self.pixels[offset+2] = color.b;
        self.pixels[offset+3] = color.a;
        self.z_buffer[y as usize * self.width + x as usize] = z;
    }
    pub fn write_line(&mut self, p1: &Point3D, p2: &Point3D, color: &Color) {
        let x1 = p1.x_coord();
        let y1 = p1.y_coord();
        let z1 = p1.z_coord_float();
        
        let x2 = p2.x_coord();
        let y2 = p2.y_coord();
        let z2 = p2.z_coord_float();

        let mut dx = x2 as f32 - x1 as f32;
        let mut dy = y2 as f32 - y1 as f32;
        let step = {
            if dx.abs() >= dy.abs() {
                dx.abs()
            }
            else {
                dy.abs()
            }
        };

        if step == 0.0 {
            return;
        }
        else {
            dx /= step;
            dy /= step;
        }

        let mut x = x1 as f32;
        let mut y = y1 as f32;
        let mut i: usize = 1;
        while i <= step as usize {
            let t = (i - 1) as f32 / step;
            self.write_pixel(
                x as isize,
                y as isize, 
                z1 * (1.0 - t) + z2 * t,
                color
            );
            x += dx;
            y += dy;
            i += 1;
        }
    }
    // write_square is always at top z_indx cuz it's just a debugging function
    // Also I'm too lazy to write 2 lines to do interp
    pub fn write_square(&mut self, x: isize, y: isize, sidelen: isize, color: &Color) {
        let halfside = sidelen as isize / 2;
        for indx in -halfside..halfside {
            if x + indx >= 0 {
                for indy in -halfside..halfside {
                    if y + indy >= 0 {
                        self.write_pixel(x + indx, y + indy, 1.0, color);
                    }
                }
            } 
        }
    }
    #[inline(always)]
    fn bary_sign(px: f32, py: f32, v1x: f32, v1y: f32, v2x: f32, v2y: f32) -> f32 {
        (px - v2x) * (v1y - v2y) - (v1x - v2x) * (py - v2y)
    }
    // write_triangle is always at top cuz I'm too lazy to do barycentric interp
    // TODO: handle barycentric interp
    pub fn write_triangle(&mut self, v1: &(usize, usize), v2: &(usize, usize), v3: &(usize, usize), color: &Color) {
        /*
        let min_x = cmp::min(cmp::min(v1.0, v2.0), v3.0);
        let min_y = cmp::min(cmp::min(v1.1, v2.1), v3.1);

        let max_x = cmp::max(cmp::max(v1.0, v2.0), v3.0);
        let max_y = cmp::max(cmp::max(v1.1, v2.1), v3.1);

        
        self.write_line(v1, v2, color);
        self.write_line(v1, v3, color);
        self.write_line(v2, v3, color);
        
        for indx in min_x..max_x {
            for indy in min_y..max_y {
                let s1 = Self::bary_sign(indx as f32, indy as f32, v1.0 as f32, v1.1 as f32, v2.0 as f32, v2.1 as f32);
                let s2 = Self::bary_sign(indx as f32, indy as f32, v2.0 as f32, v2.1 as f32, v3.0 as f32, v3.1 as f32);
                let s3 = Self::bary_sign(indx as f32, indy as f32, v3.0 as f32, v3.1 as f32, v1.0 as f32, v1.1 as f32);
                
                let has_negative = (s1 < 0.0) || (s2 < 0.0) || (s3 < 0.0);
                let has_positive = (s1 > 0.0) || (s2 > 0.0) || (s3 > 0.0);

                if !(has_negative && has_positive) {
                    if (indx >= 0) && (indx < self.width) && (indy >= 0) && (indy < self.height) {
                        self.write_pixel(indx, indy, 1.0, color);
                    }
                }
            }
        }
        */
    }
    pub fn draw_triface(&mut self, p1: &Point3D, p2: &Point3D, p3: &Point3D, texture: &Color) {
    }
    pub fn draw_cuboid(&mut self, position: &Point3D, orientation: &(f32, f32, f32), dimensions: &(usize, usize, usize)) {
        let x = position.x_coord();
        let y = position.y_coord();
        let z = position.z_coord();

        let (sx, sy, sz) = dimensions;
        let side_x = *sx as isize / 2;
        let side_y = *sy as isize / 2;
        let side_z = *sz as isize / 2;

        let (pitch, roll, yaw) = orientation;

        let vertices = RenderMatrices::bundle_points(&[
            Point3D::from_euc_coords(x - side_x, y - side_y, z + side_z),
            Point3D::from_euc_coords(x + side_x, y - side_y, z + side_z),
            Point3D::from_euc_coords(x - side_x, y + side_y, z + side_z),
            Point3D::from_euc_coords(x + side_x, y + side_y, z + side_z),
            Point3D::from_euc_coords(x - side_x, y - side_y, z - side_z),
            Point3D::from_euc_coords(x + side_x, y - side_y, z - side_z),
            Point3D::from_euc_coords(x - side_x, y + side_y, z - side_z),
            Point3D::from_euc_coords(x + side_x, y + side_y, z - side_z), 
        ]);

        let forward = self.camera.view().matrix_mul(&
            RenderMatrices::rotation_3d(
                *pitch, 
                *roll, 
                *yaw, 
                Some(&(x as f32, y as f32, z as f32))
            )
        );

        let reverse = self.camera.reverse();

        let transformed_vertices = RenderMatrices::split_points(
            &reverse.matrix_mul(&forward.matrix_mul(&vertices)),
        );

        let white = Color::new(255, 255, 255, 255);
        let c1 = Color::new(255, 255, 255, 255);
        let c2 = Color::new(255, 0, 0, 255);
        let c3 = Color::new(0, 255, 0, 255);
        let c4 = Color::new(0, 0, 255, 255);
        let c5 = Color::new(255, 255, 0, 255);
        let c6 = Color::new(0, 255, 255, 255);
        let c7 = Color::new(255, 0, 255, 255);

        for vertex in transformed_vertices.iter() {
            self.write_square(vertex.x_coord(), vertex.y_coord(), 5, &white);
        }

        self.write_line(&transformed_vertices[0], &transformed_vertices[1], &c1);
        self.write_line(&transformed_vertices[0], &transformed_vertices[2], &c2);
        self.write_line(&transformed_vertices[2], &transformed_vertices[3], &c3);
        self.write_line(&transformed_vertices[1], &transformed_vertices[3], &c4);

        self.write_line(&transformed_vertices[4], &transformed_vertices[5], &c5);
        self.write_line(&transformed_vertices[4], &transformed_vertices[6], &c6);
        self.write_line(&transformed_vertices[6], &transformed_vertices[7], &c7);
        self.write_line(&transformed_vertices[5], &transformed_vertices[7], &c1);

        self.write_line(&transformed_vertices[0], &transformed_vertices[4], &c2);
        self.write_line(&transformed_vertices[1], &transformed_vertices[5], &c3);
        self.write_line(&transformed_vertices[2], &transformed_vertices[6], &c4);
        self.write_line(&transformed_vertices[3], &transformed_vertices[7], &c5);
    }
}

impl Camera {
    pub fn new(position: Point3D, target: Point3D, near: isize, far: isize) -> Self {
        Self {
            position,
            target,
            near,
            far,
        }
    }
    pub fn translate(&mut self, dx: isize, dy: isize, dz: isize) {
        self.position.set_x_coord(self.position.x_coord() + dx);
        self.position.set_y_coord(self.position.y_coord() + dy);
        self.position.set_z_coord(self.position.z_coord() + dz);

        self.target.set_x_coord(self.target.x_coord() + dx);
        self.target.set_y_coord(self.target.y_coord() + dy);
        self.target.set_z_coord(self.target.z_coord() + dz);
    }
    pub fn look(&mut self, target: Point3D) {
        self.target = target;
    }
    pub fn view(&self) -> Matrix {
        let mut forward = (self.target.position.minus(&self.position.position));
        forward.normalize_inplace();
        let right = Vector::with_data(vec![0.0, 1.0, 0.0]).cross(&forward);
        let up = forward.cross(&right);

        let projection_matrix = RenderMatrices::projection(self.near, self.far);

        let coord_matrix = Matrix::with_2d_data(&vec![
            vec![right.get(0), up.get(0), forward.get(0), 0.0],
            vec![right.get(1), up.get(1), forward.get(1), 0.0],
            vec![right.get(2), up.get(2), forward.get(2), 0.0],
            vec![0.0,          0.0,       0.0,            1.0],
        ]);

        let translation_matrix = RenderMatrices::translation(-1.0 * (self.position.x_coord() as f32), -1.0 * (self.position.y_coord() as f32), -1.0 * (self.position.z_coord() as f32));

        projection_matrix.matrix_mul(&coord_matrix.matrix_mul(&translation_matrix))
    }
    pub fn reverse(&self) -> Matrix {
        RenderMatrices::translation(self.position.x_coord() as f32, self.position.y_coord() as f32, self.position.z_coord() as f32)
    }
}