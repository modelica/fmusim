pub struct Adder {
    value: i32,
}

impl Adder {
    pub fn new(initial: i32) -> Self {
        Self { value: initial }
    }

    pub fn add(&mut self, n: i32) {
        self.value += n;
    }

    pub fn value(&self) -> i32 {
        self.value
    }
}

pub fn adder_create(initial: i32) -> *mut Adder {
    Box::into_raw(Box::new(Adder::new(initial)))
}

pub unsafe fn adder_add(ptr: *mut Adder, n: i32) {
    if let Some(adder) = unsafe { ptr.as_mut() } {
        adder.add(n);
    }
}

pub unsafe fn adder_value(ptr: *const Adder) -> i32 {
    unsafe { ptr.as_ref().map(Adder::value).unwrap_or(0) }
}

pub unsafe fn adder_destroy(ptr: *mut Adder) {
    if !ptr.is_null() {
        drop(unsafe { Box::from_raw(ptr) });
    }
}