//! FPS stats debug overlay with rolling window.
//!
//! Provides [`DebugOverlay`] for tracking frame timing statistics using a
//! rolling window of recent frame times. Stats are cached and recomputed
//! at a configurable interval to minimize per-frame overhead.

use std::collections::VecDeque;

/// Maximum number of frame times stored in the rolling window.
const DEFAULT_WINDOW_CAPACITY: usize = 120;

// =============================================================================
// FPS Statistics
// =============================================================================

/// Frame timing statistics computed from the rolling window.
///
/// All FPS values are in frames-per-second; `frame_time_ms` is the most
/// recent frame time in milliseconds.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct FpsStats {
    /// Current (most recent) FPS.
    pub current_fps: f32,
    /// Minimum FPS observed in the rolling window.
    pub min_fps: f32,
    /// Maximum FPS observed in the rolling window.
    pub max_fps: f32,
    /// Average FPS across the rolling window.
    pub avg_fps: f32,
    /// Most recent frame time in milliseconds.
    pub frame_time_ms: f32,
}

// =============================================================================
// Render Metrics
// =============================================================================

/// Per-frame render metrics for draw call counting, culling stats, and timing.
///
/// Populated each frame from `SpriteBatch`, `TextBatch`, and `UiRenderSystem`
/// statistics. All timing values are in milliseconds.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct RenderMetrics {
    /// Total draw calls across all render subsystems.
    pub draw_call_count: u32,
    /// Total sprites submitted before culling.
    pub sprites_submitted: u32,
    /// Sprites that passed culling and were drawn.
    pub sprites_drawn: u32,
    /// Sprites rejected by frustum culling.
    pub sprites_culled: u32,
    /// Number of sprite batches submitted.
    pub batches_submitted: u32,
    /// Average sprites per batch (batch efficiency).
    pub avg_sprites_per_batch: f32,
    /// Time spent rendering sprites (ms).
    pub sprite_render_ms: f32,
    /// Time spent rendering text (ms).
    pub text_render_ms: f32,
    /// Time spent rendering UI (ms).
    pub ui_render_ms: f32,
    /// Total render phase time (ms). Currently only includes UI render time;
    /// sprite and text phase timing will be added in a future update.
    pub total_render_ms: f32,
    /// Draw calls from text rendering.
    pub text_draw_calls: u32,
    /// Glyphs rendered this frame.
    pub text_glyph_count: u32,
    /// Draw calls from UI rendering.
    pub ui_draw_calls: u32,
}

impl From<crate::core::debugger::RenderMetricsV1> for RenderMetrics {
    fn from(rm: crate::core::debugger::RenderMetricsV1) -> Self {
        Self {
            draw_call_count: rm.draw_call_count,
            sprites_submitted: rm.sprites_submitted,
            sprites_drawn: rm.sprites_drawn,
            sprites_culled: rm.sprites_culled,
            batches_submitted: rm.batches_submitted,
            avg_sprites_per_batch: rm.avg_sprites_per_batch,
            sprite_render_ms: rm.sprite_render_ms,
            text_render_ms: rm.text_render_ms,
            ui_render_ms: rm.ui_render_ms,
            total_render_ms: rm.total_render_ms,
            text_draw_calls: rm.text_draw_calls,
            text_glyph_count: rm.text_glyph_count,
            ui_draw_calls: rm.ui_draw_calls,
        }
    }
}

// =============================================================================
// Overlay Corner
// =============================================================================

/// Screen corner where the overlay is displayed.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(C)]
pub enum OverlayCorner {
    /// Top-left corner of the screen.
    #[default]
    TopLeft = 0,
    /// Top-right corner of the screen.
    TopRight = 1,
    /// Bottom-left corner of the screen.
    BottomLeft = 2,
    /// Bottom-right corner of the screen.
    BottomRight = 3,
}

// =============================================================================
// Debug Overlay
// =============================================================================

/// Tracks frame timing and computes FPS statistics over a rolling window.
///
/// Stats are cached and only recomputed every `update_interval` seconds
/// to avoid impacting frame timing.
#[derive(Debug, Clone)]
pub struct DebugOverlay {
    /// Whether the overlay is enabled.
    enabled: bool,
    /// Which corner to display the overlay in.
    corner: OverlayCorner,
    /// How often (in seconds) to recompute statistics.
    update_interval: f32,
    /// Rolling window of recent frame times (in seconds).
    frame_times: VecDeque<f32>,
    /// Cached statistics (recomputed every `update_interval`).
    cached_stats: FpsStats,
    /// Time accumulated since the last stats recomputation.
    time_since_update: f32,
}

impl DebugOverlay {
    /// Creates a new overlay with the given stats update interval.
    ///
    /// The rolling window holds up to 120 frame-time samples.
    pub fn new(update_interval: f32) -> Self {
        Self {
            enabled: false,
            corner: OverlayCorner::default(),
            update_interval,
            frame_times: VecDeque::with_capacity(DEFAULT_WINDOW_CAPACITY),
            cached_stats: FpsStats::default(),
            time_since_update: 0.0,
        }
    }

    /// Records a frame and recomputes stats if the update interval has elapsed.
    pub fn update(&mut self, delta_time: f32) {
        // Always record frame times so stats are ready when queried.
        if self.frame_times.len() >= DEFAULT_WINDOW_CAPACITY {
            self.frame_times.pop_front();
        }
        self.frame_times.push_back(delta_time);

        self.time_since_update += delta_time;
        if self.time_since_update >= self.update_interval {
            self.time_since_update = 0.0;
            self.recompute_stats(delta_time);
        }
    }

    /// Returns the most recently cached FPS statistics.
    #[inline]
    pub fn stats(&self) -> FpsStats {
        self.cached_stats
    }

    /// Enables or disables the overlay.
    #[inline]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns whether the overlay is enabled.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets the display corner.
    #[inline]
    pub fn set_corner(&mut self, corner: OverlayCorner) {
        self.corner = corner;
    }

    /// Returns the current display corner.
    #[inline]
    pub fn corner(&self) -> OverlayCorner {
        self.corner
    }

    /// Sets the stats update interval in seconds.
    #[inline]
    pub fn set_update_interval(&mut self, interval: f32) {
        self.update_interval = interval;
    }

    // -------------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------------

    fn recompute_stats(&mut self, current_delta: f32) {
        if self.frame_times.is_empty() {
            self.cached_stats = FpsStats::default();
            return;
        }

        let mut sum: f32 = 0.0;
        let mut min_dt = f32::MAX;
        let mut max_dt: f32 = 0.0;

        for &dt in &self.frame_times {
            sum += dt;
            if dt < min_dt {
                min_dt = dt;
            }
            if dt > max_dt {
                max_dt = dt;
            }
        }

        let count = self.frame_times.len() as f32;
        let avg_dt = sum / count;

        self.cached_stats = FpsStats {
            current_fps: if current_delta > 0.0 {
                1.0 / current_delta
            } else {
                0.0
            },
            min_fps: if max_dt > 0.0 { 1.0 / max_dt } else { 0.0 },
            max_fps: if min_dt > 0.0 { 1.0 / min_dt } else { 0.0 },
            avg_fps: if avg_dt > 0.0 { 1.0 / avg_dt } else { 0.0 },
            frame_time_ms: current_delta * 1000.0,
        };
    }
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self::new(0.5)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_overlay_has_default_stats() {
        let overlay = DebugOverlay::new(0.5);
        let stats = overlay.stats();
        assert_eq!(stats.current_fps, 0.0);
        assert_eq!(stats.avg_fps, 0.0);
        assert_eq!(stats.min_fps, 0.0);
        assert_eq!(stats.max_fps, 0.0);
        assert_eq!(stats.frame_time_ms, 0.0);
    }

    #[test]
    fn test_overlay_disabled_by_default() {
        let overlay = DebugOverlay::new(0.5);
        assert!(!overlay.is_enabled());
    }

    #[test]
    fn test_set_enabled() {
        let mut overlay = DebugOverlay::new(0.5);
        overlay.set_enabled(true);
        assert!(overlay.is_enabled());
        overlay.set_enabled(false);
        assert!(!overlay.is_enabled());
    }

    #[test]
    fn test_set_corner() {
        let mut overlay = DebugOverlay::new(0.5);
        assert_eq!(overlay.corner(), OverlayCorner::TopLeft);
        overlay.set_corner(OverlayCorner::BottomRight);
        assert_eq!(overlay.corner(), OverlayCorner::BottomRight);
    }

    #[test]
    fn test_stats_computed_after_interval() {
        let mut overlay = DebugOverlay::new(0.5);

        // Feed frames that do NOT reach the 0.5s interval
        for _ in 0..10 {
            overlay.update(0.016); // 10 * 0.016 = 0.16s < 0.5s
        }
        // Stats should still be zero (interval not reached)
        let stats = overlay.stats();
        assert_eq!(stats.current_fps, 0.0);

        // Now push enough to cross the interval
        // We need 0.5 - 0.16 = 0.34 more seconds
        overlay.update(0.34);
        let stats = overlay.stats();
        // current_fps should be 1/0.34
        assert!((stats.current_fps - 1.0 / 0.34).abs() < 0.01);
        assert!(stats.avg_fps > 0.0);
    }

    #[test]
    fn test_stats_with_known_frame_times() {
        // Use a very short interval so stats recompute immediately
        let mut overlay = DebugOverlay::new(0.0);

        // Push three known frame times: 10ms, 20ms, 30ms
        overlay.update(0.010);
        overlay.update(0.020);
        overlay.update(0.030);

        let stats = overlay.stats();

        // current_fps = 1/0.030 ~= 33.33
        assert!((stats.current_fps - 33.333).abs() < 0.1);

        // avg_dt = (0.01+0.02+0.03)/3 = 0.02 => avg_fps = 50
        assert!((stats.avg_fps - 50.0).abs() < 0.1);

        // min_fps = 1/max_dt = 1/0.03 ~= 33.33
        assert!((stats.min_fps - 33.333).abs() < 0.1);

        // max_fps = 1/min_dt = 1/0.01 = 100
        assert!((stats.max_fps - 100.0).abs() < 0.1);

        // frame_time_ms = 0.030 * 1000 = 30
        assert!((stats.frame_time_ms - 30.0).abs() < 0.1);
    }

    #[test]
    fn test_rolling_window_eviction() {
        let mut overlay = DebugOverlay::new(0.0);

        // Fill beyond capacity (120)
        for _ in 0..150 {
            overlay.update(0.016);
        }

        // Internal window should be capped at 120
        assert_eq!(overlay.frame_times.len(), DEFAULT_WINDOW_CAPACITY);
    }

    #[test]
    fn test_single_frame() {
        let mut overlay = DebugOverlay::new(0.0);
        overlay.update(0.016);

        let stats = overlay.stats();
        assert!((stats.current_fps - 62.5).abs() < 0.1);
        assert!((stats.avg_fps - 62.5).abs() < 0.1);
        assert!((stats.frame_time_ms - 16.0).abs() < 0.1);
    }

    #[test]
    fn test_zero_delta_time() {
        let mut overlay = DebugOverlay::new(0.0);
        overlay.update(0.0);

        let stats = overlay.stats();
        // Zero delta => 0 fps, 0 frame_time_ms
        assert_eq!(stats.current_fps, 0.0);
        assert_eq!(stats.frame_time_ms, 0.0);
    }

    #[test]
    fn test_update_interval_respects_timing() {
        let mut overlay = DebugOverlay::new(1.0);

        // 60 frames at 16ms = 0.96s (under 1.0s interval)
        for _ in 0..60 {
            overlay.update(0.016);
        }
        assert_eq!(overlay.stats().current_fps, 0.0); // not yet recomputed

        // One more frame pushes over 1.0s
        overlay.update(0.016); // total ~0.976s... still under
                               // Need a bit more
        overlay.update(0.04); // total now > 1.0s
        assert!(overlay.stats().current_fps > 0.0);
    }

    #[test]
    fn test_set_update_interval() {
        let mut overlay = DebugOverlay::new(1.0);
        overlay.set_update_interval(0.1);

        // Now a shorter window should trigger recomputation
        for _ in 0..10 {
            overlay.update(0.016);
        }
        // 10 * 0.016 = 0.16s > 0.1s interval
        assert!(overlay.stats().current_fps > 0.0);
    }

    #[test]
    fn test_default_overlay() {
        let overlay = DebugOverlay::default();
        assert!(!overlay.is_enabled());
        assert_eq!(overlay.corner(), OverlayCorner::TopLeft);
    }

    #[test]
    fn test_overlay_corner_default() {
        assert_eq!(OverlayCorner::default(), OverlayCorner::TopLeft);
    }

    #[test]
    fn test_fps_stats_default() {
        let stats = FpsStats::default();
        assert_eq!(stats.current_fps, 0.0);
        assert_eq!(stats.min_fps, 0.0);
        assert_eq!(stats.max_fps, 0.0);
        assert_eq!(stats.avg_fps, 0.0);
        assert_eq!(stats.frame_time_ms, 0.0);
    }

    #[test]
    fn test_render_metrics_default_is_zeroed() {
        let m = RenderMetrics::default();
        assert_eq!(m.draw_call_count, 0);
        assert_eq!(m.sprites_submitted, 0);
        assert_eq!(m.sprites_drawn, 0);
        assert_eq!(m.sprites_culled, 0);
        assert_eq!(m.batches_submitted, 0);
        assert_eq!(m.avg_sprites_per_batch, 0.0);
        assert_eq!(m.sprite_render_ms, 0.0);
        assert_eq!(m.text_render_ms, 0.0);
        assert_eq!(m.ui_render_ms, 0.0);
        assert_eq!(m.total_render_ms, 0.0);
        assert_eq!(m.text_draw_calls, 0);
        assert_eq!(m.text_glyph_count, 0);
        assert_eq!(m.ui_draw_calls, 0);
    }

    #[test]
    fn test_render_metrics_submitted_equals_drawn_plus_culled() {
        let m = RenderMetrics {
            sprites_submitted: 100,
            sprites_drawn: 75,
            sprites_culled: 25,
            ..Default::default()
        };
        assert_eq!(m.sprites_submitted, m.sprites_drawn + m.sprites_culled);
    }
}
