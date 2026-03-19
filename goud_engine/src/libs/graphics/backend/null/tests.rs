//! Unit tests for the NullBackend.

use super::NullBackend;
use crate::libs::graphics::backend::types::{
    BufferType, BufferUsage, PrimitiveTopology, RenderTargetDesc, TextureFilter, TextureFormat,
    TextureWrap,
};
use crate::libs::graphics::backend::{
    BufferOps, ClearOps, DrawOps, FrameOps, RenderBackend, RenderTargetOps, ShaderOps, StateOps,
    TextureOps,
};

#[test]
fn test_buffer_lifecycle() {
    let mut backend = NullBackend::new();
    let data = [1u8, 2, 3, 4];

    let handle = backend
        .create_buffer(BufferType::Vertex, BufferUsage::Static, &data)
        .expect("create_buffer should succeed");

    assert!(backend.is_buffer_valid(handle));
    assert_eq!(backend.buffer_size(handle), Some(4));

    backend
        .update_buffer(handle, 0, &[5, 6])
        .expect("update_buffer should succeed");

    assert!(backend.destroy_buffer(handle));
    assert!(!backend.is_buffer_valid(handle));
}

#[test]
fn test_texture_lifecycle() {
    let mut backend = NullBackend::new();

    let handle = backend
        .create_texture(
            256,
            128,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &[],
        )
        .expect("create_texture should succeed");

    assert!(backend.is_texture_valid(handle));
    assert_eq!(backend.texture_size(handle), Some((256, 128)));

    assert!(backend.destroy_texture(handle));
    assert!(!backend.is_texture_valid(handle));
}

#[test]
fn test_shader_lifecycle() {
    let mut backend = NullBackend::new();

    let handle = backend
        .create_shader("vertex", "fragment")
        .expect("create_shader should succeed");

    assert!(backend.is_shader_valid(handle));

    assert!(backend.destroy_shader(handle));
    assert!(!backend.is_shader_valid(handle));
}

#[test]
fn test_double_destroy_returns_false() {
    let mut backend = NullBackend::new();

    let buf = backend
        .create_buffer(BufferType::Index, BufferUsage::Dynamic, &[0; 16])
        .unwrap();
    assert!(backend.destroy_buffer(buf));
    assert!(!backend.destroy_buffer(buf));

    let tex = backend
        .create_texture(
            64,
            64,
            TextureFormat::RGB8,
            TextureFilter::Nearest,
            TextureWrap::ClampToEdge,
            &[],
        )
        .unwrap();
    assert!(backend.destroy_texture(tex));
    assert!(!backend.destroy_texture(tex));

    let shader = backend.create_shader("v", "f").unwrap();
    assert!(backend.destroy_shader(shader));
    assert!(!backend.destroy_shader(shader));
}

#[test]
fn test_state_methods_do_not_panic() {
    let mut backend = NullBackend::new();

    backend.set_clear_color(1.0, 0.0, 0.0, 1.0);
    assert_eq!(backend.clear_color, [1.0, 0.0, 0.0, 1.0]);

    backend.enable_depth_test();
    assert!(backend.depth_test_enabled);
    backend.disable_depth_test();
    assert!(!backend.depth_test_enabled);

    backend.enable_blending();
    assert!(backend.blending_enabled);
    backend.disable_blending();
    assert!(!backend.blending_enabled);

    backend.enable_culling();
    assert!(backend.culling_enabled);
    backend.disable_culling();
    assert!(!backend.culling_enabled);

    backend.set_viewport(10, 20, 1920, 1080);
    assert_eq!(backend.viewport, (10, 20, 1920, 1080));

    backend.set_line_width(2.5);
    assert!((backend.line_width - 2.5).abs() < f32::EPSILON);

    backend.set_depth_mask(false);
    assert!(!backend.depth_mask_enabled);
}

#[test]
fn test_frame_lifecycle() {
    let mut backend = NullBackend::new();
    backend.begin_frame().expect("begin_frame should succeed");
    backend.clear();
    backend.end_frame().expect("end_frame should succeed");
}

#[test]
fn test_draw_calls_do_not_panic() {
    let mut backend = NullBackend::new();

    backend
        .draw_arrays(PrimitiveTopology::Triangles, 0, 3)
        .expect("draw_arrays should succeed");
    backend
        .draw_indexed(PrimitiveTopology::Triangles, 6, 0)
        .expect("draw_indexed should succeed");
    backend
        .draw_indexed_u16(PrimitiveTopology::Lines, 4, 0)
        .expect("draw_indexed_u16 should succeed");
    backend
        .draw_arrays_instanced(PrimitiveTopology::Triangles, 0, 6, 100)
        .expect("draw_arrays_instanced should succeed");
    backend
        .draw_indexed_instanced(PrimitiveTopology::Triangles, 6, 0, 50)
        .expect("draw_indexed_instanced should succeed");

    assert_eq!(backend.draw_arrays_calls(), 1);
    assert_eq!(backend.draw_indexed_calls(), 2);
    assert_eq!(backend.draw_arrays_instanced_calls(), 1);
    assert_eq!(backend.draw_indexed_instanced_calls(), 1);
}

#[test]
fn test_info_values() {
    let backend = NullBackend::new();
    let info = backend.info();

    assert_eq!(info.name, "Null");
    assert_eq!(info.version, "1.0");
    assert_eq!(info.vendor, "Software");
    assert_eq!(info.renderer, "NullBackend");
    assert!(info.capabilities.supports_instancing);
    assert!(!info.capabilities.supports_compute_shaders);
}

#[test]
fn test_destroyed_handle_operations_return_errors() {
    let mut backend = NullBackend::new();

    let buf = backend
        .create_buffer(BufferType::Vertex, BufferUsage::Static, &[1, 2, 3])
        .unwrap();
    backend.destroy_buffer(buf);
    assert!(backend.update_buffer(buf, 0, &[4, 5]).is_err());
    assert!(backend.bind_buffer(buf).is_err());
    assert_eq!(backend.buffer_size(buf), None);

    let tex = backend
        .create_texture(
            32,
            32,
            TextureFormat::RGBA8,
            TextureFilter::Linear,
            TextureWrap::Repeat,
            &[],
        )
        .unwrap();
    backend.destroy_texture(tex);
    assert!(backend.update_texture(tex, 0, 0, 16, 16, &[]).is_err());
    assert!(backend.bind_texture(tex, 0).is_err());
    assert_eq!(backend.texture_size(tex), None);

    let shader = backend.create_shader("v", "f").unwrap();
    backend.destroy_shader(shader);
    assert!(backend.bind_shader(shader).is_err());
    assert_eq!(backend.get_uniform_location(shader, "u_mvp"), None);
}

#[test]
fn test_default_trait() {
    let backend = NullBackend::default();
    assert_eq!(backend.info().name, "Null");
}

#[test]
fn test_render_target_lifecycle() {
    let mut backend = NullBackend::new();
    backend.set_viewport(5, 7, 320, 180);

    let render_target = backend
        .create_render_target(&RenderTargetDesc {
            width: 128,
            height: 64,
            format: TextureFormat::RGBA8,
            has_depth: true,
        })
        .expect("create_render_target should succeed");

    assert!(backend.is_render_target_valid(render_target));
    let color_texture = backend
        .render_target_texture(render_target)
        .expect("render target should expose its color texture");
    assert!(backend.is_texture_valid(color_texture));
    assert_eq!(backend.texture_size(color_texture), Some((128, 64)));

    backend
        .bind_render_target(Some(render_target))
        .expect("bind_render_target should succeed");
    assert_eq!(backend.viewport, (0, 0, 128, 64));

    backend
        .bind_render_target(None)
        .expect("bind_render_target(None) should succeed");
    assert_eq!(backend.viewport, (5, 7, 320, 180));

    assert!(backend.destroy_render_target(render_target));
    assert!(!backend.is_render_target_valid(render_target));
    assert!(!backend.is_texture_valid(color_texture));
}
