---
globs:
  - "goud_engine/src/ui/**"
alwaysApply: false
---

# UI Subsystem Patterns

The UI system (`goud_engine/src/ui/`) is a Layer 3 (Services) module. It MAY import from `core/` and `ecs/`, and is consumed by higher layers (`sdk/`, `rendering/`, `ffi/`, `wasm/`). It MUST NOT import from any Layer 4 or Layer 5 module.

## Node Tree

- The UI tree is standalone and separate from the ECS `World` — do not conflate `UiNode` with an ECS entity.
- Nodes are identified by generational `UiNodeId` values (index + generation). Never store or compare raw indices; a stale ID must fail validation, not alias a reused slot.
- `UiNodeAllocator` hands out and recycles node IDs. `UiManager` owns the full tree and is the only entry point for creating, reparenting, or destroying nodes.
- Each `UiNode` carries an optional `UiComponent`, its parent/child links, layout properties (anchor, position mode, size, margin, padding), computed rect, visibility, and input/focus flags.

## Widgets

- `UiComponent` is a closed enum: `Panel`, `Button`, `Label`, `Image`, `Slider`. Add new widget kinds as variants here, not as external types.
- Widget structs (`UiButton`, `UiLabel`, `UiImage`, `UiSlider`) hold intrinsic settings/state only — plain data, no side-effecting methods.
- The FFI/SDK widget-kind integer codes are mapped in `component_from_widget_kind` (`mod.rs`). Keep that mapping in sync when adding a variant, and update every SDK.

## Layout

- Layout primitives live in `layout.rs` (`PositionMode`, `UiAnchor`, `UiEdges`, `UiLayout`, `UiFlexLayout`, `UiJustify`, `UiAlign`, `UiFlexDirection`).
- `PositionMode::Relative` (default) means the layout system computes the rect; `Absolute` means the position is set explicitly. Read the computed rect, set the local properties.
- Layout, input hit-testing, and render-command emission are split across `manager/layout.rs`, `manager/input.rs`, and `manager/render.rs`. Keep each concern in its own module.

## Events and Theming

- `UiManager` produces `UiEvent` values (`HoverEnter`, `HoverLeave`, `FocusChanged`, `Click`). `map_ui_event` packs them into `PackedUiEvent` (node IDs packed as `index | generation << 32`) for FFI/WASM consumers.
- Visual styling flows through `theme.rs` (`UiTheme`, `UiVisualStyle`, `UiStyleOverrides`) and is resolved per node via `resolve_widget_visual` against the node's `UiInteractionState`. Do not hardcode colors or fonts in nodes; use the theme plus optional per-node overrides.
- Rendering is data-driven: the manager emits `UiRenderCommand` values (`UiQuadCommand`, `UiTexturedQuadCommand`, `UiTextCommand`) rather than calling the renderer directly.
