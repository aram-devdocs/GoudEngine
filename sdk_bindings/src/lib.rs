use std::ffi::c_void;
// use std::mem;
// use std::ptr;

#[repr(C)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[no_mangle]
pub extern "C" fn create_point(x: f64, y: f64) -> *mut Point {
    let point = Box::new(Point { x, y });
    Box::into_raw(point)
}

#[no_mangle]
pub extern "C" fn get_x(point: *const Point) -> f64 {
    unsafe { (*point).x }
}

#[no_mangle]
pub extern "C" fn get_y(point: *const Point) -> f64 {
    unsafe { (*point).y }
}

#[no_mangle]
pub extern "C" fn free_point(point: *mut c_void) {
    if !point.is_null() {
        unsafe {
           let _ = Box::from_raw(point as *mut Point);
        }
    }
}
