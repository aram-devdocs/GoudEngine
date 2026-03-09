use wasm_bindgen::prelude::*;

use super::super::WasmGame;
use super::{apply_spatial_state, parse_player_id};

#[wasm_bindgen]
impl WasmGame {
    /// Updates attenuation for a playing spatial source in 2D (`z = 0`).
    pub fn audio_update_spatial_volume(
        &mut self,
        player_id: f64,
        source_x: f32,
        source_y: f32,
        listener_x: f32,
        listener_y: f32,
        max_distance: f32,
        rolloff: f32,
    ) -> i32 {
        self.audio_update_spatial_volume_3d(
            player_id,
            source_x,
            source_y,
            0.0,
            listener_x,
            listener_y,
            0.0,
            max_distance,
            rolloff,
        )
    }

    /// Updates attenuation for a playing spatial source in 3D.
    pub fn audio_update_spatial_volume_3d(
        &mut self,
        player_id: f64,
        source_x: f32,
        source_y: f32,
        source_z: f32,
        listener_x: f32,
        listener_y: f32,
        listener_z: f32,
        max_distance: f32,
        rolloff: f32,
    ) -> i32 {
        let Some(id) = parse_player_id(player_id) else {
            return -1;
        };

        let listener_position = [listener_x, listener_y, listener_z];
        self.audio_state.listener_position = listener_position;

        let Some(player) = self.audio_state.players.get_mut(&id) else {
            return -1;
        };

        player.set_source_position([source_x, source_y, source_z], max_distance, rolloff);
        apply_spatial_state(listener_position, player);
        0
    }

    /// Sets the global spatial listener position in 2D (`z = 0`).
    pub fn audio_set_listener_position(&mut self, x: f32, y: f32) -> i32 {
        self.audio_set_listener_position_3d(x, y, 0.0)
    }

    /// Sets the global spatial listener position in 3D.
    pub fn audio_set_listener_position_3d(&mut self, x: f32, y: f32, z: f32) -> i32 {
        self.audio_state.listener_position = [x, y, z];
        self.audio_state.refresh_spatial();
        0
    }

    /// Sets (or updates) a player source position in 2D (`z = 0`).
    pub fn audio_set_source_position(
        &mut self,
        player_id: f64,
        x: f32,
        y: f32,
        max_distance: f32,
        rolloff: f32,
    ) -> i32 {
        self.audio_set_source_position_3d(player_id, x, y, 0.0, max_distance, rolloff)
    }

    /// Sets (or updates) a player source position in 3D.
    pub fn audio_set_source_position_3d(
        &mut self,
        player_id: f64,
        x: f32,
        y: f32,
        z: f32,
        max_distance: f32,
        rolloff: f32,
    ) -> i32 {
        let Some(id) = parse_player_id(player_id) else {
            return -1;
        };
        let listener = self.audio_state.listener_position;
        let Some(player) = self.audio_state.players.get_mut(&id) else {
            return -1;
        };

        player.set_source_position([x, y, z], max_distance, rolloff);
        apply_spatial_state(listener, player);
        0
    }

    /// Sets a player's base volume in `[0, 1]`.
    pub fn audio_set_player_volume(&mut self, player_id: f64, volume: f32) -> i32 {
        let Some(id) = parse_player_id(player_id) else {
            return -1;
        };
        let listener = self.audio_state.listener_position;
        let Some(player) = self.audio_state.players.get_mut(&id) else {
            return -1;
        };

        player.set_base_volume(volume);
        apply_spatial_state(listener, player);
        0
    }

    /// Sets a player's playback speed in `[0.1, 10.0]`.
    pub fn audio_set_player_speed(&mut self, player_id: f64, speed: f32) -> i32 {
        let Some(id) = parse_player_id(player_id) else {
            return -1;
        };
        let Some(player) = self.audio_state.players.get_mut(&id) else {
            return -1;
        };

        player.set_speed(speed);
        0
    }
}
