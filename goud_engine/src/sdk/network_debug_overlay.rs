//! Runtime state and formatting helpers for the native network debug overlay.

/// Network metrics displayed by the overlay.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct NetworkOverlayMetrics {
    /// Latest round-trip-time sample in milliseconds.
    pub rtt_ms: f32,
    /// Rolling send bandwidth in bytes per second.
    pub send_bandwidth_bytes_per_sec: f32,
    /// Rolling receive bandwidth in bytes per second.
    pub receive_bandwidth_bytes_per_sec: f32,
    /// Rolling packet-loss percentage.
    pub packet_loss_percent: f32,
    /// Rolling RTT jitter in milliseconds.
    pub jitter_ms: f32,
}

/// Per-context network overlay state.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct NetworkOverlayState {
    visible: bool,
    active_handle: Option<i64>,
}

impl NetworkOverlayState {
    /// Returns whether the network overlay is currently visible.
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Toggles overlay visibility.
    #[inline]
    pub fn toggle_visibility(&mut self) {
        self.visible = !self.visible;
    }

    /// Sets overlay visibility explicitly.
    #[inline]
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    /// Returns the active network handle for this context.
    #[inline]
    pub fn active_handle(&self) -> Option<i64> {
        self.active_handle
    }

    /// Sets the active network handle for this context.
    #[inline]
    pub fn set_active_handle(&mut self, handle: Option<i64>) {
        self.active_handle = handle;
    }
}

/// Formats overlay lines for the built-in debug text renderer.
pub fn format_overlay_lines(handle: i64, metrics: NetworkOverlayMetrics) -> [String; 6] {
    [
        format!("H: {}", handle),
        format!("RTT: {:.1} MS", metrics.rtt_ms),
        format!("SND: {:.1} BPS", metrics.send_bandwidth_bytes_per_sec),
        format!("RCV: {:.1} BPS", metrics.receive_bandwidth_bytes_per_sec),
        format!("LOS: {:.1}%", metrics.packet_loss_percent),
        format!("JIT: {:.1} MS", metrics.jitter_ms),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_state_toggle_and_handle_tracking() {
        let mut state = NetworkOverlayState::default();
        assert!(!state.is_visible());
        assert_eq!(state.active_handle(), None);

        state.toggle_visibility();
        assert!(state.is_visible());

        state.set_active_handle(Some(42));
        assert_eq!(state.active_handle(), Some(42));

        state.set_active_handle(None);
        assert_eq!(state.active_handle(), None);
    }

    #[test]
    fn test_overlay_line_formatting_contains_required_metrics() {
        let lines = format_overlay_lines(
            9,
            NetworkOverlayMetrics {
                rtt_ms: 12.4,
                send_bandwidth_bytes_per_sec: 1200.0,
                receive_bandwidth_bytes_per_sec: 987.6,
                packet_loss_percent: 1.5,
                jitter_ms: 0.7,
            },
        );

        assert!(lines[0].contains("H: 9"));
        assert!(lines[1].contains("RTT"));
        assert!(lines[2].contains("SND"));
        assert!(lines[3].contains("RCV"));
        assert!(lines[4].contains("LOS"));
        assert!(lines[5].contains("JIT"));
    }
}
