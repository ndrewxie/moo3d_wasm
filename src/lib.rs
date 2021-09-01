#![allow(clippy::not_unsafe_ptr_arg_deref)]

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod wasm_interopt;
extern crate moo3d_core;

#[no_mangle]
pub extern "C" fn make_game_state(width: usize, height: usize) -> *mut moo3d_core::GameState {
    Box::into_raw(Box::new(moo3d_core::GameState::new(width, height)))
}

#[no_mangle]
pub extern "C" fn get_pixel_data(
    input: *mut moo3d_core::GameState,
) -> *mut wasm_interopt::Uint8Array {
    unsafe { wasm_interopt::Uint8Array::new((*input).get_mut_pixels()) }
}

#[no_mangle]
pub unsafe extern "C" fn render_game(input: *mut moo3d_core::GameState, curr_time: usize) {
    (*input).render(curr_time);
}

#[no_mangle]
pub unsafe extern "C" fn translate_camera(
    input: *mut moo3d_core::GameState,
    trans_x: isize,
    trans_y: isize,
    trans_z: isize,
) {
    (*input).translate_camera(trans_x, trans_y, trans_z);
}

#[no_mangle]
pub unsafe extern "C" fn translate_camera_look(
    input: *mut moo3d_core::GameState,
    trans_x: f32,
    trans_y: f32,
    trans_z: f32,
) {
    (*input).translate_camera_look(trans_x, trans_y, trans_z);
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
