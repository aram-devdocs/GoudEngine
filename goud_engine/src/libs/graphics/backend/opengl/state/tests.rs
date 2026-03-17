use super::clamp_line_width;

#[test]
fn clamp_line_width_respects_supported_range() {
    assert_eq!(clamp_line_width(3.0, [1.0, 1.0]), Some(1.0));
    assert_eq!(clamp_line_width(0.5, [1.0, 4.0]), Some(1.0));
    assert_eq!(clamp_line_width(2.0, [1.0, 4.0]), Some(2.0));
}

#[test]
fn clamp_line_width_rejects_invalid_values() {
    assert_eq!(clamp_line_width(0.0, [1.0, 4.0]), None);
    assert_eq!(clamp_line_width(f32::NAN, [1.0, 4.0]), None);
}
