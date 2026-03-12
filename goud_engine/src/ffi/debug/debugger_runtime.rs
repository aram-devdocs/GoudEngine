use crate::core::error::{set_last_error, GoudError};
use crate::core::{debugger, debugger::CapabilityStateV1};
use crate::ffi::context::get_context_registry;
use crate::ffi::context::{GoudContextId, GOUD_INVALID_CONTEXT_ID};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
/// FFI-safe memory stats for a single debugger memory category.
pub struct GoudMemoryCategoryStats {
    /// Current bytes tracked for the category.
    pub current_bytes: u64,
    /// Peak bytes observed for the category.
    pub peak_bytes: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
/// FFI-safe aggregate debugger memory summary for one context.
pub struct GoudMemorySummary {
    /// Rendering subsystem memory totals.
    pub rendering: GoudMemoryCategoryStats,
    /// Asset subsystem memory totals.
    pub assets: GoudMemoryCategoryStats,
    /// ECS subsystem memory totals.
    pub ecs: GoudMemoryCategoryStats,
    /// UI subsystem memory totals.
    pub ui: GoudMemoryCategoryStats,
    /// Audio subsystem memory totals.
    pub audio: GoudMemoryCategoryStats,
    /// Network subsystem memory totals.
    pub network: GoudMemoryCategoryStats,
    /// Debugger subsystem memory totals.
    pub debugger: GoudMemoryCategoryStats,
    /// Catch-all memory totals for uncategorized engine allocations.
    pub other: GoudMemoryCategoryStats,
    /// Sum of all tracked current bytes.
    pub total_current_bytes: u64,
    /// Sum of all tracked peak bytes.
    pub total_peak_bytes: u64,
}

impl From<debugger::MemoryCategoryStatsV1> for GoudMemoryCategoryStats {
    fn from(value: debugger::MemoryCategoryStatsV1) -> Self {
        Self {
            current_bytes: value.current_bytes,
            peak_bytes: value.peak_bytes,
        }
    }
}

impl From<debugger::MemorySummaryV1> for GoudMemorySummary {
    fn from(value: debugger::MemorySummaryV1) -> Self {
        Self {
            rendering: value.rendering.into(),
            assets: value.assets.into(),
            ecs: value.ecs.into(),
            ui: value.ui.into(),
            audio: value.audio.into(),
            network: value.network.into(),
            debugger: value.debugger.into(),
            other: value.other.into(),
            total_current_bytes: value.total_current_bytes,
            total_peak_bytes: value.total_peak_bytes,
        }
    }
}

pub(crate) fn refresh_debugger_snapshot(context_id: GoudContextId) -> Result<(), GoudError> {
    let route_id = debugger::route_for_context(context_id).ok_or(GoudError::InvalidContext)?;
    let selected_entity = debugger::snapshot_for_route(&route_id)
        .and_then(|snapshot| snapshot.selection.entity_id);

    let registry = get_context_registry()
        .lock()
        .map_err(|_| GoudError::InternalError("Failed to lock context registry".to_string()))?;
    let context = registry.get(context_id).ok_or(GoudError::InvalidContext)?;
    let scene_manager = context.scene_manager();
    let active_scene_name = scene_manager
        .get_scene_name(context.current_scene())
        .unwrap_or("default")
        .to_string();

    let mut entities = Vec::new();
    let mut entity_count = 0usize;
    let mut ecs_bytes = 0u64;
    for scene_id in scene_manager.active_scenes() {
        let Some(world) = scene_manager.get_scene(*scene_id) else {
            continue;
        };
        entity_count = entity_count.saturating_add(world.entity_count());
        let scene_name = scene_manager
            .get_scene_name(*scene_id)
            .unwrap_or("unknown")
            .to_string();
        let scene_entities = crate::context_registry::scene::collect_debugger_entities(
            world,
            scene_name,
            selected_entity,
        );
        ecs_bytes = ecs_bytes.saturating_add(
            serde_json::to_vec(&scene_entities)
                .map(|bytes| bytes.len() as u64)
                .unwrap_or_default(),
        );
        entities.extend(scene_entities);
    }

    let _ = debugger::with_snapshot_mut(&route_id, |snapshot| {
        snapshot.scene.active_scene = active_scene_name.clone();
        snapshot.scene.entity_count = entity_count as u32;
        snapshot.selection.scene_id = active_scene_name;
        snapshot.entities = entities;
    });
    let _ = debugger::update_memory_category_for_context(context_id, "ecs", ecs_bytes);
    let _ =
        debugger::set_service_state_for_context(context_id, "debugger", CapabilityStateV1::Ready, None);
    Ok(())
}

pub(crate) unsafe fn write_string_result(value: &str, buf: *mut u8, buf_len: usize) -> i32 {
    if buf.is_null() || buf_len == 0 {
        return i32::try_from(value.len().saturating_add(1))
            .map(|len| -len)
            .unwrap_or(i32::MIN);
    }

    let bytes = value.as_bytes();
    let copy_len = bytes.len().min(buf_len - 1);
    // SAFETY: Caller guarantees buf is valid for buf_len bytes.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, copy_len);
        *buf.add(copy_len) = 0;
    }
    copy_len as i32
}

/// Writes the latest debugger snapshot JSON into a caller-owned buffer.
///
/// # Safety
///
/// `buf` must be valid for `buf_len` bytes, or null to query required size.
#[no_mangle]
pub unsafe extern "C" fn goud_debugger_get_snapshot_json(
    context_id: GoudContextId,
    buf: *mut u8,
    buf_len: usize,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    if let Err(err) = refresh_debugger_snapshot(context_id) {
        set_last_error(err);
        return -1;
    }

    let json = match debugger::snapshot_json_for_context(context_id) {
        Some(json) => json,
        None => {
            set_last_error(GoudError::InvalidContext);
            return -1;
        }
    };

    // SAFETY: Caller either provided a valid buffer or requested the required size with null/zero.
    unsafe { write_string_result(&json, buf, buf_len) }
}

/// Writes the current debugger manifest JSON into a caller-owned buffer.
///
/// # Safety
///
/// `buf` must be valid for `buf_len` bytes, or null to query required size.
#[no_mangle]
pub unsafe extern "C" fn goud_debugger_get_manifest_json(buf: *mut u8, buf_len: usize) -> i32 {
    let manifest = match debugger::current_manifest().and_then(|manifest| manifest.to_json().ok()) {
        Some(json) => json,
        None => return 0,
    };

    // SAFETY: Caller either provided a valid buffer or requested the required size with null/zero.
    unsafe { write_string_result(&manifest, buf, buf_len) }
}

/// Enables or disables debugger profiling for one context.
#[no_mangle]
pub extern "C" fn goud_debugger_set_profiling_enabled(
    context_id: GoudContextId,
    enabled: bool,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    if debugger::set_profiling_enabled_for_context(context_id, enabled) {
        0
    } else {
        set_last_error(GoudError::InvalidContext);
        -1
    }
}

/// Selects one entity for expanded inspector output.
#[no_mangle]
pub extern "C" fn goud_debugger_set_selected_entity(
    context_id: GoudContextId,
    entity_id: u64,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    if debugger::set_selected_entity_for_context(context_id, Some(entity_id)) {
        0
    } else {
        set_last_error(GoudError::InvalidContext);
        -1
    }
}

/// Clears the currently selected entity for one route.
#[no_mangle]
pub extern "C" fn goud_debugger_clear_selected_entity(context_id: GoudContextId) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }

    if debugger::set_selected_entity_for_context(context_id, None) {
        0
    } else {
        set_last_error(GoudError::InvalidContext);
        -1
    }
}

/// Returns the debugger-owned memory summary for one route.
///
/// # Safety
///
/// `out_summary` must point to writable storage for one [`GoudMemorySummary`].
#[no_mangle]
pub unsafe extern "C" fn goud_debugger_get_memory_summary(
    context_id: GoudContextId,
    out_summary: *mut GoudMemorySummary,
) -> i32 {
    if context_id == GOUD_INVALID_CONTEXT_ID {
        set_last_error(GoudError::InvalidContext);
        return -1;
    }
    if out_summary.is_null() {
        set_last_error(GoudError::InvalidState(
            "out_summary pointer is null".to_string(),
        ));
        return -2;
    }

    if let Err(err) = refresh_debugger_snapshot(context_id) {
        set_last_error(err);
        return -1;
    }

    let Some(summary) = debugger::get_memory_summary_for_context(context_id) else {
        set_last_error(GoudError::InvalidContext);
        return -1;
    };

    // SAFETY: `out_summary` is non-null and points to writable storage for one summary struct.
    unsafe {
        *out_summary = summary.into();
    }
    0
}
