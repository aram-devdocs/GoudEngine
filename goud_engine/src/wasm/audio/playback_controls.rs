use wasm_bindgen::prelude::*;

use super::super::WasmGame;
use super::{apply_spatial_state, parse_player_id, spawn_play};

#[wasm_bindgen]
impl WasmGame {
    /// Pauses a player's playback.
    pub fn audio_pause(&mut self, player_id: f64) -> i32 {
        let Some(id) = parse_player_id(player_id) else {
            return -1;
        };
        let Some(player) = self.audio_state.players.get(&id) else {
            return -1;
        };

        let _ = player.element.pause();
        0
    }

    /// Resumes a paused player.
    pub fn audio_resume(&mut self, player_id: f64) -> i32 {
        let Some(id) = parse_player_id(player_id) else {
            return -1;
        };
        let Some(player) = self.audio_state.players.get(&id) else {
            return -1;
        };

        spawn_play(player.element.clone());
        0
    }

    /// Stops a player and releases its WebAudio resources.
    pub fn audio_stop(&mut self, player_id: f64) -> i32 {
        let Some(id) = parse_player_id(player_id) else {
            return -1;
        };

        if self.audio_state.remove_player(id) {
            0
        } else {
            -1
        }
    }

    /// Stops all active players and releases their WebAudio resources.
    pub fn audio_stop_all(&mut self) -> i32 {
        let ids: Vec<u64> = self.audio_state.players.keys().copied().collect();
        for id in ids {
            let _ = self.audio_state.remove_player(id);
        }
        0
    }

    /// Applies an immediate two-player mix in `[0, 1]`.
    pub fn audio_crossfade(&mut self, from_player_id: f64, to_player_id: f64, mix: f32) -> i32 {
        let Some(from_id) = parse_player_id(from_player_id) else {
            return -1;
        };
        let Some(to_id) = parse_player_id(to_player_id) else {
            return -1;
        };

        let t = mix.clamp(0.0, 1.0);

        if from_id == to_id {
            return self.audio_set_player_volume(from_player_id, 1.0);
        }

        if !self.audio_state.players.contains_key(&from_id)
            || !self.audio_state.players.contains_key(&to_id)
        {
            return -1;
        }

        let listener = self.audio_state.listener_position;

        if let Some(from_state) = self.audio_state.players.get_mut(&from_id) {
            from_state.set_base_volume(1.0 - t);
            apply_spatial_state(listener, from_state);
        }
        if let Some(to_state) = self.audio_state.players.get_mut(&to_id) {
            to_state.set_base_volume(t);
            apply_spatial_state(listener, to_state);
        }

        0
    }
}
