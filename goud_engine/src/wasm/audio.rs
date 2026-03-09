//! WebAudio-backed audio methods for `WasmGame`.

use std::collections::HashMap;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};

use web_sys::{
    AudioContext, AudioContextState, GainNode, HtmlAudioElement, HtmlMediaElement,
    MediaElementAudioSourceNode, StereoPannerNode,
};

use super::WasmGame;

pub(super) struct WasmAudioState {
    context: Option<AudioContext>,
    listener_position: [f32; 3],
    next_player_id: u64,
    players: HashMap<u64, WasmAudioPlayerState>,
}

impl WasmAudioState {
    pub(super) fn new() -> Self {
        Self {
            context: AudioContext::new().ok(),
            listener_position: [0.0, 0.0, 0.0],
            next_player_id: 0,
            players: HashMap::new(),
        }
    }

    fn ensure_context(&mut self) -> Option<AudioContext> {
        if self.context.is_none() {
            self.context = AudioContext::new().ok();
        }
        self.context.clone()
    }

    fn allocate_player_id(&mut self) -> u64 {
        let id = self.next_player_id;
        self.next_player_id = self.next_player_id.wrapping_add(1);
        id
    }

    fn refresh_spatial(&self) {
        for player in self.players.values() {
            apply_spatial_state(self.listener_position, player);
        }
    }

    fn remove_player(&mut self, id: u64) -> bool {
        let Some(player) = self.players.remove(&id) else {
            return false;
        };

        let _ = player.element.pause();
        let _ = player.element.set_current_time(0.0);
        let _ = web_sys::Url::revoke_object_url(&player.object_url);
        true
    }
}

struct WasmAudioPlayerState {
    source_position: [f32; 3],
    max_distance: f32,
    rolloff: f32,
    base_volume: f32,
    speed: f32,
    object_url: String,
    element: HtmlAudioElement,
    source_node: MediaElementAudioSourceNode,
    gain_node: GainNode,
    panner_node: Option<StereoPannerNode>,
}

impl WasmAudioPlayerState {
    fn set_source_position(&mut self, source_position: [f32; 3], max_distance: f32, rolloff: f32) {
        self.source_position = source_position;
        self.max_distance = max_distance.max(0.1);
        self.rolloff = rolloff.max(0.01);
    }

    fn set_base_volume(&mut self, volume: f32) {
        self.base_volume = volume.clamp(0.0, 1.0);
    }

    fn set_speed(&mut self, speed: f32) {
        self.speed = speed.clamp(0.1, 10.0);
        self.element.set_playback_rate(self.speed as f64);
    }
}

#[inline]
fn parse_player_id(player_id: f64) -> Option<u64> {
    if !player_id.is_finite() || player_id < 0.0 || player_id.fract() != 0.0 {
        return None;
    }
    if player_id > u64::MAX as f64 {
        return None;
    }
    Some(player_id as u64)
}

#[inline]
fn compute_linear_attenuation(distance: f32, max_distance: f32, rolloff: f32) -> f32 {
    if max_distance <= 0.0 {
        return 1.0;
    }
    if distance >= max_distance {
        return 0.0;
    }

    let normalized = distance / max_distance;
    (1.0 - normalized.powf(rolloff.max(0.01))).clamp(0.0, 1.0)
}

#[inline]
fn spatial_attenuation_3d(
    source_position: [f32; 3],
    listener_position: [f32; 3],
    max_distance: f32,
    rolloff: f32,
) -> f32 {
    let dx = source_position[0] - listener_position[0];
    let dy = source_position[1] - listener_position[1];
    let dz = source_position[2] - listener_position[2];
    let distance = (dx * dx + dy * dy + dz * dz).sqrt();
    compute_linear_attenuation(distance, max_distance, rolloff)
}

#[inline]
fn stereo_pan(source_position: [f32; 3], listener_position: [f32; 3]) -> f32 {
    let dx = source_position[0] - listener_position[0];
    if dx.abs() <= f32::EPSILON {
        0.0
    } else {
        (dx / (dx.abs() + 1.0)).clamp(-1.0, 1.0)
    }
}

fn apply_spatial_state(listener_position: [f32; 3], player: &WasmAudioPlayerState) {
    let attenuation = spatial_attenuation_3d(
        player.source_position,
        listener_position,
        player.max_distance,
        player.rolloff,
    );
    let effective_volume = (player.base_volume * attenuation).clamp(0.0, 1.0);
    player.gain_node.gain().set_value(effective_volume);

    if let Some(panner) = &player.panner_node {
        panner
            .pan()
            .set_value(stereo_pan(player.source_position, listener_position));
    }

    let _ = player.source_node.number_of_inputs();
}

fn spawn_play(element: HtmlAudioElement) {
    if let Ok(promise) = element.play() {
        spawn_local(async move {
            let _ = JsFuture::from(promise).await;
        });
    }
}

fn create_audio_url(bytes: &[u8]) -> Result<String, JsValue> {
    let payload = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&payload.buffer());
    let blob = web_sys::Blob::new_with_u8_array_sequence(&parts)?;
    web_sys::Url::create_object_url_with_blob(&blob)
}

fn connect_audio_graph(
    context: &AudioContext,
    element: &HtmlAudioElement,
) -> Result<
    (
        MediaElementAudioSourceNode,
        GainNode,
        Option<StereoPannerNode>,
    ),
    JsValue,
> {
    let media: &HtmlMediaElement = element.as_ref();
    let source = context.create_media_element_source(media)?;
    let gain = context.create_gain()?;
    gain.gain().set_value(1.0);

    let panner = context.create_stereo_panner().ok();

    if let Some(panner_node) = &panner {
        source.connect_with_audio_node(panner_node)?;
        panner_node.connect_with_audio_node(&gain)?;
    } else {
        source.connect_with_audio_node(&gain)?;
    }

    gain.connect_with_audio_node(&context.destination())?;

    Ok((source, gain, panner))
}

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
