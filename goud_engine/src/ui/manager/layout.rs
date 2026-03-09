use crate::core::math::{Rect, Vec2};

use super::UiManager;
use crate::ui::layout::{UiAlign, UiAnchor, UiEdges, UiFlexDirection, UiJustify, UiLayout};
use crate::ui::node::UiNode;
use crate::ui::node_id::UiNodeId;

impl UiManager {
    /// Recomputes layout if marked dirty.
    pub fn update(&mut self) {
        self.recompute_layout_if_needed();
    }

    pub(super) fn recompute_layout_if_needed(&mut self) {
        if !self.layout_dirty {
            return;
        }

        let viewport_rect = Rect::new(
            0.0,
            0.0,
            self.viewport_size.0 as f32,
            self.viewport_size.1 as f32,
        );

        let roots = self.root_nodes();
        for root in roots {
            self.layout_node(root, viewport_rect);
        }

        self.layout_dirty = false;
        self.layout_epoch = self.layout_epoch.saturating_add(1);
        self.clear_stale_ui_state();
    }

    fn layout_node(&mut self, node_id: UiNodeId, parent_content_rect: Rect) {
        let (node_rect, content_rect, layout, children) = {
            let node = match self.nodes.get(&node_id) {
                Some(node) if node.layout_enabled() => node,
                _ => return,
            };

            let node_rect = resolve_anchored_rect(node, parent_content_rect);
            let content_rect = inset_rect(node_rect, node.padding());
            let layout = node.layout();
            let children = node.children().to_vec();
            (node_rect, content_rect, layout, children)
        };

        if let Some(node_mut) = self.nodes.get_mut(&node_id) {
            node_mut.set_computed_rect(node_rect);
        }

        match layout {
            UiLayout::None => {
                for child_id in children {
                    self.layout_node(child_id, content_rect);
                }
            }
            UiLayout::Flex(flex) => {
                self.layout_flex_children(
                    children,
                    content_rect,
                    flex.direction,
                    flex.justify,
                    flex.align_items,
                    flex.spacing,
                );
            }
        }
    }

    fn layout_flex_children(
        &mut self,
        child_ids: Vec<UiNodeId>,
        parent_content_rect: Rect,
        direction: UiFlexDirection,
        justify: UiJustify,
        align_items: UiAlign,
        spacing: f32,
    ) {
        struct FlexItem {
            id: UiNodeId,
            size: Vec2,
            margin: UiEdges,
        }

        let items: Vec<_> = child_ids
            .into_iter()
            .filter_map(|id| {
                self.nodes.get(&id).and_then(|node| {
                    if node.layout_enabled() && node.visible() {
                        Some(FlexItem {
                            id,
                            size: node.size(),
                            margin: node.margin(),
                        })
                    } else {
                        None
                    }
                })
            })
            .collect();

        if items.is_empty() {
            return;
        }

        let available_main = axis_main(direction, parent_content_rect.size()).max(0.0);
        let available_cross = axis_cross(direction, parent_content_rect.size()).max(0.0);

        let spacing_total = spacing.max(0.0) * (items.len().saturating_sub(1) as f32);
        let mut children_main_total = 0.0;
        for item in &items {
            children_main_total += axis_main(direction, item.size).max(0.0);
            children_main_total += axis_main_margin_before(direction, item.margin);
            children_main_total += axis_main_margin_after(direction, item.margin);
        }
        let total_main = children_main_total + spacing_total;
        let remaining_main = (available_main - total_main).max(0.0);

        let start_main_offset = match justify {
            UiJustify::Start => 0.0,
            UiJustify::Center => remaining_main * 0.5,
            UiJustify::End => remaining_main,
        };

        let mut cursor = axis_main_start(direction, parent_content_rect) + start_main_offset;

        for item in items {
            let main_size = axis_main(direction, item.size).max(0.0);
            let mut cross_size = axis_cross(direction, item.size).max(0.0);
            let margin_cross_before = axis_cross_margin_before(direction, item.margin);
            let margin_cross_after = axis_cross_margin_after(direction, item.margin);

            if matches!(align_items, UiAlign::Stretch) {
                cross_size = (available_cross - margin_cross_before - margin_cross_after).max(0.0);
            }

            let main_pos = cursor + axis_main_margin_before(direction, item.margin);
            let cross_pos = match align_items {
                UiAlign::Start | UiAlign::Stretch => {
                    axis_cross_start(direction, parent_content_rect) + margin_cross_before
                }
                UiAlign::Center => {
                    axis_cross_start(direction, parent_content_rect)
                        + (available_cross - cross_size) * 0.5
                        + margin_cross_before
                        - margin_cross_after
                }
                UiAlign::End => {
                    axis_cross_start(direction, parent_content_rect) + available_cross
                        - margin_cross_after
                        - cross_size
                }
            };

            let rect = compose_axis_rect(direction, main_pos, cross_pos, main_size, cross_size);
            if let Some(node_mut) = self.nodes.get_mut(&item.id) {
                node_mut.set_computed_rect(rect);
            }

            let child_content = {
                let padding = self
                    .nodes
                    .get(&item.id)
                    .map(UiNode::padding)
                    .unwrap_or_default();
                inset_rect(rect, padding)
            };

            let child_layout = self
                .nodes
                .get(&item.id)
                .map(UiNode::layout)
                .unwrap_or(UiLayout::None);

            let child_children = self
                .nodes
                .get(&item.id)
                .map(|n| n.children().to_vec())
                .unwrap_or_default();

            match child_layout {
                UiLayout::None => {
                    for child in child_children {
                        self.layout_node(child, child_content);
                    }
                }
                UiLayout::Flex(flex) => {
                    self.layout_flex_children(
                        child_children,
                        child_content,
                        flex.direction,
                        flex.justify,
                        flex.align_items,
                        flex.spacing,
                    );
                }
            }

            cursor += axis_main_margin_before(direction, item.margin)
                + main_size
                + axis_main_margin_after(direction, item.margin)
                + spacing.max(0.0);
        }
    }
}

fn resolve_anchored_rect(node: &UiNode, parent_content: Rect) -> Rect {
    let margin = node.margin();
    let anchor = node.anchor();
    let mut width = node.size().x.max(0.0);
    let mut height = node.size().y.max(0.0);

    if matches!(anchor, UiAnchor::Stretch) {
        width = (parent_content.width - margin.horizontal()).max(0.0);
        height = (parent_content.height - margin.vertical()).max(0.0);
    }

    let center = parent_content.center();
    let x = match anchor {
        UiAnchor::TopLeft | UiAnchor::Stretch => parent_content.x + margin.left,
        UiAnchor::Center => center.x - width * 0.5 + margin.left - margin.right,
        UiAnchor::BottomRight => parent_content.x + parent_content.width - margin.right - width,
    };

    let y = match anchor {
        UiAnchor::TopLeft | UiAnchor::Stretch => parent_content.y + margin.top,
        UiAnchor::Center => center.y - height * 0.5 + margin.top - margin.bottom,
        UiAnchor::BottomRight => parent_content.y + parent_content.height - margin.bottom - height,
    };

    Rect::new(x, y, width.max(0.0), height.max(0.0))
}

fn inset_rect(rect: Rect, insets: UiEdges) -> Rect {
    Rect::new(
        rect.x + insets.left,
        rect.y + insets.top,
        (rect.width - insets.horizontal()).max(0.0),
        (rect.height - insets.vertical()).max(0.0),
    )
}

fn axis_main(direction: UiFlexDirection, size: Vec2) -> f32 {
    match direction {
        UiFlexDirection::Row => size.x,
        UiFlexDirection::Column => size.y,
    }
}

fn axis_cross(direction: UiFlexDirection, size: Vec2) -> f32 {
    match direction {
        UiFlexDirection::Row => size.y,
        UiFlexDirection::Column => size.x,
    }
}

fn axis_main_start(direction: UiFlexDirection, rect: Rect) -> f32 {
    match direction {
        UiFlexDirection::Row => rect.x,
        UiFlexDirection::Column => rect.y,
    }
}

fn axis_cross_start(direction: UiFlexDirection, rect: Rect) -> f32 {
    match direction {
        UiFlexDirection::Row => rect.y,
        UiFlexDirection::Column => rect.x,
    }
}

fn axis_main_margin_before(direction: UiFlexDirection, edges: UiEdges) -> f32 {
    match direction {
        UiFlexDirection::Row => edges.left,
        UiFlexDirection::Column => edges.top,
    }
}

fn axis_main_margin_after(direction: UiFlexDirection, edges: UiEdges) -> f32 {
    match direction {
        UiFlexDirection::Row => edges.right,
        UiFlexDirection::Column => edges.bottom,
    }
}

fn axis_cross_margin_before(direction: UiFlexDirection, edges: UiEdges) -> f32 {
    match direction {
        UiFlexDirection::Row => edges.top,
        UiFlexDirection::Column => edges.left,
    }
}

fn axis_cross_margin_after(direction: UiFlexDirection, edges: UiEdges) -> f32 {
    match direction {
        UiFlexDirection::Row => edges.bottom,
        UiFlexDirection::Column => edges.right,
    }
}

fn compose_axis_rect(
    direction: UiFlexDirection,
    main_pos: f32,
    cross_pos: f32,
    main_size: f32,
    cross_size: f32,
) -> Rect {
    match direction {
        UiFlexDirection::Row => Rect::new(main_pos, cross_pos, main_size, cross_size),
        UiFlexDirection::Column => Rect::new(cross_pos, main_pos, cross_size, main_size),
    }
}
