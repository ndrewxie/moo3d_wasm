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
            z_buffer: vec![100000.0; width * height],
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
        for indx in 0..self.width {
            for indy in 0..self.height {
                self.internal_write_pixel(indx as isize, indy as isize, 100000.0, color, false, false);
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
    fn to_render(&self, x: isize, y: isize, z: Option<f32>) -> bool {
        if x < 0 || x >= self.width as isize {
            return false;
        }
        if y < 0 || y >= self.height as isize {
            return false;
        }
        if z.is_some() {
            let unwrapped = z.unwrap();
            if unwrapped > 1.0 || unwrapped < 0.0 {
                return false;
            }    
        }
        return true;
    }
    #[inline(always)]
    fn internal_write_pixel(&mut self, x: isize, y: isize, z: f32, color: &Color, respect_bounds: bool, respect_z: bool) {
        if respect_bounds && !self.to_render(x, y, Some(z)) {
            return;
        }
		let pixel_offset = y as usize * self.width + x as usize;
        if respect_z && z > self.z_buffer[pixel_offset] {
            return;
        }

        let offset = 4 * pixel_offset;
        self.pixels[offset] = color.r;
        self.pixels[offset+1] = color.g;
        self.pixels[offset+2] = color.b;
        self.pixels[offset+3] = color.a;
        self.z_buffer[pixel_offset] = z;
    }
    #[inline(always)]
    pub fn write_pixel(&mut self, x: isize, y: isize, z: f32, color: &Color) {
        self.internal_write_pixel(x, y, z, color, true, true);
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
    pub fn write_square(&mut self, p: &Point3D, sidelen: isize, color: &Color) {
        let x = p.x_coord();
        let y = p.y_coord();
        let z = p.z_coord_float();

        if !self.to_render(x, y, Some(z)) {
            return;
        }

        let halfside = sidelen as isize / 2;
        for indx in -halfside..halfside {
            for indy in -halfside..halfside {
                if self.to_render(x + indx, y + indy, None) {
                    self.write_pixel(x + indx, y + indy, 1.0, color);
                }
            }
        }
    }
    fn interp_barycentric(u: f32, v: f32, w: f32, value_a: f32, value_b: f32, value_c: f32) -> f32 {
        let to_return = (1.0 / value_a) * u + (1.0 / value_b) * v + (1.0 / value_c) * w;
        1.0 / to_return
    }
    pub fn draw_triface(&mut self, v1: &Point3D, v2: &Point3D, v3: &Point3D, color: &Color) {
        let vertices = RenderMatrices::bundle_points(&[v1, v2, v3]);

        let forward = self.camera.view();
        let reverse = self.camera.reverse();

        let proj = RenderMatrices::split_points(
            &reverse.matrix_mul(&forward.matrix_mul(&vertices))
        );

        let p1 = (proj[0].x_coord(), proj[0].y_coord(), proj[0].z_coord_float());
        let p2 = (proj[1].x_coord(), proj[1].y_coord(), proj[1].z_coord_float());
        let p3 = (proj[2].x_coord(), proj[2].y_coord(), proj[2].z_coord_float());

        let render1 = self.to_render(p1.0, p1.1, Some(p1.2));
        let render2 = self.to_render(p2.0, p2.1, Some(p2.2));
        let render3 = self.to_render(p3.0, p3.1, Some(p3.2));
		let respect_bounds = !(render1 && render2 && render3);
        if (!render1) && (!render2) && (!render3) {
            return;
        }

        let min_x = cmp::min(cmp::min(p1.0, p2.0), p3.0);
        let min_y = cmp::min(cmp::min(p1.1, p2.1), p3.1);
        let max_x = cmp::max(cmp::max(p1.0, p2.0), p3.0);
        let max_y = cmp::max(cmp::max(p1.1, p2.1), p3.1);
    
		let (mut row_u, mut row_v, mut row_w, dudx, dvdx, dwdx, dudy, dvdy, dwdy) = RenderMatrices::barycentric_params(
			min_x as f32, min_y as f32,
            p1.0 as f32, p1.1 as f32,
            p2.0 as f32, p2.1 as f32,
            p3.0 as f32, p3.1 as f32,
		);
		for indy in min_y..max_y {
			let mut column_u: f32 = 0.0;
			let mut column_v: f32 = 0.0;
			let mut column_w: f32 = 0.0;
            for indx in min_x..max_x {
				let u = row_u + column_u;
				let v = row_v + column_v;
				let w = row_w + column_w;
                if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                    let z = Self::interp_barycentric(u, v, w, p1.2, p2.2, p3.2);
                    self.internal_write_pixel(indx as isize, indy as isize, z, color, respect_bounds, true);
                }
				column_u += dudx;
				column_v += dvdx;
				column_w += dwdx;
            }
			row_u += dudy;
			row_v += dvdy;
			row_w += dwdy;
        }
    }
    pub fn draw_quadface(&mut self, v1: &Point3D, v2: &Point3D, v3: &Point3D, v4: &Point3D, color: &Color) {
        self.draw_triface(v1, v2, v3, &color);
        self.draw_triface(v1, v3, v4, &color);
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

        let vertices = RenderMatrices::split_points(
            &RenderMatrices::rotation_3d(
                *pitch, 
                *roll, 
                *yaw, 
                Some(&(x as f32, y as f32, z as f32))
            ).matrix_mul(&RenderMatrices::bundle_points(&[
                &Point3D::from_euc_coords(x - side_x, y - side_y, z + side_z), // 0
                &Point3D::from_euc_coords(x + side_x, y - side_y, z + side_z), // 1
                &Point3D::from_euc_coords(x + side_x, y + side_y, z + side_z), // 2
                &Point3D::from_euc_coords(x - side_x, y + side_y, z + side_z), // 3
                &Point3D::from_euc_coords(x - side_x, y - side_y, z - side_z), // 4
                &Point3D::from_euc_coords(x + side_x, y - side_y, z - side_z), // 5
                &Point3D::from_euc_coords(x + side_x, y + side_y, z - side_z), // 6
                &Point3D::from_euc_coords(x - side_x, y + side_y, z - side_z), // 7
            ]))
        );

        let c1 = Color::new(255, 255, 255, 255);
        let c2 = Color::new(255, 0, 0, 255);
        let c3 = Color::new(0, 255, 0, 255);
        let c4 = Color::new(0, 0, 255, 255);
        let c5 = Color::new(255, 255, 0, 255);
        let c6 = Color::new(0, 255, 255, 255);
        let c7 = Color::new(255, 0, 255, 255);

        self.draw_quadface(&vertices[0], &vertices[1], &vertices[2], &vertices[3], &c1);
        self.draw_quadface(&vertices[4], &vertices[5], &vertices[6], &vertices[7], &c2);
        
        self.draw_quadface(&vertices[2], &vertices[6], &vertices[5], &vertices[1], &c3);
        self.draw_quadface(&vertices[0], &vertices[3], &vertices[7], &vertices[4], &c4);

        self.draw_quadface(&vertices[0], &vertices[1], &vertices[5], &vertices[4], &c5);
        self.draw_quadface(&vertices[3], &vertices[2], &vertices[6], &vertices[7], &c6);
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
    pub fn translate_look(&mut self, dx: f32, dy: f32, dz: f32) {
        self.target.set(0, self.target.get(0) + dx);
        self.target.set(1, self.target.get(1) + dy);
        self.target.set(2, self.target.get(2) + dz);
    }
    pub fn look(&mut self, target: Point3D) {
        self.target = target;
    }
    pub fn view(&self) -> Matrix {
        let mut forward = self.target.position.minus(&self.position.position);
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
        RenderMatrices::translation(self.position.x_coord() as f32, self.position.y_coord() as f32, 0.0)
    }
}