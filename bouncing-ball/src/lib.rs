mod add_core;

use add_core::Adder;

#[unsafe(no_mangle)]
pub extern "C" fn adder_create(initial: f64) -> *mut Adder {
    add_core::adder_create(initial)
}

#[unsafe(no_mangle)]
pub extern "C" fn adder_add(ptr: *mut Adder, n: f64) {
    unsafe { add_core::adder_add(ptr, n) }
}

#[unsafe(no_mangle)]
pub extern "C" fn adder_value(ptr: *const Adder, out: *mut f64, len: i32) {
    unsafe { add_core::adder_value(ptr, out, len) }
}

#[unsafe(no_mangle)]
pub extern "C" fn adder_destroy(ptr: *mut Adder) {
    unsafe { add_core::adder_destroy(ptr) }
}
