//! 3D Renderer FFI Module
//!
//! Provides C-compatible functions for 3D rendering operations.
//! Wraps the Renderer3D from libs/graphics/renderer3d.rs
//!
//! ## Example Usage (C#)
//!
//! ```csharp
//! // Create a primitive
//! uint cubeId = game.CreateCube(textureId, 1.0f, 1.0f, 1.0f);
//! game.SetObjectPosition(cubeId, 0, 1, 0);
//!
//! // Add a light
//! uint lightId = game.AddLight(LightType.Point, x, y, z, ...);
//!
//! // In game loop
//! game.Render3D();
//! ```

mod animation;
mod camera;
mod environment;
mod lighting;
mod materials;
mod model;
mod postprocess;
mod primitives;
mod scene;
mod skinned;
mod state;

// Re-export constants so external callers see the same public surface.
pub use primitives::{
    GOUD_INVALID_LIGHT, GOUD_INVALID_OBJECT, GOUD_LIGHT_TYPE_DIRECTIONAL, GOUD_LIGHT_TYPE_POINT,
    GOUD_LIGHT_TYPE_SPOT, GOUD_PRIMITIVE_CUBE, GOUD_PRIMITIVE_CYLINDER, GOUD_PRIMITIVE_PLANE,
    GOUD_PRIMITIVE_SPHERE,
};

// FFI functions are `#[no_mangle] extern "C"` and therefore globally visible; the
// `pub use` below makes them importable via the module path as well.
pub use animation::{
    goud_renderer3d_blend_animations, goud_renderer3d_get_animation_count,
    goud_renderer3d_get_animation_name, goud_renderer3d_get_animation_progress,
    goud_renderer3d_is_animation_playing, goud_renderer3d_play_animation,
    goud_renderer3d_set_animation_speed, goud_renderer3d_stop_animation,
    goud_renderer3d_transition_animation, goud_renderer3d_update_animations,
};
pub use camera::{goud_renderer3d_set_camera_position, goud_renderer3d_set_camera_rotation};
pub use environment::{
    goud_renderer3d_configure_fog, goud_renderer3d_configure_grid,
    goud_renderer3d_configure_skybox, goud_renderer3d_get_culled_object_count,
    goud_renderer3d_get_draw_calls, goud_renderer3d_get_visible_object_count,
    goud_renderer3d_render, goud_renderer3d_render_all, goud_renderer3d_set_animation_lod_enabled,
    goud_renderer3d_set_fog_enabled, goud_renderer3d_set_frustum_culling_enabled,
    goud_renderer3d_set_grid_enabled, goud_renderer3d_set_material_sorting_enabled,
    goud_renderer3d_set_shared_animation_eval, goud_renderer3d_set_skinning_mode,
};
pub use lighting::{
    goud_renderer3d_add_light, goud_renderer3d_remove_light, goud_renderer3d_update_light,
};
pub use materials::{
    goud_renderer3d_create_material, goud_renderer3d_get_object_material,
    goud_renderer3d_remove_material, goud_renderer3d_set_object_material,
    goud_renderer3d_update_material, GOUD_INVALID_MATERIAL, GOUD_MATERIAL_TYPE_PBR,
    GOUD_MATERIAL_TYPE_PHONG, GOUD_MATERIAL_TYPE_UNLIT,
};
pub use model::{
    goud_renderer3d_destroy_model, goud_renderer3d_get_model_bounding_box,
    goud_renderer3d_get_model_mesh_count, goud_renderer3d_instantiate_model,
    goud_renderer3d_load_model, goud_renderer3d_set_model_material,
    goud_renderer3d_set_model_position, goud_renderer3d_set_model_rotation,
    goud_renderer3d_set_model_scale, GOUD_INVALID_MODEL,
};
pub use postprocess::{
    goud_renderer3d_add_bloom_pass, goud_renderer3d_add_blur_pass,
    goud_renderer3d_add_color_grade_pass, goud_renderer3d_postprocess_pass_count,
    goud_renderer3d_remove_postprocess_pass,
};
pub use primitives::{
    goud_renderer3d_create_cube, goud_renderer3d_create_cylinder, goud_renderer3d_create_plane,
    goud_renderer3d_create_sphere, goud_renderer3d_destroy_object,
    goud_renderer3d_set_object_position, goud_renderer3d_set_object_rotation,
    goud_renderer3d_set_object_scale, goud_renderer3d_set_object_static,
};
pub use scene::{
    goud_renderer3d_add_light_to_scene, goud_renderer3d_add_model_to_scene,
    goud_renderer3d_add_object_to_scene, goud_renderer3d_clear_current_scene,
    goud_renderer3d_create_scene, goud_renderer3d_destroy_scene, goud_renderer3d_get_current_scene,
    goud_renderer3d_remove_light_from_scene, goud_renderer3d_remove_model_from_scene,
    goud_renderer3d_remove_object_from_scene, goud_renderer3d_set_current_scene,
    GOUD_INVALID_SCENE,
};
pub use skinned::{
    goud_renderer3d_create_skinned_mesh, goud_renderer3d_remove_skinned_mesh,
    goud_renderer3d_set_skinned_mesh_bones, goud_renderer3d_set_skinned_mesh_position,
    goud_renderer3d_set_skinned_mesh_rotation, goud_renderer3d_set_skinned_mesh_scale,
    GOUD_INVALID_SKINNED_MESH,
};
