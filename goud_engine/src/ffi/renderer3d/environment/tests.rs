//! Tests for environment FFI functions.

use super::*;
use crate::core::debugger::{
    self, dispatch_request_json_for_route, register_context, DebuggerConfig, RuntimeSurfaceKind,
};
use crate::ffi::context::GoudContextId;
#[cfg(feature = "rapier3d")]
use crate::ffi::physics::{
    goud_physics3d_add_collider, goud_physics3d_add_rigid_body, goud_physics3d_create,
    goud_physics3d_destroy,
};
use serde_json::json;

#[test]
fn resolved_runtime_debug_draw_shapes_returns_empty_until_debug_draw_enabled() {
    let _guard = debugger::test_lock();
    debugger::reset_for_tests();

    let context_id = GoudContextId::new(761, 1);
    let route = register_context(
        context_id,
        RuntimeSurfaceKind::HeadlessContext,
        &DebuggerConfig {
            enabled: true,
            publish_local_attach: true,
            route_label: Some("renderer3d-debug-draw".to_string()),
        },
    );

    #[cfg(feature = "rapier3d")]
    {
        assert_eq!(goud_physics3d_create(context_id, 0.0, -9.8, 0.0), 0);
        let body = goud_physics3d_add_rigid_body(context_id, 1, 0.0, 0.0, 0.0, 1.0);
        assert!(body > 0);
        let collider =
            goud_physics3d_add_collider(context_id, body as u64, 1, 1.0, 1.0, 1.0, 0.0, 0.5, 0.0);
        assert!(collider > 0);
    }

    assert!(resolved_runtime_debug_draw_shapes(context_id).is_empty());

    let response = dispatch_request_json_for_route(
        &route,
        &json!({ "verb": "set_debug_draw_enabled", "enabled": true }).to_string(),
    )
    .expect("dispatcher should return JSON");
    assert_eq!(response["ok"], true);

    #[cfg(feature = "rapier3d")]
    assert_eq!(resolved_runtime_debug_draw_shapes(context_id).len(), 1);
    #[cfg(not(feature = "rapier3d"))]
    assert!(resolved_runtime_debug_draw_shapes(context_id).is_empty());
    #[cfg(feature = "rapier3d")]
    assert_eq!(goud_physics3d_destroy(context_id), 0);
}

// =========================================================================
// Config getter / setter tests with invalid context
// =========================================================================

#[test]
fn test_config_getters_invalid_context_return_error_values() {
    assert!(
        (goud_renderer3d_get_frustum_culling_fov(GOUD_INVALID_CONTEXT_ID) - (-1.0)).abs()
            < f32::EPSILON,
    );
    assert!(
        (goud_renderer3d_get_frustum_culling_near_plane(GOUD_INVALID_CONTEXT_ID) - (-1.0)).abs()
            < f32::EPSILON,
    );
    assert!(
        (goud_renderer3d_get_frustum_culling_far_plane(GOUD_INVALID_CONTEXT_ID) - (-1.0)).abs()
            < f32::EPSILON,
    );
    assert!(
        (goud_renderer3d_get_animation_lod_distance(GOUD_INVALID_CONTEXT_ID) - (-1.0)).abs()
            < f32::EPSILON,
    );
    assert!(
        (goud_renderer3d_get_animation_lod_skip_distance(GOUD_INVALID_CONTEXT_ID) - (-1.0)).abs()
            < f32::EPSILON,
    );
    assert!(
        (goud_renderer3d_get_baked_animation_sample_rate(GOUD_INVALID_CONTEXT_ID) - (-1.0)).abs()
            < f32::EPSILON,
    );
    assert!(
        (goud_renderer3d_get_shadow_bias(GOUD_INVALID_CONTEXT_ID) - (-1.0)).abs() < f32::EPSILON,
    );
}

#[test]
fn test_config_int_getters_invalid_context_return_error_values() {
    assert_eq!(
        goud_renderer3d_get_skinning_mode(GOUD_INVALID_CONTEXT_ID),
        -1
    );
    assert_eq!(
        goud_renderer3d_get_min_instances_for_batching(GOUD_INVALID_CONTEXT_ID),
        -1
    );
    assert_eq!(
        goud_renderer3d_get_shadow_map_size(GOUD_INVALID_CONTEXT_ID),
        -1
    );
}

#[test]
fn test_config_bool_getters_invalid_context_return_false() {
    assert!(!goud_renderer3d_get_frustum_culling_enabled(
        GOUD_INVALID_CONTEXT_ID
    ));
    assert!(!goud_renderer3d_get_material_sorting_enabled(
        GOUD_INVALID_CONTEXT_ID
    ));
    assert!(!goud_renderer3d_get_static_batching_enabled(
        GOUD_INVALID_CONTEXT_ID
    ));
    assert!(!goud_renderer3d_get_instancing_enabled(
        GOUD_INVALID_CONTEXT_ID
    ));
    assert!(!goud_renderer3d_get_animation_lod_enabled(
        GOUD_INVALID_CONTEXT_ID
    ));
    assert!(!goud_renderer3d_get_shared_animation_eval(
        GOUD_INVALID_CONTEXT_ID
    ));
    assert!(!goud_renderer3d_get_shadows_enabled(
        GOUD_INVALID_CONTEXT_ID
    ));
}

#[test]
fn test_config_setters_invalid_context_return_error() {
    assert_eq!(
        goud_renderer3d_set_frustum_culling_enabled(GOUD_INVALID_CONTEXT_ID, true),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_skinning_mode(GOUD_INVALID_CONTEXT_ID, 0),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_material_sorting_enabled(GOUD_INVALID_CONTEXT_ID, true),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_animation_lod_enabled(GOUD_INVALID_CONTEXT_ID, true),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_shared_animation_eval(GOUD_INVALID_CONTEXT_ID, true),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_animation_lod_distance(GOUD_INVALID_CONTEXT_ID, 50.0),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_animation_lod_skip_distance(GOUD_INVALID_CONTEXT_ID, 100.0),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_static_batching_enabled(GOUD_INVALID_CONTEXT_ID, true),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_instancing_enabled(GOUD_INVALID_CONTEXT_ID, true),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_baked_animation_sample_rate(GOUD_INVALID_CONTEXT_ID, 60.0),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_min_instances_for_batching(GOUD_INVALID_CONTEXT_ID, 4),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_default_material_color(GOUD_INVALID_CONTEXT_ID, 1.0, 0.0, 0.0, 1.0),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_frustum_culling_fov(GOUD_INVALID_CONTEXT_ID, 90.0),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_frustum_culling_near_plane(GOUD_INVALID_CONTEXT_ID, 0.5),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_frustum_culling_far_plane(GOUD_INVALID_CONTEXT_ID, 500.0),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_shadows_enabled(GOUD_INVALID_CONTEXT_ID, true),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_shadow_map_size(GOUD_INVALID_CONTEXT_ID, 1024),
        -1
    );
    assert_eq!(
        goud_renderer3d_set_shadow_bias(GOUD_INVALID_CONTEXT_ID, 0.01),
        -1
    );
}

#[test]
fn test_stats_getters_invalid_context_return_error() {
    assert_eq!(goud_renderer3d_get_draw_calls(GOUD_INVALID_CONTEXT_ID), -1);
    assert_eq!(
        goud_renderer3d_get_visible_object_count(GOUD_INVALID_CONTEXT_ID),
        -1
    );
    assert_eq!(
        goud_renderer3d_get_culled_object_count(GOUD_INVALID_CONTEXT_ID),
        -1
    );
    assert_eq!(
        goud_renderer3d_get_instanced_draw_calls(GOUD_INVALID_CONTEXT_ID),
        -1
    );
    assert_eq!(
        goud_renderer3d_get_active_instance_count(GOUD_INVALID_CONTEXT_ID),
        -1
    );
    assert_eq!(
        goud_renderer3d_get_animation_evaluation_count(GOUD_INVALID_CONTEXT_ID),
        -1
    );
    assert_eq!(
        goud_renderer3d_get_animation_evaluation_saved_count(GOUD_INVALID_CONTEXT_ID),
        -1
    );
    assert_eq!(
        goud_renderer3d_get_bone_matrix_upload_count(GOUD_INVALID_CONTEXT_ID),
        -1
    );
}

#[test]
fn test_environment_functions_invalid_context_return_false() {
    assert!(!goud_renderer3d_configure_grid(
        GOUD_INVALID_CONTEXT_ID,
        true,
        10.0,
        10
    ));
    assert!(!goud_renderer3d_configure_skybox(
        GOUD_INVALID_CONTEXT_ID,
        true,
        0.5,
        0.5,
        0.8,
        1.0
    ));
    assert!(!goud_renderer3d_configure_fog(
        GOUD_INVALID_CONTEXT_ID,
        true,
        0.5,
        0.5,
        0.5,
        0.1
    ));
    assert!(!goud_renderer3d_set_fog_enabled(
        GOUD_INVALID_CONTEXT_ID,
        true
    ));
    assert!(!goud_renderer3d_set_grid_enabled(
        GOUD_INVALID_CONTEXT_ID,
        true
    ));
    assert!(!goud_renderer3d_render(GOUD_INVALID_CONTEXT_ID));
    assert!(!goud_renderer3d_render_all(GOUD_INVALID_CONTEXT_ID));
}
