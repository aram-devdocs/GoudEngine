use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{AudioContextState, HtmlAudioElement};

use super::super::WasmGame;
use super::{
    apply_spatial_state, connect_audio_graph, create_audio_url, spawn_play, WasmAudioPlayerState,
};

#[wasm_bindgen]
impl WasmGame {
    /// Creates or resumes the WebAudio context.
    ///
    /// Call this from the first user gesture (click/touch/key) to satisfy
    /// browser autoplay restrictions.
    pub fn audio_activate(&mut self) -> i32 {
        let Some(context) = self.audio_state.ensure_context() else {
            return -1;
        };

        if context.state() == AudioContextState::Running {
            return 0;
        }

        match context.resume() {
            Ok(promise) => {
                spawn_local(async move {
                    let _ = JsFuture::from(promise).await;
                });
                0
            }
            Err(_) => -1,
        }
    }

    /// Plays audio without spatialization defaults.
    pub fn audio_play(&mut self, asset_data: &[u8]) -> f64 {
        let listener = self.audio_state.listener_position;
        self.audio_play_spatial_3d(
            asset_data,
            0.0,
            0.0,
            0.0,
            listener[0],
            listener[1],
            listener[2],
            100.0,
            1.0,
        )
    }

    /// Plays audio with spatial attenuation in 2D (`z = 0`).
    pub fn audio_play_spatial(
        &mut self,
        asset_data: &[u8],
        source_x: f32,
        source_y: f32,
        listener_x: f32,
        listener_y: f32,
        max_distance: f32,
        rolloff: f32,
    ) -> f64 {
        self.audio_play_spatial_3d(
            asset_data,
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

    /// Plays audio with spatial attenuation in 3D.
    pub fn audio_play_spatial_3d(
        &mut self,
        asset_data: &[u8],
        source_x: f32,
        source_y: f32,
        source_z: f32,
        listener_x: f32,
        listener_y: f32,
        listener_z: f32,
        max_distance: f32,
        rolloff: f32,
    ) -> f64 {
        if asset_data.is_empty() {
            return -1.0;
        }

        let Some(context) = self.audio_state.ensure_context() else {
            return -1.0;
        };

        if context.state() != AudioContextState::Running {
            if let Ok(promise) = context.resume() {
                spawn_local(async move {
                    let _ = JsFuture::from(promise).await;
                });
            }
        }

        let object_url = match create_audio_url(asset_data) {
            Ok(url) => url,
            Err(_) => return -1.0,
        };

        let element = match HtmlAudioElement::new_with_src(&object_url) {
            Ok(element) => element,
            Err(_) => {
                let _ = web_sys::Url::revoke_object_url(&object_url);
                return -1.0;
            }
        };

        element.set_preload("auto");
        element.set_loop(false);
        element.set_playback_rate(1.0);

        let (source_node, gain_node, panner_node) = match connect_audio_graph(&context, &element) {
            Ok(graph) => graph,
            Err(_) => {
                let _ = web_sys::Url::revoke_object_url(&object_url);
                return -1.0;
            }
        };

        self.audio_state.listener_position = [listener_x, listener_y, listener_z];

        let player_id = self.audio_state.allocate_player_id();
        let mut player = WasmAudioPlayerState {
            source_position: [source_x, source_y, source_z],
            max_distance: max_distance.max(0.1),
            rolloff: rolloff.max(0.01),
            base_volume: 1.0,
            speed: 1.0,
            object_url,
            element,
            source_node,
            gain_node,
            panner_node,
        };

        player.set_speed(1.0);
        apply_spatial_state(self.audio_state.listener_position, &player);

        spawn_play(player.element.clone());
        self.audio_state.players.insert(player_id, player);

        player_id as f64
    }
}
