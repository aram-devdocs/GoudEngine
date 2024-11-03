
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[repr(C)]
pub struct Point {
    x: f64,
    y: f64,
}

#[no_mangle]
pub extern "C" fn create_point(x: f64, y: f64) -> Point {
    Point { x, y }
}