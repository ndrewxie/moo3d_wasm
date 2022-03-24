#![allow(clippy::not_unsafe_ptr_arg_deref)]
use std::mem;

#[repr(C)]
pub struct Uint8Array {
    pub data: *mut u8,
    pub length: usize,
    data_owner: Option<Vec<u8>>,
}
impl Uint8Array {
    pub fn new(buf: &mut [u8]) -> *mut Self {
        let len = buf.len();
        let ptr = buf.as_mut_ptr();
        //mem::forget(buf);
        let to_return = Self {
            data: ptr,
            length: len,
            data_owner: None,
        };
        Box::into_raw(Box::new(to_return))
    }
    pub unsafe fn from_vec(mut input: Vec<u8>) -> *mut Self {
        //let mut owner = Box::new(input);
        let length = input.len();
        let ptr = input.as_mut_ptr();
        let to_return = Self {
            data: ptr,
            length,
            data_owner: Some(input),
        };
        Box::into_raw(Box::new(to_return))
    }
    pub fn get_data(self) -> Vec<u8> {
        self.data_owner.unwrap()
    }
}

#[no_mangle]
pub unsafe extern "C" fn get_array_data(input: *mut Uint8Array) -> *mut u8 {
    (*input).data
}

#[no_mangle]
pub unsafe extern "C" fn get_array_length(input: *mut Uint8Array) -> usize {
    (*input).length
}

#[no_mangle]
pub unsafe extern "C" fn free_uint8_arr(input: *mut Uint8Array) {
    drop(Box::from_raw(input));
}

#[no_mangle]
pub unsafe extern "C" fn new_uint8_arr(length: usize) -> *mut Uint8Array {
    let new_vec: Vec<u8> = vec![0; length];
    Uint8Array::from_vec(new_vec)
}
