use std::cmp;

pub const TEXTURE_SIZE: isize = 128;
const MAX_TEXTURE_COORD: isize = TEXTURE_SIZE - 1;
const TEXTURE_LEN: isize = TEXTURE_SIZE * MAX_TEXTURE_COORD + MAX_TEXTURE_COORD;
const FTEXTURE_SIZE: f32 = TEXTURE_SIZE as f32;

pub const MTEXCOORD: f32 = MAX_TEXTURE_COORD as f32;

//const TEXTURE_MASK: isize = TEXTURE_SIZE - 1;

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
}
impl Texture {
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
        Self {
            data: to_return,
        }
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
            *self.data.get_unchecked((
                (FTEXTURE_SIZE * v.trunc() + u).to_int_unchecked::<isize>() & TEXTURE_LEN
            ) as usize)
        }
    }
}
