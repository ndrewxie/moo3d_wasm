use std::cmp;

pub mod gfx;
mod pixeliterator;

use crate::camera::{CameraCache, CameraData};
use crate::rendermath::{Matrix, Point3D, RenderMatrices, Vector};
use gfx::{Color, Texture};
use pixeliterator::PixelIterator;

pub struct Renderer {
    pub width: usize,
    pub height: usize,
    pixels: Vec<u8>,
    z_buffer: Vec<f32>,

    pub textures: Vec<Texture>,
}
#[derive(Clone, Copy, Debug)]
pub enum CubeFace {
    PlusX = 0,
    PlusY = 1,
    PlusZ = 2,
    MinusX = 3,
    MinusY = 4,
    MinusZ = 5,
}

impl Renderer {
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
            pixels: vec![0; 4 * width * height],
            z_buffer: vec![100000.0; width * height],
            width,
            height,
            textures,
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
    fn pixel_iterator(
        &self,
        x: usize,
        y: usize,
        params: &(f32, f32, f32, f32, f32, f32, f32, f32, f32),
    ) -> PixelIterator {
        PixelIterator::new(self.width, self.height, x, y, params)
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
    #[inline(always)]
    fn fails_z_check(&mut self, pixel_offset: usize, z: f32) -> bool {
        unsafe { (z >= *self.z_buffer.get_unchecked(pixel_offset)) || (z < 0.0) || (z > 1.0) }
    }
    fn write_pixel_internal(&mut self, pixel_offset: usize, offset: usize, z: f32, color: Color) {
        unsafe {
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

        if self.fails_z_check(pixel_offset, z) {
            return;
        }

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
    pub fn draw_triface(
        &mut self,
        screen_space: &mut Option<Matrix>,
        camera_data: &CameraData,
        v1: &Point3D,
        v2: &Point3D,
        v3: &Point3D,
        light_color_1: Color,
        light_color_2: Color,
        light_color_3: Color,
        texture: (f32, f32, f32, f32, f32, f32, u16),
    ) {
        let normal = RenderMatrices::triface_normal(v1, v2, v3);
        if normal.dot(&RenderMatrices::triface_center(v1, v2, v3)) >= 0.0 {
            return;
        }

        let reverse = CameraCache::to_screen_space(screen_space, camera_data);

        let projected1 = v1.transform(&reverse);
        let projected2 = v2.transform(&reverse);
        let projected3 = v3.transform(&reverse);

        let p1x = projected1.x_coord();
        let p1y = projected1.y_coord();

        let p2x = projected2.x_coord();
        let p2y = projected2.y_coord();

        let p3x = projected3.x_coord();
        let p3y = projected3.y_coord();

        if (!self.to_render(p1x, p1y, Some(projected1.z_coord_float())))
            && (!self.to_render(p2x, p2y, Some(projected2.z_coord_float())))
            && (!self.to_render(p3x, p3y, Some(projected3.z_coord_float())))
        {
            return;
        }

        let (tc1x, tc1y, tc2x, tc2y, tc3x, tc3y, texture_id) = texture;

        let bary_interp_params = Self::barycentric_interp_params(
            v1.z_coord_float(),
            v2.z_coord_float(),
            v3.z_coord_float(),
        );

        let min_x = cmp::max(0, cmp::min(cmp::min(p1x, p2x), p3x));
        let min_y = cmp::max(0, cmp::min(cmp::min(p1y, p2y), p3y));
        let max_x = cmp::min(self.width as isize - 1, cmp::max(cmp::max(p1x, p2x), p3x));
        let max_y = cmp::min(self.height as isize - 1, cmp::max(cmp::max(p1y, p2y), p3y));

        let barycentric_params_wrapped = RenderMatrices::barycentric_params(
            min_x as f32,
            min_y as f32,
            projected1.get(0),
            projected1.get(1),
            projected2.get(0),
            projected2.get(1),
            projected3.get(0),
            projected3.get(1),
        );
        if barycentric_params_wrapped.is_none() {
            return;
        }
        let barycentric_params = barycentric_params_wrapped.unwrap();

        let mut pixel_iterator =
            self.pixel_iterator(min_x as usize, min_y as usize, &barycentric_params);

        let z_map_denominator = 1.0 / (camera_data.far - camera_data.near);

        for _indy in min_y..=max_y {
            let [x_start, x_end] = pixel_iterator.solve_x_range(min_x, max_x);
            pixel_iterator.set_x(x_start + min_x);

            for _indx in x_start..=x_end {
                let u = pixel_iterator.u;
                let v = pixel_iterator.v;
                let w = pixel_iterator.w;

                let interp_z = Self::interp_barycentric_z(&bary_interp_params, u, v, w);
                let actual_z = (interp_z - camera_data.near) * z_map_denominator;

                if !self.fails_z_check(pixel_iterator.pixel_offset, actual_z) {
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
                    let mut pixel_color = self.textures[texture_id as usize].sample(tcx, tcy);
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
                        actual_z,
                        pixel_color,
                    );
                }
                pixel_iterator.next_column();
            }
            pixel_iterator.next_row();
        }
    }
    pub fn draw_quadface(
        &mut self,
        screen_space: &mut Option<Matrix>,
        camera_data: &CameraData,
        v1: &Point3D,
        v2: &Point3D,
        v3: &Point3D,
        v4: &Point3D,
        light_color_1: Color,
        light_color_2: Color,
        light_color_3: Color,
        light_color_4: Color,
        fill: (f32, f32, f32, f32, f32, f32, f32, f32, u16),
    ) {
        let (tc1x, tc1y, tc2x, tc2y, tc3x, tc3y, tc4x, tc4y, tex) = fill;
        self.draw_triface(
            screen_space,
            camera_data,
            v1,
            v2,
            v3,
            light_color_1,
            light_color_2,
            light_color_3,
            (tc1x, tc1y, tc2x, tc2y, tc3x, tc3y, tex),
        );
        self.draw_triface(
            screen_space,
            camera_data,
            v3,
            v4,
            v1,
            light_color_3,
            light_color_4,
            light_color_1,
            (tc3x, tc3y, tc4x, tc4y, tc1x, tc1y, tex),
        );
    }
    pub fn draw_cubeface<LightingCalculator: Fn(&Point3D, &Vector) -> Color>(
        &mut self,
        screen_space: &mut Option<Matrix>,
        camera_data: &CameraData,
        center: &Point3D,
        side: CubeFace,
        halfsides: &[f32],
        post_transform: &Matrix,
        calculate_lighting: &LightingCalculator,
        texture: u16,
    ) {
        let x = center.get(0);
        let y = center.get(1);
        let z = center.get(2);

        let (offset_x, offset_y, offset_z, axis_1, axis_2) = {
            match side {
                CubeFace::PlusZ => (0.0, 0.0, halfsides[2] as f32, 0, 1),
                CubeFace::MinusX => (-(halfsides[0] as f32), 0.0, 0.0, 2, 1),
                CubeFace::PlusY => (0.0, halfsides[1] as f32, 0.0, 2, 0),
                CubeFace::PlusX => (halfsides[0] as f32, 0.0, 0.0, 1, 2),
                CubeFace::MinusY => (0.0, -(halfsides[1] as f32), 0.0, 0, 2),
                CubeFace::MinusZ => (0.0, 0.0, -(halfsides[2] as f32), 1, 0),
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

        let normal = RenderMatrices::triface_normal(&p1, &p2, &p3);
        let color_1 = calculate_lighting(&p1, &normal);
        let color_2 = calculate_lighting(&p2, &normal);
        let color_3 = calculate_lighting(&p3, &normal);
        let color_4 = calculate_lighting(&p4, &normal);

        p1 = p1.transform(post_transform);
        p2 = p2.transform(post_transform);
        p3 = p3.transform(post_transform);
        p4 = p4.transform(post_transform);

        self.draw_quadface(
            screen_space,
            camera_data,
            &p1,
            &p2,
            &p3,
            &p4,
            color_1,
            color_2,
            color_3,
            color_4,
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
}
