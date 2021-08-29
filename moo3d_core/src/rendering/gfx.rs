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
    width: isize,
    height: isize,
    max: isize,
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
            width,
            height,
            data: to_return,
            max: (width * height) as isize,
        }
    }
    #[inline(never)]
    pub fn sample(&self, u: f32, v: f32) -> &Color {
        let mut indx = (self.height as f32 * v + 0.5) as isize * self.width + (self.width as f32 * u + 0.5) as isize;
        if (indx < 0) | (indx >= self.max) {
            indx = 0;
        }
        unsafe { &self.data.get_unchecked(indx as usize) }
    }
}
