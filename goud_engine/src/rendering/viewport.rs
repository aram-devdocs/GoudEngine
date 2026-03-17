//! Viewport sizing policy for 2D and 3D rendering.

/// How logical content maps to the physical framebuffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewportScaleMode {
    /// Scale the logical viewport to fill the framebuffer.
    #[default]
    Stretch,
    /// Preserve aspect ratio and add bars when needed.
    Letterbox,
}

/// Resolved viewport rectangle plus its logical render size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{compute_render_viewport, RenderViewport, ViewportScaleMode};

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
}
