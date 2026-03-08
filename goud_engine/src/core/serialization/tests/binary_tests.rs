use crate::core::math::{Color, Rect, Vec2, Vec3, Vec4};
use crate::core::serialization::binary;

// =============================================================================
// Vec2
// =============================================================================

#[test]
fn test_binary_roundtrip_vec2() {
    // Arrange
    let original = Vec2::new(42.5, -17.3);

    // Act
    let bytes = binary::encode(&original).unwrap();
    let decoded: Vec2 = binary::decode(&bytes).unwrap();

    // Assert
    assert_eq!(original, decoded);
}

// =============================================================================
// Vec3
// =============================================================================

#[test]
fn test_binary_roundtrip_vec3() {
    // Arrange
    let original = Vec3::new(1.0, 2.0, 3.0);

    // Act
    let bytes = binary::encode(&original).unwrap();
    let decoded: Vec3 = binary::decode(&bytes).unwrap();

    // Assert
    assert_eq!(original, decoded);
}

// =============================================================================
// Vec4
// =============================================================================

#[test]
fn test_binary_roundtrip_vec4() {
    // Arrange
    let original = Vec4::new(10.0, 20.0, 30.0, 40.0);

    // Act
    let bytes = binary::encode(&original).unwrap();
    let decoded: Vec4 = binary::decode(&bytes).unwrap();

    // Assert
    assert_eq!(original, decoded);
}

// =============================================================================
// Color
// =============================================================================

#[test]
fn test_binary_roundtrip_color() {
    // Arrange
    let original = Color::rgba(0.2, 0.4, 0.6, 0.8);

    // Act
    let bytes = binary::encode(&original).unwrap();
    let decoded: Color = binary::decode(&bytes).unwrap();

    // Assert
    assert_eq!(original, decoded);
}

// =============================================================================
// Rect
// =============================================================================

#[test]
fn test_binary_roundtrip_rect() {
    // Arrange
    let original = Rect::new(10.0, 20.0, 100.0, 50.0);

    // Act
    let bytes = binary::encode(&original).unwrap();
    let decoded: Rect = binary::decode(&bytes).unwrap();

    // Assert
    assert_eq!(original, decoded);
}

// =============================================================================
// Error case
// =============================================================================

#[test]
fn test_binary_decode_corrupt_bytes() {
    // Arrange
    let corrupt = vec![0xFF, 0xFE, 0x00, 0x01, 0x02];

    // Act
    let result = binary::decode::<Vec2>(&corrupt);

    // Assert
    assert!(result.is_err());
}

#[test]
fn test_binary_decode_empty_bytes() {
    // Arrange
    let empty: &[u8] = &[];

    // Act
    let result = binary::decode::<Vec3>(empty);

    // Assert
    assert!(result.is_err());
}
