//! In-memory ring buffer for capturing log entries.
//!
//! [`LogCaptureSink`] stores the most recent log entries in a bounded
//! `VecDeque` and implements [`DiagnosticsSource`] for integration with the
//! debugger snapshot pipeline.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::core::providers::diagnostics::DiagnosticsSource;

const MAX_LOG_ENTRIES: usize = 500;

/// A single captured log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntryV1 {
    /// Timestamp in milliseconds since an unspecified epoch.
    pub timestamp_ms: u64,
    /// Log level (e.g. "info", "warn", "error").
    pub level: String,
    /// The log message body.
    pub message: String,
    /// The log target / module path.
    pub target: String,
}

/// Thread-safe ring buffer that stores the most recent log entries.
#[derive(Debug, Clone)]
pub struct LogCaptureSink {
    entries: Arc<Mutex<VecDeque<LogEntryV1>>>,
}

impl LogCaptureSink {
    /// Creates a new, empty log capture sink.
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_LOG_ENTRIES))),
        }
    }

    /// Pushes a log entry into the ring buffer, evicting the oldest if full.
    pub fn push(&self, entry: LogEntryV1) {
        if let Ok(mut entries) = self.entries.lock() {
            if entries.len() >= MAX_LOG_ENTRIES {
                entries.pop_front();
            }
            entries.push_back(entry);
        }
    }

    /// Returns a snapshot of all current entries.
    pub fn entries(&self) -> Vec<LogEntryV1> {
        self.entries
            .lock()
            .map(|e| e.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Returns entries since a given frame. Currently returns all entries;
    /// frame-level filtering will be added when `frame_index` is tracked
    /// per entry.
    pub fn entries_since_frame(&self, _since_frame: u64) -> Vec<LogEntryV1> {
        self.entries()
    }
}

impl Default for LogCaptureSink {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticsSource for LogCaptureSink {
    fn diagnostics_key(&self) -> &str {
        "logs"
    }

    fn collect_diagnostics(&self) -> serde_json::Value {
        let entries = self.entries();
        serde_json::json!({
            "entry_count": entries.len(),
            "entries": entries,
        })
    }
}
