---
globs:
  - "**/ui/**"
  - "**/ffi/ui/**"
---

# UI Subsystem Patterns

## Architecture

- `UiManager` owns a hierarchical node tree, separate from the ECS world
- Nodes use generational `UiNodeId` for safe references — stale IDs return errors, not panics
- Tree structure: each node has an optional parent and a Vec of children
- `UiComponent` enum defines widget type (Button, Panel, Text, Image)
- Node allocation uses a generational arena (`UiNodeAllocator`)

## Node Tree Rules

- Nodes without a parent are root nodes
- Setting a parent triggers cycle detection — circular hierarchies are rejected
- Removing a node removes its entire subtree
- Never store raw node indices; always use `UiNodeId`

## Components

- Components are data-only — rendering and layout are handled by the UI rendering system
- One component per node (optional)
- Source in `goud_engine/src/ui/`

## FFI

- UI FFI in `ffi/ui/` with `manager.rs` and `node.rs` modules
- Node creation, destruction, reparenting, and component operations exposed via FFI

## Testing

- UI tree tests do not require GL context
- Test cycle detection, subtree removal, and generational ID invalidation
