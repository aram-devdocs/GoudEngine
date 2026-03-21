//! UI widget/style mutation FFI exports.

use crate::ui::{UiButton, UiComponent, UiImage, UiLabel, UiManager, UiSlider, UiStyleOverrides};

use super::{component_from_widget_kind, unpack_node_id, FfiUiStyle, ERR_NULL_MANAGER, ERR_NULL_PTR};
const ERR_NODE_NOT_FOUND: i32 = -3;
const ERR_INVALID_UTF8: i32 = -4;
const ERR_UNKNOWN_WIDGET: i32 = -5;

fn read_utf8_bytes(ptr: *const u8, len: usize) -> Result<String, i32> {
    if len == 0 {
        return Ok(String::new());
    }
    if ptr.is_null() {
        return Err(ERR_NULL_PTR);
    }
    // SAFETY: Caller provides a non-null pointer valid for `len` bytes.
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    std::str::from_utf8(bytes)
        .map(|s| s.to_string())
        .map_err(|_| ERR_INVALID_UTF8)
}

/// Sets or clears the widget component on an existing UI node.
///
/// Widget kinds:
/// - `-1` = none
/// - `0` = panel
/// - `1` = button
/// - `2` = label
/// - `3` = image
/// - `4` = slider
///
/// # Returns
/// * `0` on success
/// * `-1` if manager is null
/// * `-3` if node does not exist
/// * `-5` if widget kind is unknown
///
/// # Safety
/// `mgr` must be null (handled) or a valid exclusive pointer to `UiManager`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_set_widget(
    mgr: *mut UiManager,
    node_id: u64,
    widget_kind: i32,
) -> i32 {
    if mgr.is_null() {
        return ERR_NULL_MANAGER;
    }
    let Some(component) = component_from_widget_kind(widget_kind) else {
        return ERR_UNKNOWN_WIDGET;
    };

    // SAFETY: Caller guarantees `mgr` is a valid exclusive pointer.
    let manager = unsafe { &mut *mgr };
    let id = unpack_node_id(node_id);
    let Some(node) = manager.get_node_mut(id) else {
        return ERR_NODE_NOT_FOUND;
    };
    node.set_component(component);
    0
}

/// Sets style overrides on an existing node via a C-safe payload.
///
/// If no `has_*` fields are set to true, this clears style overrides.
///
/// # Returns
/// * `0` on success
/// * `-1` if manager is null
/// * `-2` if style pointer is null or required strings are null
/// * `-3` if node does not exist
/// * `-4` if UTF-8 decoding fails
///
/// # Safety
/// `mgr` must be null (handled) or a valid exclusive pointer to `UiManager`.
/// `style` must be null (handled as error) or point to a valid `FfiUiStyle`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_set_style(
    mgr: *mut UiManager,
    node_id: u64,
    style: *const FfiUiStyle,
) -> i32 {
    if mgr.is_null() {
        return ERR_NULL_MANAGER;
    }
    if style.is_null() {
        return ERR_NULL_PTR;
    }

    // SAFETY: `style` is checked non-null above and expected valid by caller.
    let style = unsafe { &*style };

    let mut overrides = UiStyleOverrides::default();
    let mut any_override = false;
    if style.has_background_color {
        overrides.background_color = Some(style.background_color);
        any_override = true;
    }
    if style.has_foreground_color {
        overrides.foreground_color = Some(style.foreground_color);
        any_override = true;
    }
    if style.has_border_color {
        overrides.border_color = Some(style.border_color);
        any_override = true;
    }
    if style.has_border_width {
        overrides.border_width = Some(style.border_width);
        any_override = true;
    }
    if style.has_font_family {
        match read_utf8_bytes(style.font_family_ptr, style.font_family_len) {
            Ok(value) => {
                overrides.font_family = Some(value);
                any_override = true;
            }
            Err(code) => return code,
        }
    }
    if style.has_font_size {
        overrides.font_size = Some(style.font_size);
        any_override = true;
    }
    if style.has_texture_path {
        match read_utf8_bytes(style.texture_path_ptr, style.texture_path_len) {
            Ok(value) => {
                overrides.texture_path = Some(value);
                any_override = true;
            }
            Err(code) => return code,
        }
    }
    if style.has_widget_spacing {
        overrides.widget_spacing = Some(style.widget_spacing);
        any_override = true;
    }

    // SAFETY: Caller guarantees `mgr` is a valid exclusive pointer.
    let manager = unsafe { &mut *mgr };
    let id = unpack_node_id(node_id);
    let Some(node) = manager.get_node_mut(id) else {
        return ERR_NODE_NOT_FOUND;
    };

    if any_override {
        node.set_style_overrides(overrides);
    } else {
        node.clear_style_overrides();
    }

    0
}

/// Sets or creates a label component and updates its text.
///
/// # Safety
/// `mgr` must be null (handled) or a valid exclusive pointer to `UiManager`.
/// `text_ptr` must be null only when `text_len == 0`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_set_label_text(
    mgr: *mut UiManager,
    node_id: u64,
    text_ptr: *const u8,
    text_len: usize,
) -> i32 {
    if mgr.is_null() {
        return ERR_NULL_MANAGER;
    }
    let text = match read_utf8_bytes(text_ptr, text_len) {
        Ok(text) => text,
        Err(code) => return code,
    };
    // SAFETY: Caller guarantees `mgr` is a valid exclusive pointer.
    let manager = unsafe { &mut *mgr };
    let id = unpack_node_id(node_id);
    let Some(node) = manager.get_node_mut(id) else {
        return ERR_NODE_NOT_FOUND;
    };
    node.set_component(Some(UiComponent::Label(UiLabel::new(text))));
    0
}

/// Sets or creates a button component and updates its enabled flag.
///
/// # Safety
/// `mgr` must be null (handled) or a valid exclusive pointer to `UiManager`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_set_button_enabled(
    mgr: *mut UiManager,
    node_id: u64,
    enabled: bool,
) -> i32 {
    if mgr.is_null() {
        return ERR_NULL_MANAGER;
    }
    // SAFETY: Caller guarantees `mgr` is a valid exclusive pointer.
    let manager = unsafe { &mut *mgr };
    let id = unpack_node_id(node_id);
    let Some(node) = manager.get_node_mut(id) else {
        return ERR_NODE_NOT_FOUND;
    };
    node.set_component(Some(UiComponent::Button(UiButton { enabled })));
    0
}

/// Sets or creates an image component and updates its texture path.
///
/// # Safety
/// `mgr` must be null (handled) or a valid exclusive pointer to `UiManager`.
/// `path_ptr` must be null only when `path_len == 0`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_set_image_texture_path(
    mgr: *mut UiManager,
    node_id: u64,
    path_ptr: *const u8,
    path_len: usize,
) -> i32 {
    if mgr.is_null() {
        return ERR_NULL_MANAGER;
    }
    let texture_path = match read_utf8_bytes(path_ptr, path_len) {
        Ok(path) => path,
        Err(code) => return code,
    };
    // SAFETY: Caller guarantees `mgr` is a valid exclusive pointer.
    let manager = unsafe { &mut *mgr };
    let id = unpack_node_id(node_id);
    let Some(node) = manager.get_node_mut(id) else {
        return ERR_NODE_NOT_FOUND;
    };
    node.set_component(Some(UiComponent::Image(UiImage::new(texture_path))));
    0
}

/// Sets or creates a slider component and updates range/value/enabled.
///
/// # Safety
/// `mgr` must be null (handled) or a valid exclusive pointer to `UiManager`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_set_slider(
    mgr: *mut UiManager,
    node_id: u64,
    min: f32,
    max: f32,
    value: f32,
    enabled: bool,
) -> i32 {
    if mgr.is_null() {
        return ERR_NULL_MANAGER;
    }
    let mut slider = UiSlider::new(min, max, value);
    slider.enabled = enabled;
    // SAFETY: Caller guarantees `mgr` is a valid exclusive pointer.
    let manager = unsafe { &mut *mgr };
    let id = unpack_node_id(node_id);
    let Some(node) = manager.get_node_mut(id) else {
        return ERR_NODE_NOT_FOUND;
    };
    node.set_component(Some(UiComponent::Slider(slider)));
    0
}

/// Sets absolute screen-space position for a UI node.
///
/// Switches the node to absolute positioning mode so it is no longer
/// placed by the layout system.
///
/// # Returns
/// * `0` on success
/// * `-1` if manager is null
/// * `-3` if node does not exist
///
/// # Safety
/// `mgr` must be null (handled) or a valid exclusive pointer to `UiManager`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_set_node_position(
    mgr: *mut UiManager,
    node_id: u64,
    x: f32,
    y: f32,
) -> i32 {
    if mgr.is_null() {
        return ERR_NULL_MANAGER;
    }
    // SAFETY: Caller guarantees `mgr` is a valid exclusive pointer.
    let manager = unsafe { &mut *mgr };
    let id = unpack_node_id(node_id);
    let Some(node) = manager.get_node_mut(id) else {
        return ERR_NODE_NOT_FOUND;
    };
    node.set_position(x, y);
    manager.mark_layout_dirty();
    0
}

/// Sets visibility for a UI node. Hidden nodes are not rendered.
///
/// # Returns
/// * `0` on success
/// * `-1` if manager is null
/// * `-3` if node does not exist
///
/// # Safety
/// `mgr` must be null (handled) or a valid exclusive pointer to `UiManager`.
#[no_mangle]
pub unsafe extern "C" fn goud_ui_set_node_visible(
    mgr: *mut UiManager,
    node_id: u64,
    visible: bool,
) -> i32 {
    if mgr.is_null() {
        return ERR_NULL_MANAGER;
    }
    // SAFETY: Caller guarantees `mgr` is a valid exclusive pointer.
    let manager = unsafe { &mut *mgr };
    let id = unpack_node_id(node_id);
    let Some(node) = manager.get_node_mut(id) else {
        return ERR_NODE_NOT_FOUND;
    };
    node.set_visible(visible);
    manager.mark_layout_dirty();
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::ui::manager::{goud_ui_manager_create, goud_ui_manager_destroy};
    use crate::ui::{UiComponent, UiStyleOverrides};

    #[test]
    fn test_set_widget_maps_all_widget_kinds() {
        let mgr = goud_ui_manager_create();
        // SAFETY: `mgr` is valid for the duration of this scope.
        unsafe {
            let node = super::super::node::goud_ui_create_node(mgr, 0);
            assert_eq!(goud_ui_set_widget(mgr, node, -1), 0);
            assert_eq!(goud_ui_set_widget(mgr, node, 0), 0);
            assert_eq!(goud_ui_set_widget(mgr, node, 1), 0);
            assert_eq!(goud_ui_set_widget(mgr, node, 2), 0);
            assert_eq!(goud_ui_set_widget(mgr, node, 3), 0);
            assert_eq!(goud_ui_set_widget(mgr, node, 4), 0);
            assert_eq!(goud_ui_set_widget(mgr, node, 99), ERR_UNKNOWN_WIDGET);
            goud_ui_manager_destroy(mgr);
        }
    }

    #[test]
    fn test_set_style_sets_and_clears_overrides() {
        let mgr = goud_ui_manager_create();
        // SAFETY: `mgr` is valid for the duration of this scope.
        unsafe {
            let node = super::super::node::goud_ui_create_node(mgr, 0);
            let font = "F05";
            let tex = "ui://tex";
            let style = FfiUiStyle {
                has_background_color: true,
                background_color: crate::core::math::Color::RED,
                has_foreground_color: true,
                foreground_color: crate::core::math::Color::WHITE,
                has_border_color: true,
                border_color: crate::core::math::Color::BLUE,
                has_border_width: true,
                border_width: 2.0,
                has_font_family: true,
                font_family_ptr: font.as_ptr(),
                font_family_len: font.len(),
                has_font_size: true,
                font_size: 18.0,
                has_texture_path: true,
                texture_path_ptr: tex.as_ptr(),
                texture_path_len: tex.len(),
                has_widget_spacing: true,
                widget_spacing: 6.0,
            };
            assert_eq!(goud_ui_set_style(mgr, node, &style), 0);

            let manager = &*mgr;
            let id = super::super::unpack_node_id(node);
            let node_ref = manager.get_node(id).unwrap();
            let got = node_ref.style_overrides().cloned().unwrap();
            assert_eq!(got.background_color, Some(crate::core::math::Color::RED));
            assert_eq!(got.foreground_color, Some(crate::core::math::Color::WHITE));
            assert_eq!(got.border_color, Some(crate::core::math::Color::BLUE));
            assert_eq!(got.border_width, Some(2.0));
            assert_eq!(got.font_family, Some(font.to_string()));
            assert_eq!(got.font_size, Some(18.0));
            assert_eq!(got.texture_path, Some(tex.to_string()));
            assert_eq!(got.widget_spacing, Some(6.0));

            let clear_style = FfiUiStyle::default();
            assert_eq!(goud_ui_set_style(mgr, node, &clear_style), 0);
            let node_ref = manager.get_node(id).unwrap();
            assert_eq!(node_ref.style_overrides(), None);

            let _ = UiStyleOverrides::default();
            goud_ui_manager_destroy(mgr);
        }
    }

    #[test]
    fn test_widget_specific_mutators_create_components() {
        let mgr = goud_ui_manager_create();
        // SAFETY: `mgr` is valid for the duration of this scope.
        unsafe {
            let node = super::super::node::goud_ui_create_node(mgr, -1);

            let text = "Play";
            assert_eq!(
                goud_ui_set_label_text(mgr, node, text.as_ptr(), text.len()),
                0
            );
            let id = super::super::unpack_node_id(node);
            let manager = &*mgr;
            assert!(matches!(
                manager.get_node(id).unwrap().component(),
                Some(UiComponent::Label(label)) if label.text == "Play"
            ));

            assert_eq!(goud_ui_set_button_enabled(mgr, node, false), 0);
            assert!(matches!(
                manager.get_node(id).unwrap().component(),
                Some(UiComponent::Button(button)) if !button.enabled
            ));

            let path = "assets/ui/button.png";
            assert_eq!(
                goud_ui_set_image_texture_path(mgr, node, path.as_ptr(), path.len()),
                0
            );
            assert!(matches!(
                manager.get_node(id).unwrap().component(),
                Some(UiComponent::Image(image)) if image.texture_path == path
            ));

            assert_eq!(goud_ui_set_slider(mgr, node, -5.0, 5.0, 1.0, true), 0);
            assert!(matches!(
                manager.get_node(id).unwrap().component(),
                Some(UiComponent::Slider(slider))
                if slider.min == -5.0 && slider.max == 5.0 && slider.value == 1.0 && slider.enabled
            ));

            goud_ui_manager_destroy(mgr);
        }
    }

    #[test]
    fn test_set_node_position() {
        let mgr = goud_ui_manager_create();
        // SAFETY: `mgr` is valid for the duration of this scope.
        unsafe {
            let node = super::super::node::goud_ui_create_node(mgr, 0);
            assert_eq!(goud_ui_set_node_position(mgr, node, 100.0, 200.0), 0);
            let id = super::super::unpack_node_id(node);
            let manager = &*mgr;
            let node_ref = manager.get_node(id).unwrap();
            assert_eq!(node_ref.position().x, 100.0);
            assert_eq!(node_ref.position().y, 200.0);
            assert_eq!(node_ref.position_mode(), crate::ui::PositionMode::Absolute);

            goud_ui_manager_destroy(mgr);
        }
    }

    #[test]
    fn test_set_node_position_null_mgr() {
        // SAFETY: Null is explicitly handled.
        let result = unsafe { goud_ui_set_node_position(std::ptr::null_mut(), 0, 0.0, 0.0) };
        assert_eq!(result, ERR_NULL_MANAGER);
    }

    #[test]
    fn test_set_node_visible() {
        let mgr = goud_ui_manager_create();
        // SAFETY: `mgr` is valid for the duration of this scope.
        unsafe {
            let node = super::super::node::goud_ui_create_node(mgr, 0);

            assert_eq!(goud_ui_set_node_visible(mgr, node, false), 0);
            let id = super::super::unpack_node_id(node);
            let manager = &*mgr;
            assert!(!manager.get_node(id).unwrap().visible());
            assert_eq!(goud_ui_set_node_visible(mgr, node, true), 0);
            assert!(manager.get_node(id).unwrap().visible());

            goud_ui_manager_destroy(mgr);
        }
    }

    #[test]
    fn test_set_node_visible_null_mgr() {
        // SAFETY: Null is explicitly handled.
        let result = unsafe { goud_ui_set_node_visible(std::ptr::null_mut(), 0, true) };
        assert_eq!(result, ERR_NULL_MANAGER);
    }
}
