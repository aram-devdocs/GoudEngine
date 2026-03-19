//! Material management methods for [`Renderer3D`].

use super::core::Renderer3D;
use super::types::Material3D;

impl Renderer3D {
    /// Create a material and return its ID.
    pub fn create_material(&mut self, material: Material3D) -> u32 {
        let id = self.next_material_id;
        self.next_material_id += 1;
        self.materials.insert(id, material);
        id
    }

    /// Update an existing material. Returns `true` if the material existed.
    pub fn update_material(&mut self, id: u32, material: Material3D) -> bool {
        use std::collections::hash_map::Entry;
        if let Entry::Occupied(mut e) = self.materials.entry(id) {
            e.insert(material);
            true
        } else {
            false
        }
    }

    /// Remove a material by ID. Returns `true` if it existed.
    pub fn remove_material(&mut self, id: u32) -> bool {
        self.materials.remove(&id).is_some()
    }

    /// Bind a material to an object. Returns `true` if the object exists.
    pub fn set_object_material(&mut self, object_id: u32, material_id: u32) -> bool {
        if self.objects.contains_key(&object_id) {
            self.object_materials.insert(object_id, material_id);
            true
        } else {
            false
        }
    }

    /// Get the material ID bound to an object, if any.
    pub fn get_object_material(&self, object_id: u32) -> Option<u32> {
        self.object_materials.get(&object_id).copied()
    }

    /// Get a reference to a material by ID.
    pub fn get_material(&self, id: u32) -> Option<&Material3D> {
        self.materials.get(&id)
    }
}
