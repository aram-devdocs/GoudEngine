//! Thread-local per-frame phase timing cache.
//!
//! Stores the latest frame phase timings so they are accessible without
//! requiring the debugger route to be active. Each phase is recorded in
//! microseconds by the wgpu backend and renderer3d during frame rendering.

use std::cell::RefCell;

/// Per-frame phase timings (all values in microseconds).
#[derive(Debug, Clone, Copy, Default)]
pub struct FramePhaseTimings {
    /// Time to acquire the next surface texture.
    pub surface_acquire_us: u64,
    /// Uniform upload and pipeline creation time.
    pub uniform_upload_us: u64,
    /// GPU render pass recording time.
    pub render_pass_us: u64,
    /// GPU command submission time.
    pub gpu_submit_us: u64,
    /// GPU readback stall time.
    pub readback_stall_us: u64,
    /// Surface present / vsync wait time.
    pub surface_present_us: u64,
    /// GPU shadow depth pass recording and execution time.
    pub shadow_pass_us: u64,
    /// Shadow map build time.
    pub shadow_build_us: u64,
    /// 3D scene render time.
    pub render3d_scene_us: u64,
    /// Animation evaluation time.
    pub anim_eval_us: u64,
    /// Bone matrix packing time.
    pub bone_pack_us: u64,
    /// Bone matrix GPU upload time.
    pub bone_upload_us: u64,
}

thread_local! {
    static FRAME_TIMINGS: RefCell<FramePhaseTimings> = RefCell::new(FramePhaseTimings::default());
}

/// Records a single phase timing value by name.
pub fn record_timing(field: &str, value: u64) {
    FRAME_TIMINGS.with(|t| {
        let mut t = t.borrow_mut();
        match field {
            "surface_acquire" => t.surface_acquire_us = value,
            "uniform_upload" => t.uniform_upload_us = value,
            "render_pass" => t.render_pass_us = value,
            "gpu_submit" => t.gpu_submit_us = value,
            "readback_stall" => t.readback_stall_us = value,
            "surface_present" => t.surface_present_us = value,
            "shadow_pass" => t.shadow_pass_us = value,
            "shadow_build" => t.shadow_build_us = value,
            "render3d_scene" => t.render3d_scene_us = value,
            "anim_eval" => t.anim_eval_us = value,
            "bone_pack" => t.bone_pack_us = value,
            "bone_upload" => t.bone_upload_us = value,
            _ => {}
        }
    });
}

/// Records a single phase timing into both the thread-local cache and the
/// debugger runtime. Replaces dual-call sites that previously called
/// `record_timing()` and `debugger::record_phase_duration()` separately.
pub fn record_phase(name: &str, us: u64) {
    record_timing(name, us);
    crate::core::debugger::record_phase_duration(name, us);
}

/// Returns the latest frame phase timings.
pub fn latest_timings() -> FramePhaseTimings {
    FRAME_TIMINGS.with(|t| *t.borrow())
}

/// Resets all phase timings to zero for the start of a new frame.
pub fn reset_timings() {
    FRAME_TIMINGS.with(|t| *t.borrow_mut() = FramePhaseTimings::default());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_read_timings() {
        reset_timings();
        record_timing("surface_acquire", 100);
        record_timing("shadow_pass", 150);
        record_timing("shadow_build", 200);
        record_timing("render3d_scene", 300);
        record_timing("uniform_upload", 400);
        record_timing("render_pass", 500);
        record_timing("gpu_submit", 600);
        record_timing("readback_stall", 700);
        record_timing("surface_present", 800);
        record_timing("anim_eval", 900);
        record_timing("bone_pack", 1000);
        record_timing("bone_upload", 1100);

        let t = latest_timings();
        assert_eq!(t.surface_acquire_us, 100);
        assert_eq!(t.shadow_pass_us, 150);
        assert_eq!(t.shadow_build_us, 200);
        assert_eq!(t.render3d_scene_us, 300);
        assert_eq!(t.uniform_upload_us, 400);
        assert_eq!(t.render_pass_us, 500);
        assert_eq!(t.gpu_submit_us, 600);
        assert_eq!(t.readback_stall_us, 700);
        assert_eq!(t.surface_present_us, 800);
        assert_eq!(t.anim_eval_us, 900);
        assert_eq!(t.bone_pack_us, 1000);
        assert_eq!(t.bone_upload_us, 1100);
    }

    #[test]
    fn reset_clears_all_timings() {
        record_timing("surface_acquire", 999);
        record_timing("render3d_scene", 888);
        reset_timings();

        let t = latest_timings();
        assert_eq!(t.surface_acquire_us, 0);
        assert_eq!(t.render3d_scene_us, 0);
    }

    #[test]
    fn unknown_field_is_ignored() {
        reset_timings();
        record_timing("nonexistent_phase", 12345);
        let t = latest_timings();
        assert_eq!(t.surface_acquire_us, 0);
        assert_eq!(t.shadow_build_us, 0);
    }

    #[test]
    fn default_timings_are_zero() {
        let t = FramePhaseTimings::default();
        assert_eq!(t.surface_acquire_us, 0);
        assert_eq!(t.shadow_pass_us, 0);
        assert_eq!(t.shadow_build_us, 0);
        assert_eq!(t.render3d_scene_us, 0);
        assert_eq!(t.uniform_upload_us, 0);
        assert_eq!(t.render_pass_us, 0);
        assert_eq!(t.gpu_submit_us, 0);
        assert_eq!(t.readback_stall_us, 0);
        assert_eq!(t.surface_present_us, 0);
    }
}
