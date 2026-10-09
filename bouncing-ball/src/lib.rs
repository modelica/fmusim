mod add_core;

use add_core::Adder;

#[unsafe(no_mangle)]
pub extern "C" fn adder_create(initial: i32) -> *mut Adder {
    add_core::adder_create(initial)
}

#[unsafe(no_mangle)]
pub extern "C" fn adder_add(ptr: *mut Adder, n: i32) {
    unsafe { add_core::adder_add(ptr, n) }
}

#[unsafe(no_mangle)]
pub extern "C" fn adder_value(ptr: *const Adder) -> i32 {
    unsafe { add_core::adder_value(ptr) }
}

#[unsafe(no_mangle)]
pub extern "C" fn adder_destroy(ptr: *mut Adder) {
    unsafe { add_core::adder_destroy(ptr) }
}
