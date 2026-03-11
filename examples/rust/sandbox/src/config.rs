use std::fs;

use serde_json::Value;

#[derive(Clone)]
pub(crate) struct Assets {
    pub(crate) background: String,
    pub(crate) sprite: String,
    pub(crate) accent: String,
    pub(crate) texture3d: String,
    pub(crate) font: String,
}

#[derive(Clone)]
pub(crate) struct HudConfig {
    pub(crate) overview_title: String,
    pub(crate) status_title: String,
    pub(crate) next_steps_title: String,
    pub(crate) tagline: String,
    pub(crate) overview: Vec<String>,
    pub(crate) next_steps: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct SceneEntry {
    pub(crate) key: String,
    pub(crate) mode: String,
    pub(crate) label: String,
}

pub(crate) struct SandboxConfig {
    pub(crate) title: String,
    pub(crate) assets: Assets,
    pub(crate) network_port: u16,
    pub(crate) packet_version: String,
    pub(crate) hud: HudConfig,
    pub(crate) scenes: Vec<SceneEntry>,
}

pub(crate) fn parse_start_mode(scene_count: usize) -> usize {
    std::env::var("GOUD_SANDBOX_START_MODE")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| (1..=scene_count).contains(value))
        .map(|value| value - 1)
        .unwrap_or(0)
}

pub(crate) fn read_manifest() -> SandboxConfig {
    let path = "examples/shared/sandbox/manifest.json";
    let raw = fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    let root: Value =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("failed to parse {path}: {e}"));
    let assets = root
        .get("assets")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("missing assets in {path}"));
    let network = root
        .get("network")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("missing network in {path}"));
    let hud = root
        .get("hud")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("missing hud in {path}"));
    let scenes = root
        .get("scenes")
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("missing scenes in {path}"));

    let scene_entries = scenes
        .iter()
        .map(|entry| SceneEntry {
            key: value_str(entry, "key"),
            mode: value_str(entry, "mode"),
            label: value_str(entry, "label"),
        })
        .collect::<Vec<_>>();

    SandboxConfig {
        title: value_str(&root, "title"),
        assets: Assets {
            background: value_str_obj(assets, "background"),
            sprite: value_str_obj(assets, "sprite"),
            accent: value_str_obj(assets, "accent_sprite"),
            texture3d: value_str_obj(assets, "texture3d"),
            font: value_str_obj(assets, "font"),
        },
        network_port: network
            .get("port")
            .and_then(Value::as_u64)
            .and_then(|v| u16::try_from(v).ok())
            .unwrap_or(38491),
        packet_version: network
            .get("packet_version")
            .and_then(Value::as_str)
            .unwrap_or("v1")
            .to_string(),
        hud: HudConfig {
            overview_title: value_str_obj(hud, "overview_title"),
            status_title: value_str_obj(hud, "status_title"),
            next_steps_title: value_str_obj(hud, "next_steps_title"),
            tagline: value_str_obj(hud, "tagline"),
            overview: value_str_array_obj(hud, "overview"),
            next_steps: value_str_array_obj(hud, "next_steps"),
        },
        scenes: scene_entries,
    }
}

pub(crate) fn mode_color(mode: &str) -> (f32, f32, f32, f32) {
    match mode {
        "2D" => (0.20, 0.55, 0.95, 0.85),
        "3D" => (0.62, 0.35, 0.90, 0.85),
        _ => (0.30, 0.72, 0.50, 0.85),
    }
}

fn value_str(root: &Value, key: &str) -> String {
    root.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing key {key}"))
        .to_string()
}

fn value_str_obj(root: &serde_json::Map<String, Value>, key: &str) -> String {
    root.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("missing key {key}"))
        .to_string()
}

fn value_str_array_obj(root: &serde_json::Map<String, Value>, key: &str) -> Vec<String> {
    root.get(key)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("missing array key {key}"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("non-string entry in array key {key}"))
                .to_string()
        })
        .collect()
}
