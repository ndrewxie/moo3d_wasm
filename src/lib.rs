#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod wasm_interopt;
extern crate m3d_core;

#[no_mangle]
pub unsafe extern "C" fn make_game_state(
    width: usize,
    height: usize,
    tex: *mut wasm_interopt::Uint8Array,
) -> *mut m3d_core::GameState {
    let owned_textures = Box::from_raw(tex);
    let texture_array = owned_textures.get_data();

    Box::into_raw(Box::new(m3d_core::GameState::new(
        width,
        height,
        &texture_array,
    )))
}

#[no_mangle]
pub extern "C" fn get_pixel_data(
    input: *mut m3d_core::GameState,
) -> *mut wasm_interopt::Uint8Array {
    unsafe { wasm_interopt::Uint8Array::new((*input).get_mut_pixels()) }
}

#[no_mangle]
pub unsafe extern "C" fn render_game(input: *mut m3d_core::GameState, curr_time: usize) {
    (*input).render(curr_time);
}

#[no_mangle]
pub unsafe extern "C" fn translate_camera(
    input: *mut m3d_core::GameState,
    trans_x: isize,
    trans_y: isize,
    trans_z: isize,
) {
    (*input).translate_camera(trans_x, trans_y, trans_z);
}

#[no_mangle]
pub unsafe extern "C" fn rotate_camera(
    input: *mut m3d_core::GameState,
    d_rotation: f32,
    d_inclination: f32,
) {
    (*input).rotate_camera(d_rotation, d_inclination);
}

// tests
#[no_mangle]
pub extern "C" fn test_return_5() -> usize {
    return 5;
}

#[no_mangle]
pub unsafe extern "C" fn test_return_arr() -> *mut wasm_interopt::Uint8Array {
    wasm_interopt::Uint8Array::from_vec(vec![1, 2, 3, 4, 5])
}
