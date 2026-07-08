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

#[cfg(all(
    feature = "native",
    feature = "wgpu-backend",
    any(target_os = "linux", target_os = "macos", target_os = "windows")
))]
use goud_engine::core::input_manager::InputManager;
use goud_engine::libs::graphics::backend::null::NullBackend;
use goud_engine::libs::graphics::backend::ShaderLanguage;
use goud_engine::libs::graphics::renderer3d::{
    Light, LightType, Material3D, PrimitiveCreateInfo, PrimitiveType, Render3DConfig, Renderer3D,
};
#[cfg(all(
    feature = "native",
    feature = "wgpu-backend",
    any(target_os = "linux", target_os = "macos", target_os = "windows")
))]
use goud_engine::libs::graphics::{
    backend::{
        native_backend::{NativeRenderBackend, SharedNativeRenderBackend},
        wgpu_backend::WgpuBackend,
        FrameOps,
    },
    renderer3d::Renderer3D as RealWgpuRenderer3D,
};
#[cfg(all(
    feature = "native",
    feature = "wgpu-backend",
    any(target_os = "linux", target_os = "macos", target_os = "windows")
))]
use goud_engine::libs::platform::{winit_platform::WinitPlatform, PlatformBackend, WindowConfig};
#[cfg(all(
    feature = "native",
    feature = "wgpu-backend",
    any(target_os = "linux", target_os = "macos", target_os = "windows")
))]
use std::time::Duration;

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
    /// Primitive geometry to create for every object.
    pub primitive_type: PrimitiveType,
    /// Mark every primitive static (exercises the static-batch path).
    pub static_objects: bool,
    /// Enable the GPU shadow pre-pass (requires a Wgsl backend + directional light).
    pub shadows: bool,
    /// Enable material sorting of the visible draw list.
    pub material_sorting: bool,
    /// Enable real frustum culling instead of forcing every object visible.
    pub frustum_culling_enabled: bool,
}

impl SceneOptions {
    /// Default options for `object_count` objects: dynamic, unsorted-off,
    /// no shadows, [`DEFAULT_MATERIALS`] materials.
    pub fn new(object_count: usize) -> Self {
        Self {
            object_count,
            material_count: DEFAULT_MATERIALS,
            primitive_type: PrimitiveType::Plane,
            static_objects: false,
            shadows: false,
            material_sorting: true,
            frustum_culling_enabled: false,
        }
    }
}

/// Builds a [`Renderer3D`] over a Wgsl [`NullBackend`], populated per `opts`.
///
/// Determinism guarantees relied on by the companion `#[test]` assertions:
/// * Frustum culling is disabled by default, so every object is "visible" and
///   the per-object counts are pinned to `object_count`.
/// * The grid/axis overlay is disabled so it does not add draw commands.
/// * `material_count` distinct materials (ids `1..=material_count`) are assigned
///   round-robin; all primitives share the same geometry.
/// * Transforms vary only by index.
pub fn build_scene(opts: SceneOptions) -> Renderer3D {
    let backend = Box::new(NullBackend::with_shader_language(ShaderLanguage::Wgsl));
    let mut renderer =
        Renderer3D::new(backend, WIDTH, HEIGHT).expect("Renderer3D::new over NullBackend");
    populate_scene(&mut renderer, opts);
    renderer
}

fn populate_scene(renderer: &mut Renderer3D, opts: SceneOptions) {
    // Deterministic, benchmark-friendly configuration.
    let mut config = Render3DConfig::default();
    config.frustum_culling.enabled = opts.frustum_culling_enabled;
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

    // Shared geometry: reused for every primitive in the scene.
    let primitive = primitive_create_info(opts.primitive_type);

    // Create the shared material palette; `create_material` allocates ids
    // sequentially from 1, so the palette occupies `1..=material_count`.
    let material_count = opts.material_count.max(1);
    for _ in 0..material_count {
        renderer.create_material(Material3D::default());
    }

    for i in 0..opts.object_count {
        let id = renderer.create_primitive(primitive.clone());
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

fn primitive_create_info(primitive_type: PrimitiveType) -> PrimitiveCreateInfo {
    match primitive_type {
        PrimitiveType::Plane => PrimitiveCreateInfo {
            primitive_type,
            width: 1.0,
            height: 0.0,
            depth: 1.0,
            segments: 1,
            texture_id: 0,
        },
        PrimitiveType::Cube => PrimitiveCreateInfo {
            primitive_type,
            width: 1.0,
            height: 1.0,
            depth: 1.0,
            segments: 1,
            texture_id: 0,
        },
        PrimitiveType::Sphere | PrimitiveType::Cylinder => PrimitiveCreateInfo {
            primitive_type,
            width: 1.0,
            height: 1.0,
            depth: 1.0,
            segments: 8,
            texture_id: 0,
        },
    }
}

fn cull_visible_position(index: usize) -> (f32, f32, f32) {
    let x = ((index % 100) as f32) * 0.05 - 2.5;
    let z = ((index / 100) as f32) * 0.05;
    (x, 0.0, z)
}

fn cull_hidden_position(index: usize) -> (f32, f32, f32) {
    (10_000.0 + index as f32, 0.0, 0.0)
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

/// A dynamic scene with `primitive_type` for every object.
pub fn dynamic_primitive_scene(n: usize, primitive_type: PrimitiveType) -> Renderer3D {
    build_scene(SceneOptions {
        primitive_type,
        ..SceneOptions::new(n)
    })
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

/// A culling-enabled scene with `visible_count` objects kept in front of the
/// default camera and the remainder placed far outside the frustum.
pub fn cull_scaling_scene(total_count: usize, visible_count: usize) -> Renderer3D {
    let mut renderer = build_scene(SceneOptions {
        frustum_culling_enabled: true,
        ..SceneOptions::new(total_count)
    });

    for i in 0..total_count {
        let id = (i as u32) + 1;
        let (x, y, z) = if i < visible_count {
            cull_visible_position(i)
        } else {
            cull_hidden_position(i - visible_count)
        };
        renderer.set_object_position(id, x, y, z);
    }

    renderer
}

#[cfg(all(
    feature = "native",
    feature = "wgpu-backend",
    any(target_os = "linux", target_os = "macos", target_os = "windows")
))]
pub struct RealWgpuScene {
    platform: WinitPlatform,
    input: InputManager,
    backend: SharedNativeRenderBackend,
    pub renderer: RealWgpuRenderer3D,
}

#[cfg(all(
    feature = "native",
    feature = "wgpu-backend",
    any(target_os = "linux", target_os = "macos", target_os = "windows")
))]
impl RealWgpuScene {
    pub fn shadow_scene(n: usize) -> Result<Self, String> {
        let platform = WinitPlatform::new(&WindowConfig {
            width: WIDTH,
            height: HEIGHT,
            title: "renderer3d-real-wgpu-bench".to_string(),
            vsync: false,
            resizable: false,
            ..WindowConfig::default()
        })
        .map_err(|e| format!("WinitPlatform::new failed: {e}"))?;

        let backend = SharedNativeRenderBackend::new(NativeRenderBackend::Wgpu(Box::new(
            WgpuBackend::new(platform.window().clone(), false)
                .map_err(|e| format!("WgpuBackend::new failed: {e}"))?,
        )));
        let renderer = Self::build_shadow_renderer(&backend, n)?;

        Ok(Self {
            platform,
            input: InputManager::new(),
            backend,
            renderer,
        })
    }

    pub fn reset_shadow_scene(&mut self, n: usize) -> Result<(), String> {
        self.renderer = Self::build_shadow_renderer(&self.backend, n)?;
        Ok(())
    }

    fn build_shadow_renderer(
        backend: &SharedNativeRenderBackend,
        n: usize,
    ) -> Result<RealWgpuRenderer3D, String> {
        let mut renderer = RealWgpuRenderer3D::new(Box::new(backend.clone()), WIDTH, HEIGHT)
            .map_err(|e| format!("Renderer3D::new failed: {e}"))?;
        populate_scene(
            &mut renderer,
            SceneOptions {
                shadows: true,
                ..SceneOptions::new(n)
            },
        );
        Ok(renderer)
    }

    pub fn render_frame(&mut self) -> Result<(), String> {
        for attempt in 0..5 {
            self.platform.window().request_redraw();
            let _ = self.platform.poll_events(&mut self.input);
            self.backend
                .begin_frame()
                .map_err(|e| format!("begin_frame failed: {e}"))?;
            self.renderer.render(None);

            match self.backend.end_frame() {
                Ok(()) => return Ok(()),
                Err(err) if attempt < 4 && err.to_string().contains("No active frame") => {
                    std::thread::sleep(Duration::from_millis(16));
                }
                Err(err) => return Err(format!("end_frame failed: {err}")),
            }
        }

        Err("end_frame failed: no active frame after retries".to_string())
    }
}

pub fn real_wgpu_shadow_bench_env_var() -> &'static str {
    "GOUD_BENCH_REAL_WGPU_SHADOW"
}

pub fn real_wgpu_shadow_bench_skip_reason() -> Option<String> {
    if std::env::var_os(real_wgpu_shadow_bench_env_var()).is_none() {
        return Some(format!(
            "set {}=1 to opt into the real-GPU shadow bench",
            real_wgpu_shadow_bench_env_var()
        ));
    }
    if std::env::var_os("CI").is_some() {
        return Some("real-GPU shadow bench is disabled in CI".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        if std::env::var_os("DISPLAY").is_none() && std::env::var_os("WAYLAND_DISPLAY").is_none() {
            return Some("real-GPU shadow bench requires a desktop display server".to_string());
        }
    }

    #[cfg(not(all(
        feature = "native",
        feature = "wgpu-backend",
        any(target_os = "linux", target_os = "macos", target_os = "windows")
    )))]
    {
        return Some("real-GPU shadow bench requires the native wgpu desktop backend".to_string());
    }

    None
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
