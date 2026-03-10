use crate::core::math::{Color, Rect, Vec2, Vec3, Vec4};
use crate::core::serialization::delta::DeltaEncode;
use crate::ecs::components::transform::Quat;
use crate::ecs::components::{Transform, Transform2D};

// =============================================================================
// Vec2 delta tests
// =============================================================================

#[test]
fn test_vec2_no_changes_returns_none() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(1.0, 2.0);

    assert!(b.delta_from(&a).is_none());
}

#[test]
fn test_vec2_all_fields_changed() {
    let baseline = Vec2::new(1.0, 2.0);
    let current = Vec2::new(3.0, 4.0);

    let delta = current.delta_from(&baseline).unwrap();

    assert_eq!(delta.mask, 0b11);
    assert_eq!(delta.data.len(), 8); // 2 x f32
}

#[test]
fn test_vec2_partial_change_x_only() {
    let baseline = Vec2::new(1.0, 2.0);
    let current = Vec2::new(5.0, 2.0);

    let delta = current.delta_from(&baseline).unwrap();

    assert_eq!(delta.mask, 0b01);
    assert_eq!(delta.data.len(), 4); // 1 x f32
}

#[test]
fn test_vec2_apply_delta_reconstructs() {
    let baseline = Vec2::new(1.0, 2.0);
    let target = Vec2::new(5.0, 2.0);

    let delta = target.delta_from(&baseline).unwrap();
    let reconstructed = baseline.apply_delta(&delta).unwrap();

    assert_eq!(reconstructed, target);
}

// =============================================================================
// Vec3 delta tests
// =============================================================================

#[test]
fn test_vec3_no_changes_returns_none() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    assert!(a.delta_from(&a).is_none());
}

#[test]
fn test_vec3_all_fields_changed() {
    let baseline = Vec3::new(1.0, 2.0, 3.0);
    let current = Vec3::new(4.0, 5.0, 6.0);

    let delta = current.delta_from(&baseline).unwrap();

    assert_eq!(delta.mask, 0b111);
    assert_eq!(delta.data.len(), 12);
}

#[test]
fn test_vec3_partial_change_z_only() {
    let baseline = Vec3::new(1.0, 2.0, 3.0);
    let current = Vec3::new(1.0, 2.0, 9.0);

    let delta = current.delta_from(&baseline).unwrap();

    assert_eq!(delta.mask, 0b100);
    assert_eq!(delta.data.len(), 4);
}

#[test]
fn test_vec3_apply_delta_reconstructs() {
    let baseline = Vec3::new(1.0, 2.0, 3.0);
    let target = Vec3::new(1.0, 5.0, 3.0);

    let delta = target.delta_from(&baseline).unwrap();
    let reconstructed = baseline.apply_delta(&delta).unwrap();

    assert_eq!(reconstructed, target);
}

// =============================================================================
// Vec4 delta tests
// =============================================================================

#[test]
fn test_vec4_no_changes_returns_none() {
    let a = Vec4::new(1.0, 2.0, 3.0, 4.0);
    assert!(a.delta_from(&a).is_none());
}

#[test]
fn test_vec4_all_fields_changed() {
    let baseline = Vec4::new(1.0, 2.0, 3.0, 4.0);
    let current = Vec4::new(5.0, 6.0, 7.0, 8.0);

    let delta = current.delta_from(&baseline).unwrap();

    assert_eq!(delta.mask, 0b1111);
    assert_eq!(delta.data.len(), 16);
}

#[test]
fn test_vec4_partial_change_w_only() {
    let baseline = Vec4::new(1.0, 2.0, 3.0, 4.0);
    let current = Vec4::new(1.0, 2.0, 3.0, 9.0);

    let delta = current.delta_from(&baseline).unwrap();

    assert_eq!(delta.mask, 0b1000);
    assert_eq!(delta.data.len(), 4);
}

#[test]
fn test_vec4_apply_delta_reconstructs() {
    let baseline = Vec4::new(1.0, 2.0, 3.0, 4.0);
    let target = Vec4::new(1.0, 2.0, 9.0, 4.0);

    let delta = target.delta_from(&baseline).unwrap();
    let reconstructed = baseline.apply_delta(&delta).unwrap();

    assert_eq!(reconstructed, target);
}

// =============================================================================
// Color delta tests
// =============================================================================

#[test]
fn test_color_no_changes_returns_none() {
    let a = Color::rgba(0.1, 0.2, 0.3, 0.4);
    assert!(a.delta_from(&a).is_none());
}

#[test]
fn test_color_all_fields_changed() {
    let baseline = Color::rgba(0.1, 0.2, 0.3, 0.4);
    let current = Color::rgba(0.5, 0.6, 0.7, 0.8);

    let delta = current.delta_from(&baseline).unwrap();

    assert_eq!(delta.mask, 0b1111);
    assert_eq!(delta.data.len(), 16);
}

#[test]
fn test_color_partial_change_alpha_only() {
    let baseline = Color::rgba(1.0, 1.0, 1.0, 1.0);
    let current = Color::rgba(1.0, 1.0, 1.0, 0.5);

    let delta = current.delta_from(&baseline).unwrap();

    assert_eq!(delta.mask, 0b1000);
    assert_eq!(delta.data.len(), 4);
}

#[test]
fn test_color_apply_delta_reconstructs() {
    let baseline = Color::rgba(0.1, 0.2, 0.3, 1.0);
    let target = Color::rgba(0.1, 0.9, 0.3, 1.0);

    let delta = target.delta_from(&baseline).unwrap();
    let reconstructed = baseline.apply_delta(&delta).unwrap();

    assert_eq!(reconstructed, target);
}

// =============================================================================
// Rect delta tests
// =============================================================================

#[test]
fn test_rect_no_changes_returns_none() {
    let a = Rect::new(0.0, 0.0, 100.0, 50.0);
    assert!(a.delta_from(&a).is_none());
}

#[test]
fn test_rect_all_fields_changed() {
    let baseline = Rect::new(0.0, 0.0, 100.0, 50.0);
    let current = Rect::new(10.0, 20.0, 200.0, 100.0);

    let delta = current.delta_from(&baseline).unwrap();

    assert_eq!(delta.mask, 0b1111);
    assert_eq!(delta.data.len(), 16);
}

#[test]
fn test_rect_partial_change_width_height() {
    let baseline = Rect::new(0.0, 0.0, 100.0, 50.0);
    let current = Rect::new(0.0, 0.0, 200.0, 75.0);

    let delta = current.delta_from(&baseline).unwrap();

    assert_eq!(delta.mask, 0b1100);
    assert_eq!(delta.data.len(), 8);
}

#[test]
fn test_rect_apply_delta_reconstructs() {
    let baseline = Rect::new(0.0, 0.0, 100.0, 50.0);
    let target = Rect::new(5.0, 0.0, 100.0, 75.0);

    let delta = target.delta_from(&baseline).unwrap();
    let reconstructed = baseline.apply_delta(&delta).unwrap();

    assert_eq!(reconstructed, target);
}

// =============================================================================
// Transform2D delta tests
// =============================================================================

#[test]
fn test_transform2d_apply_delta_reconstructs_roundtrip() {
    let baseline = Transform2D::new(Vec2::new(1.0, 2.0), 0.25, Vec2::new(1.0, 1.0));
    let target = Transform2D::new(Vec2::new(4.0, 2.0), 1.5, Vec2::new(2.0, 0.5));

    let delta = target.delta_from(&baseline).unwrap();
    let reconstructed = baseline.apply_delta(&delta).unwrap();

    assert_eq!(reconstructed, target);
}

// =============================================================================
// Transform delta tests
// =============================================================================

#[test]
fn test_transform_apply_delta_reconstructs_roundtrip() {
    let baseline = Transform::new(
        Vec3::new(1.0, 2.0, 3.0),
        Quat::new(0.0, 0.0, 0.0, 1.0),
        Vec3::new(1.0, 1.0, 1.0),
    );
    let target = Transform::new(
        Vec3::new(5.0, 2.0, -2.0),
        Quat::new(0.1, -0.2, 0.3, 0.9),
        Vec3::new(1.5, 0.5, 2.0),
    );

    let delta = target.delta_from(&baseline).unwrap();
    let reconstructed = baseline.apply_delta(&delta).unwrap();

    assert_eq!(reconstructed, target);
}
