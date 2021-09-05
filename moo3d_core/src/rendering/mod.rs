use std::cmp;

pub mod camera;
pub mod gfx;
pub mod rendermath;

use camera::{Camera, CameraCache};
use gfx::{Color, Light, Texture, MTEXCOORD};
use rendermath::{Matrix, Point3D, RenderMatrices, Vector};

struct PixelIterator {
    width: usize,
    height: usize,

    x: usize,
    y: usize,

    pub pixel_offset: usize,
    pub offset: usize,
}
#[repr(C)]
pub struct Renderer {
    pub width: usize,
    pub height: usize,
    pub scale: usize, // side length of 1 block
    pixels: Vec<u8>,
    z_buffer: Vec<f32>,
    pub camera: Camera,

    pub textures: Vec<Texture>,
    pub lights: Vec<Light>,
}

impl Renderer {
    pub fn new(width: usize, height: usize, fov_horizontal: f32, texture_array: &[u8]) -> Self {
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

        let near = (width as f32 / 2.0) / (fov_horizontal / 2.0).tan();
        let far = near * 200.0;
        let scale = width / 7;

        Self {
            pixels: vec![0; 4 * width * height],
            z_buffer: vec![100000.0; width * height],
            width,
            height,
            scale,
            camera: Camera::new(
                Point3D::from_euc_coords(width as isize / 2, height as isize / 2, 0),
                Point3D::from_euc_coords(width as isize / 2, height as isize / 2, 1),
                near as isize,
                far as isize,
            ),
            textures,
            lights: vec![Light::new(
                Color::new(255, 0, 0, 255),
                5000,
                Point3D::from_euc_coords(
                    width as isize / 2,
                    (height as isize / 2) - 1 * (scale as isize),
                    2 * near as isize + 7 * scale as isize,
                ),
            )],
        }
    }
    pub fn clear(&mut self) {
        self.pixels.fill(255);
        self.z_buffer.fill(100000.0);
    }
    pub fn get_pixels(&self) -> &[u8] {
        &self.pixels
    }
    pub fn get_mut_pixels(&mut self) -> &mut [u8] {
        &mut self.pixels
    }
    pub fn get_near(&self) -> isize {
        self.camera.data.near
    }
    pub fn get_far(&self) -> isize {
        self.camera.data.far
    }
    fn pixel_iterator(&self, x: usize, y: usize) -> PixelIterator {
        PixelIterator::new(self.width, self.height, x, y)
    }
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
    fn write_pixel_internal(&mut self, pixel_offset: usize, offset: usize, z: f32, color: Color) {
        unsafe {
            if z >= *self.z_buffer.get_unchecked(pixel_offset) {
                return;
            }

            let pixel_slice = self.pixels.as_mut_slice();
            *pixel_slice.get_unchecked_mut(offset) = color.r;
            *pixel_slice.get_unchecked_mut(offset + 1) = color.g;
            *pixel_slice.get_unchecked_mut(offset + 2) = color.b;
            *pixel_slice.get_unchecked_mut(offset + 3) = color.a;

            *self.z_buffer.get_unchecked_mut(pixel_offset) = z;
        }
    }
    #[inline(always)]
    fn write_pixel_unchecked(&mut self, x: isize, y: isize, z: f32, color: Color) {
        let pixel_offset = y as usize * self.width + x as usize;
        self.write_pixel_internal(pixel_offset, 4 * pixel_offset, z, color);
    }
    #[inline(always)]
    pub fn write_pixel(&mut self, x: isize, y: isize, z: f32, color: Color) {
        if !self.to_render(x, y, Some(z)) {
            return;
        }
        self.write_pixel_unchecked(x, y, z, color);
    }
    pub fn write_line(&mut self, p1: &Point3D, p2: &Point3D, color: Color) {
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
            } else {
                dy.abs()
            }
        };

        if step == 0.0 {
            return;
        } else {
            dx /= step;
            dy /= step;
        }

        let mut x = x1 as f32;
        let mut y = y1 as f32;
        let mut i: usize = 1;
        while i <= step as usize {
            let t = (i - 1) as f32 / step;
            self.write_pixel(x as isize, y as isize, z1 * (1.0 - t) + z2 * t, color);
            x += dx;
            y += dy;
            i += 1;
        }
    }
    // write_square is always at top z_indx cuz it's just a debugging function
    // Also I'm too lazy to write 2 lines to do interp
    pub fn write_square(&mut self, p: &Point3D, sidelen: isize, color: Color) {
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
    fn barycentric_interp_params(z_a: f32, z_b: f32, z_c: f32) -> (f32, f32, f32) {
        (1.0 / z_a, 1.0 / z_b, 1.0 / z_c)
    }
    #[inline(always)]
    fn interp_barycentric_z(params: &(f32, f32, f32), u: f32, v: f32, w: f32) -> f32 {
        1.0 / (params.0 * u + params.1 * v + params.2 * w)
    }
    #[inline(always)]
    fn interp_barycentric(
        params: &(f32, f32, f32),
        u: f32,
        v: f32,
        w: f32,
        z: f32,
        v_a: f32,
        v_b: f32,
        v_c: f32,
    ) -> f32 {
        z * (v_a * params.0 * u + v_b * params.1 * v + v_c * params.2 * w)
    }
    // Solves for the range of x-coordinates (euclidean) to make points in the triangle
    // Call it once for each of the 3 barycentric coordinates (u, v, w)
    // a is the current row's barycentric coordinate, b is the RECIPROCAL of du/dx, or dv/dx, or
    // dw/dx
    fn solve_bary_range(lower: &mut f32, upper: &mut f32, a: f32, b: f32) {
        let calculated = -a * b;
        if b >= 0.0 {
            if *lower < calculated {
                *lower = calculated;
            }
        } else {
            if *upper > calculated {
                *upper = calculated;
            }
        }
    }
    pub fn draw_triface(
        &mut self,
        v1: &Point3D,
        v2: &Point3D,
        v3: &Point3D,
        texture: (f32, f32, f32, f32, f32, f32, usize),
    ) {
        let normal = RenderMatrices::triface_normal(v1, v2, v3);
        if normal.dot(
            &RenderMatrices::triface_center(v1, v2, v3).minus(&self.camera.data.position.position),
        ) > 0.0
        {
            return;
        }

        let (tc1x, tc1y, tc2x, tc2y, tc3x, tc3y, texture_id) = texture;
        let light_color_1 =
            self.lights[0].intensity(v1, &self.camera.data.position, &normal, self.scale);
        let light_color_2 =
            self.lights[0].intensity(v2, &self.camera.data.position, &normal, self.scale);
        let light_color_3 =
            self.lights[0].intensity(v3, &self.camera.data.position, &normal, self.scale);

        let mut vertices = RenderMatrices::bundle_points(&[v1, v2, v3]);

        let forward = CameraCache::view(&mut self.camera.cache.view, &self.camera.data);
        let reverse =
            CameraCache::reverse(&mut self.camera.cache.reverse, &self.camera.data).matrix_mul(
                CameraCache::projection(&mut self.camera.cache.projection, &self.camera.data),
            );

        vertices = forward.matrix_mul(&vertices);

        let bary_interp_params = Self::barycentric_interp_params(
            vertices.get(0, 2),
            vertices.get(1, 2),
            vertices.get(2, 2),
        );

        let proj = RenderMatrices::split_points(&reverse.matrix_mul(&vertices));

        let p1x = proj[0].x_coord();
        let p1y = proj[0].y_coord();
        let p1x_f = proj[0].get(0);
        let p1y_f = proj[0].get(1);
        let p1z = proj[0].z_coord_float();

        let p2x = proj[1].x_coord();
        let p2y = proj[1].y_coord();
        let p2x_f = proj[1].get(0);
        let p2y_f = proj[1].get(1);
        let p2z = proj[1].z_coord_float();

        let p3x = proj[2].x_coord();
        let p3y = proj[2].y_coord();
        let p3x_f = proj[2].get(0);
        let p3y_f = proj[2].get(1);
        let p3z = proj[2].z_coord_float();

        let render1 = self.to_render(p1x, p1y, Some(p1z));
        let render2 = self.to_render(p2x, p2y, Some(p2z));
        let render3 = self.to_render(p3x, p3y, Some(p3z));
        if (!render1) && (!render2) && (!render3) {
            return;
        }

        let min_x = cmp::max(0, cmp::min(cmp::min(p1x, p2x), p3x));
        let min_y = cmp::max(0, cmp::min(cmp::min(p1y, p2y), p3y));
        let max_x = cmp::min(self.width as isize - 1, cmp::max(cmp::max(p1x, p2x), p3x));
        let max_y = cmp::min(self.height as isize - 1, cmp::max(cmp::max(p1y, p2y), p3y));

        let (mut row_u, mut row_v, mut row_w, dudx, dvdx, dwdx, dudy, dvdy, dwdy) =
            RenderMatrices::barycentric_params(
                min_x as f32,
                min_y as f32,
                p1x_f,
                p1y_f,
                p2x_f,
                p2y_f,
                p3x_f,
                p3y_f,
            );
        let inv_dudx: Option<f32> = if dudx.abs() <= 0.00001 {
            None
        } else {
            Some(1.0 / dudx)
        };
        let inv_dvdx: Option<f32> = if dvdx.abs() <= 0.00001 {
            None
        } else {
            Some(1.0 / dvdx)
        };
        let inv_dwdx: Option<f32> = if dwdx.abs() <= 0.00001 {
            None
        } else {
            Some(1.0 / dwdx)
        };

        let mut pixel_iterator = self.pixel_iterator(min_x as usize, min_y as usize);

        for indy in min_y..=max_y {
            let (x_start, x_end, mut column_u, mut column_v, mut column_w) = {
                let mut low = 0.0;
                let mut high = (max_x - min_x) as f32;
                if inv_dudx.is_some() {
                    Self::solve_bary_range(&mut low, &mut high, row_u, inv_dudx.unwrap())
                }
                if inv_dvdx.is_some() {
                    Self::solve_bary_range(&mut low, &mut high, row_v, inv_dvdx.unwrap())
                }
                if inv_dwdx.is_some() {
                    Self::solve_bary_range(&mut low, &mut high, row_w, inv_dwdx.unwrap())
                }
                (
                    low as isize,
                    (high + 0.5) as isize,
                    dudx * low,
                    dvdx * low,
                    dwdx * low,
                )
            };
            pixel_iterator.move_to((x_start + min_x) as usize, indy as usize);

            for indx in x_start..=x_end {
                let u = row_u + column_u;
                let v = row_v + column_v;
                let w = row_w + column_w;

                let interp_z = Self::interp_barycentric_z(&bary_interp_params, u, v, w);
                let tcx = Self::interp_barycentric(
                    &bary_interp_params,
                    u,
                    v,
                    w,
                    interp_z,
                    tc1x,
                    tc2x,
                    tc3x,
                );
                let tcy = Self::interp_barycentric(
                    &bary_interp_params,
                    u,
                    v,
                    w,
                    interp_z,
                    tc1y,
                    tc2y,
                    tc3y,
                );
                let mut pixel_color = self.textures[texture_id].sample(tcx, tcy);
                pixel_color.compose(Color::interp_barycentric(
                    &bary_interp_params,
                    u,
                    v,
                    w,
                    interp_z,
                    light_color_1,
                    light_color_2,
                    light_color_3,
                ));

                self.write_pixel_internal(
                    pixel_iterator.pixel_offset,
                    pixel_iterator.offset,
                    interp_z,
                    pixel_color,
                );

                column_u += dudx;
                column_v += dvdx;
                column_w += dwdx;
                pixel_iterator.next_column();
            }
            row_u += dudy;
            row_v += dvdy;
            row_w += dwdy;
        }
    }
    pub fn draw_quadface(
        &mut self,
        v1: &Point3D,
        v2: &Point3D,
        v3: &Point3D,
        v4: &Point3D,
        fill: (f32, f32, f32, f32, f32, f32, f32, f32, usize),
    ) {
        let (tc1x, tc1y, tc2x, tc2y, tc3x, tc3y, tc4x, tc4y, tex) = fill;
        self.draw_triface(v1, v2, v3, (tc1x, tc1y, tc2x, tc2y, tc3x, tc3y, tex));
        self.draw_triface(v3, v4, v1, (tc3x, tc3y, tc4x, tc4y, tc1x, tc1y, tex));
    }
    // 1 is bottom, 2 is left side, 3 is far side, 4 is right side, 5 is near side, 6 is top
    pub fn draw_cubeface(
        &mut self,
        center: &Point3D,
        side: usize,
        halfsides: &[f32],
        transform: &Matrix,
        texture: usize,
    ) {
        let x = center.get(0);
        let y = center.get(1);
        let z = center.get(2);

        let (offset_x, offset_y, offset_z, axis_1, axis_2) = {
            match side {
                1 => (0.0, 0.0, halfsides[2] as f32, 0, 1),
                2 => (-(halfsides[0] as f32), 0.0, 0.0, 2, 1),
                3 => (0.0, halfsides[1] as f32, 0.0, 2, 0),
                4 => (halfsides[0] as f32, 0.0, 0.0, 1, 2),
                5 => (0.0, -(halfsides[1] as f32), 0.0, 0, 2),
                6 => (0.0, 0.0, -(halfsides[2] as f32), 1, 0),
                _ => {
                    unreachable!()
                }
            }
        };

        let mut p1 = Point3D::from_euc_coords_float(x + offset_x, y + offset_y, z + offset_z);
        let mut p2 = Point3D::from_euc_coords_float(x + offset_x, y + offset_y, z + offset_z);
        let mut p3 = Point3D::from_euc_coords_float(x + offset_x, y + offset_y, z + offset_z);
        let mut p4 = Point3D::from_euc_coords_float(x + offset_x, y + offset_y, z + offset_z);

        p1.set(axis_1, p1.get(axis_1) - halfsides[axis_1]);
        p1.set(axis_2, p1.get(axis_2) - halfsides[axis_2]);

        p2.set(axis_1, p2.get(axis_1) + halfsides[axis_1]);
        p2.set(axis_2, p2.get(axis_2) - halfsides[axis_2]);

        p3.set(axis_1, p3.get(axis_1) + halfsides[axis_1]);
        p3.set(axis_2, p3.get(axis_2) + halfsides[axis_2]);

        p4.set(axis_1, p4.get(axis_1) - halfsides[axis_1]);
        p4.set(axis_2, p4.get(axis_2) + halfsides[axis_2]);

        /*
        p1.set(axis_1, p1.get(axis_1) - halfsides[axis_1]);
        p1.set(axis_2, p1.get(axis_2) + halfsides[axis_2]);

        p2.set(axis_1, p2.get(axis_1) + halfsides[axis_1]);
        p2.set(axis_2, p2.get(axis_2) + halfsides[axis_2]);

        p3.set(axis_1, p3.get(axis_1) + halfsides[axis_1]);
        p3.set(axis_2, p3.get(axis_2) - halfsides[axis_2]);

        p4.set(axis_1, p4.get(axis_1) - halfsides[axis_1]);
        p4.set(axis_2, p4.get(axis_2) - halfsides[axis_2]);
        */

        p1 = p1.transform(transform);
        p2 = p2.transform(transform);
        p3 = p3.transform(transform);
        p4 = p4.transform(transform);

        self.draw_quadface(
            &p1,
            &p2,
            &p3,
            &p4,
            (
                0.0,
                gfx::MTEXCOORD,
                gfx::MTEXCOORD,
                gfx::MTEXCOORD,
                gfx::MTEXCOORD,
                0.0,
                0.0,
                0.0,
                texture,
            ),
        );
    }
    pub fn draw_cuboid(
        &mut self,
        position: &Point3D,
        orientation: &(f32, f32, f32),
        dimensions: &[f32],
        texture_id: usize,
    ) {
        let x = position.x_coord();
        let y = position.y_coord();
        let z = position.z_coord();

        let (pitch, roll, yaw) = orientation;
        let halfsides = [
            dimensions[0] / 2.0,
            dimensions[1] / 2.0,
            dimensions[2] / 2.0,
        ];

        let transform =
            RenderMatrices::rotation_3d(*pitch, *roll, *yaw, Some(&(x as f32, y as f32, z as f32)));

        self.draw_cubeface(position, 1, &halfsides, &transform, texture_id);
        self.draw_cubeface(position, 2, &halfsides, &transform, texture_id);
        self.draw_cubeface(position, 3, &halfsides, &transform, texture_id);
        self.draw_cubeface(position, 4, &halfsides, &transform, texture_id);
        self.draw_cubeface(position, 5, &halfsides, &transform, texture_id);
        self.draw_cubeface(position, 6, &halfsides, &transform, texture_id);
    }
}

impl PixelIterator {
    pub fn new(width: usize, height: usize, x: usize, y: usize) -> Self {
        let pixel_offset: usize = y * width + x;
        Self {
            width,
            height,
            x,
            y,
            pixel_offset,
            offset: 4 * pixel_offset,
        }
    }
    pub fn next_row(&mut self) {
        self.y += 1;
        self.pixel_offset += self.width;
        self.offset += 4 * self.width;
    }
    pub fn next_column(&mut self) {
        self.x += 1;
        self.pixel_offset += 1;
        self.offset += 4;
    }
    pub fn move_to(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
        self.pixel_offset = y * self.width + x;
        self.offset = 4 * self.pixel_offset;
    }
}
