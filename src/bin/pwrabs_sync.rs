#![no_std]
#![feature(collections)]
#![feature(alloc)]
#![feature(box_patterns)]

extern crate alloc;
extern crate collections;
extern crate pwrabs;
extern crate fst;

use alloc::boxed::Box;
use collections::{Vec};
use fst::raw::Fst;
use pwrabs::Verifier;

#[no_mangle]
pub extern fn buf_create() -> *mut Vec<u8> {
    Box::into_raw(Box::new(Vec::new()))
}
#[no_mangle]
pub extern fn buf_destroy(buf: *mut Vec<u8>) {
    unsafe { Box::from_raw(buf) };
}

#[no_mangle]
pub extern fn buf_write(buf: *mut Vec<u8>, len: usize) -> *mut u8 {
    let buf = unsafe { &mut *buf };
    let cap = buf.capacity();
    if len > cap {
        buf.reserve(len - cap);
    }
    unsafe { buf.set_len(len) };
    buf.as_mut_ptr()
}

#[no_mangle]
pub extern fn pwrabs_create() -> *mut Verifier
{
    let pwfst = include_bytes!(concat!(env!("OUT_DIR"), "/pws.fst"));
    let pwmap = Fst::from_static_slice(pwfst).expect("");
    
    let verifier = Verifier::new(pwmap);
    Box::into_raw(Box::new(verifier))
}

#[no_mangle]
pub extern fn pwrabs_verify(verifier: *mut Verifier, set: *const Vec<u8>) -> *const u8 {
    use core::ptr;
    
    let verifier = unsafe { &mut *verifier };
    let set = unsafe { &*set };
    
    match verifier.check(set as &[u8]) {
        Some(err) => err.as_ptr(),
        None => ptr::null()
    }
}

#[no_mangle]
pub extern fn pwrabs_free(verifier: *mut Verifier) {
    drop( unsafe { Box::from_raw(verifier) } );
}

fn main() {}

