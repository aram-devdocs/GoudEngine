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

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_vec_opaque_type() {
        // Test that Vec is a zero-sized type (ZST)
        assert_eq!(mem::size_of::<Vec>(), 0);
        
        // Test that we can create an instance (though it's not useful)
        let _vec = Vec { _private: [] };
    }

    #[test]
    fn test_vector3_fields() {
        let vec3 = Vector3 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        
        assert_eq!(vec3.x, 1.0);
        assert_eq!(vec3.y, 2.0);
        assert_eq!(vec3.z, 3.0);
        
        // Test size is 3 floats
        assert_eq!(mem::size_of::<Vector3>(), 3 * mem::size_of::<f32>());
    }

    #[test]
    fn test_glfw_opaque_type() {
        assert_eq!(mem::size_of::<Glfw>(), 0);
        let _glfw = Glfw { _private: [] };
    }

    #[test]
    fn test_receiver_opaque_type() {
        assert_eq!(mem::size_of::<Receiver>(), 0);
        let _receiver = Receiver { _private: [] };
    }

    #[test]
    fn test_hashset_opaque_type() {
        assert_eq!(mem::size_of::<HashSet>(), 0);
        let _hashset = HashSet { _private: [] };
    }

    #[test]
    fn test_shader_program_opaque_type() {
        assert_eq!(mem::size_of::<ShaderProgram>(), 0);
        let _shader = ShaderProgram { _private: [] };
    }

    #[test]
    fn test_vao_opaque_type() {
        assert_eq!(mem::size_of::<Vao>(), 0);
        let _vao = Vao { _private: [] };
    }

    #[test]
    fn test_duration_fields() {
        let duration = Duration {
            secs: 42,
            nanos: 500_000_000, // 0.5 seconds
        };
        
        assert_eq!(duration.secs, 42);
        assert_eq!(duration.nanos, 500_000_000);
        
        // Test size - Duration may have padding for alignment
        // It should be at least the size of its fields
        assert!(mem::size_of::<Duration>() >= mem::size_of::<u64>() + mem::size_of::<u32>());
        
        // Due to repr(C), the struct is likely padded to 16 bytes for u64 alignment
        assert_eq!(mem::size_of::<Duration>(), 16);
    }

    #[test]
    fn test_instant_fields() {
        let instant = Instant {
            secs: 1_234_567_890,
            nanos: 123_456_789,
        };
        
        assert_eq!(instant.secs, 1_234_567_890);
        assert_eq!(instant.nanos, 123_456_789);
        
        // Test size - Instant may have padding for alignment
        // It should be at least the size of its fields
        assert!(mem::size_of::<Instant>() >= mem::size_of::<u64>() + mem::size_of::<u32>());
        
        // Due to repr(C), the struct is likely padded to 16 bytes for u64 alignment
        assert_eq!(mem::size_of::<Instant>(), 16);
    }

    #[test]
    fn test_ecs_opaque_type() {
        assert_eq!(mem::size_of::<Ecs>(), 0);
        let _ecs = Ecs { _private: [] };
    }

    #[test]
    fn test_hashmap_opaque_type() {
        assert_eq!(mem::size_of::<HashMap>(), 0);
        let _hashmap = HashMap { _private: [] };
    }

    #[test]
    fn test_glfw_receiver_opaque_type() {
        assert_eq!(mem::size_of::<GlfwReceiver>(), 0);
        let _receiver = GlfwReceiver { _private: [] };
    }

    #[test]
    fn test_pwindow_opaque_type() {
        assert_eq!(mem::size_of::<PWindow>(), 0);
        let _window = PWindow { _private: [] };
    }

    #[test]
    fn test_skybox_opaque_type() {
        assert_eq!(mem::size_of::<Skybox>(), 0);
        let _skybox = Skybox { _private: [] };
    }

    #[test]
    fn test_all_opaque_types_are_zero_sized() {
        // Comprehensive test to ensure all opaque types are zero-sized
        let opaque_sizes = vec![
            ("Vec", mem::size_of::<Vec>()),
            ("Glfw", mem::size_of::<Glfw>()),
            ("Receiver", mem::size_of::<Receiver>()),
            ("HashSet", mem::size_of::<HashSet>()),
            ("ShaderProgram", mem::size_of::<ShaderProgram>()),
            ("Vao", mem::size_of::<Vao>()),
            ("Ecs", mem::size_of::<Ecs>()),
            ("HashMap", mem::size_of::<HashMap>()),
            ("GlfwReceiver", mem::size_of::<GlfwReceiver>()),
            ("PWindow", mem::size_of::<PWindow>()),
            ("Skybox", mem::size_of::<Skybox>()),
        ];
        
        for (name, size) in opaque_sizes {
            assert_eq!(size, 0, "{} should be zero-sized", name);
        }
    }

    #[test]
    fn test_repr_c_alignment() {
        // Test that Duration and Instant have proper alignment for C interop
        // These should match the alignment of their largest field (u64)
        assert_eq!(mem::align_of::<Duration>(), mem::align_of::<u64>());
        assert_eq!(mem::align_of::<Instant>(), mem::align_of::<u64>());
        
        // Vector3 should have f32 alignment
        assert_eq!(mem::align_of::<Vector3>(), mem::align_of::<f32>());
    }

    #[test]
    fn test_duration_nanos_range() {
        // Nanoseconds should be less than 1 second (1_000_000_000 nanos)
        let valid_duration = Duration {
            secs: 10,
            nanos: 999_999_999,
        };
        
        assert!(valid_duration.nanos < 1_000_000_000);
    }

    #[test]
    fn test_instant_nanos_range() {
        // Similar to Duration, nanos should be less than 1 second
        let valid_instant = Instant {
            secs: 10,
            nanos: 999_999_999,
        };
        
        assert!(valid_instant.nanos < 1_000_000_000);
    }
}
