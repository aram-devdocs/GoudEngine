use super::*;

#[test]
fn layout_resolves_anchor_margin_padding_and_stretch() {
    let mut ui = UiManager::new();
    let root = ui.create_node(Some(UiComponent::Panel));
    let top_left = ui.create_node(Some(UiComponent::Panel));
    let center = ui.create_node(Some(UiComponent::Panel));
    let bottom_right = ui.create_node(Some(UiComponent::Panel));
    let stretch = ui.create_node(Some(UiComponent::Panel));

    ui.set_parent(top_left, Some(root)).unwrap();
    ui.set_parent(center, Some(root)).unwrap();
    ui.set_parent(bottom_right, Some(root)).unwrap();
    ui.set_parent(stretch, Some(root)).unwrap();

    {
        let root_node = ui.get_node_mut(root).unwrap();
        root_node.set_anchor(UiAnchor::Stretch);
        root_node.set_padding(UiEdges::all(10.0));
    }

    {
        let node = ui.get_node_mut(top_left).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(100.0, 50.0));
        node.set_margin(UiEdges::new(5.0, 0.0, 7.0, 0.0));
    }

    {
        let node = ui.get_node_mut(center).unwrap();
        node.set_anchor(UiAnchor::Center);
        node.set_size(Vec2::new(100.0, 50.0));
    }

    {
        let node = ui.get_node_mut(bottom_right).unwrap();
        node.set_anchor(UiAnchor::BottomRight);
        node.set_size(Vec2::new(80.0, 40.0));
        node.set_margin(UiEdges::new(0.0, 8.0, 0.0, 6.0));
    }

    {
        let node = ui.get_node_mut(stretch).unwrap();
        node.set_anchor(UiAnchor::Stretch);
        node.set_margin(UiEdges::new(2.0, 4.0, 6.0, 8.0));
    }

    ui.set_viewport_size(800, 600);
    ui.update();

    assert_rect_eq(
        ui.computed_rect(root).unwrap(),
        Rect::new(0.0, 0.0, 800.0, 600.0),
    );
    assert_rect_eq(
        ui.computed_rect(top_left).unwrap(),
        Rect::new(15.0, 17.0, 100.0, 50.0),
    );
    assert_rect_eq(
        ui.computed_rect(center).unwrap(),
        Rect::new(350.0, 275.0, 100.0, 50.0),
    );
    assert_rect_eq(
        ui.computed_rect(bottom_right).unwrap(),
        Rect::new(702.0, 544.0, 80.0, 40.0),
    );
    assert_rect_eq(
        ui.computed_rect(stretch).unwrap(),
        Rect::new(12.0, 16.0, 774.0, 566.0),
    );
}

#[test]
fn layout_resolves_flex_row_alignment_and_spacing() {
    let mut ui = UiManager::new();
    let container = ui.create_node(Some(UiComponent::Panel));
    let a = ui.create_node(Some(UiComponent::Panel));
    let b = ui.create_node(Some(UiComponent::Panel));

    ui.set_parent(a, Some(container)).unwrap();
    ui.set_parent(b, Some(container)).unwrap();

    {
        let node = ui.get_node_mut(container).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(300.0, 100.0));
        node.set_layout(UiLayout::Flex(UiFlexLayout {
            direction: UiFlexDirection::Row,
            justify: UiJustify::Center,
            align_items: UiAlign::End,
            spacing: 10.0,
        }));
    }

    {
        let node = ui.get_node_mut(a).unwrap();
        node.set_size(Vec2::new(50.0, 20.0));
    }

    {
        let node = ui.get_node_mut(b).unwrap();
        node.set_size(Vec2::new(30.0, 10.0));
    }

    ui.set_viewport_size(800, 600);
    ui.update();

    assert_rect_eq(
        ui.computed_rect(container).unwrap(),
        Rect::new(0.0, 0.0, 300.0, 100.0),
    );
    assert_rect_eq(
        ui.computed_rect(a).unwrap(),
        Rect::new(105.0, 80.0, 50.0, 20.0),
    );
    assert_rect_eq(
        ui.computed_rect(b).unwrap(),
        Rect::new(165.0, 90.0, 30.0, 10.0),
    );
}

#[test]
fn layout_resolves_flex_column_alignment_and_spacing() {
    let mut ui = UiManager::new();
    let container = ui.create_node(Some(UiComponent::Panel));
    let a = ui.create_node(Some(UiComponent::Panel));
    let b = ui.create_node(Some(UiComponent::Panel));
    let c = ui.create_node(Some(UiComponent::Panel));

    ui.set_parent(a, Some(container)).unwrap();
    ui.set_parent(b, Some(container)).unwrap();
    ui.set_parent(c, Some(container)).unwrap();

    {
        let node = ui.get_node_mut(container).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(160.0, 220.0));
        node.set_layout(UiLayout::Flex(UiFlexLayout {
            direction: UiFlexDirection::Column,
            justify: UiJustify::End,
            align_items: UiAlign::Center,
            spacing: 15.0,
        }));
    }

    {
        let node = ui.get_node_mut(a).unwrap();
        node.set_size(Vec2::new(30.0, 40.0));
    }

    {
        let node = ui.get_node_mut(b).unwrap();
        node.set_size(Vec2::new(60.0, 20.0));
    }

    {
        let node = ui.get_node_mut(c).unwrap();
        node.set_size(Vec2::new(20.0, 30.0));
    }

    ui.set_viewport_size(800, 600);
    ui.update();

    assert_rect_eq(
        ui.computed_rect(container).unwrap(),
        Rect::new(0.0, 0.0, 160.0, 220.0),
    );
    assert_rect_eq(
        ui.computed_rect(a).unwrap(),
        Rect::new(65.0, 100.0, 30.0, 40.0),
    );
    assert_rect_eq(
        ui.computed_rect(b).unwrap(),
        Rect::new(50.0, 155.0, 60.0, 20.0),
    );
    assert_rect_eq(
        ui.computed_rect(c).unwrap(),
        Rect::new(70.0, 190.0, 20.0, 30.0),
    );
}

#[test]
fn layout_resolves_flex_cross_axis_center_with_asymmetric_margins() {
    let mut ui = UiManager::new();
    let container = ui.create_node(Some(UiComponent::Panel));
    let child = ui.create_node(Some(UiComponent::Panel));

    ui.set_parent(child, Some(container)).unwrap();

    {
        let node = ui.get_node_mut(container).unwrap();
        node.set_anchor(UiAnchor::TopLeft);
        node.set_size(Vec2::new(200.0, 100.0));
        node.set_layout(UiLayout::Flex(UiFlexLayout {
            direction: UiFlexDirection::Row,
            justify: UiJustify::Start,
            align_items: UiAlign::Center,
            spacing: 0.0,
        }));
    }

    {
        let node = ui.get_node_mut(child).unwrap();
        node.set_size(Vec2::new(40.0, 20.0));
        node.set_margin(UiEdges::new(0.0, 0.0, 10.0, 0.0));
    }

    ui.set_viewport_size(800, 600);
    ui.update();

    assert_rect_eq(
        ui.computed_rect(container).unwrap(),
        Rect::new(0.0, 0.0, 200.0, 100.0),
    );
    // Outer height is 10(top) + 20(content) + 0(bottom) = 30.
    // Center outer box in 100px cross-axis -> start at 35, then add top margin => y = 45.
    assert_rect_eq(
        ui.computed_rect(child).unwrap(),
        Rect::new(0.0, 45.0, 40.0, 20.0),
    );
}

#[test]
fn layout_recomputes_on_tree_changes_and_window_resize() {
    let mut ui = UiManager::new();
    let root = ui.create_node(Some(UiComponent::Panel));

    {
        let node = ui.get_node_mut(root).unwrap();
        node.set_anchor(UiAnchor::Stretch);
    }

    ui.set_viewport_size(100, 100);
    ui.update();
    let first_epoch = ui.layout_epoch();
    assert_rect_eq(
        ui.computed_rect(root).unwrap(),
        Rect::new(0.0, 0.0, 100.0, 100.0),
    );

    // Multiple changes in one frame should coalesce into one recompute.
    ui.set_viewport_size(300, 200);
    let child = ui.create_node(Some(UiComponent::Panel));
    ui.set_parent(child, Some(root)).unwrap();
    {
        let node = ui.get_node_mut(child).unwrap();
        node.set_anchor(UiAnchor::BottomRight);
        node.set_size(Vec2::new(10.0, 20.0));
    }

    ui.update();

    assert_eq!(ui.layout_epoch(), first_epoch + 1);
    assert_rect_eq(
        ui.computed_rect(root).unwrap(),
        Rect::new(0.0, 0.0, 300.0, 200.0),
    );
    assert_rect_eq(
        ui.computed_rect(child).unwrap(),
        Rect::new(290.0, 180.0, 10.0, 20.0),
    );
}
