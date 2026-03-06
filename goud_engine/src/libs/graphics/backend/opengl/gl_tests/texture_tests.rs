//! Tests for OpenGL texture operations (require GL context).

use crate::libs::graphics::backend::opengl::backend::OpenGLBackend;
use crate::libs::graphics::backend::types::{TextureFilter, TextureFormat, TextureWrap};
use crate::libs::graphics::backend::RenderBackend;

#[test]
#[ignore] // Requires OpenGL context
fn test_texture_lifecycle() {
    let mut backend = OpenGLBackend::new().unwrap();

    let pixels: Vec<u8> = vec![255; 256 * 256 * 4];
    let handle = backend
        .create_texture(
            256,
            256,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &pixels,
        )
        .unwrap();

    assert!(backend.is_texture_valid(handle));
    assert_eq!(backend.texture_size(handle), Some((256, 256)));

    assert!(backend.destroy_texture(handle));
    assert!(!backend.is_texture_valid(handle));
    assert_eq!(backend.texture_size(handle), None);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_texture_empty_data() {
    let mut backend = OpenGLBackend::new().unwrap();

    let handle = backend
        .create_texture(
            512,
            512,
            TextureFormat::RGBA8,
            TextureFilter::Nearest,
            TextureWrap::ClampToEdge,
            &[],
        )
        .unwrap();

    assert!(backend.is_texture_valid(handle));
    assert_eq!(backend.texture_size(handle), Some((512, 512)));

    backend.destroy_texture(handle);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_texture_update() {
    let mut backend = OpenGLBackend::new().unwrap();

    let pixels: Vec<u8> = vec![0; 256 * 256 * 4];
    let handle = backend
        .create_texture(
            256,
            256,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &pixels,
        )
        .unwrap();

    let new_pixels: Vec<u8> = vec![255; 64 * 64 * 4];
    backend
        .update_texture(handle, 0, 0, 64, 64, &new_pixels)
        .unwrap();

    backend.destroy_texture(handle);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_texture_update_bounds_check() {
    let mut backend = OpenGLBackend::new().unwrap();

    let pixels: Vec<u8> = vec![0; 128 * 128 * 4];
    let handle = backend
        .create_texture(
            128,
            128,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &pixels,
        )
        .unwrap();

    // Try to update a region that exceeds bounds
    let new_pixels: Vec<u8> = vec![255; 64 * 64 * 4];
    let result = backend.update_texture(handle, 100, 100, 64, 64, &new_pixels);
    assert!(result.is_err());

    backend.destroy_texture(handle);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_texture_bind() {
    let mut backend = OpenGLBackend::new().unwrap();

    let pixels: Vec<u8> = vec![255; 64 * 64 * 4];
    let handle = backend
        .create_texture(
            64,
            64,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &pixels,
        )
        .unwrap();

    backend.bind_texture(handle, 0).unwrap();
    backend.unbind_texture(0);

    backend.destroy_texture(handle);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_texture_multiple_units() {
    let mut backend = OpenGLBackend::new().unwrap();

    let pixels1: Vec<u8> = vec![255; 64 * 64 * 4];
    let pixels2: Vec<u8> = vec![128; 64 * 64 * 4];

    let handle1 = backend
        .create_texture(
            64,
            64,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &pixels1,
        )
        .unwrap();
    let handle2 = backend
        .create_texture(
            64,
            64,
            TextureFormat::RGBA8,
            TextureFilter::Nearest,
            TextureWrap::ClampToEdge,
            &pixels2,
        )
        .unwrap();

    backend.bind_texture(handle1, 0).unwrap();
    backend.bind_texture(handle2, 1).unwrap();
    backend.unbind_texture(0);
    backend.unbind_texture(1);

    backend.destroy_texture(handle1);
    backend.destroy_texture(handle2);
}

#[test]
#[ignore] // Requires OpenGL context
fn test_texture_invalid_dimensions() {
    let mut backend = OpenGLBackend::new().unwrap();

    let result = backend.create_texture(
        0,
        256,
        TextureFormat::RGBA8,
        TextureFilter::Linear,
        TextureWrap::Repeat,
        &[],
    );
    assert!(result.is_err());

    let result = backend.create_texture(
        256,
        0,
        TextureFormat::RGBA8,
        TextureFilter::Linear,
        TextureWrap::Repeat,
        &[],
    );
    assert!(result.is_err());
}

#[test]
#[ignore] // Requires OpenGL context
fn test_texture_slot_reuse() {
    let mut backend = OpenGLBackend::new().unwrap();

    let pixels: Vec<u8> = vec![255; 64 * 64 * 4];
    let handle1 = backend
        .create_texture(
            64,
            64,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &pixels,
        )
        .unwrap();
    backend.destroy_texture(handle1);

    let handle2 = backend
        .create_texture(
            64,
            64,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &pixels,
        )
        .unwrap();

    assert_eq!(handle1.index(), handle2.index());
    assert_ne!(handle1.generation(), handle2.generation());
    assert!(!backend.is_texture_valid(handle1));
    assert!(backend.is_texture_valid(handle2));

    backend.destroy_texture(handle2);
}
