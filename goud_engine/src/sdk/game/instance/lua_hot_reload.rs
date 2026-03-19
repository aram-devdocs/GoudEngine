//! Lua script hot-reload watcher.
//!
//! Watches a directory for `.lua` file changes and reports them so the
//! runtime can re-execute updated scripts.

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};

use crate::core::error::{GoudError, GoudResult};

/// Default debounce interval for Lua script changes.
const DEFAULT_DEBOUNCE_MS: u64 = 200;

/// Watches a directory tree for `.lua` file modifications and provides a
/// non-blocking poll interface to retrieve changed paths.
pub(crate) struct LuaScriptWatcher {
    /// File system watcher (kept alive to receive events).
    _watcher: RecommendedWatcher,

    /// Channel receiver for file system events.
    receiver: Receiver<notify::Result<Event>>,

    /// Debounce tracking: path -> last event time.
    debounce_map: HashMap<PathBuf, Instant>,

    /// Debounce duration.
    debounce_duration: Duration,
}

impl LuaScriptWatcher {
    /// Creates a new watcher that recursively monitors `dir` for `.lua` changes.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying file-system watcher cannot be created
    /// or if the directory cannot be watched.
    pub(crate) fn new(dir: &Path) -> GoudResult<Self> {
        let (sender, receiver) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = sender.send(res);
            },
            Config::default(),
        )
        .map_err(|e| GoudError::InitializationFailed(format!("Lua watcher init failed: {e}")))?;

        watcher.watch(dir, RecursiveMode::Recursive).map_err(|e| {
            GoudError::InitializationFailed(format!("Lua watcher watch failed: {e}"))
        })?;

        log::info!("Lua hot-reload watcher started on {:?}", dir);

        Ok(Self {
            _watcher: watcher,
            receiver,
            debounce_map: HashMap::new(),
            debounce_duration: Duration::from_millis(DEFAULT_DEBOUNCE_MS),
        })
    }

    /// Returns paths of `.lua` files that have changed since the last poll.
    ///
    /// This call is non-blocking: it drains the internal event channel and
    /// applies debounce filtering before returning.
    pub(crate) fn poll_changes(&mut self) -> Vec<PathBuf> {
        let now = Instant::now();
        let mut changed = Vec::new();

        while let Ok(event_result) = self.receiver.try_recv() {
            let event = match event_result {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Only care about modifications and creations.
            match &event.kind {
                EventKind::Modify(_) | EventKind::Create(_) => {}
                _ => continue,
            }

            for path in &event.paths {
                // Extension filter: only `.lua` files.
                let is_lua = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e == "lua")
                    .unwrap_or(false);
                if !is_lua {
                    continue;
                }

                // Ignore hidden and temp files.
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.')
                        || name.ends_with('~')
                        || name.ends_with(".tmp")
                        || name.ends_with(".swp")
                    {
                        continue;
                    }
                }

                // Debounce.
                if let Some(last) = self.debounce_map.get(path) {
                    if now.duration_since(*last) < self.debounce_duration {
                        continue;
                    }
                }
                self.debounce_map.insert(path.clone(), now);
                changed.push(path.clone());
            }
        }

        // Periodic cleanup of stale debounce entries.
        if self.debounce_map.len() > 500 {
            self.debounce_map
                .retain(|_, time| now.duration_since(*time) < Duration::from_secs(10));
        }

        changed
    }
}

impl std::fmt::Debug for LuaScriptWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LuaScriptWatcher")
            .field("debounce_ms", &self.debounce_duration.as_millis())
            .field("debounce_entries", &self.debounce_map.len())
            .finish()
    }
}
