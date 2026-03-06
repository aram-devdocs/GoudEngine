//! Tests for OpenGL buffer operations (require GL context).

use crate::core::error::GoudError;
use crate::libs::graphics::backend::opengl::backend::OpenGLBackend;
use crate::libs::graphics::backend::types::{BufferHandle, BufferType, BufferUsage};
use crate::libs::graphics::backend::RenderBackend;

#[test]
#[ignore] // Requires OpenGL context
fn test_opengl_backend_creation() {
    let result = OpenGLBackend::new();
    if result.is_ok() {
        let backend = result.unwrap();
        assert_eq!(backend.info().name, "OpenGL");
        assert!(backend.info().version.contains("3.") || backend.info().version.contains("4."));
    }
}

#[test]
#[ignore] // Requires OpenGL context
fn test_buffer_lifecycle() {
    let mut backend = OpenGLBackend::new().unwrap();

    let data: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];
    let handle = backend
        .create_buffer(BufferType::Vertex, BufferUsage::Static, &data)
        .unwrap();

    assert!(backend.is_buffer_valid(handle));
    assert_eq!(backend.buffer_size(handle), Some(8));

    assert!(backend.destroy_buffer(handle));
    assert!(!backend.is_buffer_valid(handle));
    assert_eq!(backend.buffer_size(handle), None);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_buffer_update() {
    let mut backend = OpenGLBackend::new().unwrap();

    let data: Vec<u8> = vec![0; 16];
    let handle = backend
        .create_buffer(BufferType::Vertex, BufferUsage::Dynamic, &data)
        .unwrap();

    let new_data: Vec<u8> = vec![1, 2, 3, 4];
    backend.update_buffer(handle, 0, &new_data).unwrap();

    backend.destroy_buffer(handle);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_buffer_bind() {
    let mut backend = OpenGLBackend::new().unwrap();

    let data: Vec<u8> = vec![1, 2, 3, 4];
    let handle = backend
        .create_buffer(BufferType::Vertex, BufferUsage::Static, &data)
        .unwrap();

    backend.bind_buffer(handle).unwrap();
    backend.unbind_buffer(BufferType::Vertex);

    backend.destroy_buffer(handle);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_multiple_buffers() {
    let mut backend = OpenGLBackend::new().unwrap();

    let data1: Vec<u8> = vec![1, 2, 3, 4];
    let data2: Vec<u8> = vec![5, 6, 7, 8];

    let handle1 = backend
        .create_buffer(BufferType::Vertex, BufferUsage::Static, &data1)
        .unwrap();
    let handle2 = backend
        .create_buffer(BufferType::Index, BufferUsage::Static, &data2)
        .unwrap();

    assert!(backend.is_buffer_valid(handle1));
    assert!(backend.is_buffer_valid(handle2));
    assert_ne!(handle1, handle2);

    backend.destroy_buffer(handle1);
    backend.destroy_buffer(handle2);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_buffer_update_out_of_bounds() {
    let mut backend = OpenGLBackend::new().unwrap();

    let data: Vec<u8> = vec![0; 8];
    let handle = backend
        .create_buffer(BufferType::Vertex, BufferUsage::Dynamic, &data)
        .unwrap();

    let new_data: Vec<u8> = vec![1; 16]; // Too large
    let result = backend.update_buffer(handle, 0, &new_data);
    assert!(result.is_err());

    backend.destroy_buffer(handle);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_invalid_handle_operations() {
    let mut backend = OpenGLBackend::new().unwrap();
    let invalid_handle = BufferHandle::INVALID;

    assert!(!backend.is_buffer_valid(invalid_handle));
    assert_eq!(backend.buffer_size(invalid_handle), None);
    assert!(backend.bind_buffer(invalid_handle).is_err());
    assert!(!backend.destroy_buffer(invalid_handle));
}

#[test]
#[ignore] // Requires OpenGL context
fn test_buffer_slot_reuse() {
    let mut backend = OpenGLBackend::new().unwrap();

    let data: Vec<u8> = vec![1, 2, 3, 4];
    let handle1 = backend
        .create_buffer(BufferType::Vertex, BufferUsage::Static, &data)
        .unwrap();
    backend.destroy_buffer(handle1);

    // Create another buffer - should reuse the slot
    let handle2 = backend
        .create_buffer(BufferType::Vertex, BufferUsage::Static, &data)
        .unwrap();

    // Handles should have same index but different generation
    assert_eq!(handle1.index(), handle2.index());
    assert_ne!(handle1.generation(), handle2.generation());

    assert!(!backend.is_buffer_valid(handle1));
    assert!(backend.is_buffer_valid(handle2));

    backend.destroy_buffer(handle2);
}
