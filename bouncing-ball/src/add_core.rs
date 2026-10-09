pub struct Adder {
    value: f64,
}

impl Adder {
    pub fn new(initial: f64) -> Self {
        Self { value: initial }
    }

    pub fn add(&mut self, n: f64) {
        self.value += n;
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

pub fn adder_create(initial: f64) -> *mut Adder {
    Box::into_raw(Box::new(Adder::new(initial)))
}

pub unsafe fn adder_add(ptr: *mut Adder, n: f64) {
    if let Some(adder) = unsafe { ptr.as_mut() } {
        adder.add(n);
    }
}

/// Writes the adder's current value into every element of `out[0..len]`.
pub unsafe fn adder_value(ptr: *const Adder, out: *mut f64, len: i32) {
    if out.is_null() || len <= 0 {
        return;
    }
    let value = match unsafe { ptr.as_ref() } {
        Some(adder) => adder.value(),
        None => return,
    };
    let out = unsafe { std::slice::from_raw_parts_mut(out, len as usize) };
    out.fill(value);
}

pub unsafe fn adder_destroy(ptr: *mut Adder) {
    if !ptr.is_null() {
        drop(unsafe { Box::from_raw(ptr) });
    }
}
