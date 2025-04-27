#[repr(C)]
#[allow(dead_code)]
pub struct Vec {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]
pub struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
}

// Opaque pointers for additional structures
#[repr(C)]
#[allow(dead_code)]
pub struct Glfw {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]

pub struct Receiver {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]

pub struct HashSet {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]

pub struct ShaderProgram {
    _private: [u8; 0],
}

// vao vec

#[repr(C)]
#[allow(dead_code)]

pub struct Vao {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]

pub struct Duration {
    secs: u64,
    nanos: u32, // Duration is a struct with two fields: secs and nanos. Nanos is nanoseconds coming from std::time::Duration.
}

#[repr(C)]
#[allow(dead_code)]

pub struct Instant {
    secs: u64,
    nanos: u32, // Instant is a struct with two fields: secs and nanos. Nanos is nanoseconds coming from std::time::Instant.
}

#[repr(C)]
#[allow(dead_code)]

pub struct Ecs {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]
pub struct HashMap {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]
pub struct GlfwReceiver {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]
pub struct PWindow {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(dead_code)]
pub struct Skybox {
    _private: [u8; 0],
}
