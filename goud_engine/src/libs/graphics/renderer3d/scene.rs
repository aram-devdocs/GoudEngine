//! Named 3D scene containing a subset of renderer objects, models, lights, and environment config.

use std::collections::HashSet;

use super::types::{FogConfig, GridConfig, SkyboxConfig};

/// A named 3D scene containing a subset of renderer objects.
///
/// When a scene is active, only the objects, models, and lights belonging to it
/// are rendered, and its per-scene environment configs (fog, skybox, grid) are
/// used instead of the renderer-level defaults.
pub struct Scene3D {
    /// Human-readable scene name.
    pub name: String,
    /// [`Object3D`](super::types::Object3D) IDs belonging to this scene.
    pub objects: HashSet<u32>,
    /// [`Model3D`](super::model::Model3D) / [`ModelInstance3D`](super::model::ModelInstance3D) IDs.
    pub models: HashSet<u32>,
    /// [`Light`](super::types::Light) IDs belonging to this scene.
    pub lights: HashSet<u32>,
    /// Per-scene fog configuration.
    pub fog: FogConfig,
    /// Per-scene skybox configuration.
    pub skybox: SkyboxConfig,
    /// Per-scene grid configuration.
    pub grid: GridConfig,
}

impl Scene3D {
    /// Create a new empty scene with the given name and default environment configs.
    pub fn new(name: String) -> Self {
        Self {
            name,
            objects: HashSet::new(),
            models: HashSet::new(),
            lights: HashSet::new(),
            fog: FogConfig::default(),
            skybox: SkyboxConfig::default(),
            grid: GridConfig::default(),
        }
    }

    /// Add an object ID to the scene. Returns `true` if the ID was newly inserted.
    pub fn add_object(&mut self, id: u32) -> bool {
        self.objects.insert(id)
    }

    /// Remove an object ID from the scene. Returns `true` if the ID was present.
    pub fn remove_object(&mut self, id: u32) -> bool {
        self.objects.remove(&id)
    }

    /// Add a model ID to the scene. Returns `true` if the ID was newly inserted.
    pub fn add_model(&mut self, id: u32) -> bool {
        self.models.insert(id)
    }

    /// Remove a model ID from the scene. Returns `true` if the ID was present.
    pub fn remove_model(&mut self, id: u32) -> bool {
        self.models.remove(&id)
    }

    /// Add a light ID to the scene. Returns `true` if the ID was newly inserted.
    pub fn add_light(&mut self, id: u32) -> bool {
        self.lights.insert(id)
    }

    /// Remove a light ID from the scene. Returns `true` if the ID was present.
    pub fn remove_light(&mut self, id: u32) -> bool {
        self.lights.remove(&id)
    }

    /// Returns `true` if the scene contains the given object ID.
    pub fn contains_object(&self, id: u32) -> bool {
        self.objects.contains(&id)
    }

    /// Returns `true` if the scene contains the given model ID.
    pub fn contains_model(&self, id: u32) -> bool {
        self.models.contains(&id)
    }

    /// Returns `true` if the scene contains the given light ID.
    pub fn contains_light(&self, id: u32) -> bool {
        self.lights.contains(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_new_has_empty_sets() {
        let scene = Scene3D::new("test".to_string());
        assert_eq!(scene.name, "test");
        assert!(scene.objects.is_empty());
        assert!(scene.models.is_empty());
        assert!(scene.lights.is_empty());
    }

    #[test]
    fn test_scene_add_remove_objects() {
        let mut scene = Scene3D::new("s".to_string());
        assert!(scene.add_object(1));
        assert!(!scene.add_object(1)); // duplicate
        assert!(scene.contains_object(1));
        assert!(!scene.contains_object(2));
        assert!(scene.remove_object(1));
        assert!(!scene.remove_object(1)); // already removed
        assert!(!scene.contains_object(1));
    }

    #[test]
    fn test_scene_add_remove_models() {
        let mut scene = Scene3D::new("s".to_string());
        assert!(scene.add_model(10));
        assert!(!scene.add_model(10));
        assert!(scene.contains_model(10));
        assert!(scene.remove_model(10));
        assert!(!scene.contains_model(10));
    }

    #[test]
    fn test_scene_add_remove_lights() {
        let mut scene = Scene3D::new("s".to_string());
        assert!(scene.add_light(5));
        assert!(!scene.add_light(5));
        assert!(scene.contains_light(5));
        assert!(scene.remove_light(5));
        assert!(!scene.contains_light(5));
    }
}
