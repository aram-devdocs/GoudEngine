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
    pub(crate) contract: HudContract,
}

#[derive(Clone)]
pub(crate) struct HudContract {
    pub(crate) overview_items: Vec<String>,
    pub(crate) status_rows: Vec<String>,
    pub(crate) next_step_items: Vec<String>,
    pub(crate) next_step_dynamic_rows: Vec<String>,
    pub(crate) layout: HudLayout,
}

#[derive(Clone, Copy)]
pub(crate) struct HudRect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct OverviewTextLayout {
    pub(crate) x: f32,
    pub(crate) title_y: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct StatusTextLayout {
    pub(crate) x: f32,
    pub(crate) title_y: f32,
    pub(crate) max_width: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct NextTextLayout {
    pub(crate) x: f32,
    pub(crate) title_y: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct SceneLabelLayout {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) max_width: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct HudLayout {
    pub(crate) overview_panel: HudRect,
    pub(crate) status_panel: HudRect,
    pub(crate) next_panel: HudRect,
    pub(crate) scene_badge: HudRect,
    pub(crate) overview_text: OverviewTextLayout,
    pub(crate) status_text: StatusTextLayout,
    pub(crate) next_text: NextTextLayout,
    pub(crate) scene_label: SceneLabelLayout,
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
    let contract_path = root
        .get("contract")
        .and_then(Value::as_str)
        .unwrap_or("examples/shared/sandbox/contract.json");
    let contract_raw = fs::read_to_string(contract_path)
        .unwrap_or_else(|e| panic!("failed to read {contract_path}: {e}"));
    let contract: Value = serde_json::from_str(&contract_raw)
        .unwrap_or_else(|e| panic!("failed to parse {contract_path}: {e}"));
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
    let contract_obj = contract
        .as_object()
        .unwrap_or_else(|| panic!("invalid contract root in {contract_path}"));
    let layout = contract_obj
        .get("layout")
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("missing layout in {contract_path}"));
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
            contract: HudContract {
                overview_items: value_str_array_obj(contract_obj, "overview_items"),
                status_rows: value_str_array_obj(contract_obj, "status_rows"),
                next_step_items: value_str_array_obj(contract_obj, "next_step_items"),
                next_step_dynamic_rows: value_str_array_obj(contract_obj, "next_step_dynamic_rows"),
                layout: HudLayout {
                    overview_panel: value_rect_obj(layout, "overview_panel"),
                    status_panel: value_rect_obj(layout, "status_panel"),
                    next_panel: value_rect_obj(layout, "next_panel"),
                    scene_badge: value_rect_obj(layout, "scene_badge"),
                    overview_text: value_overview_text_obj(layout, "overview_text"),
                    status_text: value_status_text_obj(layout, "status_text"),
                    next_text: value_next_text_obj(layout, "next_text"),
                    scene_label: value_scene_label_obj(layout, "scene_label"),
                },
            },
        },
        scenes: scene_entries,
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

fn value_rect_obj(root: &serde_json::Map<String, Value>, key: &str) -> HudRect {
    let rect = value_obj(root, key);
    HudRect {
        x: value_f32_obj(rect, "x"),
        y: value_f32_obj(rect, "y"),
        width: value_f32_obj(rect, "width"),
        height: value_f32_obj(rect, "height"),
    }
}

fn value_overview_text_obj(root: &serde_json::Map<String, Value>, key: &str) -> OverviewTextLayout {
    let layout = value_obj(root, key);
    OverviewTextLayout {
        x: value_f32_obj(layout, "x"),
        title_y: value_f32_obj(layout, "title_y"),
    }
}

fn value_status_text_obj(root: &serde_json::Map<String, Value>, key: &str) -> StatusTextLayout {
    let layout = value_obj(root, key);
    StatusTextLayout {
        x: value_f32_obj(layout, "x"),
        title_y: value_f32_obj(layout, "title_y"),
        max_width: value_f32_obj(layout, "max_width"),
    }
}

fn value_next_text_obj(root: &serde_json::Map<String, Value>, key: &str) -> NextTextLayout {
    let layout = value_obj(root, key);
    NextTextLayout {
        x: value_f32_obj(layout, "x"),
        title_y: value_f32_obj(layout, "title_y"),
    }
}

fn value_scene_label_obj(root: &serde_json::Map<String, Value>, key: &str) -> SceneLabelLayout {
    let layout = value_obj(root, key);
    SceneLabelLayout {
        x: value_f32_obj(layout, "x"),
        y: value_f32_obj(layout, "y"),
        max_width: value_f32_obj(layout, "max_width"),
    }
}

fn value_obj<'a>(
    root: &'a serde_json::Map<String, Value>,
    key: &str,
) -> &'a serde_json::Map<String, Value> {
    root.get(key)
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("missing object key {key}"))
}

fn value_f32_obj(root: &serde_json::Map<String, Value>, key: &str) -> f32 {
    root.get(key)
        .and_then(Value::as_f64)
        .unwrap_or_else(|| panic!("missing float key {key}")) as f32
}
