//! Object, light, camera, and config manipulation methods for [`Renderer3D`].

use super::super::config::Render3DConfig;
use super::super::types::{AntiAliasingMode, Light, Object3D, Renderer3DStats};
use super::Renderer3D;
use cgmath::Vector3;

#[allow(missing_docs)]
impl Renderer3D {
    pub fn set_object_position(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        let ok = self.mutate_object(id, |obj| {
            obj.position = Vector3::new(x, y, z);
        });
        if ok {
            if self.objects.get(&id).is_some_and(|o| o.is_static) {
                self.static_batch_dirty = true;
            }
            self.spatial_index_refresh(id);
        }
        ok
    }
    pub fn set_object_rotation(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        // Rotation does not change the bounding-sphere AABB, so no spatial-index
        // refresh is needed here.
        //
        // INVARIANT: only safe while the per-object bound is a sphere
        // (uniform radius around `bounds.center`). If `Object3D::bounds` is
        // ever switched to an axis-aligned or oriented box, this skip must
        // be removed and `spatial_index_refresh` called below.
        let ok = self.mutate_object(id, |obj| obj.rotation = Vector3::new(x, y, z));
        if ok && self.objects.get(&id).is_some_and(|o| o.is_static) {
            self.static_batch_dirty = true;
        }
        ok
    }
    pub fn set_object_scale(&mut self, id: u32, x: f32, y: f32, z: f32) -> bool {
        let ok = self.mutate_object(id, |obj| {
            obj.scale = Vector3::new(x, y, z);
        });
        if ok {
            if self.objects.get(&id).is_some_and(|o| o.is_static) {
                self.static_batch_dirty = true;
            }
            self.spatial_index_refresh(id);
        }
        ok
    }

    /// Mark an object as static (transform never changes) or dynamic.
    ///
    /// Static objects are batched into a single VBO when
    /// [`BatchingConfig::static_batching_enabled`] is `true`, reducing draw calls.
    pub fn set_object_static(&mut self, id: u32, is_static: bool) -> bool {
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.is_static = is_static;
            self.static_batch_dirty = true;
            true
        } else {
            false
        }
    }

    fn mutate_object(&mut self, id: u32, f: impl FnOnce(&mut Object3D)) -> bool {
        if let Some(obj) = self.objects.get_mut(&id) {
            f(obj);
            true
        } else {
            false
        }
    }

    pub fn remove_object(&mut self, id: u32) -> bool {
        if let Some(obj) = self.objects.remove(&id) {
            if obj.is_static {
                self.static_batch_dirty = true;
            }
            self.spatial_index_remove(id);
            self.backend.destroy_buffer(obj.buffer);
            true
        } else {
            false
        }
    }

    pub fn add_light(&mut self, light: Light) -> u32 {
        let id = self.next_light_id;
        self.next_light_id += 1;
        self.lights.insert(id, light);
        id
    }

    pub fn update_light(&mut self, id: u32, light: Light) -> bool {
        use std::collections::hash_map::Entry;
        if let Entry::Occupied(mut e) = self.lights.entry(id) {
            e.insert(light);
            true
        } else {
            false
        }
    }

    pub fn remove_light(&mut self, id: u32) -> bool {
        self.lights.remove(&id).is_some()
    }

    pub fn set_camera_position(&mut self, x: f32, y: f32, z: f32) {
        self.camera.position = Vector3::new(x, y, z);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.window_width = width.max(1);
        self.window_height = height.max(1);
    }

    pub fn set_viewport(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.viewport = (x, y, width.max(1), height.max(1));
    }

    pub fn set_camera_rotation(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.camera.rotation = Vector3::new(pitch, yaw, roll);
    }

    pub fn stats(&self) -> Renderer3DStats {
        self.stats
    }

    pub fn anti_aliasing_mode(&self) -> AntiAliasingMode {
        self.anti_aliasing_mode
    }

    pub fn msaa_samples(&self) -> u32 {
        self.msaa_samples
    }

    pub fn set_anti_aliasing_mode(&mut self, mode: AntiAliasingMode) -> Result<(), String> {
        self.anti_aliasing_mode = mode;
        self.backend.set_multisampling_enabled(mode.uses_msaa());
        Ok(())
    }

    pub fn set_msaa_samples(&mut self, samples: u32) {
        self.msaa_samples = match samples {
            2 | 4 | 8 => samples,
            _ => 1,
        };
    }

    pub fn set_shadow_bias(&mut self, bias: f32) {
        self.config.shadows.bias = bias.max(0.0);
    }

    pub fn shadow_bias(&self) -> f32 {
        self.config.shadows.bias
    }

    pub fn set_shadows_enabled(&mut self, enabled: bool) {
        self.config.shadows.enabled = enabled;
    }

    pub fn shadows_enabled(&self) -> bool {
        self.config.shadows.enabled
    }

    pub fn render_config(&self) -> &Render3DConfig {
        &self.config
    }

    pub fn set_render_config(&mut self, config: Render3DConfig) {
        self.config = config;
    }
}
