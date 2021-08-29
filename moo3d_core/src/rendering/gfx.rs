use std::cmp;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Color {
    #[inline(always)]
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

pub struct Texture {
    data: Vec<Color>,
    width: usize,
    height: usize,
}
impl Texture {
    pub fn checkerboard() -> Self {
        let width = 120;
        let height = 120;
        let mut to_return = Vec::with_capacity(width * height);
        for indy in 0..height {
            for indx in 0..width {
                let x = indx as usize / 30;
                let y = indy as usize / 30;

                if (x + y) % 2 == 0 {
                    to_return.push(Color::new(0, 0, 0, 255));
                } else {
                    to_return.push(Color::new(255, 255, 255, 255));
                }
            }
        }
        Self {
            width,
            height,
            data: to_return,
        }
    }
    pub fn sample(&self, u: f32, v: f32) -> &Color {
        let clamped_u = cmp::max(
            cmp::min(self.width - 1, (self.width as f32 * u + 0.5) as usize),
            0,
        );
        let clamped_v = cmp::max(
            cmp::min(self.height - 1, (self.height as f32 * v + 0.5) as usize),
            0,
        );

        unsafe { &self.data.get_unchecked(clamped_v * self.width + clamped_u) }
    }
}
