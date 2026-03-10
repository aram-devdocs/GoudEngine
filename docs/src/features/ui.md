# UI System

GoudEngine provides a hierarchical UI node tree for building game interfaces.

## UiManager

Each context has a `UiManager` that owns UI nodes in a tree structure. The tree is separate from the ECS world — UI nodes are not entities.

## Node Tree

### UiNodeId

Nodes are identified by generational IDs (`UiNodeId`). Like entity IDs, the generation counter detects stale references — using a node ID after the node is removed returns an error.

### Creating Nodes

Create a node and optionally set its parent. Nodes without a parent are root nodes.

### Parent/Child Relationships

- Each node has at most one parent
- Each node can have multiple children
- Setting a parent is validated: cycle detection prevents circular hierarchies
- Removing a parent detaches the node (becomes a root)
- Removing a node removes its entire subtree

## UiComponent

A `UiComponent` can be attached to a node to define its visual role:

- Button
- Panel
- Text
- Image

Components are data-only — rendering and layout are handled by the UI rendering system.

## Layout

The layout system supports both anchor-based and flex-style placement:

- Anchors: top-left, center, bottom-right, and stretch behaviors
- Edge spacing: margin and padding fields on nodes
- Flex containers: row/column direction, alignment, and spacing
- Deterministic recompute when the UI tree changes or the window size changes

The implementation is in `goud_engine/src/ui/layout.rs` and `goud_engine/src/ui/manager/layout.rs`.

## Input Semantics

UI input is processed before game-level input polling.

- Click dispatch targets the topmost interactive node under the cursor
- Hover state emits enter/leave transitions as the pointer moves
- Focus traversal supports Tab and Shift+Tab
- Enter/Space activates focused buttons
- Consumed UI input is masked so gameplay input queries do not re-handle the same event

The input flow is implemented in `goud_engine/src/ui/manager/input.rs` with per-frame integration in the game loop.

## Cycle Detection

`set_parent` validates the relationship before applying it. Attempting to create a circular hierarchy returns an error immediately; it does not silently corrupt the tree.

## FFI

UI FFI functions are in `goud_engine/src/ffi/ui/`:

- Node creation, destruction, and reparenting
- Component attachment and modification
- Tree traversal queries
