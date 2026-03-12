use crate::config::SandboxConfig;
use crate::network::NetworkState;
use goud_engine::sdk::GoudGame;

const BODY_COLOR: (f32, f32, f32, f32) = (0.94, 0.97, 1.0, 1.0);
const TITLE_COLOR: (f32, f32, f32, f32) = (1.0, 1.0, 1.0, 1.0);
const SECTION_COLOR: (f32, f32, f32, f32) = (0.95, 0.90, 0.40, 1.0);
const BADGE_COLOR: (f32, f32, f32, f32) = (0.20, 0.55, 0.95, 0.84);

pub(crate) struct HudSnapshot {
    pub(crate) current_mode: String,
    pub(crate) mode_index: usize,
    pub(crate) mouse_x: f32,
    pub(crate) mouse_y: f32,
    pub(crate) render_texture_size: u32,
    pub(crate) render_supports_instancing: bool,
    pub(crate) physics_supports_joints: bool,
    pub(crate) physics_max_bodies: u32,
    pub(crate) audio_supports_spatial: bool,
    pub(crate) audio_max_channels: u32,
    pub(crate) audio_activated: bool,
}

struct HudLine {
    text: String,
    size: f32,
    max_width: f32,
    line_spacing: f32,
    line_advance: f32,
    color: (f32, f32, f32, f32),
}

pub(crate) fn draw(
    game: &mut GoudGame,
    font_path: &str,
    config: &SandboxConfig,
    network: &NetworkState,
    snapshot: &HudSnapshot,
) {
    let layout = &config.hud.contract.layout;
    let panel_alpha = if matches!(snapshot.current_mode.as_str(), "3D" | "Hybrid") {
        0.48
    } else {
        0.72
    };
    let bottom_alpha = if matches!(snapshot.current_mode.as_str(), "3D" | "Hybrid") {
        0.55
    } else {
        0.78
    };

    game.draw_quad(
        layout.overview_panel.x,
        layout.overview_panel.y,
        layout.overview_panel.width,
        layout.overview_panel.height,
        0.05,
        0.08,
        0.12,
        panel_alpha,
    );
    game.draw_quad(
        layout.status_panel.x,
        layout.status_panel.y,
        layout.status_panel.width,
        layout.status_panel.height,
        0.08,
        0.12,
        0.18,
        panel_alpha,
    );
    game.draw_quad(
        layout.next_panel.x,
        layout.next_panel.y,
        layout.next_panel.width,
        layout.next_panel.height,
        0.05,
        0.08,
        0.12,
        bottom_alpha,
    );
    game.draw_quad(
        layout.scene_badge.x,
        layout.scene_badge.y,
        layout.scene_badge.width,
        layout.scene_badge.height,
        BADGE_COLOR.0,
        BADGE_COLOR.1,
        BADGE_COLOR.2,
        BADGE_COLOR.3,
    );
    game.draw_quad(
        snapshot.mouse_x,
        snapshot.mouse_y,
        12.0,
        12.0,
        0.95,
        0.85,
        0.20,
        0.95,
    );

    let overview = build_overview_lines(config);
    let status = build_status_lines(config, network, snapshot);
    let next_steps = build_next_step_lines(config, network, snapshot.audio_activated);

    draw_block(
        game,
        font_path,
        layout.overview_text.x,
        layout.overview_text.title_y,
        &overview,
    );
    draw_block(
        game,
        font_path,
        layout.status_text.x,
        layout.status_text.title_y,
        &status,
    );
    draw_block(
        game,
        font_path,
        layout.next_text.x,
        layout.next_text.title_y,
        &next_steps,
    );

    let scene_label = current_scene_label(config, snapshot.mode_index);
    let scene_type = config.hud.contract.typography.scene_label;
    let _ = game.draw_text(
        font_path,
        &scene_label,
        layout.scene_label.x,
        layout.scene_label.y,
        scene_type.size,
        layout.scene_label.max_width,
        scene_type.line_spacing,
        TITLE_COLOR.0,
        TITLE_COLOR.1,
        TITLE_COLOR.2,
        TITLE_COLOR.3,
    );
}

fn build_overview_lines(config: &SandboxConfig) -> Vec<HudLine> {
    let type_scale = config.hud.contract.typography.overview;
    let mut lines = vec![
        HudLine {
            text: config.hud.overview_title.clone(),
            size: type_scale.title_size,
            max_width: config.hud.contract.layout.overview_text.max_width,
            line_spacing: type_scale.line_spacing,
            line_advance: type_scale.line_advances.title,
            color: TITLE_COLOR,
        },
        HudLine {
            text: config.hud.tagline.clone(),
            size: type_scale.tagline_size,
            max_width: config.hud.contract.layout.overview_text.max_width,
            line_spacing: type_scale.line_spacing,
            line_advance: type_scale.line_advances.tagline,
            color: TITLE_COLOR,
        },
    ];
    for item in &config.hud.contract.overview_items {
        lines.push(HudLine {
            text: item.clone(),
            size: type_scale.body_size,
            max_width: config.hud.contract.layout.overview_text.max_width,
            line_spacing: type_scale.line_spacing,
            line_advance: type_scale.line_advances.body,
            color: BODY_COLOR,
        });
    }
    lines
}

fn build_status_lines(
    config: &SandboxConfig,
    network: &NetworkState,
    snapshot: &HudSnapshot,
) -> Vec<HudLine> {
    let type_scale = config.hud.contract.typography.status;
    let mut lines = vec![HudLine {
        text: config.hud.status_title.clone(),
        size: type_scale.title_size,
        max_width: config.hud.contract.layout.status_text.max_width,
        line_spacing: type_scale.line_spacing,
        line_advance: type_scale.line_advances.title,
        color: SECTION_COLOR,
    }];
    for row in &config.hud.contract.status_rows {
        lines.push(HudLine {
            text: render_status_row(config, network, snapshot, row),
            size: type_scale.body_size,
            max_width: config.hud.contract.layout.status_text.max_width,
            line_spacing: type_scale.line_spacing,
            line_advance: type_scale.line_advances.body,
            color: BODY_COLOR,
        });
    }
    lines
}

fn build_next_step_lines(
    config: &SandboxConfig,
    network: &NetworkState,
    audio_activated: bool,
) -> Vec<HudLine> {
    let type_scale = config.hud.contract.typography.next;
    let mut lines = vec![HudLine {
        text: config.hud.next_steps_title.clone(),
        size: type_scale.title_size,
        max_width: config.hud.contract.layout.next_text.max_width,
        line_spacing: type_scale.line_spacing,
        line_advance: type_scale.line_advances.title,
        color: SECTION_COLOR,
    }];
    for item in &config.hud.contract.next_step_items {
        lines.push(HudLine {
            text: item.clone(),
            size: type_scale.body_size,
            max_width: config.hud.contract.layout.next_text.max_width,
            line_spacing: type_scale.line_spacing,
            line_advance: type_scale.line_advances.body,
            color: BODY_COLOR,
        });
    }
    for row in &config.hud.contract.next_step_dynamic_rows {
        lines.push(HudLine {
            text: render_next_step_row(network, audio_activated, row),
            size: type_scale.body_size,
            max_width: config.hud.contract.layout.next_text.max_width,
            line_spacing: type_scale.line_spacing,
            line_advance: type_scale.line_advances.body,
            color: BODY_COLOR,
        });
    }
    lines
}

fn draw_block(game: &mut GoudGame, font_path: &str, x: f32, start_y: f32, lines: &[HudLine]) {
    let mut y = start_y;
    for line in lines {
        let _ = game.draw_text(
            font_path,
            &line.text,
            x,
            y,
            line.size,
            line.max_width,
            line.line_spacing,
            line.color.0,
            line.color.1,
            line.color.2,
            line.color.3,
        );
        y += line.line_advance * wrapped_line_count(&line.text, line.max_width, line.size) as f32;
    }
}

fn wrapped_line_count(text: &str, max_width: f32, font_size: f32) -> usize {
    if text.trim().is_empty() || max_width <= 0.0 {
        return 1;
    }

    let approx_glyph_width = (font_size * 0.52).max(1.0);
    let max_chars = (max_width / approx_glyph_width).floor().max(1.0) as usize;
    let mut total_lines = 0usize;

    for raw_line in text.split('\n') {
        let words: Vec<&str> = raw_line.split_whitespace().collect();
        if words.is_empty() {
            total_lines += 1;
            continue;
        }
        let mut current_len = 0usize;
        let mut wrapped = 1usize;
        for word in words {
            let word_len = word.chars().count();
            if current_len == 0 {
                current_len = word_len;
                continue;
            }
            if current_len + 1 + word_len <= max_chars {
                current_len += 1 + word_len;
            } else {
                wrapped += 1;
                current_len = word_len;
            }
        }
        total_lines += wrapped;
    }

    total_lines.max(1)
}

fn render_status_row(
    config: &SandboxConfig,
    network: &NetworkState,
    snapshot: &HudSnapshot,
    row: &str,
) -> String {
    let scene = current_scene(config, snapshot.mode_index);
    match row {
        "scene" => format!("Scene: {} ({} to switch)", scene.label, scene.key),
        "mouse" => format!(
            "Mouse marker: ({:.0}, {:.0})",
            snapshot.mouse_x, snapshot.mouse_y
        ),
        "render_caps" => format!(
            "Render caps: tex={} instancing={}",
            snapshot.render_texture_size,
            bool_word(snapshot.render_supports_instancing)
        ),
        "physics_caps" => format!(
            "Physics caps: joints={} maxBodies={}",
            bool_word(snapshot.physics_supports_joints),
            snapshot.physics_max_bodies
        ),
        "audio_caps" => format!(
            "Audio caps: spatial={} channels={}",
            bool_word(snapshot.audio_supports_spatial),
            snapshot.audio_max_channels
        ),
        "scene_count" => format!(
            "Scene count: {} active mode={}",
            config.scenes.len(),
            snapshot.current_mode
        ),
        "target" => "Target: desktop".to_string(),
        "network_role" => format!(
            "Network role: {} peers={} label={}",
            network.role.as_str(),
            network.peer_count,
            network.label
        ),
        "network_detail" => format!("Network detail: {}", network.detail()),
        _ => row.to_string(),
    }
}

fn render_next_step_row(network: &NetworkState, audio_activated: bool, row: &str) -> String {
    match row {
        "audio_status" => format!(
            "Audio status: {}",
            if audio_activated {
                "active"
            } else {
                "press SPACE to activate"
            }
        ),
        "network_probe" => {
            if let Some(remote) = &network.remote {
                format!("Peer sprite live at ({:.0}, {:.0})", remote.x, remote.y)
            } else {
                "Networking: open a second native sandbox to confirm peer sync.".to_string()
            }
        }
        _ => row.to_string(),
    }
}

fn current_scene(config: &SandboxConfig, mode_index: usize) -> &crate::config::SceneEntry {
    config
        .scenes
        .get(mode_index)
        .unwrap_or_else(|| config.scenes.first().expect("sandbox scenes missing"))
}

fn current_scene_label(config: &SandboxConfig, mode_index: usize) -> String {
    current_scene(config, mode_index).label.clone()
}

fn bool_word(value: bool) -> &'static str {
    if value {
        "True"
    } else {
        "False"
    }
}

#[cfg(test)]
mod tests {
    use super::build_overview_lines;
    use crate::config::read_manifest_from;
    use std::path::PathBuf;

    #[test]
    fn overview_lines_keep_full_contract_strings_without_manual_prerender_wrap() {
        let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../shared/sandbox/manifest.json")
            .to_string_lossy()
            .into_owned();
        let config = read_manifest_from(&manifest_path);
        let lines = build_overview_lines(&config);
        assert_eq!(lines.len(), config.hud.contract.overview_items.len() + 2);
        assert_eq!(lines[0].text, config.hud.overview_title);
        assert_eq!(lines[1].text, config.hud.tagline);
        for (line, source) in lines
            .iter()
            .skip(2)
            .zip(config.hud.contract.overview_items.iter())
        {
            assert_eq!(&line.text, source);
        }
    }
}
