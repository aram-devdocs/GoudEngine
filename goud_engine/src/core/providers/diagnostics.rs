//! Diagnostics types for provider subsystems.
//!
//! Each provider trait includes a required diagnostics method returning one of
//! these versioned structs. Because the method has no default implementation,
//! any new provider **must** implement it or the build fails -- guaranteeing
//! diagnostics coverage across all backends.

use serde::{Deserialize, Serialize};

// =============================================================================
// Per-provider diagnostics (V1)
// =============================================================================

/// Render subsystem diagnostics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenderDiagnosticsV1 {
    /// Number of draw calls submitted this frame.
    pub draw_calls: u32,
    /// Number of triangles rendered this frame.
    pub triangles: u32,
    /// Number of texture bind operations this frame.
    pub texture_binds: u32,
    /// Number of shader bind operations this frame.
    pub shader_binds: u32,
    /// Number of textures currently allocated.
    pub active_textures: u32,
    /// Number of shader programs currently allocated.
    pub active_shaders: u32,
}

/// 2D physics subsystem diagnostics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PhysicsDiagnosticsV1 {
    /// Number of active rigid bodies.
    pub body_count: u32,
    /// Number of active colliders.
    pub collider_count: u32,
    /// Number of active joints.
    pub joint_count: u32,
    /// Number of active contact pairs.
    pub contact_pair_count: u32,
    /// Current gravity vector `[x, y]`.
    pub gravity: [f32; 2],
    /// Physics timestep in seconds.
    pub timestep: f32,
}

/// 3D physics subsystem diagnostics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Physics3DDiagnosticsV1 {
    /// Number of active rigid bodies.
    pub body_count: u32,
    /// Number of active colliders.
    pub collider_count: u32,
    /// Number of active joints.
    pub joint_count: u32,
    /// Number of active contact pairs.
    pub contact_pair_count: u32,
    /// Current gravity vector `[x, y, z]`.
    pub gravity: [f32; 3],
    /// Physics timestep in seconds.
    pub timestep: f32,
}

/// Audio subsystem diagnostics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioDiagnosticsV1 {
    /// Number of currently active playback instances.
    pub active_playbacks: u32,
    /// Current master volume (0.0 to 1.0).
    pub master_volume: f32,
}

/// Input subsystem diagnostics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputDiagnosticsV1 {
    /// List of currently pressed key codes (as strings).
    pub pressed_keys: Vec<String>,
    /// Current mouse position `[x, y]` in window coordinates.
    pub mouse_position: [f32; 2],
    /// List of currently pressed mouse buttons (as strings).
    pub mouse_buttons_pressed: Vec<String>,
    /// Number of connected gamepads.
    pub connected_gamepads: u32,
    /// Scroll wheel delta since last frame `[dx, dy]`.
    pub scroll_delta: [f32; 2],
}

/// Network subsystem diagnostics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkDiagnosticsV1 {
    /// Total bytes sent since provider creation.
    pub bytes_sent: u64,
    /// Total bytes received since provider creation.
    pub bytes_received: u64,
    /// Total packets sent since provider creation.
    pub packets_sent: u64,
    /// Total packets received since provider creation.
    pub packets_received: u64,
    /// Current round-trip time estimate in milliseconds.
    pub rtt_ms: f32,
    /// Number of currently active connections.
    pub active_connections: u32,
}

/// Window subsystem diagnostics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowDiagnosticsV1 {
    /// Window size in screen coordinates `[width, height]`.
    pub size: [u32; 2],
    /// Framebuffer size in pixels `[width, height]` (may differ on HiDPI).
    pub framebuffer_size: [u32; 2],
}

/// Trait for non-provider subsystems that want to contribute diagnostics.
pub trait DiagnosticsSource: Send + Sync {
    /// Returns a unique key identifying this diagnostics source.
    fn diagnostics_key(&self) -> &str;

    /// Collect current diagnostics as a type-erased JSON value.
    fn collect_diagnostics(&self) -> serde_json::Value;
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Default value tests -- all diagnostics structs derive Default
    // =========================================================================

    #[test]
    fn test_render_diagnostics_default_is_zeroed() {
        let d = RenderDiagnosticsV1::default();
        assert_eq!(d.draw_calls, 0);
        assert_eq!(d.triangles, 0);
        assert_eq!(d.texture_binds, 0);
        assert_eq!(d.shader_binds, 0);
        assert_eq!(d.active_textures, 0);
        assert_eq!(d.active_shaders, 0);
    }

    #[test]
    fn test_physics_diagnostics_default_is_zeroed() {
        let d = PhysicsDiagnosticsV1::default();
        assert_eq!(d.body_count, 0);
        assert_eq!(d.collider_count, 0);
        assert_eq!(d.joint_count, 0);
        assert_eq!(d.contact_pair_count, 0);
        assert_eq!(d.gravity, [0.0, 0.0]);
        assert_eq!(d.timestep, 0.0);
    }

    #[test]
    fn test_physics3d_diagnostics_default_is_zeroed() {
        let d = Physics3DDiagnosticsV1::default();
        assert_eq!(d.body_count, 0);
        assert_eq!(d.collider_count, 0);
        assert_eq!(d.joint_count, 0);
        assert_eq!(d.contact_pair_count, 0);
        assert_eq!(d.gravity, [0.0, 0.0, 0.0]);
        assert_eq!(d.timestep, 0.0);
    }

    #[test]
    fn test_audio_diagnostics_default_is_zeroed() {
        let d = AudioDiagnosticsV1::default();
        assert_eq!(d.active_playbacks, 0);
        assert_eq!(d.master_volume, 0.0);
    }

    #[test]
    fn test_input_diagnostics_default_is_zeroed() {
        let d = InputDiagnosticsV1::default();
        assert!(d.pressed_keys.is_empty());
        assert_eq!(d.mouse_position, [0.0, 0.0]);
        assert!(d.mouse_buttons_pressed.is_empty());
        assert_eq!(d.connected_gamepads, 0);
        assert_eq!(d.scroll_delta, [0.0, 0.0]);
    }

    #[test]
    fn test_network_diagnostics_default_is_zeroed() {
        let d = NetworkDiagnosticsV1::default();
        assert_eq!(d.bytes_sent, 0);
        assert_eq!(d.bytes_received, 0);
        assert_eq!(d.packets_sent, 0);
        assert_eq!(d.packets_received, 0);
        assert_eq!(d.rtt_ms, 0.0);
        assert_eq!(d.active_connections, 0);
    }

    #[test]
    fn test_window_diagnostics_default_is_zeroed() {
        let d = WindowDiagnosticsV1::default();
        assert_eq!(d.size, [0, 0]);
        assert_eq!(d.framebuffer_size, [0, 0]);
    }

    // =========================================================================
    // Serialization round-trip tests
    // =========================================================================

    #[test]
    fn test_render_diagnostics_serializes_to_valid_json() {
        let d = RenderDiagnosticsV1 {
            draw_calls: 150,
            triangles: 50_000,
            texture_binds: 12,
            shader_binds: 4,
            active_textures: 8,
            active_shaders: 3,
        };
        let json = serde_json::to_value(&d).expect("should serialize");
        assert_eq!(json["draw_calls"], 150);
        assert_eq!(json["triangles"], 50_000);
        let roundtrip: RenderDiagnosticsV1 =
            serde_json::from_value(json).expect("should deserialize");
        assert_eq!(roundtrip.draw_calls, d.draw_calls);
        assert_eq!(roundtrip.triangles, d.triangles);
    }

    #[test]
    fn test_input_diagnostics_serializes_to_valid_json() {
        let d = InputDiagnosticsV1 {
            pressed_keys: vec!["W".to_string(), "Space".to_string()],
            mouse_position: [320.0, 240.0],
            mouse_buttons_pressed: vec!["Left".to_string()],
            connected_gamepads: 1,
            scroll_delta: [0.0, -3.0],
        };
        let json = serde_json::to_value(&d).expect("should serialize");
        let keys = json["pressed_keys"]
            .as_array()
            .expect("pressed_keys should be array");
        assert_eq!(keys.len(), 2);
        let roundtrip: InputDiagnosticsV1 =
            serde_json::from_value(json).expect("should deserialize");
        assert_eq!(roundtrip.pressed_keys, d.pressed_keys);
    }

    // =========================================================================
    // DiagnosticsSource trait test
    // =========================================================================

    struct FakeSource;

    impl DiagnosticsSource for FakeSource {
        fn diagnostics_key(&self) -> &str {
            "fake"
        }
        fn collect_diagnostics(&self) -> serde_json::Value {
            serde_json::json!({ "status": "ok", "count": 42 })
        }
    }

    #[test]
    fn test_diagnostics_source_trait_returns_expected_values() {
        let source = FakeSource;
        assert_eq!(source.diagnostics_key(), "fake");
        let diag = source.collect_diagnostics();
        assert_eq!(diag["status"], "ok");
        assert_eq!(diag["count"], 42);
    }
}
