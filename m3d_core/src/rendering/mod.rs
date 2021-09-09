use std::cmp;

pub mod camera;
pub mod gfx;
mod pixeliterator;
pub mod rendermath;

use camera::{Camera, CameraCache};
use gfx::{Color, FarLight, Light, NearLight, Texture};
use pixeliterator::PixelIterator;
use rendermath::{Matrix, Point3D, RenderMatrices, Vector};

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
#[derive(Clone, Copy)]
pub enum CubeFace {
    PlusX,
    PlusY,
    PlusZ,
    MinusX,
    MinusY,
    MinusZ,
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
                (0.0, 0.0),
                near,
                far,
                width,
                height,
            ),
            textures,
            lights: vec![
                Light::Near(NearLight::new(
                    Color::new(255, 0, 0, 255),
                    3500,
                    Point3D::from_euc_coords(
                        (width as isize / 2) - 3 * (scale as isize),
                        (height as isize / 2) - 1 * (scale as isize),
                        2 * near as isize + 1 * scale as isize,
                    ),
                )),
                Light::Near(NearLight::new(
                    Color::new(0, 255, 0, 255),
                    3000,
                    Point3D::from_euc_coords(
                        (width as isize / 2) + 3 * (scale as isize),
                        (height as isize / 2) - 1 * (scale as isize),
                        2 * near as isize + 1 * scale as isize,
                    ),
                )),
                Light::Far(FarLight::new(
                    Color::new(0, 0, 255, 255),
                    175,
                    Vector::with_data(vec![0.0, 1.0, 0.0]),
                )),
            ],
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
    pub fn get_near(&self) -> f32 {
        self.camera.data.near
    }
    pub fn get_far(&self) -> f32 {
        self.camera.data.far
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
    fn write_pixel_internal(&mut self, pixel_offset: usize, offset: usize, z: f32, color: Color) {
        unsafe {
            if (z >= *self.z_buffer.get_unchecked(pixel_offset)) | (z < 0.0) | (z > 1.0) {
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
    fn vertex_lighting(&self, vertex: &Point3D, normal: &Vector) -> Color {
        let mut to_return = Color::zero();
        for light in &self.lights {
            to_return.add(light.intensity(vertex, normal, self.scale));
        }
        to_return
    }
    pub fn draw_triface(
        &mut self,
        v1: &Point3D,
        v2: &Point3D,
        v3: &Point3D,
        texture: (f32, f32, f32, f32, f32, f32, u16),
    ) {
        let normal = RenderMatrices::triface_normal(v1, v2, v3);
        if normal.dot(
            &RenderMatrices::triface_center(v1, v2, v3).minus(&self.camera.data.position.position),
        ) > 0.0
        {
            return;
        }

        let forward = CameraCache::view(&mut self.camera.cache.view, &self.camera.data);
        let reverse =
            CameraCache::reverse(&mut self.camera.cache.reverse, &self.camera.data).matrix_mul(
                CameraCache::projection(&mut self.camera.cache.projection, &self.camera.data),
            );

        let vertex1 = v1.transform(&forward);
        let vertex2 = v2.transform(&forward);
        let vertex3 = v3.transform(&forward);

        let projected1 = vertex1.transform(&reverse);
        let projected2 = vertex2.transform(&reverse);
        let projected3 = vertex3.transform(&reverse);

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
        let light_color_1 = self.vertex_lighting(v1, &normal);
        let light_color_2 = self.vertex_lighting(v2, &normal);
        let light_color_3 = self.vertex_lighting(v3, &normal);

        let bary_interp_params = Self::barycentric_interp_params(
            vertex1.z_coord_float(),
            vertex2.z_coord_float(),
            vertex3.z_coord_float(),
        );

        let min_x = cmp::max(0, cmp::min(cmp::min(p1x, p2x), p3x));
        let min_y = cmp::max(0, cmp::min(cmp::min(p1y, p2y), p3y));
        let max_x = cmp::min(self.width as isize - 1, cmp::max(cmp::max(p1x, p2x), p3x));
        let max_y = cmp::min(self.height as isize - 1, cmp::max(cmp::max(p1y, p2y), p3y));

        let barycentric_params = RenderMatrices::barycentric_params(
            min_x as f32,
            min_y as f32,
            projected1.get(0),
            projected1.get(1),
            projected2.get(0),
            projected2.get(1),
            projected3.get(0),
            projected3.get(1),
        );

        let mut pixel_iterator =
            self.pixel_iterator(min_x as usize, min_y as usize, &barycentric_params);
        let z_map_denominator = 1.0 / (self.get_far() - self.get_near());

        for _indy in min_y..=max_y {
            let [x_start, x_end] = pixel_iterator.solve_x_range(min_x, max_x);
            pixel_iterator.set_x(x_start + min_x);

            for _indx in x_start..=x_end {
                let u = pixel_iterator.u;
                let v = pixel_iterator.v;
                let w = pixel_iterator.w;

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
                    (interp_z - self.get_near()) * z_map_denominator,
                    pixel_color,
                );
                pixel_iterator.next_column();
            }
            pixel_iterator.next_row();
        }
    }
    pub fn draw_quadface(
        &mut self,
        v1: &Point3D,
        v2: &Point3D,
        v3: &Point3D,
        v4: &Point3D,
        fill: (f32, f32, f32, f32, f32, f32, f32, f32, u16),
    ) {
        let (tc1x, tc1y, tc2x, tc2y, tc3x, tc3y, tc4x, tc4y, tex) = fill;
        self.draw_triface(v1, v2, v3, (tc1x, tc1y, tc2x, tc2y, tc3x, tc3y, tex));
        self.draw_triface(v3, v4, v1, (tc3x, tc3y, tc4x, tc4y, tc1x, tc1y, tex));
    }
    // 1 is bottom, 2 is left side, 3 is far side, 4 is right side, 5 is near side, 6 is top
    pub fn draw_cubeface(
        &mut self,
        center: &Point3D,
        side: CubeFace,
        halfsides: &[f32],
        transform: &Matrix,
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
        texture_id: u16,
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

        self.draw_cubeface(
            position,
            CubeFace::PlusZ,
            &halfsides,
            &transform,
            texture_id,
        );
        self.draw_cubeface(
            position,
            CubeFace::MinusX,
            &halfsides,
            &transform,
            texture_id,
        );
        self.draw_cubeface(
            position,
            CubeFace::PlusY,
            &halfsides,
            &transform,
            texture_id,
        );
        self.draw_cubeface(
            position,
            CubeFace::PlusX,
            &halfsides,
            &transform,
            texture_id,
        );
        self.draw_cubeface(
            position,
            CubeFace::MinusY,
            &halfsides,
            &transform,
            texture_id,
        );
        self.draw_cubeface(
            position,
            CubeFace::MinusZ,
            &halfsides,
            &transform,
            texture_id,
        );
    }
}
