use std::fs;

pub fn main() {
    println!("cargo:rerun-if-changed=./img");

    let images_names = fs::read_to_string("./img/index.txt").expect("Image packaging failed");
    let images_split = images_names.split("\n").collect::<Vec<&str>>();
    let mut acc = String::new();

    for (i, img_name) in images_split.iter().enumerate() {
        let contents =
            fs::read_to_string("./img/".to_string() + img_name).expect("Couldn't open image file");
        acc += &contents;
        if i + 1 < images_split.len() {
            acc += "\n";
        }
    }
    fs::write("./images.txt", acc).unwrap();
}
