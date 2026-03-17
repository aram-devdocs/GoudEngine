use crate::core::math::{Rect, Vec2};
use crate::core::providers::input_types::{KeyCode as Key, MouseButton};
use crate::ecs::InputManager;
use crate::ui::{
    UiAlign, UiAnchor, UiButton, UiComponent, UiEdges, UiEvent, UiFlexDirection, UiFlexLayout,
    UiJustify, UiLayout, UiManager,
};

mod input;
mod layout;
mod render_commands;
mod theme;

fn assert_rect_eq(actual: Rect, expected: Rect) {
    assert!(
        (actual.x - expected.x).abs() < 0.001,
        "x mismatch: {:?} != {:?}",
        actual,
        expected
    );
    assert!(
        (actual.y - expected.y).abs() < 0.001,
        "y mismatch: {:?} != {:?}",
        actual,
        expected
    );
    assert!(
        (actual.width - expected.width).abs() < 0.001,
        "width mismatch: {:?} != {:?}",
        actual,
        expected
    );
    assert!(
        (actual.height - expected.height).abs() < 0.001,
        "height mismatch: {:?} != {:?}",
        actual,
        expected
    );
}
