use std::sync::atomic::AtomicBool;

// Flag to indicate if we've tried initialization and failed
#[allow(dead_code)]
static MOCK_MODE: AtomicBool = AtomicBool::new(true);

/// For testing purposes, always return that we should use mocks
#[allow(dead_code)]
pub fn init_test_context() -> bool {
    println!("Using mock mode for all tests");
    false
}

/// For tests that don't need to check rendering results, just create a mock shader
pub struct MockShaderProgram {
    #[allow(dead_code)]
    program_id: u32,
}

impl MockShaderProgram {
    #[allow(dead_code)]
    pub fn new() -> Result<Self, String> {
        println!("Creating mock 2D shader");
        Ok(MockShaderProgram { program_id: 1 })
    }

    #[allow(dead_code)]
    pub fn new_3d() -> Result<Self, String> {
        println!("Creating mock 3D shader");
        Ok(MockShaderProgram { program_id: 2 })
    }

    #[allow(dead_code)]
    pub fn new_skybox() -> Result<Self, String> {
        println!("Creating mock skybox shader");
        Ok(MockShaderProgram { program_id: 3 })
    }

    #[allow(dead_code)]
    pub fn bind(&self) {
        println!("Mock shader bind called");
    }

    #[allow(dead_code)]
    pub fn terminate(&self) {
        println!("Mock shader terminate called");
    }

    #[allow(dead_code)]
    pub fn create_uniform(&mut self, uniform_name: &str) -> Result<(), String> {
        println!("Mock create_uniform called with: {}", uniform_name);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_uniform_int(&self, uniform_name: &str, value: i32) -> Result<(), String> {
        println!("Mock set_uniform_int called: {} = {}", uniform_name, value);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_uniform_vec3(
        &self,
        uniform_name: &str,
        _value: &cgmath::Vector3<f32>,
    ) -> Result<(), String> {
        println!("Mock set_uniform_vec3 called: {}", uniform_name);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_uniform_vec4(
        &self,
        uniform_name: &str,
        _value: &cgmath::Vector4<f32>,
    ) -> Result<(), String> {
        println!("Mock set_uniform_vec4 called: {}", uniform_name);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_uniform_mat4(
        &self,
        uniform_name: &str,
        _value: &cgmath::Matrix4<f32>,
    ) -> Result<(), String> {
        println!("Mock set_uniform_mat4 called: {}", uniform_name);
        if uniform_name == "test_vec3" || uniform_name == "test_vec4" || uniform_name == "test_mat4"
        {
            Err(format!("Could not find uniform: {}", uniform_name))
        } else {
            Ok(())
        }
    }
}
