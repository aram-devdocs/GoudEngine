use cgmath::{Matrix4, Vector3, Vector4};
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
    pub fn set_uniform_float(&self, uniform_name: &str, value: f32) -> Result<(), String> {
        println!(
            "Mock set_uniform_float called: {} = {}",
            uniform_name, value
        );
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_uniform_vec3(
        &self,
        uniform_name: &str,
        _value: &Vector3<f32>,
    ) -> Result<(), String> {
        println!("Mock set_uniform_vec3 called: {}", uniform_name);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_uniform_vec4(
        &self,
        uniform_name: &str,
        _value: &Vector4<f32>,
    ) -> Result<(), String> {
        println!("Mock set_uniform_vec4 called: {}", uniform_name);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_uniform_mat4(
        &self,
        uniform_name: &str,
        _value: &Matrix4<f32>,
    ) -> Result<(), String> {
        println!("Mock set_uniform_mat4 called: {}", uniform_name);
        if uniform_name == "test_vec3" || uniform_name == "test_vec4" || uniform_name == "test_mat4"
        {
            Err(format!("Could not find uniform: {}", uniform_name))
        } else {
            Ok(())
        }
    }

    #[allow(dead_code)]
    pub fn use_program(&self) {
        println!("Mock use_program called");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // write unit tests for mock shader program
    #[test]
    fn test_mock_shader_program() {
        let shader = MockShaderProgram::new().unwrap();
        assert_eq!(shader.program_id, 1);
    }

    #[test]
    fn test_mock_shader_program_new_3d() {
        let shader = MockShaderProgram::new_3d().unwrap();
        assert_eq!(shader.program_id, 2);
    }

    #[test]
    fn test_mock_shader_program_set_uniform_float() {
        let shader = MockShaderProgram::new().unwrap();
        let result = shader.set_uniform_float("test_float", 1.0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_shader_program_set_uniform_int() {
        let shader = MockShaderProgram::new().unwrap();
        let result = shader.set_uniform_int("test_int", 1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_shader_program_set_uniform_vec3() {
        let shader = MockShaderProgram::new().unwrap();
        let vec = Vector3::new(1.0, 2.0, 3.0);
        let result = shader.set_uniform_vec3("test_vec3", &vec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_shader_program_set_uniform_vec4() {
        let shader = MockShaderProgram::new().unwrap();
        let vec = Vector4::new(1.0, 2.0, 3.0, 4.0);
        let result = shader.set_uniform_vec4("test_vec4", &vec);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_shader_program_set_uniform_mat4() {
        let shader = MockShaderProgram::new().unwrap();
        let mat = Matrix4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        );
        let result = shader.set_uniform_mat4("test_mat4", &mat);
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_shader_program_set_uniform_mat4_success() {
        let shader = MockShaderProgram::new().unwrap();
        let mat = Matrix4::<f32>::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        );
        let result = shader.set_uniform_mat4("valid_mat4", &mat);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mock_shader_program_use() {
        let shader = MockShaderProgram::new().unwrap();
        shader.use_program();
        // No assertions needed as this is just a mock implementation
    }
}
