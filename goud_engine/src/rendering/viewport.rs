//! Viewport sizing policy for 2D and 3D rendering.

/// Locks the viewport to a specific aspect ratio.
///
/// When active, the viewport will use [`ViewportScaleMode::Letterbox`]
/// behaviour with the locked ratio, adding bars as needed to preserve the
/// target aspect ratio regardless of the actual window dimensions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum AspectRatioLock {
    /// No aspect ratio lock — the viewport follows the framebuffer dimensions.
    #[default]
    Free = 0,
    /// Lock to 4:3 (1.333...).
    Ratio4x3 = 1,
    /// Lock to 16:9 (1.777...).
    Ratio16x9 = 2,
    /// Lock to 16:10 (1.6).
    Ratio16x10 = 3,
}

impl AspectRatioLock {
    /// Converts an FFI/backend code into an aspect ratio lock.
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Free),
            1 => Some(Self::Ratio4x3),
            2 => Some(Self::Ratio16x9),
            3 => Some(Self::Ratio16x10),
            _ => None,
        }
    }

    /// Returns the aspect ratio as a float, or `None` for [`Free`](Self::Free).
    pub fn ratio(&self) -> Option<f32> {
        match self {
            Self::Free => None,
            Self::Ratio4x3 => Some(4.0 / 3.0),
            Self::Ratio16x9 => Some(16.0 / 9.0),
            Self::Ratio16x10 => Some(16.0 / 10.0),
        }
    }
}

/// How logical content maps to the physical framebuffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewportScaleMode {
    /// Scale the logical viewport to fill the framebuffer.
    #[default]
    Stretch,
    /// Preserve aspect ratio and add bars when needed.
    Letterbox,
}

// Re-export `SafeAreaInsets` from the platform layer so that higher-level
// consumers can access it without reaching into `libs::platform` directly.
pub use crate::libs::platform::SafeAreaInsets;

/// Resolved viewport rectangle plus its logical render size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderViewport {
    /// Viewport X origin in framebuffer pixels.
    pub x: i32,
    /// Viewport Y origin in framebuffer pixels.
    pub y: i32,
    /// Viewport width in framebuffer pixels.
    pub width: u32,
    /// Viewport height in framebuffer pixels.
    pub height: u32,
    /// Logical content width used for projection/layout.
    pub logical_width: u32,
    /// Logical content height used for projection/layout.
    pub logical_height: u32,
    /// Display scale factor (DPI ratio). 1.0 = standard, 2.0 = Retina/xxhdpi.
    pub scale_factor: f32,
}

impl RenderViewport {
    /// Creates a fullscreen viewport for the given framebuffer size.
    #[must_use]
    pub fn fullscreen(framebuffer_size: (u32, u32)) -> Self {
        Self {
            x: 0,
            y: 0,
            width: framebuffer_size.0.max(1),
            height: framebuffer_size.1.max(1),
            logical_width: framebuffer_size.0.max(1),
            logical_height: framebuffer_size.1.max(1),
            scale_factor: 1.0,
        }
    }

    /// Returns the logical render size.
    #[must_use]
    pub fn logical_size(self) -> (u32, u32) {
        (self.logical_width, self.logical_height)
    }
}

impl Default for RenderViewport {
    fn default() -> Self {
        Self::fullscreen((800, 600))
    }
}

/// Resolves the active viewport rectangle for the framebuffer and policy.
#[must_use]
pub fn compute_render_viewport(
    framebuffer_size: (u32, u32),
    logical_size: (u32, u32),
    mode: ViewportScaleMode,
) -> RenderViewport {
    let framebuffer_width = framebuffer_size.0.max(1);
    let framebuffer_height = framebuffer_size.1.max(1);
    let logical_width = logical_size.0.max(1);
    let logical_height = logical_size.1.max(1);

    match mode {
        ViewportScaleMode::Stretch => RenderViewport {
            x: 0,
            y: 0,
            width: framebuffer_width,
            height: framebuffer_height,
            logical_width,
            logical_height,
            scale_factor: 1.0,
        },
        ViewportScaleMode::Letterbox => {
            let framebuffer_aspect = framebuffer_width as f32 / framebuffer_height as f32;
            let logical_aspect = logical_width as f32 / logical_height as f32;

            let (width, height) = if framebuffer_aspect > logical_aspect {
                let height = framebuffer_height;
                let width = ((height as f32 * logical_aspect).round() as u32).max(1);
                (width.min(framebuffer_width), height)
            } else {
                let width = framebuffer_width;
                let height = ((width as f32 / logical_aspect).round() as u32).max(1);
                (width, height.min(framebuffer_height))
            };

            RenderViewport {
                x: ((framebuffer_width - width) / 2) as i32,
                y: ((framebuffer_height - height) / 2) as i32,
                width,
                height,
                logical_width,
                logical_height,
                scale_factor: 1.0,
            }
        }
    }
}

/// Resolves the viewport rectangle, applying an aspect ratio lock if active.
///
/// When a lock is active, this forces [`ViewportScaleMode::Letterbox`] with a
/// logical size derived from the locked ratio. When the lock is
/// [`AspectRatioLock::Free`], this delegates to [`compute_render_viewport`].
#[must_use]
pub fn compute_render_viewport_with_aspect_lock(
    framebuffer_size: (u32, u32),
    logical_size: (u32, u32),
    mode: ViewportScaleMode,
    lock: AspectRatioLock,
) -> RenderViewport {
    match lock.ratio() {
        Some(target_aspect) => {
            let logical_height = logical_size.1.max(1);
            let locked_width = (logical_height as f32 * target_aspect).round() as u32;
            let locked_logical = (locked_width.max(1), logical_height);
            compute_render_viewport(
                framebuffer_size,
                locked_logical,
                ViewportScaleMode::Letterbox,
            )
        }
        None => compute_render_viewport(framebuffer_size, logical_size, mode),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compute_render_viewport, compute_render_viewport_with_aspect_lock, AspectRatioLock,
        RenderViewport, SafeAreaInsets, ViewportScaleMode,
    };

    #[test]
    fn fullscreen_viewport_matches_framebuffer() {
        assert_eq!(
            RenderViewport::fullscreen((320, 180)),
            RenderViewport {
                x: 0,
                y: 0,
                width: 320,
                height: 180,
                logical_width: 320,
                logical_height: 180,
                scale_factor: 1.0,
            }
        );
    }

    #[test]
    fn stretch_mode_fills_framebuffer() {
        let viewport =
            compute_render_viewport((1920, 1080), (320, 180), ViewportScaleMode::Stretch);
        assert_eq!(viewport.x, 0);
        assert_eq!(viewport.y, 0);
        assert_eq!(viewport.width, 1920);
        assert_eq!(viewport.height, 1080);
        assert_eq!(viewport.logical_size(), (320, 180));
    }

    #[test]
    fn letterbox_mode_preserves_aspect_ratio() {
        let viewport =
            compute_render_viewport((1920, 1200), (320, 180), ViewportScaleMode::Letterbox);
        assert_eq!(viewport.width, 1920);
        assert_eq!(viewport.height, 1080);
        assert_eq!(viewport.x, 0);
        assert_eq!(viewport.y, 60);
        assert_eq!(viewport.logical_size(), (320, 180));
    }

    #[test]
    fn letterbox_mode_handles_tall_framebuffer() {
        let viewport =
            compute_render_viewport((900, 1600), (320, 180), ViewportScaleMode::Letterbox);
        assert_eq!(viewport.width, 900);
        assert_eq!(viewport.height, 506);
        assert_eq!(viewport.x, 0);
        assert_eq!(viewport.y, 547);
    }

    // =========================================================================
    // AspectRatioLock tests
    // =========================================================================

    #[test]
    fn aspect_ratio_lock_from_u32_round_trips() {
        assert_eq!(AspectRatioLock::from_u32(0), Some(AspectRatioLock::Free));
        assert_eq!(
            AspectRatioLock::from_u32(1),
            Some(AspectRatioLock::Ratio4x3)
        );
        assert_eq!(
            AspectRatioLock::from_u32(2),
            Some(AspectRatioLock::Ratio16x9)
        );
        assert_eq!(
            AspectRatioLock::from_u32(3),
            Some(AspectRatioLock::Ratio16x10)
        );
        assert_eq!(AspectRatioLock::from_u32(4), None);
    }

    #[test]
    fn aspect_ratio_lock_free_delegates_to_normal() {
        let normal = compute_render_viewport((1920, 1080), (800, 600), ViewportScaleMode::Stretch);
        let locked = compute_render_viewport_with_aspect_lock(
            (1920, 1080),
            (800, 600),
            ViewportScaleMode::Stretch,
            AspectRatioLock::Free,
        );
        assert_eq!(normal, locked);
    }

    #[test]
    fn aspect_ratio_lock_4x3_on_16x9_framebuffer() {
        // 16:9 framebuffer with 4:3 lock should produce pillarboxing.
        let viewport = compute_render_viewport_with_aspect_lock(
            (1920, 1080),
            (800, 600),
            ViewportScaleMode::Stretch,
            AspectRatioLock::Ratio4x3,
        );
        // 4:3 on a 16:9 frame => pillarbox (bars on left and right).
        let expected_aspect = 4.0_f32 / 3.0;
        let actual_aspect = viewport.width as f32 / viewport.height as f32;
        assert!((actual_aspect - expected_aspect).abs() < 0.02);
        assert!(viewport.x > 0, "pillarbox should offset X");
        assert_eq!(viewport.height, 1080);
    }

    #[test]
    fn aspect_ratio_lock_16x9_on_4x3_framebuffer() {
        // 4:3 framebuffer with 16:9 lock should produce letterboxing.
        let viewport = compute_render_viewport_with_aspect_lock(
            (1024, 768),
            (800, 600),
            ViewportScaleMode::Stretch,
            AspectRatioLock::Ratio16x9,
        );
        let expected_aspect = 16.0_f32 / 9.0;
        let actual_aspect = viewport.width as f32 / viewport.height as f32;
        assert!((actual_aspect - expected_aspect).abs() < 0.02);
        assert!(viewport.y > 0, "letterbox should offset Y");
        assert_eq!(viewport.width, 1024);
    }

    // =========================================================================
    // Scale factor and SafeAreaInsets tests
    // =========================================================================

    #[test]
    fn render_viewport_default_scale_factor() {
        let vp = RenderViewport::default();
        assert!((vp.scale_factor - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn safe_area_insets_default_is_zero() {
        let insets = SafeAreaInsets::default();
        assert!((insets.top).abs() < f32::EPSILON);
        assert!((insets.bottom).abs() < f32::EPSILON);
        assert!((insets.left).abs() < f32::EPSILON);
        assert!((insets.right).abs() < f32::EPSILON);
    }

    #[test]
    fn safe_area_insets_repr_c_fields() {
        let insets = SafeAreaInsets {
            top: 47.0,
            bottom: 34.0,
            left: 0.0,
            right: 0.0,
        };
        assert!((insets.top - 47.0).abs() < f32::EPSILON);
        assert!((insets.bottom - 34.0).abs() < f32::EPSILON);
    }

    // =========================================================================
    // Mobile screen size viewport tests
    // =========================================================================

    /// iPhone 15: 393x852 logical, 1179x2556 physical, @3x scale
    #[test]
    fn mobile_iphone15_stretch_viewport() {
        let logical = (393u32, 852u32);
        let physical = (1179u32, 2556u32);
        let vp = compute_render_viewport(physical, logical, ViewportScaleMode::Stretch);
        assert_eq!(vp.width, physical.0);
        assert_eq!(vp.height, physical.1);
        assert_eq!(vp.logical_width, logical.0);
        assert_eq!(vp.logical_height, logical.1);
        assert_eq!(vp.x, 0);
        assert_eq!(vp.y, 0);
    }

    /// iPhone 15 in letterbox with a 16:9 design resolution (portrait).
    /// The device aspect (~0.461) is narrower than 9:16 (~0.5625), so the
    /// viewport should be pillarboxed (bars left/right) or letterboxed
    /// depending on orientation.
    #[test]
    fn mobile_iphone15_letterbox_16x9_portrait() {
        let physical = (1179u32, 2556u32);
        // 9:16 portrait design
        let logical = (540u32, 960u32);
        let vp = compute_render_viewport(physical, logical, ViewportScaleMode::Letterbox);
        let design_aspect = 540.0_f32 / 960.0;
        let actual_aspect = vp.width as f32 / vp.height as f32;
        assert!(
            (actual_aspect - design_aspect).abs() < 0.02,
            "aspect ratio should be preserved"
        );
        // Either x or y should be non-zero for letterbox/pillarbox.
        assert!(
            vp.x > 0 || vp.y > 0,
            "letterbox should offset on at least one axis"
        );
    }

    /// Pixel 7: 412x915 logical, 1080x2400 physical, @2.625x scale
    #[test]
    fn mobile_pixel7_stretch_viewport() {
        let logical = (412u32, 915u32);
        let physical = (1080u32, 2400u32);
        let vp = compute_render_viewport(physical, logical, ViewportScaleMode::Stretch);
        assert_eq!(vp.width, physical.0);
        assert_eq!(vp.height, physical.1);
        assert_eq!(vp.logical_size(), logical);
    }

    /// Pixel 7 letterbox with a 16:9 portrait design.
    #[test]
    fn mobile_pixel7_letterbox_portrait() {
        let physical = (1080u32, 2400u32);
        let logical = (540u32, 960u32);
        let vp = compute_render_viewport(physical, logical, ViewportScaleMode::Letterbox);
        let design_aspect = 540.0_f32 / 960.0;
        let actual_aspect = vp.width as f32 / vp.height as f32;
        assert!(
            (actual_aspect - design_aspect).abs() < 0.02,
            "aspect ratio should be preserved for Pixel 7"
        );
    }

    /// iPad Pro 12.9": 1024x1366 logical, 2048x2732 physical, @2x scale
    #[test]
    fn mobile_ipad_pro_stretch_viewport() {
        let logical = (1024u32, 1366u32);
        let physical = (2048u32, 2732u32);
        let vp = compute_render_viewport(physical, logical, ViewportScaleMode::Stretch);
        assert_eq!(vp.width, physical.0);
        assert_eq!(vp.height, physical.1);
        assert_eq!(vp.logical_size(), logical);
    }

    /// iPad Pro 12.9" letterbox with a 16:9 landscape design on a 3:4 screen.
    #[test]
    fn mobile_ipad_pro_letterbox_landscape() {
        let physical = (2732u32, 2048u32);
        let logical = (1920u32, 1080u32);
        let vp = compute_render_viewport(physical, logical, ViewportScaleMode::Letterbox);
        let design_aspect = 1920.0_f32 / 1080.0;
        let actual_aspect = vp.width as f32 / vp.height as f32;
        assert!(
            (actual_aspect - design_aspect).abs() < 0.02,
            "aspect ratio should be preserved for iPad Pro landscape"
        );
        // 16:9 is wider than 4:3, so we expect letterbox bars top/bottom.
        assert!(vp.y > 0, "should have letterbox bars on Y axis");
    }

    /// Verify that the scale factor propagates through the struct correctly.
    #[test]
    fn scale_factor_propagation() {
        let mut vp = compute_render_viewport((1179, 2556), (393, 852), ViewportScaleMode::Stretch);
        // compute_render_viewport returns 1.0 by default; the caller sets it.
        assert!((vp.scale_factor - 1.0).abs() < f32::EPSILON);
        vp.scale_factor = 3.0;
        assert!((vp.scale_factor - 3.0).abs() < f32::EPSILON);
    }
}
