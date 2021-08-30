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
    width: f32,
    height: f32,
    max: f32,
}
impl Texture {
    pub fn checkerboard() -> Self {
        let width: isize = 120;
        let height: isize = 120;
        let mut to_return = Vec::with_capacity((width * height) as usize);
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
            width: width as f32,
            height: height as f32,
            data: to_return,
            max: (width * height) as f32,
        }
    }
    pub fn sample(&self, u: f32, v: f32) -> &Color {
        let indx = (self.height * v).round() * self.width + (self.width * u).round();
        unsafe { &self.data.get_unchecked(indx.clamp(0.0, self.max-1.0) as usize) }
    }
}
