//! [`SceneManager`] implementation -- manages multiple isolated ECS worlds.

use std::collections::HashMap;

use crate::context_registry::scene::transition::TransitionState;
use crate::core::error::GoudError;
use crate::ecs::World;

/// Unique identifier for a scene within a [`SceneManager`].
pub type SceneId = u32;

/// Name assigned to the auto-created default scene.
pub const DEFAULT_SCENE_NAME: &str = "default";

// =============================================================================
// SceneSlot
// =============================================================================

/// Internal storage for a scene slot (occupied or free).
///
/// `World` is boxed to avoid a large size difference between variants.
#[derive(Debug)]
enum SceneSlot {
    /// Slot contains a live scene.
    Occupied { name: String, world: Box<World> },
    /// Slot has been freed and can be reused.
    Free,
}

// =============================================================================
// SceneManager
// =============================================================================

/// Manages multiple isolated ECS worlds (scenes).
///
/// On construction a "default" scene is created and set as the sole active
/// scene. The default scene cannot be destroyed.
///
/// # Entity Isolation
///
/// Entities spawned in one scene are completely invisible to other scenes.
/// Each scene owns its own [`World`] with independent entity allocators,
/// component storage, and archetypes.
#[derive(Debug)]
pub struct SceneManager {
    /// Scene storage slots (indexed by `SceneId`).
    scenes: Vec<SceneSlot>,
    /// Maps scene names to their IDs for lookup-by-name.
    name_to_id: HashMap<String, SceneId>,
    /// List of currently active scene IDs (order-preserving).
    pub(crate) active_scenes: Vec<SceneId>,
    /// ID of the default scene (always 0).
    default_scene: SceneId,
    /// In-progress scene transition, if any.
    pub(crate) active_transition: Option<TransitionState>,
}

impl SceneManager {
    /// Creates a new `SceneManager` with a "default" scene already created
    /// and marked active.
    pub fn new() -> Self {
        let mut manager = Self {
            scenes: Vec::new(),
            name_to_id: HashMap::new(),
            active_scenes: Vec::new(),
            default_scene: 0,
            active_transition: None,
        };
        // Create the default scene. This cannot fail because no scenes exist yet.
        let id = manager
            .create_scene(DEFAULT_SCENE_NAME)
            .expect("creating the default scene must not fail");
        manager.default_scene = id;
        // The default scene is active from the start.
        manager.active_scenes.push(id);
        manager
    }

    // =========================================================================
    // Scene Lifecycle
    // =========================================================================

    /// Creates a new scene with the given name.
    ///
    /// Returns the [`SceneId`] on success. Returns an error if a scene with
    /// the same name already exists.
    pub fn create_scene(&mut self, name: &str) -> Result<SceneId, GoudError> {
        if self.name_to_id.contains_key(name) {
            return Err(GoudError::ResourceAlreadyExists(format!(
                "Scene '{}' already exists",
                name
            )));
        }

        // Find a free slot or append.
        let id = self.find_free_slot().unwrap_or_else(|| {
            let id = self.scenes.len() as SceneId;
            self.scenes.push(SceneSlot::Free);
            id
        });

        self.scenes[id as usize] = SceneSlot::Occupied {
            name: name.to_string(),
            world: Box::new(World::new()),
        };
        self.name_to_id.insert(name.to_string(), id);

        Ok(id)
    }

    /// Destroys a scene and frees its resources.
    ///
    /// Returns an error if the scene does not exist or if attempting to
    /// destroy the default scene.
    pub fn destroy_scene(&mut self, id: SceneId) -> Result<(), GoudError> {
        if id == self.default_scene {
            return Err(GoudError::InvalidState(
                "Cannot destroy the default scene".to_string(),
            ));
        }

        // Prevent destroying scenes involved in an active transition.
        if let Some(ref transition) = self.active_transition {
            if transition.from_scene == id || transition.to_scene == id {
                return Err(GoudError::InvalidState(format!(
                    "Cannot destroy scene {} while it is part of an active transition",
                    id
                )));
            }
        }

        let index = id as usize;
        if index >= self.scenes.len() {
            return Err(GoudError::ResourceNotFound(format!(
                "Scene id {} not found",
                id
            )));
        }

        match &self.scenes[index] {
            SceneSlot::Occupied { name, .. } => {
                self.name_to_id.remove(name);
            }
            SceneSlot::Free => {
                return Err(GoudError::ResourceNotFound(format!(
                    "Scene id {} not found",
                    id
                )));
            }
        }

        self.scenes[index] = SceneSlot::Free;
        self.active_scenes.retain(|&s| s != id);

        Ok(())
    }

    // =========================================================================
    // Scene Access
    // =========================================================================

    /// Returns a reference to the [`World`] for the given scene.
    pub fn get_scene(&self, id: SceneId) -> Option<&World> {
        self.scenes.get(id as usize).and_then(|slot| match slot {
            SceneSlot::Occupied { world, .. } => Some(world.as_ref()),
            SceneSlot::Free => None,
        })
    }

    /// Returns a mutable reference to the [`World`] for the given scene.
    pub fn get_scene_mut(&mut self, id: SceneId) -> Option<&mut World> {
        self.scenes
            .get_mut(id as usize)
            .and_then(|slot| match slot {
                SceneSlot::Occupied { world, .. } => Some(world.as_mut()),
                SceneSlot::Free => None,
            })
    }

    /// Looks up a scene by name, returning its ID if found.
    pub fn get_scene_by_name(&self, name: &str) -> Option<SceneId> {
        self.name_to_id.get(name).copied()
    }

    /// Returns the name of the scene with the given ID, if it exists.
    pub fn get_scene_name(&self, id: SceneId) -> Option<&str> {
        self.scenes.get(id as usize).and_then(|slot| match slot {
            SceneSlot::Occupied { name, .. } => Some(name.as_str()),
            SceneSlot::Free => None,
        })
    }

    // =========================================================================
    // Active Scene Management
    // =========================================================================

    /// Sets whether a scene is active.
    ///
    /// Active scenes are the ones that participate in the game loop (update,
    /// render, etc.). Returns an error if the scene does not exist.
    pub fn set_active(&mut self, id: SceneId, active: bool) -> Result<(), GoudError> {
        if !self.scene_exists(id) {
            return Err(GoudError::ResourceNotFound(format!(
                "Scene id {} not found",
                id
            )));
        }

        if active {
            if !self.active_scenes.contains(&id) {
                self.active_scenes.push(id);
            }
        } else {
            self.active_scenes.retain(|&s| s != id);
        }

        Ok(())
    }

    /// Returns `true` if the given scene is currently active.
    pub fn is_active(&self, id: SceneId) -> bool {
        self.active_scenes.contains(&id)
    }

    /// Returns a slice of all currently active scene IDs.
    pub fn active_scenes(&self) -> &[SceneId] {
        &self.active_scenes
    }

    // =========================================================================
    // Queries
    // =========================================================================

    /// Returns the number of occupied scenes (including the default).
    pub fn scene_count(&self) -> usize {
        self.scenes
            .iter()
            .filter(|s| matches!(s, SceneSlot::Occupied { .. }))
            .count()
    }

    /// Returns the ID of the default scene.
    #[inline]
    pub fn default_scene(&self) -> SceneId {
        self.default_scene
    }

    // =========================================================================
    // Internal Helpers
    // =========================================================================

    /// Finds the first free slot, if any.
    fn find_free_slot(&self) -> Option<SceneId> {
        self.scenes
            .iter()
            .position(|s| matches!(s, SceneSlot::Free))
            .map(|i| i as SceneId)
    }

    /// Returns `true` if the given scene ID refers to an occupied slot.
    pub(crate) fn scene_exists(&self, id: SceneId) -> bool {
        self.scenes
            .get(id as usize)
            .is_some_and(|s| matches!(s, SceneSlot::Occupied { .. }))
    }
}

impl Default for SceneManager {
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
    use crate::ecs::Component;

    /// Trivial component used in isolation tests.
    #[derive(Debug, Clone, PartialEq)]
    struct Health(u32);
    impl Component for Health {}

    // ----- construction ------------------------------------------------------

    #[test]
    fn test_default_scene_on_new() {
        let mgr = SceneManager::new();

        assert_eq!(mgr.scene_count(), 1);
        assert_eq!(mgr.default_scene(), 0);
        assert!(mgr.is_active(mgr.default_scene()));
        assert_eq!(mgr.active_scenes(), &[0]);

        // The default scene world is accessible.
        assert!(mgr.get_scene(mgr.default_scene()).is_some());
    }

    // ----- create / destroy --------------------------------------------------

    #[test]
    fn test_create_scene() {
        let mut mgr = SceneManager::new();
        let id = mgr.create_scene("level_2").unwrap();

        assert_ne!(id, mgr.default_scene());
        assert_eq!(mgr.scene_count(), 2);
        assert!(mgr.get_scene(id).is_some());
    }

    #[test]
    fn test_create_duplicate_fails() {
        let mut mgr = SceneManager::new();
        mgr.create_scene("level_2").unwrap();

        let result = mgr.create_scene("level_2");
        assert!(result.is_err());
    }

    #[test]
    fn test_destroy_scene() {
        let mut mgr = SceneManager::new();
        let id = mgr.create_scene("temp").unwrap();

        mgr.destroy_scene(id).unwrap();

        assert_eq!(mgr.scene_count(), 1);
        assert!(mgr.get_scene(id).is_none());
        assert!(mgr.get_scene_by_name("temp").is_none());
    }

    #[test]
    fn test_destroy_default_fails() {
        let mut mgr = SceneManager::new();
        let result = mgr.destroy_scene(mgr.default_scene());
        assert!(result.is_err());
    }

    #[test]
    fn test_destroy_nonexistent_fails() {
        let mut mgr = SceneManager::new();
        let result = mgr.destroy_scene(999);
        assert!(result.is_err());
    }

    // ----- name lookup -------------------------------------------------------

    #[test]
    fn test_get_scene_by_name() {
        let mut mgr = SceneManager::new();
        let id = mgr.create_scene("my_scene").unwrap();

        assert_eq!(mgr.get_scene_by_name("my_scene"), Some(id));
        assert_eq!(
            mgr.get_scene_by_name(DEFAULT_SCENE_NAME),
            Some(mgr.default_scene())
        );
        assert_eq!(mgr.get_scene_by_name("nope"), None);
    }

    // ----- entity isolation --------------------------------------------------

    #[test]
    fn test_entity_isolation() {
        let mut mgr = SceneManager::new();
        let scene_a = mgr.default_scene();
        let scene_b = mgr.create_scene("b").unwrap();

        // Spawn an entity in scene A.
        let entity_a = mgr.get_scene_mut(scene_a).unwrap().spawn_empty();
        mgr.get_scene_mut(scene_a)
            .unwrap()
            .insert(entity_a, Health(100));

        // Scene A has the entity; scene B does not.
        assert_eq!(mgr.get_scene(scene_a).unwrap().entity_count(), 1);
        assert_eq!(mgr.get_scene(scene_b).unwrap().entity_count(), 0);

        // The entity ID is not alive in scene B.
        assert!(!mgr.get_scene(scene_b).unwrap().is_alive(entity_a));

        // Spawn in B -- independent.
        let entity_b = mgr.get_scene_mut(scene_b).unwrap().spawn_empty();
        mgr.get_scene_mut(scene_b)
            .unwrap()
            .insert(entity_b, Health(50));

        assert_eq!(mgr.get_scene(scene_a).unwrap().entity_count(), 1);
        assert_eq!(mgr.get_scene(scene_b).unwrap().entity_count(), 1);

        // Components are independent.
        assert_eq!(
            mgr.get_scene(scene_a).unwrap().get::<Health>(entity_a),
            Some(&Health(100))
        );
        assert_eq!(
            mgr.get_scene(scene_b).unwrap().get::<Health>(entity_b),
            Some(&Health(50))
        );
    }

    // ----- active scene management -------------------------------------------

    #[test]
    fn test_multiple_active_scenes() {
        let mut mgr = SceneManager::new();
        let b = mgr.create_scene("b").unwrap();
        let c = mgr.create_scene("c").unwrap();

        mgr.set_active(b, true).unwrap();
        mgr.set_active(c, true).unwrap();

        assert_eq!(mgr.active_scenes().len(), 3);
        assert!(mgr.is_active(mgr.default_scene()));
        assert!(mgr.is_active(b));
        assert!(mgr.is_active(c));
    }

    #[test]
    fn test_set_inactive() {
        let mut mgr = SceneManager::new();
        let b = mgr.create_scene("b").unwrap();
        mgr.set_active(b, true).unwrap();
        assert!(mgr.is_active(b));

        mgr.set_active(b, false).unwrap();
        assert!(!mgr.is_active(b));
    }

    #[test]
    fn test_set_active_nonexistent_fails() {
        let mut mgr = SceneManager::new();
        let result = mgr.set_active(999, true);
        assert!(result.is_err());
    }

    #[test]
    fn test_set_active_idempotent() {
        let mut mgr = SceneManager::new();
        let b = mgr.create_scene("b").unwrap();

        mgr.set_active(b, true).unwrap();
        mgr.set_active(b, true).unwrap();

        // Should only appear once.
        assert_eq!(mgr.active_scenes().iter().filter(|&&s| s == b).count(), 1);
    }

    // ----- slot reuse --------------------------------------------------------

    #[test]
    fn test_slot_reuse_after_destroy() {
        let mut mgr = SceneManager::new();
        let a = mgr.create_scene("a").unwrap();
        mgr.destroy_scene(a).unwrap();

        // Creating a new scene should reuse the freed slot.
        let b = mgr.create_scene("b").unwrap();
        assert_eq!(b, a);
    }

    // ----- destroy removes from active list ----------------------------------

    #[test]
    fn test_destroy_removes_from_active() {
        let mut mgr = SceneManager::new();
        let b = mgr.create_scene("b").unwrap();
        mgr.set_active(b, true).unwrap();
        assert!(mgr.is_active(b));

        mgr.destroy_scene(b).unwrap();
        assert!(!mgr.is_active(b));
    }
}

#[cfg(test)]
#[path = "transition_tests.rs"]
mod transition_tests;
