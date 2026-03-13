use super::super::{
    clear_debug_draw_transient_for_route, debug_draw_payload_for_route, register_context,
    replace_provider_debug_draw_2d_for_context, replace_provider_debug_draw_2d_for_route,
    replace_provider_debug_draw_3d_for_route, reset_for_tests, test_lock, DebugDrawShape2DV1,
    DebugDrawShape3DV1, DebuggerConfig, RuntimeSurfaceKind,
};
use crate::core::context_id::GoudContextId;
use crate::core::providers::types::{DebugShape, DebugShape3D};

#[test]
fn test_debug_draw_store_is_route_scoped_and_replaces_provider_payloads() {
    let _guard = test_lock();
    reset_for_tests();

    let route_a = register_context(
        GoudContextId::new(61, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: false,
            route_label: Some("route-a".to_string()),
        },
    );
    let route_b = register_context(
        GoudContextId::new(62, 1),
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: false,
            route_label: Some("route-b".to_string()),
        },
    );

    assert!(replace_provider_debug_draw_2d_for_route(
        &route_a,
        &[DebugShape {
            shape_type: 1,
            position: [1.0, 2.0],
            size: [3.0, 4.0],
            rotation: 0.5,
            color: [0.2, 0.3, 0.4, 0.5],
        }],
    ));
    assert!(replace_provider_debug_draw_3d_for_route(
        &route_a,
        &[DebugShape3D {
            shape_type: 0,
            position: [9.0, 8.0, 7.0],
            size: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.0, 1.0],
            color: [0.9, 0.8, 0.7, 0.6],
        }],
    ));

    let payload_a = debug_draw_payload_for_route(&route_a).expect("route-a payload should exist");
    assert_eq!(
        payload_a.provider_2d,
        vec![DebugDrawShape2DV1 {
            shape: DebugShape {
                shape_type: 1,
                position: [1.0, 2.0],
                size: [3.0, 4.0],
                rotation: 0.5,
                color: [0.2, 0.3, 0.4, 0.5],
            },
            lifetime_frames: None,
            render_layer: None,
        }]
    );
    assert_eq!(
        payload_a.provider_3d,
        vec![DebugDrawShape3DV1 {
            shape: DebugShape3D {
                shape_type: 0,
                position: [9.0, 8.0, 7.0],
                size: [1.0, 2.0, 3.0],
                rotation: [0.0, 0.0, 0.0, 1.0],
                color: [0.9, 0.8, 0.7, 0.6],
            },
            lifetime_frames: None,
            render_layer: None,
        }]
    );

    let payload_b = debug_draw_payload_for_route(&route_b).expect("route-b payload should exist");
    assert!(payload_b.provider_2d.is_empty());
    assert!(payload_b.provider_3d.is_empty());
}

#[test]
fn test_context_based_replacement_and_transient_clear_keep_provider_payload_intact() {
    let _guard = test_lock();
    reset_for_tests();

    let context_id = GoudContextId::new(63, 1);
    let route = register_context(
        context_id,
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: false,
            route_label: Some("context-route".to_string()),
        },
    );

    assert!(replace_provider_debug_draw_2d_for_context(
        context_id,
        &[DebugShape {
            shape_type: 2,
            position: [5.0, 6.0],
            size: [7.0, 8.0],
            rotation: 1.5,
            color: [1.0, 0.0, 0.0, 1.0],
        }],
    ));
    assert!(clear_debug_draw_transient_for_route(&route));

    let payload = debug_draw_payload_for_route(&route).expect("payload should exist");
    assert_eq!(payload.provider_2d.len(), 1);
    assert_eq!(payload.provider_2d[0].shape.color, [1.0, 0.0, 0.0, 1.0]);
    assert!(payload.transient_2d.is_empty());
    assert!(payload.transient_3d.is_empty());
}
