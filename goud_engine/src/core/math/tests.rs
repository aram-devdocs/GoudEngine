//! Tests for all math types.

use crate::core::math::{Color, Rect, Vec2, Vec3, Vec4};

// =========================================================================
// Vec2 Tests
// =========================================================================

#[test]
fn test_vec2_constructors() {
    assert_eq!(Vec2::new(1.0, 2.0), Vec2 { x: 1.0, y: 2.0 });
    assert_eq!(Vec2::zero(), Vec2 { x: 0.0, y: 0.0 });
    assert_eq!(Vec2::one(), Vec2 { x: 1.0, y: 1.0 });
    assert_eq!(Vec2::unit_x(), Vec2 { x: 1.0, y: 0.0 });
    assert_eq!(Vec2::unit_y(), Vec2 { x: 0.0, y: 1.0 });
}

#[test]
fn test_vec2_dot() {
    let a = Vec2::new(2.0, 3.0);
    let b = Vec2::new(4.0, 5.0);
    assert_eq!(a.dot(b), 23.0); // 2*4 + 3*5 = 8 + 15 = 23
}

#[test]
fn test_vec2_length() {
    let v = Vec2::new(3.0, 4.0);
    assert_eq!(v.length_squared(), 25.0);
    assert_eq!(v.length(), 5.0);
}

#[test]
fn test_vec2_normalize() {
    let v = Vec2::new(3.0, 4.0);
    let n = v.normalize();
    assert!((n.length() - 1.0).abs() < 0.0001);
    assert_eq!(Vec2::zero().normalize(), Vec2::zero());
}

#[test]
fn test_vec2_operators() {
    let a = Vec2::new(1.0, 2.0);
    let b = Vec2::new(3.0, 4.0);

    assert_eq!(a + b, Vec2::new(4.0, 6.0));
    assert_eq!(a - b, Vec2::new(-2.0, -2.0));
    assert_eq!(a * 2.0, Vec2::new(2.0, 4.0));
    assert_eq!(2.0 * a, Vec2::new(2.0, 4.0));
    assert_eq!(a / 2.0, Vec2::new(0.5, 1.0));
    assert_eq!(-a, Vec2::new(-1.0, -2.0));
}

#[test]
fn test_vec2_lerp() {
    let a = Vec2::new(0.0, 0.0);
    let b = Vec2::new(10.0, 20.0);
    assert_eq!(a.lerp(b, 0.0), a);
    assert_eq!(a.lerp(b, 1.0), b);
    assert_eq!(a.lerp(b, 0.5), Vec2::new(5.0, 10.0));
}

#[test]
fn test_vec2_cgmath_conversion() {
    let goud = Vec2::new(1.0, 2.0);
    let cg: cgmath::Vector2<f32> = goud.into();
    assert_eq!(cg.x, 1.0);
    assert_eq!(cg.y, 2.0);

    let back: Vec2 = cg.into();
    assert_eq!(back, goud);
}

// =========================================================================
// Vec3 Tests
// =========================================================================

#[test]
fn test_vec3_constructors() {
    assert_eq!(
        Vec3::new(1.0, 2.0, 3.0),
        Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );
    assert_eq!(
        Vec3::zero(),
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0
        }
    );
    assert_eq!(
        Vec3::one(),
        Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0
        }
    );
    assert_eq!(
        Vec3::unit_x(),
        Vec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0
        }
    );
    assert_eq!(
        Vec3::unit_y(),
        Vec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0
        }
    );
    assert_eq!(
        Vec3::unit_z(),
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0
        }
    );
}

#[test]
fn test_vec3_cross() {
    let x = Vec3::unit_x();
    let y = Vec3::unit_y();
    let z = x.cross(y);
    assert!((z - Vec3::unit_z()).length() < 0.0001);
}

#[test]
fn test_vec3_operators() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 5.0, 6.0);

    assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
    assert_eq!(a - b, Vec3::new(-3.0, -3.0, -3.0));
    assert_eq!(a * 2.0, Vec3::new(2.0, 4.0, 6.0));
    assert_eq!(2.0 * a, Vec3::new(2.0, 4.0, 6.0));
    assert_eq!(-a, Vec3::new(-1.0, -2.0, -3.0));
}

#[test]
fn test_vec3_cgmath_conversion() {
    let goud = Vec3::new(1.0, 2.0, 3.0);
    let cg: cgmath::Vector3<f32> = goud.into();
    assert_eq!(cg.x, 1.0);
    assert_eq!(cg.y, 2.0);
    assert_eq!(cg.z, 3.0);

    let back: Vec3 = cg.into();
    assert_eq!(back, goud);
}

#[test]
fn test_vec3_dot() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 5.0, 6.0);
    // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
    assert_eq!(a.dot(b), 32.0);

    // Dot product with self is length squared
    assert_eq!(a.dot(a), a.length_squared());

    // Dot product is commutative
    assert_eq!(a.dot(b), b.dot(a));

    // Perpendicular vectors have dot product of 0
    let x = Vec3::unit_x();
    let y = Vec3::unit_y();
    assert_eq!(x.dot(y), 0.0);
}

#[test]
fn test_vec3_cross_properties() {
    let a = Vec3::new(1.0, 2.0, 3.0);
    let b = Vec3::new(4.0, 5.0, 6.0);

    // Cross product is anti-commutative: a × b = -(b × a)
    let ab = a.cross(b);
    let ba = b.cross(a);
    assert!((ab + ba).length() < 0.0001);

    // Cross product is perpendicular to both inputs
    assert!(ab.dot(a).abs() < 0.0001);
    assert!(ab.dot(b).abs() < 0.0001);

    // Right-hand rule: x × y = z
    assert!((Vec3::unit_x().cross(Vec3::unit_y()) - Vec3::unit_z()).length() < 0.0001);
    assert!((Vec3::unit_y().cross(Vec3::unit_z()) - Vec3::unit_x()).length() < 0.0001);
    assert!((Vec3::unit_z().cross(Vec3::unit_x()) - Vec3::unit_y()).length() < 0.0001);
}

#[test]
fn test_vec3_length() {
    // 3-4-5 Pythagorean in 3D extended: sqrt(1 + 4 + 4) = 3
    let v = Vec3::new(1.0, 2.0, 2.0);
    assert_eq!(v.length_squared(), 9.0);
    assert_eq!(v.length(), 3.0);

    // Zero vector has zero length
    assert_eq!(Vec3::zero().length(), 0.0);

    // Unit vectors have length 1
    assert_eq!(Vec3::unit_x().length(), 1.0);
    assert_eq!(Vec3::unit_y().length(), 1.0);
    assert_eq!(Vec3::unit_z().length(), 1.0);
}

#[test]
fn test_vec3_normalize() {
    let v = Vec3::new(3.0, 4.0, 0.0);
    let n = v.normalize();
    assert!((n.length() - 1.0).abs() < 0.0001);
    assert!((n.x - 0.6).abs() < 0.0001);
    assert!((n.y - 0.8).abs() < 0.0001);
    assert_eq!(n.z, 0.0);

    // Zero vector normalizes to zero (safe behavior)
    assert_eq!(Vec3::zero().normalize(), Vec3::zero());

    // Unit vectors remain unchanged
    assert!((Vec3::unit_x().normalize() - Vec3::unit_x()).length() < 0.0001);
}

#[test]
fn test_vec3_lerp() {
    let a = Vec3::new(0.0, 0.0, 0.0);
    let b = Vec3::new(10.0, 20.0, 30.0);

    // t=0 returns start
    assert_eq!(a.lerp(b, 0.0), a);

    // t=1 returns end
    assert_eq!(a.lerp(b, 1.0), b);

    // t=0.5 returns midpoint
    assert_eq!(a.lerp(b, 0.5), Vec3::new(5.0, 10.0, 15.0));

    // Extrapolation works (t > 1)
    assert_eq!(a.lerp(b, 2.0), Vec3::new(20.0, 40.0, 60.0));

    // Negative t extrapolates backwards
    assert_eq!(a.lerp(b, -0.5), Vec3::new(-5.0, -10.0, -15.0));
}

#[test]
fn test_vec3_ffi_layout() {
    use std::mem::{align_of, size_of};

    // Verify Vec3 has expected FFI layout
    assert_eq!(size_of::<Vec3>(), 12); // 3 * f32 = 12 bytes
    assert_eq!(align_of::<Vec3>(), 4); // f32 alignment

    // Verify fields are laid out consecutively with no padding
    let v = Vec3::new(1.0, 2.0, 3.0);
    let ptr = &v as *const Vec3 as *const f32;
    // SAFETY: Vec3 is #[repr(C)] with three consecutive f32 fields.
    // Pointer arithmetic within the bounds of the struct is valid.
    unsafe {
        assert_eq!(*ptr, 1.0); // x at offset 0
        assert_eq!(*ptr.add(1), 2.0); // y at offset 4
        assert_eq!(*ptr.add(2), 3.0); // z at offset 8
    }

    // Verify Default trait
    assert_eq!(Vec3::default(), Vec3::zero());
}

// =========================================================================
// Vec4 Tests
// =========================================================================

#[test]
fn test_vec4_constructors() {
    assert_eq!(
        Vec4::new(1.0, 2.0, 3.0, 4.0),
        Vec4 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            w: 4.0
        }
    );
    assert_eq!(
        Vec4::zero(),
        Vec4 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0
        }
    );
    assert_eq!(
        Vec4::one(),
        Vec4 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
            w: 1.0
        }
    );
}

#[test]
fn test_vec4_from_vec3() {
    let v3 = Vec3::new(1.0, 2.0, 3.0);
    let v4 = Vec4::from_vec3(v3, 4.0);
    assert_eq!(v4, Vec4::new(1.0, 2.0, 3.0, 4.0));
    assert_eq!(v4.xyz(), v3);
}

#[test]
fn test_vec4_cgmath_conversion() {
    let goud = Vec4::new(1.0, 2.0, 3.0, 4.0);
    let cg: cgmath::Vector4<f32> = goud.into();
    let back: Vec4 = cg.into();
    assert_eq!(back, goud);
}

// =========================================================================
// Rect Tests
// =========================================================================

#[test]
fn test_rect_constructors() {
    let r = Rect::new(10.0, 20.0, 100.0, 50.0);
    assert_eq!(r.x, 10.0);
    assert_eq!(r.y, 20.0);
    assert_eq!(r.width, 100.0);
    assert_eq!(r.height, 50.0);
}

#[test]
fn test_rect_from_min_max() {
    let r = Rect::from_min_max(Vec2::new(10.0, 20.0), Vec2::new(110.0, 70.0));
    assert_eq!(r.x, 10.0);
    assert_eq!(r.y, 20.0);
    assert_eq!(r.width, 100.0);
    assert_eq!(r.height, 50.0);
}

#[test]
fn test_rect_accessors() {
    let r = Rect::new(10.0, 20.0, 100.0, 50.0);
    assert_eq!(r.min(), Vec2::new(10.0, 20.0));
    assert_eq!(r.max(), Vec2::new(110.0, 70.0));
    assert_eq!(r.center(), Vec2::new(60.0, 45.0));
    assert_eq!(r.size(), Vec2::new(100.0, 50.0));
    assert_eq!(r.area(), 5000.0);
}

#[test]
fn test_rect_contains() {
    let r = Rect::new(0.0, 0.0, 100.0, 100.0);
    assert!(r.contains(Vec2::new(50.0, 50.0)));
    assert!(r.contains(Vec2::new(0.0, 0.0)));
    assert!(!r.contains(Vec2::new(100.0, 100.0))); // exclusive max
    assert!(!r.contains(Vec2::new(-1.0, 50.0)));
    assert!(!r.contains(Vec2::new(50.0, -1.0)));
}

#[test]
fn test_rect_intersects() {
    let a = Rect::new(0.0, 0.0, 100.0, 100.0);
    let b = Rect::new(50.0, 50.0, 100.0, 100.0);
    let c = Rect::new(200.0, 200.0, 10.0, 10.0);

    assert!(a.intersects(&b));
    assert!(b.intersects(&a));
    assert!(!a.intersects(&c));
    assert!(!c.intersects(&a));
}

#[test]
fn test_rect_intersection() {
    let a = Rect::new(0.0, 0.0, 100.0, 100.0);
    let b = Rect::new(50.0, 50.0, 100.0, 100.0);

    let inter = a.intersection(&b).unwrap();
    assert_eq!(inter.x, 50.0);
    assert_eq!(inter.y, 50.0);
    assert_eq!(inter.width, 50.0);
    assert_eq!(inter.height, 50.0);

    let c = Rect::new(200.0, 200.0, 10.0, 10.0);
    assert!(a.intersection(&c).is_none());
}

// =========================================================================
// Color Tests
// =========================================================================

#[test]
fn test_color_constants() {
    assert_eq!(Color::WHITE, Color::rgba(1.0, 1.0, 1.0, 1.0));
    assert_eq!(Color::BLACK, Color::rgba(0.0, 0.0, 0.0, 1.0));
    assert_eq!(Color::RED, Color::rgba(1.0, 0.0, 0.0, 1.0));
    assert_eq!(Color::TRANSPARENT, Color::rgba(0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_color_from_u8() {
    let c = Color::from_u8(255, 128, 0, 255);
    assert!((c.r - 1.0).abs() < 0.01);
    assert!((c.g - 0.5).abs() < 0.01);
    assert_eq!(c.b, 0.0);
    assert_eq!(c.a, 1.0);
}

#[test]
fn test_color_from_hex() {
    let c1 = Color::from_hex(0xFF0000); // Red
    assert_eq!(c1.r, 1.0);
    assert_eq!(c1.g, 0.0);
    assert_eq!(c1.b, 0.0);
    assert_eq!(c1.a, 1.0);

    let c2 = Color::from_hex(0xFF000080); // Red with 50% alpha
    assert_eq!(c2.r, 1.0);
    assert_eq!(c2.g, 0.0);
    assert_eq!(c2.b, 0.0);
    assert!((c2.a - 0.5).abs() < 0.01);
}

#[test]
fn test_color_vec_conversions() {
    let c = Color::rgba(0.1, 0.2, 0.3, 0.4);
    let v3 = c.to_vec3();
    assert_eq!(v3, Vec3::new(0.1, 0.2, 0.3));

    let v4 = c.to_vec4();
    assert_eq!(v4, Vec4::new(0.1, 0.2, 0.3, 0.4));

    let c2 = Color::from_vec4(v4);
    assert_eq!(c2, c);
}

#[test]
fn test_color_lerp() {
    let a = Color::BLACK;
    let b = Color::WHITE;
    let mid = a.lerp(b, 0.5);
    assert!((mid.r - 0.5).abs() < 0.0001);
    assert!((mid.g - 0.5).abs() < 0.0001);
    assert!((mid.b - 0.5).abs() < 0.0001);
}

#[test]
fn test_color_with_alpha() {
    let c = Color::RED.with_alpha(0.5);
    assert_eq!(c.r, 1.0);
    assert_eq!(c.g, 0.0);
    assert_eq!(c.b, 0.0);
    assert_eq!(c.a, 0.5);
}

// =========================================================================
// FFI Layout Tests
// =========================================================================

#[test]
fn test_ffi_layout_sizes() {
    use std::mem::size_of;

    // Verify types have expected sizes for FFI
    assert_eq!(size_of::<Vec2>(), 8); // 2 * f32
    assert_eq!(size_of::<Vec3>(), 12); // 3 * f32
    assert_eq!(size_of::<Vec4>(), 16); // 4 * f32
    assert_eq!(size_of::<Rect>(), 16); // 4 * f32
    assert_eq!(size_of::<Color>(), 16); // 4 * f32
}
