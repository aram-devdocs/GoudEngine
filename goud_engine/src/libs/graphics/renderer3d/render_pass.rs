//! Post-processing pipeline types for the 3D renderer.

// ============================================================================
// Post-Processing Pipeline
// ============================================================================

/// A single render pass in the post-processing pipeline.
pub trait RenderPass: std::fmt::Debug + Send {
    /// The display name of this pass.
    fn name(&self) -> &str;

    /// Whether the pass is currently active.
    fn enabled(&self) -> bool;

    /// Process an RGBA8 image buffer in-place.
    fn process(&self, width: u32, height: u32, data: &mut [u8]);
}

/// A pipeline of chained post-processing passes.
#[derive(Debug)]
pub struct PostProcessPipeline {
    passes: Vec<Box<dyn RenderPass>>,
}

impl Default for PostProcessPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl PostProcessPipeline {
    /// Create a new empty post-processing pipeline.
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// Add a render pass to the end of the pipeline.
    pub fn add_pass(&mut self, pass: Box<dyn RenderPass>) {
        self.passes.push(pass);
    }

    /// Remove a pass by index. Returns `true` if the index was valid.
    pub fn remove_pass(&mut self, index: usize) -> bool {
        if index < self.passes.len() {
            self.passes.remove(index);
            true
        } else {
            false
        }
    }

    /// Process an image through all enabled passes in order.
    pub fn run(&self, width: u32, height: u32, data: &mut [u8]) {
        for pass in &self.passes {
            if pass.enabled() {
                pass.process(width, height, data);
            }
        }
    }

    /// Return the number of passes in the pipeline.
    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }
}

/// Bloom post-processing pass.
#[derive(Debug, Clone)]
pub struct BloomPass {
    /// Brightness threshold for bloom extraction.
    pub threshold: f32,
    /// Bloom intensity multiplier.
    pub intensity: f32,
    /// Whether this pass is enabled.
    pub enabled: bool,
}

impl Default for BloomPass {
    fn default() -> Self {
        Self {
            threshold: 0.8,
            intensity: 1.0,
            enabled: true,
        }
    }
}

impl RenderPass for BloomPass {
    fn name(&self) -> &str {
        "Bloom"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn process(&self, width: u32, height: u32, data: &mut [u8]) {
        let pixel_count = (width * height) as usize;
        if data.len() < pixel_count * 4 {
            return;
        }
        // Extract bright pixels and additively blend them back.
        let mut bright = vec![0u8; data.len()];
        for i in 0..pixel_count {
            let idx = i * 4;
            let luminance = 0.2126 * (data[idx] as f32)
                + 0.7152 * (data[idx + 1] as f32)
                + 0.0722 * (data[idx + 2] as f32);
            if luminance / 255.0 > self.threshold {
                bright[idx] = data[idx];
                bright[idx + 1] = data[idx + 1];
                bright[idx + 2] = data[idx + 2];
            }
        }
        // Simple box blur on the bright pixels (3x3).
        let blurred = box_blur_rgba(&bright, width, height);
        for i in 0..pixel_count {
            let idx = i * 4;
            for c in 0..3 {
                let combined = data[idx + c] as f32 + blurred[idx + c] as f32 * self.intensity;
                data[idx + c] = (combined.min(255.0)) as u8;
            }
        }
    }
}

/// Gaussian blur post-processing pass.
#[derive(Debug, Clone)]
pub struct GaussianBlurPass {
    /// Blur radius in pixels.
    pub radius: u32,
    /// Whether this pass is enabled.
    pub enabled: bool,
}

impl Default for GaussianBlurPass {
    fn default() -> Self {
        Self {
            radius: 2,
            enabled: true,
        }
    }
}

impl RenderPass for GaussianBlurPass {
    fn name(&self) -> &str {
        "GaussianBlur"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn process(&self, width: u32, height: u32, data: &mut [u8]) {
        let result = box_blur_rgba(data, width, height);
        let len = (width * height * 4) as usize;
        if result.len() >= len && data.len() >= len {
            data[..len].copy_from_slice(&result[..len]);
        }
    }
}

/// Color grading / tone mapping post-processing pass.
#[derive(Debug, Clone)]
pub struct ColorGradePass {
    /// Exposure adjustment.
    pub exposure: f32,
    /// Contrast adjustment (1.0 = no change).
    pub contrast: f32,
    /// Saturation adjustment (1.0 = no change).
    pub saturation: f32,
    /// Whether this pass is enabled.
    pub enabled: bool,
}

impl Default for ColorGradePass {
    fn default() -> Self {
        Self {
            exposure: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            enabled: true,
        }
    }
}

impl RenderPass for ColorGradePass {
    fn name(&self) -> &str {
        "ColorGrade"
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn process(&self, width: u32, height: u32, data: &mut [u8]) {
        let pixel_count = (width * height) as usize;
        if data.len() < pixel_count * 4 {
            return;
        }
        for i in 0..pixel_count {
            let idx = i * 4;
            for c in 0..3 {
                let mut v = data[idx + c] as f32 / 255.0;
                // Exposure
                v *= self.exposure;
                // Contrast
                v = ((v - 0.5) * self.contrast) + 0.5;
                // Saturation (simple luminance-based)
                let lum = 0.2126 * (data[idx] as f32 / 255.0)
                    + 0.7152 * (data[idx + 1] as f32 / 255.0)
                    + 0.0722 * (data[idx + 2] as f32 / 255.0);
                v = lum + (v - lum) * self.saturation;
                data[idx + c] = (v.clamp(0.0, 1.0) * 255.0) as u8;
            }
        }
    }
}

/// Simple 3x3 box blur for RGBA8 image data.
fn box_blur_rgba(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let mut output = data.to_vec();
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            for c in 0..3 {
                let mut sum: u32 = 0;
                for dy in 0..3u32 {
                    for dx in 0..3u32 {
                        let sx = x - 1 + dx as usize;
                        let sy = y - 1 + dy as usize;
                        sum += data[(sy * w + sx) * 4 + c] as u32;
                    }
                }
                output[(y * w + x) * 4 + c] = (sum / 9) as u8;
            }
        }
    }
    output
}
