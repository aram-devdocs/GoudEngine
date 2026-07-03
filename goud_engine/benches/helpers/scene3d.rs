//! Deterministic 3D scene builders shared by the renderer frame benchmarks.
//!
//! These helpers construct a real [`Renderer3D`] over a headless
//! [`NullBackend`] and populate it with plane primitives so that
//! `Renderer3D::render(None)` exercises the CPU frame-scan / cull / sort /
//! draw-record pipeline without touching a GPU.
//!
//! The backend is created in **Wgsl** mode on purpose: the wgpu draw-recording
//! path (and the GPU shadow pre-pass) in `Renderer3D` is only taken when the
//! backend reports [`ShaderLanguage::Wgsl`]. A Glsl backend would fall back to
//! the legacy CPU paths, which are not what the later renderer optimizations
//! target.
//!
//! Everything here is deterministic (no randomness — transforms vary purely by
//! index) so the companion `#[test]` assertions can pin exact draw-call and
//! culled counts.
//!
//! Included into bench/test targets via `#[path = "helpers/scene3d.rs"]`.

#![allow(dead_code)]

use goud_engine::libs::graphics::backend::null::NullBackend;
use goud_engine::libs::graphics::backend::ShaderLanguage;
use goud_engine::libs::graphics::renderer3d::{
    Light, LightType, Material3D, PrimitiveCreateInfo, PrimitiveType, Render3DConfig, Renderer3D,
};

/// Default number of distinct materials shared across a scene's primitives.
pub const DEFAULT_MATERIALS: usize = 8;

/// Window dimensions used for every benchmark scene (kept constant so the
/// projection matrix and viewport never vary between runs).
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;

/// Options controlling how a benchmark scene is assembled.
#[derive(Clone, Copy, Debug)]
pub struct SceneOptions {
    /// Number of plane primitives to create.
    pub object_count: usize,
    /// Number of distinct materials, assigned round-robin across the objects.
    pub material_count: usize,
    /// Mark every primitive static (exercises the static-batch path).
    pub static_objects: bool,
    /// Enable the GPU shadow pre-pass (requires a Wgsl backend + directional light).
    pub shadows: bool,
    /// Enable material sorting of the visible draw list.
    pub material_sorting: bool,
}

impl SceneOptions {
    /// Default options for `object_count` objects: dynamic, unsorted-off,
    /// no shadows, [`DEFAULT_MATERIALS`] materials.
    pub fn new(object_count: usize) -> Self {
        Self {
            object_count,
            material_count: DEFAULT_MATERIALS,
            static_objects: false,
            shadows: false,
            material_sorting: true,
        }
    }
}

/// Builds a [`Renderer3D`] over a Wgsl [`NullBackend`], populated per `opts`.
///
/// Determinism guarantees relied on by the companion `#[test]` assertions:
/// * Frustum culling is **disabled**, so every object is "visible" and the
///   per-object counts are pinned to `object_count`.
/// * The grid/axis overlay is disabled so it does not add draw commands.
/// * `material_count` distinct materials (ids `1..=material_count`) are assigned
///   round-robin; all primitives share the same unit-plane geometry.
/// * Transforms vary only by index.
pub fn build_scene(opts: SceneOptions) -> Renderer3D {
    let backend = Box::new(NullBackend::with_shader_language(ShaderLanguage::Wgsl));
    let mut renderer =
        Renderer3D::new(backend, WIDTH, HEIGHT).expect("Renderer3D::new over NullBackend");

    // Deterministic, benchmark-friendly configuration.
    let mut config = Render3DConfig::default();
    // Pin visible == object_count regardless of camera/positions.
    config.frustum_culling.enabled = false;
    config.batching.material_sorting_enabled = opts.material_sorting;
    // Static batching is only meaningful when the objects are static.
    config.batching.static_batching_enabled = opts.static_objects;
    // Guarantee every static object fits in a single batch buffer so the number
    // of static-batch groups equals `material_count` (no overflow → individual
    // draws). A plane is 12 vertices; 64/object is a comfortable upper bound.
    config.batching.max_static_batch_vertices = opts.object_count.saturating_mul(64).max(50_000);
    config.shadows.enabled = opts.shadows;
    renderer.set_render_config(config);

    // The grid/axis overlay draws every frame but is irrelevant to the object
    // scan being measured; disable it to keep the scene focused and the backend
    // draw counters clean.
    renderer.set_grid_enabled(false);

    // Shared geometry: a unit plane, reused for every primitive.
    let plane = PrimitiveCreateInfo {
        primitive_type: PrimitiveType::Plane,
        width: 1.0,
        height: 1.0,
        depth: 1.0,
        segments: 1,
        texture_id: 0,
    };

    // Create the shared material palette; `create_material` allocates ids
    // sequentially from 1, so the palette occupies `1..=material_count`.
    let material_count = opts.material_count.max(1);
    for _ in 0..material_count {
        renderer.create_material(Material3D::default());
    }

    for i in 0..opts.object_count {
        let id = renderer.create_primitive(plane.clone());
        let (x, y, z) = object_position(i, 0);
        renderer.set_object_position(id, x, y, z);
        let material_id = ((i % material_count) as u32) + 1;
        renderer.set_object_material(id, material_id);
        if opts.static_objects {
            renderer.set_object_static(id, true);
        }
    }

    if opts.shadows {
        // A directional light is required for the shadow light-space matrix.
        // The default direction (0, -1, 0) points straight down over the scene.
        renderer.add_light(Light {
            light_type: LightType::Directional,
            ..Light::default()
        });
    }

    renderer
}

/// Deterministic world position for object `index` at frame `frame`.
///
/// Objects are laid out on a 100-wide grid centered on the origin so the scene
/// has real bounds (needed for a meaningful shadow light-space matrix). The
/// `frame` term applies a tiny per-frame drift used by the "moving" variant.
fn object_position(index: usize, frame: u64) -> (f32, f32, f32) {
    let drift = (frame as f32) * 0.01;
    let x = ((index % 100) as f32) - 50.0 + drift;
    let z = ((index / 100) as f32) - 50.0;
    (x, 0.0, z)
}

/// A static scene of `n` plane primitives (exercises the static-batch path).
pub fn static_scene(n: usize) -> Renderer3D {
    build_scene(SceneOptions {
        static_objects: true,
        ..SceneOptions::new(n)
    })
}

/// A dynamic (non-static) scene of `n` plane primitives.
pub fn dynamic_scene(n: usize) -> Renderer3D {
    build_scene(SceneOptions::new(n))
}

/// A dynamic scene with material sorting toggled explicitly.
pub fn dynamic_scene_sorting(n: usize, material_sorting: bool) -> Renderer3D {
    build_scene(SceneOptions {
        material_sorting,
        ..SceneOptions::new(n)
    })
}

/// A dynamic scene with the GPU shadow pre-pass enabled and a directional light.
pub fn shadow_scene(n: usize) -> Renderer3D {
    build_scene(SceneOptions {
        shadows: true,
        ..SceneOptions::new(n)
    })
}

/// Moves every object by a small deterministic per-frame delta, simulating a
/// frame in which all objects changed transform.
///
/// Object ids are `1..=object_count` because `create_primitive` allocates ids
/// sequentially from 1 (see [`build_scene`]).
pub fn advance_dynamic_scene(renderer: &mut Renderer3D, object_count: usize, frame: u64) {
    for i in 0..object_count {
        let id = (i as u32) + 1;
        let (x, y, z) = object_position(i, frame);
        renderer.set_object_position(id, x, y, z);
    }
}
