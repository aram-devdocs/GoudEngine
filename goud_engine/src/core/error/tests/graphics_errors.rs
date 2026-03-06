//! Tests for GoudError graphics variants (codes 200-299).

use crate::core::error::{
    GoudError, ERR_BACKEND_NOT_SUPPORTED, ERR_BUFFER_CREATION_FAILED, ERR_DRAW_CALL_FAILED,
    ERR_RENDER_TARGET_FAILED, ERR_SHADER_COMPILATION_FAILED, ERR_SHADER_LINK_FAILED,
    ERR_TEXTURE_CREATION_FAILED,
};

#[test]
fn test_shader_compilation_failed_error_code() {
    let error = GoudError::ShaderCompilationFailed(
        "ERROR: 0:15: 'vec3' : undeclared identifier".to_string(),
    );
    assert_eq!(error.error_code(), ERR_SHADER_COMPILATION_FAILED);
    assert_eq!(error.error_code(), 200);
}

#[test]
fn test_shader_link_failed_error_code() {
    let error =
        GoudError::ShaderLinkFailed("ERROR: Varying variable 'vTexCoord' not written".to_string());
    assert_eq!(error.error_code(), ERR_SHADER_LINK_FAILED);
    assert_eq!(error.error_code(), 201);
}

#[test]
fn test_texture_creation_failed_error_code() {
    let error = GoudError::TextureCreationFailed("GL_OUT_OF_MEMORY: 4096x4096 RGBA8".to_string());
    assert_eq!(error.error_code(), ERR_TEXTURE_CREATION_FAILED);
    assert_eq!(error.error_code(), 210);
}

#[test]
fn test_buffer_creation_failed_error_code() {
    let error =
        GoudError::BufferCreationFailed("Failed to allocate 256MB vertex buffer".to_string());
    assert_eq!(error.error_code(), ERR_BUFFER_CREATION_FAILED);
    assert_eq!(error.error_code(), 211);
}

#[test]
fn test_render_target_failed_error_code() {
    let error = GoudError::RenderTargetFailed("GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT".to_string());
    assert_eq!(error.error_code(), ERR_RENDER_TARGET_FAILED);
    assert_eq!(error.error_code(), 220);
}

#[test]
fn test_backend_not_supported_error_code() {
    let error = GoudError::BackendNotSupported("Vulkan 1.2 required, found 1.0".to_string());
    assert_eq!(error.error_code(), ERR_BACKEND_NOT_SUPPORTED);
    assert_eq!(error.error_code(), 230);
}

#[test]
fn test_draw_call_failed_error_code() {
    let error =
        GoudError::DrawCallFailed("glDrawElements failed: GL_INVALID_OPERATION".to_string());
    assert_eq!(error.error_code(), ERR_DRAW_CALL_FAILED);
    assert_eq!(error.error_code(), 240);
}

#[test]
fn test_all_graphics_errors_in_graphics_category() {
    let errors: Vec<GoudError> = vec![
        GoudError::ShaderCompilationFailed("test".to_string()),
        GoudError::ShaderLinkFailed("test".to_string()),
        GoudError::TextureCreationFailed("test".to_string()),
        GoudError::BufferCreationFailed("test".to_string()),
        GoudError::RenderTargetFailed("test".to_string()),
        GoudError::BackendNotSupported("test".to_string()),
        GoudError::DrawCallFailed("test".to_string()),
    ];

    for error in errors {
        assert_eq!(
            error.category(),
            "Graphics",
            "Error {:?} should be in Graphics category",
            error
        );
    }
}

#[test]
fn test_graphics_error_codes_in_valid_range() {
    let errors: Vec<GoudError> = vec![
        GoudError::ShaderCompilationFailed("test".to_string()),
        GoudError::ShaderLinkFailed("test".to_string()),
        GoudError::TextureCreationFailed("test".to_string()),
        GoudError::BufferCreationFailed("test".to_string()),
        GoudError::RenderTargetFailed("test".to_string()),
        GoudError::BackendNotSupported("test".to_string()),
        GoudError::DrawCallFailed("test".to_string()),
    ];

    for error in errors {
        let code = error.error_code();
        assert!(
            code >= 200 && code < 300,
            "Graphics error {:?} has code {} which is outside range 200-299",
            error,
            code
        );
    }
}

#[test]
fn test_graphics_errors_preserve_message() {
    let shader_err = "ERROR: 0:42: 'sampler2D' : syntax error";
    if let GoudError::ShaderCompilationFailed(msg) =
        GoudError::ShaderCompilationFailed(shader_err.to_string())
    {
        assert_eq!(msg, shader_err);
        assert!(msg.contains("42"));
    } else {
        panic!("Expected ShaderCompilationFailed variant");
    }

    let backend_err = "Metal not available on Linux; available: OpenGL 4.5, Vulkan 1.2";
    if let GoudError::BackendNotSupported(msg) =
        GoudError::BackendNotSupported(backend_err.to_string())
    {
        assert_eq!(msg, backend_err);
    } else {
        panic!("Expected BackendNotSupported variant");
    }
}

#[test]
fn test_graphics_error_equality() {
    let err1 = GoudError::ShaderCompilationFailed("error line 10".to_string());
    let err2 = GoudError::ShaderCompilationFailed("error line 10".to_string());
    assert_eq!(err1, err2);

    let err3 = GoudError::ShaderCompilationFailed("error line 20".to_string());
    assert_ne!(err1, err3);

    let err4 = GoudError::ShaderLinkFailed("error line 10".to_string());
    assert_ne!(err1, err4);

    assert_ne!(
        GoudError::TextureCreationFailed("fail".to_string()),
        GoudError::BufferCreationFailed("fail".to_string())
    );
}

#[test]
fn test_graphics_error_debug_format() {
    let error = GoudError::ShaderCompilationFailed("syntax error at line 5".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("ShaderCompilationFailed"));
    assert!(debug_str.contains("syntax error at line 5"));

    let error2 = GoudError::DrawCallFailed("invalid state".to_string());
    let debug_str2 = format!("{:?}", error2);
    assert!(debug_str2.contains("DrawCallFailed"));
    assert!(debug_str2.contains("invalid state"));
}

#[test]
fn test_graphics_error_codes_are_distinct() {
    let codes = vec![
        ERR_SHADER_COMPILATION_FAILED,
        ERR_SHADER_LINK_FAILED,
        ERR_TEXTURE_CREATION_FAILED,
        ERR_BUFFER_CREATION_FAILED,
        ERR_RENDER_TARGET_FAILED,
        ERR_BACKEND_NOT_SUPPORTED,
        ERR_DRAW_CALL_FAILED,
    ];

    for (i, code1) in codes.iter().enumerate() {
        for (j, code2) in codes.iter().enumerate() {
            if i != j {
                assert_ne!(
                    code1, code2,
                    "Error codes at index {} and {} should be different",
                    i, j
                );
            }
        }
    }
}

#[test]
fn test_graphics_error_code_gaps_for_future_expansion() {
    assert!(ERR_SHADER_COMPILATION_FAILED == 200);
    assert!(ERR_SHADER_LINK_FAILED == 201);
    assert!(ERR_TEXTURE_CREATION_FAILED == 210);
    assert!(ERR_BUFFER_CREATION_FAILED == 211);
    assert!(ERR_RENDER_TARGET_FAILED == 220);
    assert!(ERR_BACKEND_NOT_SUPPORTED == 230);
    assert!(ERR_DRAW_CALL_FAILED == 240);
}
