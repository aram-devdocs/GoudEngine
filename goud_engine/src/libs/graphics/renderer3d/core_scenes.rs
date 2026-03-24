//! Scene management methods for [`Renderer3D`].

use super::core::Renderer3D;
use super::scene::Scene3D;

impl Renderer3D {
    /// Create a new named scene and return its ID.
    pub fn create_scene(&mut self, name: &str) -> u32 {
        let id = self.next_scene_id;
        self.next_scene_id += 1;
        self.scenes.insert(id, Scene3D::new(name.to_string()));
        id
    }

    /// Destroy a scene by ID. Returns `true` if the scene existed.
    ///
    /// If the destroyed scene was the current scene, the current scene is cleared
    /// (falls back to rendering everything).
    pub fn destroy_scene(&mut self, scene_id: u32) -> bool {
        if self.scenes.remove(&scene_id).is_some() {
            if self.current_scene == Some(scene_id) {
                self.current_scene = None;
            }
            true
        } else {
            false
        }
    }

    /// Set the current scene by ID. Returns `true` if the scene exists.
    pub fn set_current_scene(&mut self, scene_id: u32) -> bool {
        if self.scenes.contains_key(&scene_id) {
            self.current_scene = Some(scene_id);
            true
        } else {
            false
        }
    }

    /// Clear the current scene so that all objects are rendered (backward-compatible mode).
    pub fn clear_current_scene(&mut self) {
        self.current_scene = None;
    }

    /// Returns the current scene ID, or `None` if no scene is active.
    pub fn get_current_scene(&self) -> Option<u32> {
        self.current_scene
    }

    /// Add an object to a scene. Returns `true` on success.
    ///
    /// Fails if the scene or object does not exist.
    pub fn add_object_to_scene(&mut self, scene_id: u32, object_id: u32) -> bool {
        if !self.objects.contains_key(&object_id) {
            return false;
        }
        if let Some(scene) = self.scenes.get_mut(&scene_id) {
            scene.add_object(object_id);
            true
        } else {
            false
        }
    }

    /// Remove an object from a scene. Returns `true` if the scene existed and contained
    /// the object.
    pub fn remove_object_from_scene(&mut self, scene_id: u32, object_id: u32) -> bool {
        if let Some(scene) = self.scenes.get_mut(&scene_id) {
            scene.remove_object(object_id)
        } else {
            false
        }
    }

    /// Add a model to a scene. Returns `true` on success.
    ///
    /// Fails if the scene does not exist or the model/model-instance ID is unknown.
    pub fn add_model_to_scene(&mut self, scene_id: u32, model_id: u32) -> bool {
        // Collect the object IDs for this model first.
        let obj_ids = if let Some(m) = self.models.get(&model_id) {
            m.mesh_object_ids.clone()
        } else if let Some(inst) = self.model_instances.get(&model_id) {
            inst.mesh_object_ids.clone()
        } else {
            return false;
        };

        if let Some(scene) = self.scenes.get_mut(&scene_id) {
            scene.add_model(model_id);
            // Also add all the model's underlying objects to the scene.
            for obj_id in obj_ids {
                scene.add_object(obj_id);
            }
            true
        } else {
            false
        }
    }

    /// Remove a model from a scene. Returns `true` if the scene existed and contained
    /// the model.
    pub fn remove_model_from_scene(&mut self, scene_id: u32, model_id: u32) -> bool {
        // Collect the object IDs for this model first.
        let obj_ids = if let Some(m) = self.models.get(&model_id) {
            m.mesh_object_ids.clone()
        } else if let Some(inst) = self.model_instances.get(&model_id) {
            inst.mesh_object_ids.clone()
        } else {
            return false;
        };

        if let Some(scene) = self.scenes.get_mut(&scene_id) {
            let removed = scene.remove_model(model_id);
            // Also remove all the model's underlying objects from the scene.
            if removed {
                for obj_id in obj_ids {
                    scene.remove_object(obj_id);
                }
            }
            removed
        } else {
            false
        }
    }

    /// Add a light to a scene. Returns `true` on success.
    ///
    /// Fails if the scene or light does not exist.
    pub fn add_light_to_scene(&mut self, scene_id: u32, light_id: u32) -> bool {
        if !self.lights.contains_key(&light_id) {
            return false;
        }
        if let Some(scene) = self.scenes.get_mut(&scene_id) {
            scene.add_light(light_id);
            true
        } else {
            false
        }
    }

    /// Remove a light from a scene. Returns `true` if the scene existed and contained
    /// the light.
    pub fn remove_light_from_scene(&mut self, scene_id: u32, light_id: u32) -> bool {
        if let Some(scene) = self.scenes.get_mut(&scene_id) {
            scene.remove_light(light_id)
        } else {
            false
        }
    }
}
