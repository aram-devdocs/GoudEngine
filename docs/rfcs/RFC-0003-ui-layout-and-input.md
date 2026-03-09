---
rfc: "0003"
title: "UI Layout and Input Behavior"
status: draft
created: 2026-03-09
authors: ["aram-devdocs"]
tracking-issue: "#236"
---

# RFC-0003: UI Layout and Input Behavior

## 1. Summary

This RFC defines engine-internal UI layout and input behavior for `UiNode` trees. It covers anchors, margin/padding, flex row/column rules, layout recompute triggers, and input routing semantics (hit testing, hover, focus, activation, and event consumption). The scope is internal Rust engine behavior, and it does not add FFI or SDK surface yet.

---

## 2. Motivation

Issue #123 requires a clear UI behavior contract before implementation proceeds. Today, layout and input expectations are implicit, which can create inconsistent behavior across render/input code paths and follow-up features.

The engine needs:
- Deterministic positioning rules for common anchoring and container layouts.
- Predictable recomputation boundaries when UI trees change.
- Deterministic input dispatch so UI and game input do not both process the same event in one frame.

---

## 3. Design

### 3.1 Scope and Non-Goals

- Scope: Rust engine internal UI runtime behavior (`UiNode` layout + input dispatch semantics).
- Non-goal: adding or freezing public FFI/SDK APIs in this RFC.
- Non-goal: advanced text layout, grid layout, animation system, or styling/theme APIs.

### 3.2 UiNode Layout Properties

Each `UiNode` participates in layout with these conceptual properties:
- `anchor`: `TopLeft | Center | BottomRight | Stretch`
- `margin`: `{ left, right, top, bottom }` (pixels)
- `padding`: `{ left, right, top, bottom }` (pixels)
- `layout`: `None | Flex { direction, justify, align_items, spacing }`

Definitions:
- `node_rect`: node border box in parent space.
- `content_rect`: `node_rect` inset by `padding`; child layout is computed in `content_rect`.
- Margins affect placement/sizing relative to parent `content_rect`.

### 3.3 Anchor Semantics

Anchors are resolved in parent `content_rect`:

- `TopLeft`:
  - `x = parent.x + margin.left`
  - `y = parent.y + margin.top`
- `Center`:
  - Node is centered in parent, then offset by margins:
  - `x = parent.center_x - node.width/2 + margin.left - margin.right`
  - `y = parent.center_y - node.height/2 + margin.top - margin.bottom`
- `BottomRight`:
  - `x = parent.max_x - margin.right - node.width`
  - `y = parent.max_y - margin.bottom - node.height`
- `Stretch`:
  - `x = parent.x + margin.left`
  - `y = parent.y + margin.top`
  - `width = max(0, parent.width - margin.left - margin.right)`
  - `height = max(0, parent.height - margin.top - margin.bottom)`

If `Stretch` applies on an axis, explicit width/height on that axis is ignored.

### 3.4 Margin and Padding Behavior

- Margin is external spacing between a node and its parent’s `content_rect`.
- Padding is internal spacing between node border and node child content.
- Hit testing uses `node_rect` (not just `content_rect`).
- Child layout never uses parent `node_rect` directly; it always uses parent `content_rect`.

### 3.5 Flex Layout

`Flex` applies to a node’s `content_rect` and lays out direct children that are visible and layout-participating.

- `direction`:
  - `Row`: main axis = X, cross axis = Y
  - `Column`: main axis = Y, cross axis = X
- `justify`:
  - `Start | Center | End`
  - Controls child group offset on main axis after total child size + spacing is known.
- `align_items`:
  - `Start | Center | End | Stretch`
  - Controls each child on cross axis.
  - `Stretch` sets child cross size to remaining cross-axis space after margins.
- `spacing`:
  - Fixed gap inserted between adjacent children.
  - Total gap = `spacing * (child_count - 1)` when `child_count > 1`.

Children are placed in tree order.

### 3.6 Layout Dirty/Recompute Rules

Layout recomputation uses dirty flags and runs on demand.

Mark layout dirty when:
- Tree mutation: add/remove/reparent child.
- Layout-affecting property mutation: anchor, margin, padding, size constraints, flex settings, visibility/layout participation.
- Root viewport/window resize.

Rules:
- Dirty state propagates from the changed node up to root.
- The engine resolves layout at most once per frame, before render and before UI hit testing.
- Multiple dirty events in one frame coalesce into one recompute pass.
- Window resize forces a full root layout recompute.

### 3.7 Input Semantics

Input dispatch order for each frame is: `OS events -> UI system -> game input system`.

#### Topmost hit testing

- Pointer hit testing uses final layout rects from this frame.
- Traversal order for picking is reverse paint order (topmost visual node wins).
- Only visible and input-enabled nodes are hit candidates.

#### Hover enter/leave

- Engine tracks current hovered node per pointer.
- On pointer move, if hovered target changes:
  - Dispatch `HoverLeave` for previous node.
  - Dispatch `HoverEnter` for new node.
- Order is always leave then enter in the same frame.

#### Focus traversal (Tab)

- Focusable set: visible + enabled + focusable nodes.
- Traversal order: deterministic tree order.
- `Tab` moves focus forward; `Shift+Tab` moves backward.
- Traversal wraps at ends (last -> first, first -> last).

#### Keyboard activation for focused button

- If focused node is a button:
  - `Enter` triggers activation.
  - `Space` triggers activation.
- Activation emits the same logical action as pointer click on that button.

#### Event consumption boundary

- When UI handles an event (pointer hit on interactive node, focus traversal, button activation), it marks that event consumed for the current frame.
- Consumed events MUST NOT be forwarded to game input in the same frame.
- Unhandled UI events continue to game input processing.

---

## 4. Alternatives Considered

1. Immediate-mode UI only (no retained `UiNode` tree).
Rejected because current engine direction already uses retained node/component state and requires persistent focus/hover behavior.

2. Single-pass input without consumption.
Rejected because UI and gameplay would both react to the same input, causing double-activation bugs.

3. Public FFI-first UI API in this RFC.
Rejected because behavior needs stabilization internally before freezing cross-language surface.

---

## 5. Impact

- Engine internals gain a precise behavior contract for UI layout and input.
- No new FFI exports or SDK wrappers are defined by this RFC.
- Existing game input paths must honor UI event consumption to avoid duplicate handling.
- Implementation work can proceed behind internal interfaces, with FFI/API shape deferred to a later RFC.

---

## 6. Open Questions

1. Should focus traversal order be configurable (tree order vs explicit tab index) in a follow-up?
2. Should button activation differentiate keydown/keyup timing for `Space` for stricter accessibility parity?
3. Should clipping and scroll containers alter hit testing rules in a follow-up RFC?
