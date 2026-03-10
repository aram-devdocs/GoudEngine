//! Standalone UI bindings for the WASM/web API.
//!
//! This module owns the `WasmUi*` API surface used by the TypeScript web wrapper
//! for creating UI nodes, tweaking style/widget properties, and reading events.

use crate::core::math::Color;
use crate::ui::component_from_widget_kind;
use crate::ui::map_ui_event;
use crate::ui::{
    UiButton, UiComponent, UiImage, UiLabel, UiManager, UiNodeId, UiSlider, UiStyleOverrides,
};
const INVALID_NODE_U64: u64 = u64::MAX;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
/// Serializable UI event record exposed to JavaScript.
pub struct WasmUiEvent {
    /// Numeric UI event kind.
    pub event_kind: u32,
    /// Packed `UiNodeId` for the source node.
    pub node_id: u64,
    /// Packed previous focus/hover node or `u64::MAX`.
    pub previous_node_id: u64,
    /// Packed current focus/hover node or `u64::MAX`.
    pub current_node_id: u64,
}

#[wasm_bindgen]
/// Standalone UI manager exported for browser/WASM SDK consumers.
///
/// This mirrors the native `UiManager` lifecycle and widget/style/event APIs
/// while preserving Rust-side ownership of layout and event processing.
pub struct WasmUiManager {
    inner: UiManager,
    frame_events: Vec<WasmUiEvent>,
}

fn pack_ui_node_id(id: UiNodeId) -> u64 {
    (id.index() as u64) | ((id.generation() as u64) << 32)
}

fn unpack_ui_node_id(packed: u64) -> UiNodeId {
    UiNodeId::new(packed as u32, (packed >> 32) as u32)
}

#[wasm_bindgen]
impl WasmUiManager {
    /// Creates a standalone UI manager for web/WASM consumers.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: UiManager::new(),
            frame_events: Vec::new(),
        }
    }

    /// Runs a layout/input update tick and snapshots the emitted UI events.
    pub fn update(&mut self) {
        self.inner.update();
        self.frame_events = self
            .inner
            .take_events()
            .into_iter()
            .map(map_ui_event)
            .map(|event| WasmUiEvent {
                event_kind: event.event_kind,
                node_id: event.node_id,
                previous_node_id: event.previous_node_id,
                current_node_id: event.current_node_id,
            })
            .collect();
    }

    /// Builds render commands for the current UI tree.
    pub fn render(&mut self) {
        let _ = self.inner.build_render_commands();
    }

    /// Returns the number of live UI nodes.
    pub fn node_count(&self) -> u32 {
        self.inner.node_count() as u32
    }

    /// Creates a UI node with the given component type code.
    pub fn create_node(&mut self, component_type: i32) -> u64 {
        let Some(component) = component_from_widget_kind(component_type) else {
            return INVALID_NODE_U64;
        };
        pack_ui_node_id(self.inner.create_node(component))
    }

    /// Removes a UI node and its subtree.
    pub fn remove_node(&mut self, node_id: u64) -> i32 {
        if self.inner.remove_node(unpack_ui_node_id(node_id)) {
            0
        } else {
            -2
        }
    }

    /// Sets or clears a node's parent (`u64::MAX` detaches to root).
    pub fn set_parent(&mut self, child_id: u64, parent_id: u64) -> i32 {
        let parent = if parent_id == u64::MAX {
            None
        } else {
            Some(unpack_ui_node_id(parent_id))
        };
        match self.inner.set_parent(unpack_ui_node_id(child_id), parent) {
            Ok(()) => 0,
            Err(_) => -2,
        }
    }

    /// Returns a node's parent, or `u64::MAX` if missing/root.
    pub fn get_parent(&self, node_id: u64) -> u64 {
        self.inner
            .get_node(unpack_ui_node_id(node_id))
            .and_then(|node| node.parent())
            .map(pack_ui_node_id)
            .unwrap_or(u64::MAX)
    }

    /// Returns the number of children for the given node.
    pub fn get_child_count(&self, node_id: u64) -> u32 {
        self.inner
            .get_node(unpack_ui_node_id(node_id))
            .map(|node| node.children().len() as u32)
            .unwrap_or(0)
    }

    /// Returns the child at an index, or `u64::MAX` if out of bounds.
    pub fn get_child_at(&self, node_id: u64, index: u32) -> u64 {
        self.inner
            .get_node(unpack_ui_node_id(node_id))
            .and_then(|node| node.children().get(index as usize))
            .copied()
            .map(pack_ui_node_id)
            .unwrap_or(u64::MAX)
    }

    /// Sets or clears the node widget component by widget-kind code.
    pub fn set_widget(&mut self, node_id: u64, widget_kind: i32) -> i32 {
        let Some(component) = component_from_widget_kind(widget_kind) else {
            return -5;
        };
        let Some(node) = self.inner.get_node_mut(unpack_ui_node_id(node_id)) else {
            return -3;
        };
        node.set_component(component);
        0
    }

    #[allow(clippy::too_many_arguments)]
    /// Applies per-node visual style overrides.
    pub fn set_style(
        &mut self,
        node_id: u64,
        background_r: Option<f32>,
        background_g: Option<f32>,
        background_b: Option<f32>,
        background_a: Option<f32>,
        foreground_r: Option<f32>,
        foreground_g: Option<f32>,
        foreground_b: Option<f32>,
        foreground_a: Option<f32>,
        border_r: Option<f32>,
        border_g: Option<f32>,
        border_b: Option<f32>,
        border_a: Option<f32>,
        border_width: Option<f32>,
        font_family: Option<String>,
        font_size: Option<f32>,
        texture_path: Option<String>,
        widget_spacing: Option<f32>,
    ) -> i32 {
        let Some(node) = self.inner.get_node_mut(unpack_ui_node_id(node_id)) else {
            return -3;
        };

        let mut overrides = UiStyleOverrides::default();
        let mut any = false;
        if let (Some(r), Some(g), Some(b), Some(a)) =
            (background_r, background_g, background_b, background_a)
        {
            overrides.background_color = Some(Color::new(r, g, b, a));
            any = true;
        }
        if let (Some(r), Some(g), Some(b), Some(a)) =
            (foreground_r, foreground_g, foreground_b, foreground_a)
        {
            overrides.foreground_color = Some(Color::new(r, g, b, a));
            any = true;
        }
        if let (Some(r), Some(g), Some(b), Some(a)) = (border_r, border_g, border_b, border_a) {
            overrides.border_color = Some(Color::new(r, g, b, a));
            any = true;
        }
        if let Some(width) = border_width {
            overrides.border_width = Some(width);
            any = true;
        }
        if let Some(family) = font_family {
            overrides.font_family = Some(family);
            any = true;
        }
        if let Some(size) = font_size {
            overrides.font_size = Some(size);
            any = true;
        }
        if let Some(path) = texture_path {
            overrides.texture_path = Some(path);
            any = true;
        }
        if let Some(spacing) = widget_spacing {
            overrides.widget_spacing = Some(spacing);
            any = true;
        }

        if any {
            node.set_style_overrides(overrides);
        } else {
            node.clear_style_overrides();
        }
        0
    }

    /// Sets/creates a label widget and updates its text.
    pub fn set_label_text(&mut self, node_id: u64, text: String) -> i32 {
        let Some(node) = self.inner.get_node_mut(unpack_ui_node_id(node_id)) else {
            return -3;
        };
        node.set_component(Some(UiComponent::Label(UiLabel::new(text))));
        0
    }

    /// Sets/creates a button widget and updates its enabled flag.
    pub fn set_button_enabled(&mut self, node_id: u64, enabled: bool) -> i32 {
        let Some(node) = self.inner.get_node_mut(unpack_ui_node_id(node_id)) else {
            return -3;
        };
        node.set_component(Some(UiComponent::Button(UiButton { enabled })));
        0
    }

    /// Sets/creates an image widget and updates its texture path.
    pub fn set_image_texture_path(&mut self, node_id: u64, path: String) -> i32 {
        let Some(node) = self.inner.get_node_mut(unpack_ui_node_id(node_id)) else {
            return -3;
        };
        node.set_component(Some(UiComponent::Image(UiImage::new(path))));
        0
    }

    /// Sets/creates a slider widget and updates range/value/enabled state.
    pub fn set_slider(
        &mut self,
        node_id: u64,
        min: f32,
        max: f32,
        value: f32,
        enabled: bool,
    ) -> i32 {
        let Some(node) = self.inner.get_node_mut(unpack_ui_node_id(node_id)) else {
            return -3;
        };
        let mut slider = UiSlider::new(min, max, value);
        slider.enabled = enabled;
        node.set_component(Some(UiComponent::Slider(slider)));
        0
    }

    /// Returns the count of events captured during the latest `update`.
    pub fn event_count(&self) -> u32 {
        self.frame_events.len() as u32
    }

    /// Reads one captured event by index.
    pub fn event_read(&self, index: u32) -> Option<WasmUiEvent> {
        self.frame_events.get(index as usize).copied()
    }
}
