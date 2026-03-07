//! Scene loading and unloading utilities.
//!
//! Provides [`SceneLoader`] for synchronous scene load/unload/save operations
//! and [`DeferredSceneLoad`] for background JSON parsing with deferred
//! integration into the [`SceneManager`].

use std::sync::mpsc;

use crate::core::error::GoudError;

use super::data::SceneData;
use super::manager::{SceneId, SceneManager};
use super::serialization::{deserialize_scene, scene_from_json, scene_to_json, serialize_scene};

// =============================================================================
// SceneLoader
// =============================================================================

/// Static helper methods for loading and unloading scenes.
pub struct SceneLoader;

impl SceneLoader {
    /// Loads a scene from a [`SceneData`] into the manager.
    ///
    /// Creates a new scene with the given name, registers built-in
    /// serializable components, and deserializes the scene data into
    /// the world.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - A scene with the same name already exists
    /// - Deserialization of scene data fails
    pub fn load_scene(
        manager: &mut SceneManager,
        name: &str,
        data: SceneData,
    ) -> Result<SceneId, GoudError> {
        let id = manager.create_scene(name)?;
        let world = manager.get_scene_mut(id).ok_or_else(|| {
            GoudError::InternalError("Scene was created but not accessible".to_string())
        })?;
        world.register_builtin_serializables();
        deserialize_scene(&data, world)?;
        Ok(id)
    }

    /// Unloads a scene by name.
    ///
    /// Looks up the scene by name and destroys it. Dropping the
    /// [`World`] cleans up all entities and components.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No scene with the given name exists
    /// - The scene is the default scene (which cannot be destroyed)
    pub fn unload_scene(manager: &mut SceneManager, name: &str) -> Result<(), GoudError> {
        let id = manager
            .get_scene_by_name(name)
            .ok_or_else(|| GoudError::ResourceNotFound(format!("Scene '{}' not found", name)))?;
        manager.destroy_scene(id)
    }

    /// Loads a scene from a JSON string.
    ///
    /// Parses the JSON into [`SceneData`], then delegates to
    /// [`load_scene`](Self::load_scene).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The JSON is invalid or does not match the expected schema
    /// - Scene creation or deserialization fails
    pub fn load_scene_from_json(
        manager: &mut SceneManager,
        name: &str,
        json: &str,
    ) -> Result<SceneId, GoudError> {
        let data = scene_from_json(json)?;
        Self::load_scene(manager, name, data)
    }

    /// Saves a scene to a JSON string.
    ///
    /// Serializes all entities and components in the scene's world,
    /// then converts to a pretty-printed JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The scene ID does not exist
    /// - Serialization fails
    pub fn save_scene_to_json(manager: &SceneManager, id: SceneId) -> Result<String, GoudError> {
        let world = manager
            .get_scene(id)
            .ok_or_else(|| GoudError::ResourceNotFound(format!("Scene id {} not found", id)))?;
        let scene_data = serialize_scene(world, "scene")?;
        scene_to_json(&scene_data)
    }
}

// =============================================================================
// DeferredSceneLoad
// =============================================================================

/// Result of a deferred scene load operation.
struct DeferredResult {
    /// Name to use when creating the scene in the manager.
    name: String,
    /// Parsed scene data or an error from JSON parsing.
    data: Result<SceneData, GoudError>,
}

/// Deferred (background-thread) scene loader.
///
/// Allows JSON parsing to happen on a background thread while the
/// main thread continues. Call [`process_completed`](Self::process_completed)
/// each frame to integrate any finished loads into the [`SceneManager`].
///
/// # Example
///
/// ```ignore
/// let deferred = DeferredSceneLoad::new();
/// deferred.request_load("level_2".into(), json_string);
///
/// // Later (e.g., next frame):
/// let results = deferred.process_completed(&mut manager);
/// ```
pub struct DeferredSceneLoad {
    sender: mpsc::Sender<DeferredResult>,
    receiver: mpsc::Receiver<DeferredResult>,
}

impl DeferredSceneLoad {
    /// Creates a new deferred scene loader.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { sender, receiver }
    }

    /// Requests a scene load on a background thread.
    ///
    /// Spawns a thread to parse the JSON string into [`SceneData`].
    /// The result is sent through a channel and can be consumed by
    /// [`process_completed`](Self::process_completed).
    pub fn request_load(&self, name: String, json: String) {
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            let data = scene_from_json(&json);
            // Ignore send errors -- receiver may have been dropped.
            let _ = sender.send(DeferredResult { name, data });
        });
    }

    /// Drains all completed loads and integrates them into the manager.
    ///
    /// For each completed parse, calls [`SceneLoader::load_scene`] to
    /// create the scene. Returns a vec of results (one per completed
    /// load).
    pub fn process_completed(&self, manager: &mut SceneManager) -> Vec<Result<SceneId, GoudError>> {
        let mut results = Vec::new();
        while let Ok(deferred) = self.receiver.try_recv() {
            let result = match deferred.data {
                Ok(data) => SceneLoader::load_scene(manager, &deferred.name, data),
                Err(e) => Err(e),
            };
            results.push(result);
        }
        results
    }
}

impl Default for DeferredSceneLoad {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::Vec2;
    use crate::ecs::components::hierarchy::Name;
    use crate::ecs::components::Transform2D;
    use crate::ecs::World;

    /// Helper: build a SceneData with one entity (named, with transform).
    fn sample_scene_data() -> SceneData {
        let mut world = World::new();
        world.register_builtin_serializables();

        let e = world.spawn_empty();
        world.insert(e, Name::new("hero"));
        world.insert(e, Transform2D::new(Vec2::new(10.0, 20.0), 0.0, Vec2::one()));

        serialize_scene(&world, "sample").unwrap()
    }

    // ----- load from SceneData -----------------------------------------------

    #[test]
    fn test_load_scene_creates_entities() {
        let mut mgr = SceneManager::new();
        let data = sample_scene_data();

        let id = SceneLoader::load_scene(&mut mgr, "level", data).unwrap();

        let world = mgr.get_scene(id).unwrap();
        assert!(
            world.entity_count() > 0,
            "loaded scene should have entities"
        );
    }

    // ----- unload scene ------------------------------------------------------

    #[test]
    fn test_unload_scene_removes_from_manager() {
        let mut mgr = SceneManager::new();
        let data = sample_scene_data();

        SceneLoader::load_scene(&mut mgr, "level", data).unwrap();
        SceneLoader::unload_scene(&mut mgr, "level").unwrap();

        assert!(
            mgr.get_scene_by_name("level").is_none(),
            "scene should be gone after unload"
        );
    }

    #[test]
    fn test_load_then_unload_entity_count_returns_to_zero() {
        let mut mgr = SceneManager::new();

        let data = sample_scene_data();
        let id = SceneLoader::load_scene(&mut mgr, "level", data).unwrap();

        // Verify entities were loaded
        let world = mgr.get_scene(id).unwrap();
        assert!(world.entity_count() > 0, "loaded scene should have entities");

        // Unload the scene
        SceneLoader::unload_scene(&mut mgr, "level").unwrap();

        // Verify the scene is no longer accessible (World was dropped)
        assert!(
            mgr.get_scene(id).is_none(),
            "scene should be gone after unload"
        );
    }

    // ----- load from JSON ----------------------------------------------------

    #[test]
    fn test_load_scene_from_json() {
        let mut mgr = SceneManager::new();
        let data = sample_scene_data();
        let json = scene_to_json(&data).unwrap();

        let id = SceneLoader::load_scene_from_json(&mut mgr, "json_level", &json).unwrap();

        let world = mgr.get_scene(id).unwrap();
        assert!(world.entity_count() > 0);
    }

    // ----- save to JSON ------------------------------------------------------

    #[test]
    fn test_save_scene_to_json() {
        let mut mgr = SceneManager::new();
        let data = sample_scene_data();
        let id = SceneLoader::load_scene(&mut mgr, "save_test", data).unwrap();

        let json = SceneLoader::save_scene_to_json(&mgr, id).unwrap();
        assert!(json.contains("entities"));
    }

    // ----- error cases -------------------------------------------------------

    #[test]
    fn test_unload_nonexistent_scene_errors() {
        let mut mgr = SceneManager::new();
        let result = SceneLoader::unload_scene(&mut mgr, "nope");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_duplicate_name_errors() {
        let mut mgr = SceneManager::new();
        let data = sample_scene_data();

        SceneLoader::load_scene(&mut mgr, "dup", data.clone()).unwrap();
        let result = SceneLoader::load_scene(&mut mgr, "dup", data);
        assert!(result.is_err());
    }

    // ----- deferred loading --------------------------------------------------

    #[test]
    fn test_deferred_load_creates_scene() {
        let mut mgr = SceneManager::new();
        let data = sample_scene_data();
        let json = scene_to_json(&data).unwrap();

        let deferred = DeferredSceneLoad::new();
        deferred.request_load("deferred_level".to_string(), json);

        // Give the background thread time to complete.
        std::thread::sleep(std::time::Duration::from_millis(100));

        let results = deferred.process_completed(&mut mgr);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());

        let id = results[0].as_ref().unwrap();
        let world = mgr.get_scene(*id).unwrap();
        assert!(world.entity_count() > 0);
    }

    #[test]
    fn test_deferred_load_invalid_json_returns_error() {
        let mut mgr = SceneManager::new();

        let deferred = DeferredSceneLoad::new();
        deferred.request_load("bad".to_string(), "not valid json".to_string());

        std::thread::sleep(std::time::Duration::from_millis(100));

        let results = deferred.process_completed(&mut mgr);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
    }
}
