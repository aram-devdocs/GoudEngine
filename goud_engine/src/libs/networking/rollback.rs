//! Core rollback netcode engine.
//!
//! Implements GGPO-style rollback with local input prediction, state snapshots
//! stored in a ring buffer, mismatch-triggered resimulation, and hash-based
//! desync detection.

use std::collections::{HashMap, VecDeque};

/// Identifier for a player in the rollback session.
pub type PlayerId = u8;

// ---------------------------------------------------------------------------
// Traits
// ---------------------------------------------------------------------------

/// Trait that game state must implement to participate in rollback.
///
/// The state must be cloneable (for snapshotting) and hashable (for desync
/// detection). `advance` applies one tick of game logic given the collected
/// inputs for every player.
pub trait GameState: Clone {
    /// Advance the game state by one frame using the provided inputs.
    fn advance(&mut self, inputs: &HashMap<PlayerId, Vec<u8>>);

    /// Produce a deterministic hash of the current state for desync detection.
    fn state_hash(&self) -> u64;
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the rollback system.
#[derive(Debug, Clone)]
pub struct RollbackConfig {
    /// Maximum number of frames the system will roll back to correct a
    /// misprediction. Older snapshots are discarded.
    pub max_rollback_frames: usize,
    /// Number of frames of local input delay before application. Reduces the
    /// frequency of rollbacks at the cost of perceived latency.
    pub input_delay_frames: usize,
    /// When true, `check_desync` compares hashes between peers.
    pub desync_detection: bool,
}

impl Default for RollbackConfig {
    fn default() -> Self {
        Self {
            max_rollback_frames: 7,
            input_delay_frames: 2,
            desync_detection: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

/// A snapshot of the game state at a particular frame, used for rollback.
#[derive(Clone)]
struct StateSnapshot<S: GameState> {
    frame: u64,
    state: S,
    /// Inputs that were used to produce this frame's state (applied to go from
    /// frame-1 to frame). Stored for diagnostic and replay purposes.
    inputs: HashMap<PlayerId, Vec<u8>>,
}

/// Result returned by `check_desync`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DesyncResult {
    /// Hashes match -- no desync.
    InSync,
    /// Hashes differ at the given frame.
    Desync {
        /// The frame at which the desync was detected.
        frame: u64,
    },
    /// The requested frame has already been evicted from the snapshot buffer.
    FrameNotAvailable {
        /// The frame that was requested but is no longer available.
        frame: u64,
    },
}

// ---------------------------------------------------------------------------
// RollbackNetcode
// ---------------------------------------------------------------------------

/// Core rollback netcode engine parameterized over a user-provided game state.
pub struct RollbackNetcode<S: GameState> {
    config: RollbackConfig,

    /// Current simulation frame (the next frame to be produced).
    frame: u64,

    /// The latest frame for which all players' inputs have been confirmed.
    confirmed_frame: u64,

    /// The local player id.
    local_player: PlayerId,

    /// Ring buffer of state snapshots (newest at back).
    snapshots: VecDeque<StateSnapshot<S>>,

    /// Confirmed (ground-truth) input history per player, indexed by frame.
    /// Each deque is ordered by ascending frame number.
    inputs: HashMap<PlayerId, VecDeque<(u64, Vec<u8>)>>,

    /// Predicted inputs that were *used* for remote players before their real
    /// input arrived. Keyed by (player, frame).
    predictions: HashMap<(PlayerId, u64), Vec<u8>>,

    /// The live game state -- always at `frame - 1` (i.e., the result of
    /// advancing through frame `frame - 1`).
    state: S,

    /// Set of players participating in the session (including local).
    players: Vec<PlayerId>,

    /// Cache of state hashes indexed by frame. The hash at frame F is the
    /// hash of the state AFTER frame F has been processed.
    frame_hashes: VecDeque<(u64, u64)>,

    /// Flag: true when a rollback is pending because we received a remote
    /// input that differs from our prediction.
    needs_rollback: bool,

    /// The earliest frame that needs correction due to a misprediction.
    /// Used to determine how far back to rollback.
    earliest_mismatch_frame: Option<u64>,
}

impl<S: GameState> RollbackNetcode<S> {
    /// Create a new rollback session.
    ///
    /// `initial_state` is frame 0. `players` must include `local_player`.
    pub fn new(
        config: RollbackConfig,
        local_player: PlayerId,
        players: Vec<PlayerId>,
        initial_state: S,
    ) -> Self {
        let snapshots = VecDeque::with_capacity(config.max_rollback_frames + 1);

        let mut input_map: HashMap<PlayerId, VecDeque<(u64, Vec<u8>)>> = HashMap::new();
        for &p in &players {
            input_map.insert(p, VecDeque::new());
        }

        Self {
            config,
            frame: 0,
            confirmed_frame: 0,
            local_player,
            snapshots,
            inputs: input_map,
            predictions: HashMap::new(),
            state: initial_state,
            players,
            frame_hashes: VecDeque::new(),
            needs_rollback: false,
            earliest_mismatch_frame: None,
        }
    }

    // -- Public API ---------------------------------------------------------

    /// Advance the simulation by one frame.
    ///
    /// `local_input` is the raw input bytes for the local player this frame.
    /// Remote players' inputs are predicted (repeat last known) until confirmed
    /// via `receive_remote_input`.
    pub fn advance_frame(&mut self, local_input: Vec<u8>) {
        let target_frame = self.frame;

        // Record local input as confirmed.
        self.record_confirmed_input(self.local_player, target_frame, local_input.clone());

        // Save a snapshot of the state BEFORE this frame is applied.
        // This way, to rollback to frame F we restore snapshot F and
        // replay from F onward.
        self.snapshots.push_back(StateSnapshot {
            frame: target_frame,
            state: self.state.clone(),
            inputs: HashMap::new(), // filled below after we know the inputs
        });

        // Build the combined input map for this frame.
        let mut frame_inputs: HashMap<PlayerId, Vec<u8>> = HashMap::new();
        for &player in &self.players {
            if player == self.local_player {
                frame_inputs.insert(player, local_input.clone());
            } else {
                // Predict: use last confirmed input for this player, or empty.
                let predicted = self.last_confirmed_input(player);
                self.predictions
                    .insert((player, target_frame), predicted.clone());
                frame_inputs.insert(player, predicted);
            }
        }

        // Store the inputs in the snapshot we just saved.
        if let Some(snap) = self.snapshots.back_mut() {
            snap.inputs = frame_inputs.clone();
        }

        // Advance state.
        self.state.advance(&frame_inputs);

        // Record post-frame hash for desync detection.
        self.frame_hashes
            .push_back((target_frame, self.state.state_hash()));

        self.frame += 1;

        // Trim ring buffer and hash cache.
        while self.snapshots.len() > self.config.max_rollback_frames + 1 {
            self.snapshots.pop_front();
        }
        while self.frame_hashes.len() > self.config.max_rollback_frames + 1 {
            self.frame_hashes.pop_front();
        }

        // Recompute confirmed frame.
        self.recompute_confirmed_frame();
    }

    /// Receive a confirmed input from a remote player.
    ///
    /// If the input differs from what was predicted for that frame, the
    /// `needs_rollback` flag is set. The caller should then invoke
    /// `rollback_and_resimulate` before the next `advance_frame`.
    pub fn receive_remote_input(&mut self, player: PlayerId, frame: u64, input: Vec<u8>) {
        // Check if this differs from our prediction.
        if let Some(predicted) = self.predictions.get(&(player, frame)) {
            if *predicted != input {
                self.needs_rollback = true;
                // Track the earliest mismatch so rollback goes far enough.
                match self.earliest_mismatch_frame {
                    Some(existing) if existing <= frame => {}
                    _ => {
                        self.earliest_mismatch_frame = Some(frame);
                    }
                }
            }
        }

        self.record_confirmed_input(player, frame, input);
        self.recompute_confirmed_frame();
    }

    /// Returns true if a rollback is needed due to an input mismatch.
    pub fn should_rollback(&self) -> bool {
        self.needs_rollback
    }

    /// Roll back to the last confirmed state and resimulate forward to the
    /// current frame using confirmed inputs (and fresh predictions where
    /// inputs are still unconfirmed).
    ///
    /// Returns the number of frames resimulated, or 0 if no rollback was
    /// needed.
    pub fn rollback_and_resimulate(&mut self) -> usize {
        if !self.needs_rollback {
            return 0;
        }
        self.needs_rollback = false;

        // Find the snapshot at or before the earliest mismatch frame.
        let restore_frame = self.earliest_mismatch_frame.unwrap_or(self.confirmed_frame);
        self.earliest_mismatch_frame = None;
        let snapshot_idx = self
            .snapshots
            .iter()
            .rposition(|s| s.frame <= restore_frame);

        let snapshot_idx = match snapshot_idx {
            Some(idx) => idx,
            None => {
                // No usable snapshot -- cannot rollback.
                return 0;
            }
        };

        // Restore state.
        let snapshot = &self.snapshots[snapshot_idx];
        self.state = snapshot.state.clone();
        let start_frame = snapshot.frame;

        // Remove snapshots at and after the restored one (they will be
        // regenerated with corrected inputs).
        self.snapshots.truncate(snapshot_idx);

        // Clear hash cache for frames being resimulated.
        self.frame_hashes.retain(|&(f, _)| f < start_frame);

        // The restored state is the state BEFORE start_frame was processed.
        // Resimulate from start_frame onward.
        let target = self.frame;
        let players = self.players.clone();
        let mut resimulated = 0usize;
        for f in start_frame..target {
            let mut frame_inputs: HashMap<PlayerId, Vec<u8>> = HashMap::new();
            for &player in &players {
                let input = match self.confirmed_input_for(player, f) {
                    Some(confirmed) => confirmed,
                    None => {
                        // Still unconfirmed -- predict again.
                        let predicted = self.last_confirmed_input(player);
                        self.predictions.insert((player, f), predicted.clone());
                        predicted
                    }
                };
                frame_inputs.insert(player, input);
            }

            // Save pre-frame snapshot.
            self.snapshots.push_back(StateSnapshot {
                frame: f,
                state: self.state.clone(),
                inputs: frame_inputs.clone(),
            });

            self.state.advance(&frame_inputs);
            resimulated += 1;

            // Record post-frame hash.
            self.frame_hashes.push_back((f, self.state.state_hash()));
        }

        // Trim ring buffer again.
        while self.snapshots.len() > self.config.max_rollback_frames + 1 {
            self.snapshots.pop_front();
        }

        // Clear stale predictions for confirmed frames.
        self.predictions
            .retain(|&(_, f), _| f > self.confirmed_frame);

        resimulated
    }

    /// Compare a remote peer's state hash for a given frame with the local
    /// hash. The hash represents the state AFTER frame `frame` was processed.
    /// Returns a `DesyncResult`.
    pub fn check_desync(&self, remote_hash: u64, frame: u64) -> DesyncResult {
        if !self.config.desync_detection {
            return DesyncResult::InSync;
        }

        match self.frame_hashes.iter().find(|&&(f, _)| f == frame) {
            Some(&(_, local_hash)) => {
                if local_hash == remote_hash {
                    DesyncResult::InSync
                } else {
                    DesyncResult::Desync { frame }
                }
            }
            None => DesyncResult::FrameNotAvailable { frame },
        }
    }

    /// The latest frame for which all players' inputs are confirmed.
    pub fn confirmed_frame(&self) -> u64 {
        self.confirmed_frame
    }

    /// The current simulation frame (the next frame number that will be
    /// produced by `advance_frame`).
    pub fn current_frame(&self) -> u64 {
        self.frame
    }

    /// Returns a reference to the current game state.
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Returns the hash of the state AFTER the given frame was processed,
    /// if still available in the hash cache.
    pub fn state_hash_at(&self, frame: u64) -> Option<u64> {
        self.frame_hashes
            .iter()
            .find(|&&(f, _)| f == frame)
            .map(|&(_, h)| h)
    }

    // -- Internal helpers ---------------------------------------------------

    fn record_confirmed_input(&mut self, player: PlayerId, frame: u64, input: Vec<u8>) {
        let history = self.inputs.entry(player).or_default();
        // Insert in order. Most of the time this is an append at the end.
        if history.is_empty() || history.back().map(|(f, _)| *f).unwrap_or(0) < frame {
            history.push_back((frame, input));
        } else {
            // Overwrite or insert at the correct position.
            if let Some(entry) = history.iter_mut().find(|(f, _)| *f == frame) {
                entry.1 = input;
            } else {
                // Insert sorted.
                let pos = history.partition_point(|(f, _)| *f < frame);
                history.insert(pos, (frame, input));
            }
        }

        // Trim old input history beyond what snapshots could need.
        let oldest_snapshot_frame = self.snapshots.front().map(|s| s.frame).unwrap_or(0);
        while history
            .front()
            .map(|(f, _)| *f + 1 < oldest_snapshot_frame)
            .unwrap_or(false)
        {
            history.pop_front();
        }
    }

    fn confirmed_input_for(&self, player: PlayerId, frame: u64) -> Option<Vec<u8>> {
        self.inputs
            .get(&player)
            .and_then(|h| h.iter().find(|(f, _)| *f == frame).map(|(_, d)| d.clone()))
    }

    fn last_confirmed_input(&self, player: PlayerId) -> Vec<u8> {
        self.inputs
            .get(&player)
            .and_then(|h| h.back().map(|(_, d)| d.clone()))
            .unwrap_or_default()
    }

    fn recompute_confirmed_frame(&mut self) {
        // The confirmed frame is the highest frame F such that all players
        // have a confirmed input for every frame <= F.
        let mut min_confirmed = self.frame;
        for &player in &self.players {
            let max_frame = self
                .inputs
                .get(&player)
                .and_then(|h| h.back().map(|(f, _)| *f))
                .unwrap_or(0);
            if max_frame < min_confirmed {
                min_confirmed = max_frame;
            }
        }
        self.confirmed_frame = min_confirmed;
    }
}
