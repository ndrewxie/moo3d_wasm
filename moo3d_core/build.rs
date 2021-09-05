use std::fs;
extern crate image;

pub fn main() {
    println!("cargo:rerun-if-changed=./img");

    let images_names = fs::read_to_string("./img/index.txt").expect("Image packaging failed");
    let mut buffer: Vec<u8> = Vec::new();

    for img_name in images_names.split("\n") {
        let img = image::open("./img/".to_owned() + img_name)
            .unwrap()
            .to_rgba8();

        for indy in 0..img.height() {
            for indx in 0..img.width() {
                let rgba = img.get_pixel(indx, indy);
                buffer.push(rgba[0]);
                buffer.push(rgba[1]);
                buffer.push(rgba[2]);
                buffer.push(rgba[3]);
            }
        }
    }
    fs::write("./images.bin", buffer).unwrap();
}
