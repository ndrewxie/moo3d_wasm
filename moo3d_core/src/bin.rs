use moo3d_core::GameState;
use std::fs;

pub fn test_manager(n: usize) {
    let mut gs_manager = GameState::new(1265, 632, &fs::read("images.bin").unwrap());
    //gs_manager.renderer.camera.translate(0, 0, 2300);
    //let mut gs_manager = GameState::new(1266, 633);
    for j in 0..n {
        gs_manager.render(j);
    }
}

pub fn main() {
    println!("moo3d_core test starting...");
    test_manager(500); // 500
    println!("moo3d_core test finished.");
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
