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

mod camera;
mod environment;
mod lighting;
mod primitives;
mod state;

// Re-export constants so external callers see the same public surface.
pub use primitives::{
    GOUD_INVALID_LIGHT, GOUD_INVALID_OBJECT, GOUD_LIGHT_TYPE_DIRECTIONAL, GOUD_LIGHT_TYPE_POINT,
    GOUD_LIGHT_TYPE_SPOT, GOUD_PRIMITIVE_CUBE, GOUD_PRIMITIVE_CYLINDER, GOUD_PRIMITIVE_PLANE,
    GOUD_PRIMITIVE_SPHERE,
};

// FFI functions are `#[no_mangle] extern "C"` and therefore globally visible; the
// `pub use` below makes them importable via the module path as well.
pub use camera::{goud_renderer3d_set_camera_position, goud_renderer3d_set_camera_rotation};
pub use environment::{
    goud_renderer3d_configure_fog, goud_renderer3d_configure_grid,
    goud_renderer3d_configure_skybox, goud_renderer3d_render, goud_renderer3d_render_all,
    goud_renderer3d_set_fog_enabled, goud_renderer3d_set_grid_enabled,
};
pub use lighting::{
    goud_renderer3d_add_light, goud_renderer3d_remove_light, goud_renderer3d_update_light,
};
pub use primitives::{
    goud_renderer3d_create_cube, goud_renderer3d_create_cylinder, goud_renderer3d_create_plane,
    goud_renderer3d_create_sphere, goud_renderer3d_destroy_object,
    goud_renderer3d_set_object_position, goud_renderer3d_set_object_rotation,
    goud_renderer3d_set_object_scale,
};
