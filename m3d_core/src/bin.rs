use m3d_core::GameState;
use std::fs;

struct Base16Reader<'a> {
    to_read: &'a [u8],
    index: usize,
}
impl<'a> Base16Reader<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            to_read: input.as_bytes(),
            index: 0,
        }
    }
    fn lookup_char(input: u8) -> Option<u8> {
        let base16_chars = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
        ];
        for (i, val) in base16_chars.iter().enumerate() {
            if *val == input as char {
                return Some(i as u8);
            }
        }
        None
    }
    pub fn read_u8(&mut self) -> Option<u8> {
        if self.index + 2 <= self.to_read.len() {
            let to_return = 16
                * Self::lookup_char(self.to_read[self.index]).expect("Cannot lookup base16 digit")
                + Self::lookup_char(self.to_read[self.index + 1])
                    .expect("Cannot lookup base16 digit");
            self.index += 2;
            Some(to_return)
        } else {
            None
        }
    }
    pub fn read_u16(&mut self) -> Option<u16> {
        let first = self.read_u8()? as u16;
        let second = self.read_u8()? as u16;
        Some(first * 256 + second)
    }
    pub fn has_next(&self) -> bool {
        self.index + 1 < self.to_read.len()
    }
}

pub fn test_manager(n: usize) {
    let texture_string = fs::read_to_string("images.txt").expect("Cannot open images.txt");
    let mut texture_array: Vec<u8> = Vec::new();

    for texture_line in texture_string.split("\n").collect::<Vec<&str>>() {
        let mut texture_reader = Base16Reader::new(texture_line);
        let colors = {
            let num_colors = texture_reader
                .read_u16()
                .expect("Could not read number of colors");
            let mut tentative_colors: Vec<(u8, u8, u8, u8)> = Vec::new();

            for _ in 0..num_colors {
                let r = texture_reader.read_u8().unwrap();
                let g = texture_reader.read_u8().unwrap();
                let b = texture_reader.read_u8().unwrap();
                let a = texture_reader.read_u8().unwrap();
                tentative_colors.push((r, g, b, a));
            }
            tentative_colors
        };

        while texture_reader.has_next() {
            let run_length = {
                let first = texture_reader.read_u8().unwrap();
                if first == 255 {
                    texture_reader
                        .read_u16()
                        .expect("Could not read run length")
                } else {
                    first as u16
                }
            };
            let run_color = texture_reader.read_u8().expect("Could not read run color") as usize;

            for _ in 0..run_length {
                let selected_color = colors[run_color];
                texture_array.push(selected_color.0);
                texture_array.push(selected_color.1);
                texture_array.push(selected_color.2);
                texture_array.push(selected_color.3);
            }
        }
    }
    println!("{}", texture_array.len());

    let mut gs_manager = GameState::new(1265, 632, &texture_array);
    gs_manager.rotate_camera(-0.2, 0.6);
    //gs_manager.renderer.camera.translate(0, 0, 2300);
    //let mut gs_manager = GameState::new(1266, 633);
    for j in 0..n {
        gs_manager.render(j);
    }
}

pub fn main() {
    #[cfg(feature = "callgrind")]
    {
        println!("callgrind starting...");
        test_manager(5); // 500
        println!("callgrind finished.");
    }
    #[cfg(not(feature = "callgrind"))]
    {
        println!("moo3d_core test starting...");
        test_manager(500); // 500
        println!("moo3d_core test finished.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
    #[test]
    fn test_main() {
        test_manager(1);
    }
}
