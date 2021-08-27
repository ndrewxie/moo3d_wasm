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
        Self {
            r,
            g,
            b,
            a,
        }
    }
}

pub struct Texture {
    data: Vec<u8>,
    width: usize,
    height: usize,
}
impl Texture {
    pub fn new() -> Self {
        let width = 120;
        let height = 120;
        let mut to_return = vec![0; 4 * width * height];
        for indy in 0..height {
            for indx in 0..width {
                let x = indx as usize / 30;
                let y = indy as usize / 30;

                let offset = 4 * (indy * width + indx);

                if (x + y) % 2 == 0 {
                    to_return[offset] = 0;
                    to_return[offset+1] = 0;
                    to_return[offset+2] = 0;
                    to_return[offset+3] = 255;
                }
                else {
                    to_return[offset] = 255;
                    to_return[offset+1] = 255;
                    to_return[offset+2] = 255;
                    to_return[offset+3] = 255;
                }
            }
        }
        Self {
            width,
            height,
            data: to_return
        }
    }
}