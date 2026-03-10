//! WebAudio-backed audio methods for `WasmGame`.

use std::collections::HashMap;

use wasm_bindgen::JsValue;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    AudioContext, GainNode, HtmlAudioElement, HtmlMediaElement, MediaElementAudioSourceNode,
    StereoPannerNode,
};

#[path = "audio/activation_playback.rs"]
mod activation_playback;
#[path = "audio/playback_controls.rs"]
mod playback_controls;
#[path = "audio/spatial_controls.rs"]
mod spatial_controls;

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

    pub(super) fn ensure_context(&mut self) -> Option<AudioContext> {
        if self.context.is_none() {
            self.context = AudioContext::new().ok();
        }
        self.context.clone()
    }

    pub(super) fn allocate_player_id(&mut self) -> u64 {
        let id = self.next_player_id;
        self.next_player_id = self.next_player_id.wrapping_add(1);
        id
    }

    pub(super) fn refresh_spatial(&self) {
        for player in self.players.values() {
            apply_spatial_state(self.listener_position, player);
        }
    }

    pub(super) fn remove_player(&mut self, id: u64) -> bool {
        let Some(player) = self.players.remove(&id) else {
            return false;
        };

        let _ = player.element.pause();
        player.element.set_current_time(0.0);
        let _ = web_sys::Url::revoke_object_url(&player.object_url);
        true
    }
}

pub(super) struct WasmAudioPlayerState {
    pub(super) source_position: [f32; 3],
    pub(super) max_distance: f32,
    pub(super) rolloff: f32,
    pub(super) base_volume: f32,
    pub(super) speed: f32,
    pub(super) object_url: String,
    pub(super) element: HtmlAudioElement,
    pub(super) source_node: MediaElementAudioSourceNode,
    pub(super) gain_node: GainNode,
    pub(super) panner_node: Option<StereoPannerNode>,
}

impl WasmAudioPlayerState {
    pub(super) fn set_source_position(
        &mut self,
        source_position: [f32; 3],
        max_distance: f32,
        rolloff: f32,
    ) {
        self.source_position = source_position;
        self.max_distance = max_distance.max(0.1);
        self.rolloff = rolloff.max(0.01);
    }

    pub(super) fn set_base_volume(&mut self, volume: f32) {
        self.base_volume = volume.clamp(0.0, 1.0);
    }

    pub(super) fn set_speed(&mut self, speed: f32) {
        self.speed = speed.clamp(0.1, 10.0);
        self.element.set_playback_rate(self.speed as f64);
    }
}

#[inline]
pub(super) fn parse_player_id(player_id: f64) -> Option<u64> {
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

pub(super) fn apply_spatial_state(listener_position: [f32; 3], player: &WasmAudioPlayerState) {
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

pub(super) fn spawn_play(element: HtmlAudioElement) {
    if let Ok(promise) = element.play() {
        spawn_local(async move {
            let _ = JsFuture::from(promise).await;
        });
    }
}

pub(super) fn create_audio_url(bytes: &[u8]) -> Result<String, JsValue> {
    let payload = js_sys::Uint8Array::from(bytes);
    let parts = js_sys::Array::new();
    parts.push(&payload.buffer());
    let blob = web_sys::Blob::new_with_u8_array_sequence(&parts)?;
    web_sys::Url::create_object_url_with_blob(&blob)
}

pub(super) fn connect_audio_graph(
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
