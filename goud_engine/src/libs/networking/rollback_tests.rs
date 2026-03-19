//! Tests for the rollback netcode system.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::Instant;

use super::rollback::*;

// ---------------------------------------------------------------------------
// Test game state
// ---------------------------------------------------------------------------

/// A trivial game state: each player has a position that increments by their
/// input value each frame. Deterministic and easy to reason about.
#[derive(Clone, Debug)]
struct TestState {
    positions: HashMap<PlayerId, i64>,
}

impl TestState {
    fn new(players: &[PlayerId]) -> Self {
        let mut positions = HashMap::new();
        for &p in players {
            positions.insert(p, 0);
        }
        Self { positions }
    }
}

impl GameState for TestState {
    fn advance(&mut self, inputs: &HashMap<PlayerId, Vec<u8>>) {
        for (player, data) in inputs {
            let delta: i64 = if data.is_empty() { 0 } else { data[0] as i64 };
            *self.positions.entry(*player).or_insert(0) += delta;
        }
    }

    fn state_hash(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        let mut entries: Vec<_> = self.positions.iter().collect();
        entries.sort_by_key(|(k, _)| **k);
        for (k, v) in entries {
            k.hash(&mut hasher);
            v.hash(&mut hasher);
        }
        hasher.finish()
    }
}

fn input(val: u8) -> Vec<u8> {
    vec![val]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_advance_without_rollback() {
    let players = vec![0, 1];
    let state = TestState::new(&players);
    let config = RollbackConfig::default();
    let mut rb = RollbackNetcode::new(config, 0, players, state);

    // Advance 5 frames with local input = 1 each frame.
    // Remote player 1 has no confirmed input, so predictions use default (empty = 0).
    for _ in 0..5 {
        rb.advance_frame(input(1));
    }

    assert_eq!(rb.current_frame(), 5);
    // Local player moved 5 * 1 = 5.
    assert_eq!(*rb.state().positions.get(&0).unwrap(), 5);
    // Remote player was predicted with 0 each frame.
    assert_eq!(*rb.state().positions.get(&1).unwrap(), 0);
}

#[test]
fn test_advance_with_matching_remote_input() {
    let players = vec![0, 1];
    let state = TestState::new(&players);
    let config = RollbackConfig::default();
    let mut rb = RollbackNetcode::new(config, 0, players, state);

    // Frame 0: local=1, predict remote=0
    rb.advance_frame(input(1));

    // Remote player sends empty input for frame 0, matching the prediction
    // (default prediction is empty bytes when no prior input exists).
    rb.receive_remote_input(1, 0, Vec::new());

    // No rollback needed because prediction was correct.
    assert!(!rb.should_rollback());
}

#[test]
fn test_rollback_on_input_mismatch() {
    let players = vec![0, 1];
    let state = TestState::new(&players);
    let config = RollbackConfig::default();
    let mut rb = RollbackNetcode::new(config, 0, players, state);

    // Advance 3 frames. Remote player predicted as 0 each frame.
    for _ in 0..3 {
        rb.advance_frame(input(1));
    }

    // Remote player actually pressed 5 on frame 0.
    rb.receive_remote_input(1, 0, input(5));

    assert!(rb.should_rollback());

    let resimulated = rb.rollback_and_resimulate();
    assert!(resimulated > 0);
    assert!(!rb.should_rollback());

    // After resimulation:
    // Local player: still 3 (1+1+1)
    // Remote player: frame0=5, frame1=predicted(5, last confirmed), frame2=predicted(5)
    assert_eq!(*rb.state().positions.get(&0).unwrap(), 3);
    // Remote got 5 at frame 0, then predicted 5 for frames 1 and 2.
    assert_eq!(*rb.state().positions.get(&1).unwrap(), 15);
}

#[test]
fn test_rollback_restores_correct_state() {
    let players = vec![0, 1];
    let state = TestState::new(&players);
    let config = RollbackConfig {
        max_rollback_frames: 10,
        ..Default::default()
    };
    let mut rb = RollbackNetcode::new(config, 0, players, state);

    // Advance 5 frames, local=2 each frame, remote predicted as 0.
    for _ in 0..5 {
        rb.advance_frame(input(2));
    }

    // Confirm remote inputs: frame 0=1, frame 1=1, frame 2=1.
    for f in 0..3 {
        rb.receive_remote_input(1, f, input(1));
    }

    assert!(rb.should_rollback());
    rb.rollback_and_resimulate();

    // Local: 5 * 2 = 10
    assert_eq!(*rb.state().positions.get(&0).unwrap(), 10);
    // Remote: frames 0-2 = 1 each (confirmed), frames 3-4 = 1 each (predicted from last confirmed)
    assert_eq!(*rb.state().positions.get(&1).unwrap(), 5);
}

#[test]
fn test_desync_detection_matching() {
    let players = vec![0, 1];
    let state = TestState::new(&players);
    let config = RollbackConfig::default();
    let mut rb = RollbackNetcode::new(config, 0, players, state);

    rb.advance_frame(input(1));

    // Get the local hash for frame 0.
    let local_hash = rb.state_hash_at(0).unwrap();

    // Remote sends the same hash.
    let result = rb.check_desync(local_hash, 0);
    assert_eq!(result, DesyncResult::InSync);
}

#[test]
fn test_desync_detection_mismatch() {
    let players = vec![0, 1];
    let state = TestState::new(&players);
    let config = RollbackConfig::default();
    let mut rb = RollbackNetcode::new(config, 0, players, state);

    rb.advance_frame(input(1));

    // Remote sends a different hash.
    let result = rb.check_desync(99999, 0);
    assert_eq!(result, DesyncResult::Desync { frame: 0 });
}

#[test]
fn test_desync_detection_frame_not_available() {
    let players = vec![0, 1];
    let state = TestState::new(&players);
    let config = RollbackConfig {
        max_rollback_frames: 2,
        ..Default::default()
    };
    let mut rb = RollbackNetcode::new(config, 0, players, state);

    // Advance many frames to evict early snapshots.
    for _ in 0..10 {
        rb.advance_frame(input(1));
    }

    // Frame 0 should be evicted.
    let result = rb.check_desync(0, 0);
    assert_eq!(result, DesyncResult::FrameNotAvailable { frame: 0 });
}

#[test]
fn test_ring_buffer_overflow() {
    let players = vec![0];
    let state = TestState::new(&players);
    let config = RollbackConfig {
        max_rollback_frames: 3,
        ..Default::default()
    };
    let mut rb = RollbackNetcode::new(config, 0, players, state);

    // Advance 20 frames -- buffer should never exceed max_rollback_frames + 1.
    for _ in 0..20 {
        rb.advance_frame(input(1));
    }

    assert_eq!(rb.current_frame(), 20);
    assert_eq!(*rb.state().positions.get(&0).unwrap(), 20);
}

#[test]
fn test_confirmed_frame_advances() {
    let players = vec![0, 1];
    let state = TestState::new(&players);
    let config = RollbackConfig::default();
    let mut rb = RollbackNetcode::new(config, 0, players, state);

    // Advance 3 frames.
    for _ in 0..3 {
        rb.advance_frame(input(1));
    }

    // Confirmed frame should be 0 since remote has no inputs yet
    // (local has inputs for 0,1,2 but remote has none).
    assert_eq!(rb.confirmed_frame(), 0);

    // Confirm remote inputs for frames 0 and 1.
    rb.receive_remote_input(1, 0, input(0));
    rb.receive_remote_input(1, 1, input(0));

    // Now both players confirmed through frame 1.
    assert_eq!(rb.confirmed_frame(), 1);
}

#[test]
fn test_no_rollback_when_not_needed() {
    let players = vec![0];
    let state = TestState::new(&players);
    let config = RollbackConfig::default();
    let mut rb = RollbackNetcode::new(config, 0, players, state);

    rb.advance_frame(input(1));

    assert!(!rb.should_rollback());
    let resimulated = rb.rollback_and_resimulate();
    assert_eq!(resimulated, 0);
}

#[test]
fn test_multiple_rollbacks() {
    let players = vec![0, 1];
    let state = TestState::new(&players);
    let config = RollbackConfig {
        max_rollback_frames: 10,
        ..Default::default()
    };
    let mut rb = RollbackNetcode::new(config, 0, players, state);

    // Advance 5 frames.
    for _ in 0..5 {
        rb.advance_frame(input(1));
    }

    // First correction: remote had input 2 at frame 0.
    rb.receive_remote_input(1, 0, input(2));
    assert!(rb.should_rollback());
    rb.rollback_and_resimulate();
    assert!(!rb.should_rollback());

    // Advance 2 more frames.
    for _ in 0..2 {
        rb.advance_frame(input(1));
    }

    // Second correction: remote had input 3 at frame 3.
    rb.receive_remote_input(1, 3, input(3));
    assert!(rb.should_rollback());
    let resim = rb.rollback_and_resimulate();
    assert!(resim > 0);
    assert!(!rb.should_rollback());
}

#[test]
fn test_desync_detection_disabled() {
    let players = vec![0];
    let state = TestState::new(&players);
    let config = RollbackConfig {
        desync_detection: false,
        ..Default::default()
    };
    let mut rb = RollbackNetcode::new(config, 0, players, state);

    rb.advance_frame(input(1));

    // Even with a wrong hash, should report InSync when detection is off.
    let result = rb.check_desync(99999, 0);
    assert_eq!(result, DesyncResult::InSync);
}

#[test]
fn test_benchmark_resimulation_under_1ms() {
    // Use a state with more data to stress-test resimulation speed.
    #[derive(Clone, Debug)]
    struct HeavierState {
        values: Vec<i64>,
    }

    impl GameState for HeavierState {
        fn advance(&mut self, inputs: &HashMap<PlayerId, Vec<u8>>) {
            for (_, data) in inputs {
                let delta = if data.is_empty() {
                    0i64
                } else {
                    data[0] as i64
                };
                for v in self.values.iter_mut() {
                    *v += delta;
                }
            }
        }

        fn state_hash(&self) -> u64 {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            self.values.hash(&mut hasher);
            hasher.finish()
        }
    }

    let players = vec![0, 1];
    // 1000 values -- representative of a moderately complex game state.
    let state = HeavierState {
        values: vec![0; 1000],
    };
    let config = RollbackConfig {
        max_rollback_frames: 7,
        ..Default::default()
    };
    let mut rb = RollbackNetcode::new(config, 0, players, state);

    // Advance 7 frames.
    for _ in 0..7 {
        rb.advance_frame(input(1));
    }

    // Trigger rollback.
    rb.receive_remote_input(1, 0, input(2));
    assert!(rb.should_rollback());

    let start = Instant::now();
    let resimulated = rb.rollback_and_resimulate();
    let elapsed = start.elapsed();

    assert!(
        resimulated > 0,
        "Should have resimulated at least one frame"
    );
    // In debug builds, allow up to 5ms. In release builds, resimulation
    // of 7 frames with 1000-element state should be well under 1ms.
    let limit_us = if cfg!(debug_assertions) { 5000 } else { 1000 };
    assert!(
        elapsed.as_micros() < limit_us,
        "Resimulation took {}us, expected under {}us",
        elapsed.as_micros(),
        limit_us
    );
}
