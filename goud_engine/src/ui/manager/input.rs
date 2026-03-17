use crate::core::math::Vec2;
use crate::core::providers::input_types::{KeyCode as Key, MouseButton};
use crate::ecs::InputManager;

use super::UiManager;
use crate::ui::node_id::UiNodeId;
use crate::ui::UiEvent;

impl UiManager {
    /// Processes one UI input frame before game input is queried.
    pub fn process_input_frame(&mut self, input: &mut InputManager) {
        self.frame_events.clear();
        self.recompute_layout_if_needed();
        self.clear_stale_ui_state();

        self.update_hover(input);
        self.handle_focus_traversal(input);
        self.handle_pointer_click(input);
        self.handle_keyboard_activation(input);
    }

    fn update_hover(&mut self, input: &InputManager) {
        let target = self.hit_test_topmost(input.mouse_position());
        if target == self.hovered_node {
            return;
        }

        if let Some(prev) = self.hovered_node {
            self.frame_events.push(UiEvent::HoverLeave(prev));
        }
        if let Some(next) = target {
            self.frame_events.push(UiEvent::HoverEnter(next));
        }
        self.hovered_node = target;
    }

    fn handle_focus_traversal(&mut self, input: &mut InputManager) {
        if !input.key_just_pressed(Key::Tab) {
            return;
        }

        let focusables = self.focusable_nodes_in_tree_order();
        if focusables.is_empty() {
            self.set_focus(None);
            input.consume_key(Key::Tab);
            return;
        }

        let reverse = input.key_pressed(Key::LeftShift) || input.key_pressed(Key::RightShift);
        let next = match self.focused_node {
            Some(current) => {
                let idx = focusables
                    .iter()
                    .position(|&id| id == current)
                    .unwrap_or(focusables.len() - 1);
                if reverse {
                    if idx == 0 {
                        focusables[focusables.len() - 1]
                    } else {
                        focusables[idx - 1]
                    }
                } else {
                    focusables[(idx + 1) % focusables.len()]
                }
            }
            None => {
                if reverse {
                    focusables[focusables.len() - 1]
                } else {
                    focusables[0]
                }
            }
        };

        self.set_focus(Some(next));
        input.consume_key(Key::Tab);
    }

    fn handle_pointer_click(&mut self, input: &mut InputManager) {
        let left = MouseButton::Button1;
        let hit_target = self.hit_test_topmost(input.mouse_position());

        if input.mouse_button_just_pressed(left) {
            self.pressed_pointer_node = hit_target;
            if let Some(target) = hit_target {
                if self.node_focusable(target) {
                    self.set_focus(Some(target));
                }
                input.consume_mouse_button(left);
            }
        }

        if input.mouse_button_just_released(left) {
            let pressed = self.pressed_pointer_node.take();
            let mut consumed = hit_target.is_some() || pressed.is_some();

            if let (Some(pressed_node), Some(released_node)) = (pressed, hit_target) {
                if pressed_node == released_node && self.node_is_clickable_button(pressed_node) {
                    self.frame_events.push(UiEvent::Click(pressed_node));
                    consumed = true;
                }
            }

            if consumed {
                input.consume_mouse_button(left);
            }
        }
    }

    fn handle_keyboard_activation(&mut self, input: &mut InputManager) {
        let Some(focused) = self.focused_node else {
            return;
        };
        if !self.node_is_clickable_button(focused) {
            return;
        }

        let mut activated = false;
        if input.key_just_pressed(Key::Enter) || input.key_just_pressed(Key::KpEnter) {
            activated = true;
            input.consume_key(Key::Enter);
            input.consume_key(Key::KpEnter);
        }
        if input.key_just_pressed(Key::Space) {
            activated = true;
            input.consume_key(Key::Space);
        }

        if activated {
            self.frame_events.push(UiEvent::Click(focused));
        }
    }

    fn hit_test_topmost(&self, point: Vec2) -> Option<UiNodeId> {
        let mut roots = self.root_nodes();
        roots.reverse();

        for root in roots {
            if let Some(hit) = self.hit_test_subtree(root, point) {
                return Some(hit);
            }
        }

        None
    }

    fn hit_test_subtree(&self, node_id: UiNodeId, point: Vec2) -> Option<UiNodeId> {
        let node = self.nodes.get(&node_id)?;
        if !node.visible() || !node.input_enabled() {
            return None;
        }

        for &child in node.children().iter().rev() {
            if let Some(hit) = self.hit_test_subtree(child, point) {
                return Some(hit);
            }
        }

        let component = node.component()?;
        if component.is_interactive() && node.computed_rect().contains(point) {
            Some(node_id)
        } else {
            None
        }
    }

    fn focusable_nodes_in_tree_order(&self) -> Vec<UiNodeId> {
        let mut out = Vec::new();
        for root in self.root_nodes() {
            self.collect_focusables_preorder(root, &mut out);
        }
        out
    }

    fn collect_focusables_preorder(&self, id: UiNodeId, out: &mut Vec<UiNodeId>) {
        let Some(node) = self.nodes.get(&id) else {
            return;
        };

        if !node.visible() || !node.input_enabled() {
            return;
        }

        if node.focusable() {
            out.push(id);
        }

        for &child in node.children() {
            self.collect_focusables_preorder(child, out);
        }
    }
}
