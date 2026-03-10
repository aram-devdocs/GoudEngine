use super::UiManager;
use crate::ui::component::UiComponent;
use crate::ui::node_id::UiNodeId;
use crate::ui::render_commands::{
    UiQuadCommand, UiRenderCommand, UiTextCommand, UiTexturedQuadCommand,
};
use crate::ui::visuals::resolve_widget_visual;
use crate::ui::UiVisualStyle;

impl UiManager {
    /// Builds UI render commands from the current tree and interaction state.
    pub fn build_render_commands(&mut self) -> Vec<UiRenderCommand> {
        self.recompute_layout_if_needed();

        let mut commands = Vec::new();
        for root in self.root_nodes() {
            self.collect_render_commands(root, &mut commands);
        }
        commands
    }

    fn collect_render_commands(&self, node_id: UiNodeId, out: &mut Vec<UiRenderCommand>) {
        let Some(node) = self.nodes.get(&node_id) else {
            return;
        };
        if !node.visible() {
            return;
        }

        if let Some(component) = node.component() {
            let state = self.interaction_state_for(node_id);
            let visual = resolve_widget_visual(
                component.visual_kind(),
                state,
                &self.theme,
                node.style_overrides(),
            );
            let rect = node.computed_rect();

            match component {
                UiComponent::Panel | UiComponent::Button(_) => {
                    self.emit_panel_or_button_commands(node_id, rect, node, &visual, out)
                }
                UiComponent::Label(label) => {
                    self.emit_label_commands(node_id, rect, node, label, &visual, out)
                }
                UiComponent::Image(image) => {
                    self.emit_image_commands(node_id, rect, node, image, &visual, out)
                }
                UiComponent::Slider(slider) => {
                    self.emit_slider_commands(node_id, rect, node, slider, &visual, out)
                }
            }
        }

        for &child_id in node.children() {
            self.collect_render_commands(child_id, out);
        }
    }

    fn emit_panel_or_button_commands(
        &self,
        node_id: UiNodeId,
        rect: crate::core::math::Rect,
        node: &crate::ui::node::UiNode,
        visual: &UiVisualStyle,
        out: &mut Vec<UiRenderCommand>,
    ) {
        let border_width = node
            .style_overrides()
            .and_then(|o| o.border_width)
            .unwrap_or(self.theme.spacing.small)
            .max(1.0)
            .min(rect.width * 0.5)
            .min(rect.height * 0.5);
        let fill_rect = inset_rect(rect, border_width);

        out.push(UiRenderCommand::Quad(UiQuadCommand {
            node_id,
            rect,
            color: visual.border,
        }));
        out.push(UiRenderCommand::Quad(UiQuadCommand {
            node_id,
            rect: fill_rect,
            color: visual.background,
        }));
    }

    fn emit_label_commands(
        &self,
        node_id: UiNodeId,
        rect: crate::core::math::Rect,
        node: &crate::ui::node::UiNode,
        label: &crate::ui::component::UiLabel,
        visual: &UiVisualStyle,
        out: &mut Vec<UiRenderCommand>,
    ) {
        if !label.text.is_empty() {
            let font_size = node
                .style_overrides()
                .and_then(|o| o.font_size)
                .unwrap_or(self.theme.typography.default_font_size)
                .max(1.0);
            let font_family = node
                .style_overrides()
                .and_then(|o| o.font_family.clone())
                .unwrap_or_else(|| self.theme.typography.default_font_family.clone());

            out.push(UiRenderCommand::Text(UiTextCommand {
                node_id,
                text: label.text.clone(),
                position: [rect.x, rect.y],
                font_size,
                color: visual.text,
                font_family,
            }));
        }
    }

    fn emit_image_commands(
        &self,
        node_id: UiNodeId,
        rect: crate::core::math::Rect,
        node: &crate::ui::node::UiNode,
        image: &crate::ui::UiImage,
        visual: &UiVisualStyle,
        out: &mut Vec<UiRenderCommand>,
    ) {
        let texture_path = node
            .style_overrides()
            .and_then(|o| o.texture_path.clone())
            .unwrap_or_else(|| image.texture_path.clone());

        if !texture_path.is_empty() {
            out.push(UiRenderCommand::TexturedQuad(UiTexturedQuadCommand {
                node_id,
                rect,
                texture_path,
                tint: visual.text,
            }));
        } else {
            out.push(UiRenderCommand::Quad(UiQuadCommand {
                node_id,
                rect,
                color: visual.background,
            }));
        }
    }

    fn emit_slider_commands(
        &self,
        node_id: UiNodeId,
        rect: crate::core::math::Rect,
        node: &crate::ui::node::UiNode,
        slider: &crate::ui::UiSlider,
        visual: &UiVisualStyle,
        out: &mut Vec<UiRenderCommand>,
    ) {
        let track_inset = node
            .style_overrides()
            .and_then(|o| o.widget_spacing)
            .unwrap_or(self.theme.spacing.control_inner)
            .max(1.0)
            .min(rect.width * 0.5);
        let track_height = self.theme.spacing.small.max(1.0).min(rect.height.max(1.0));
        let track_rect = crate::core::math::Rect::new(
            rect.x + track_inset,
            rect.y + (rect.height - track_height) * 0.5,
            (rect.width - track_inset * 2.0).max(0.0),
            track_height,
        );
        out.push(UiRenderCommand::Quad(UiQuadCommand {
            node_id,
            rect: track_rect,
            color: visual.background,
        }));

        let fill_width =
            (track_rect.width * slider.normalized_value()).clamp(0.0, track_rect.width.max(0.0));
        let fill_rect =
            crate::core::math::Rect::new(track_rect.x, track_rect.y, fill_width, track_rect.height);
        out.push(UiRenderCommand::Quad(UiQuadCommand {
            node_id,
            rect: fill_rect,
            color: visual.text,
        }));

        let knob_size = (node
            .style_overrides()
            .and_then(|o| o.widget_spacing)
            .unwrap_or(self.theme.spacing.control_inner)
            * 2.0)
            .max(track_height * 2.0)
            .max(1.0)
            .min(rect.width.max(1.0))
            .min(rect.height.max(1.0));
        let knob_center_x = track_rect.x + fill_width;
        let knob_center_y = rect.y + rect.height * 0.5;
        let knob_x_min = rect.x;
        let knob_x_max = rect.x + (rect.width - knob_size).max(0.0);
        let knob_y_min = rect.y;
        let knob_y_max = rect.y + (rect.height - knob_size).max(0.0);
        let knob_rect = crate::core::math::Rect::new(
            (knob_center_x - knob_size * 0.5).clamp(knob_x_min, knob_x_max),
            (knob_center_y - knob_size * 0.5).clamp(knob_y_min, knob_y_max),
            knob_size,
            knob_size,
        );
        out.push(UiRenderCommand::Quad(UiQuadCommand {
            node_id,
            rect: knob_rect,
            color: visual.border,
        }));
    }
}

fn inset_rect(rect: crate::core::math::Rect, inset: f32) -> crate::core::math::Rect {
    let clamped = inset.max(0.0);
    crate::core::math::Rect::new(
        rect.x + clamped,
        rect.y + clamped,
        (rect.width - clamped * 2.0).max(0.0),
        (rect.height - clamped * 2.0).max(0.0),
    )
}
