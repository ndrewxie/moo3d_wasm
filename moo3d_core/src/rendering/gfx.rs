use crate::rendering::rendermath::{Point3D, RenderMatrices, Vector};
use std::cmp;

pub const TEXTURE_SIZE: isize = 128;
const MAX_TEXTURE_COORD: isize = TEXTURE_SIZE - 1;
pub const TEXTURE_LEN: isize = TEXTURE_SIZE * MAX_TEXTURE_COORD + MAX_TEXTURE_COORD;
const FTEXTURE_SIZE: f32 = TEXTURE_SIZE as f32;
pub const MTEXCOORD: f32 = MAX_TEXTURE_COORD as f32;

const EIGHT_PI: f32 = 8.0 * std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
#[derive(Debug)]
pub struct Texture {
    data: Vec<Color>,
}
#[derive(Debug)]
pub struct Light {
    color: Color,
    intensity: u32,
    pub position: Point3D,
}

impl Color {
    #[inline(always)]
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    pub fn compose(&self, other: Color) -> Self {
        Self {
            r: ((self.r as u32) * (other.r as u32) / 255) as u8,
            g: ((self.g as u32) * (other.g as u32) / 255) as u8,
            b: ((self.b as u32) * (other.b as u32) / 255) as u8,
            a: self.a,
        }
    }
    pub fn interp_barycentric(
        params: &(f32, f32, f32),
        u: f32,
        v: f32,
        w: f32,
        z: f32,
        c1: Self,
        c2: Self,
        c3: Self,
    ) -> Self {
        Self {
            r: (z
                * (c1.r as f32 * params.0 * u
                    + c2.r as f32 * params.1 * v
                    + c3.r as f32 * params.2 * w)) as u8,
            g: (z
                * (c1.g as f32 * params.0 * u
                    + c2.g as f32 * params.1 * v
                    + c3.g as f32 * params.2 * w)) as u8,
            b: (z
                * (c1.b as f32 * params.0 * u
                    + c2.b as f32 * params.1 * v
                    + c3.b as f32 * params.2 * w)) as u8,
            a: (z
                * (c1.a as f32 * params.0 * u
                    + c2.a as f32 * params.1 * v
                    + c3.a as f32 * params.2 * w)) as u8,
        }
    }
}
impl Texture {
    pub fn new(data: Vec<Color>) -> Self {
        Self { data }
    }
    pub fn checkerboard() -> Self {
        let mut to_return = Vec::with_capacity((TEXTURE_SIZE * TEXTURE_SIZE) as usize);
        for indy in 0..TEXTURE_SIZE {
            for indx in 0..TEXTURE_SIZE {
                let x = indx / 32;
                let y = indy / 32;

                if (x + y) % 2 == 0 {
                    to_return.push(Color::new(0, 0, 0, 255));
                } else {
                    to_return.push(Color::new(255, 255, 255, 255));
                }
            }
        }
        Self { data: to_return }
    }
    pub fn sample(&self, u: f32, v: f32) -> Color {
        /*
        unsafe {
            *self.data.get_unchecked(
                cmp::min(
                    TEXTURE_LEN,
                    cmp::max(
                        0,
                        (FTEXTURE_SIZE * v.trunc() + u).to_int_unchecked::<isize>()
                    )
                ) as usize
            )
        }
        */
        unsafe {
            *self.data.get_unchecked(
                ((FTEXTURE_SIZE * v.trunc() + u).to_int_unchecked::<isize>() & TEXTURE_LEN)
                    as usize,
            )
        }
    }
}
impl Light {
    pub fn new(color: Color, intensity: u32, position: Point3D) -> Self {
        Self {
            color,
            intensity,
            position,
        }
    }
    /*
    pub fn intensity(
        &self,
        position: &Point3D,
        camera: &Point3D,
        normal: &Vector,
        _reflectivity: f32,
    ) -> Color {
        let light_vec = self.position.position.minus(&position.position);
        if light_vec.dot(normal) < 0.0 {
            return Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            }
        }
        let camera_vec = camera.position.minus(&position.position);
        let mut half_vec = light_vec.plus(&camera_vec);
        half_vec.normalize_inplace();

        let mut to_return: f32 = normal.dot(&half_vec);
        if to_return <= 0.0 {
            to_return = 0.0;
        }

        Color {
            r: (self.color.r as f32 * to_return) as u8,
            g: (self.color.g as f32 * to_return) as u8,
            b: (self.color.b as f32 * to_return) as u8,
            a: 255,
        }
    }
    */
    // still violates conservation of energy but at least is pretty
    pub fn intensity(
        &self,
        position: &Point3D,
        camera: &Point3D,
        normal: &Vector,
        scale: usize,
    ) -> Color {
        let light_vec = self.position.position.minus(&position.position);
        if light_vec.dot(normal) < 0.0 {
            return Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            };
        }
        let mut light_distance = light_vec.scalar_mul(1.0 / scale as f32).norm2();
        if light_distance < 0.01 {
            light_distance = 0.01;
        }
        let to_return = cmp::min(255, (self.intensity as f32 / light_distance) as u32);

        Color {
            r: ((self.color.r as u32 * to_return) / 255) as u8,
            g: ((self.color.g as u32 * to_return) / 255) as u8,
            b: ((self.color.b as u32 * to_return) / 255) as u8,
            a: 255,
        }
    }
}
