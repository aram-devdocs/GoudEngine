//! Component type registry for the FFI layer.
//!
//! Tracks registered component types (size, alignment) so that the FFI layer
//! can validate caller-provided data before touching raw memory.

use std::collections::HashMap;
use std::sync::Mutex;

// ============================================================================
// Type Registry
// ============================================================================

/// Information about a registered component type.
#[derive(Debug, Clone)]
pub(super) struct ComponentTypeInfo {
    /// Size of the component in bytes.
    pub(super) size: usize,
    /// Alignment of the component in bytes.
    pub(super) align: usize,
}

/// Global registry mapping type IDs to component information.
///
/// This is used to validate component operations at the FFI boundary.
/// Types must be registered before they can be used.
static COMPONENT_TYPE_REGISTRY: Mutex<Option<HashMap<u64, ComponentTypeInfo>>> = Mutex::new(None);

/// Gets or initializes the component type registry.
pub(super) fn get_type_registry(
) -> Option<std::sync::MutexGuard<'static, Option<HashMap<u64, ComponentTypeInfo>>>> {
    COMPONENT_TYPE_REGISTRY.lock().ok()
}

/// Registers a component type with the given information.
///
/// Returns true if the type was newly registered, false if it already existed.
pub(super) fn register_component_type_internal(
    type_id_hash: u64,
    size: usize,
    align: usize,
) -> bool {
    let mut registry = match get_type_registry() {
        Some(r) => r,
        None => return false,
    };
    let map = registry.get_or_insert_with(HashMap::new);

    use std::collections::hash_map::Entry;
    match map.entry(type_id_hash) {
        Entry::Occupied(_) => false,
        Entry::Vacant(e) => {
            e.insert(ComponentTypeInfo { size, align });
            true
        }
    }
}

/// Looks up component type information by type ID hash.
pub(super) fn get_component_type_info(type_id_hash: u64) -> Option<ComponentTypeInfo> {
    let registry = get_type_registry()?;
    registry.as_ref()?.get(&type_id_hash).cloned()
}
