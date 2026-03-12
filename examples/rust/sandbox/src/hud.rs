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
        0.62
    } else {
        0.88
    };
    let bottom_alpha = if matches!(snapshot.current_mode.as_str(), "3D" | "Hybrid") {
        0.70
    } else {
        0.92
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
        overview_advance,
    );
    draw_block(
        game,
        font_path,
        layout.status_text.x,
        layout.status_text.title_y,
        &status,
        status_advance,
    );
    draw_block(
        game,
        font_path,
        layout.next_text.x,
        layout.next_text.title_y,
        &next_steps,
        next_advance,
    );

    let scene_label = current_scene_label(config, snapshot.mode_index);
    let _ = game.draw_text(
        font_path,
        &scene_label,
        layout.scene_label.x,
        layout.scene_label.y,
        20.0,
        layout.scene_label.max_width,
        1.10,
        TITLE_COLOR.0,
        TITLE_COLOR.1,
        TITLE_COLOR.2,
        TITLE_COLOR.3,
    );
}

fn build_overview_lines(config: &SandboxConfig) -> Vec<HudLine> {
    let mut lines = Vec::new();
    append_wrapped(
        &mut lines,
        &config.hud.overview_title,
        40.0,
        30,
        TITLE_COLOR,
        0.0,
        1.12,
    );
    append_wrapped(
        &mut lines,
        &config.hud.tagline,
        24.0,
        34,
        TITLE_COLOR,
        0.0,
        1.12,
    );
    for item in &config.hud.contract.overview_items {
        append_wrapped(&mut lines, item, 19.0, 36, BODY_COLOR, 0.0, 1.12);
    }
    lines
}

fn build_status_lines(
    config: &SandboxConfig,
    network: &NetworkState,
    snapshot: &HudSnapshot,
) -> Vec<HudLine> {
    let mut lines = vec![HudLine {
        text: config.hud.status_title.clone(),
        size: 30.0,
        max_width: 0.0,
        line_spacing: 1.10,
        color: SECTION_COLOR,
    }];
    for row in &config.hud.contract.status_rows {
        lines.push(HudLine {
            text: render_status_row(config, network, snapshot, row),
            size: 18.0,
            max_width: config.hud.contract.layout.status_text.max_width,
            line_spacing: 1.10,
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
    let mut lines = Vec::new();
    append_wrapped(
        &mut lines,
        &config.hud.next_steps_title,
        32.0,
        30,
        SECTION_COLOR,
        0.0,
        1.12,
    );
    for item in &config.hud.contract.next_step_items {
        append_wrapped(&mut lines, item, 19.0, 64, BODY_COLOR, 0.0, 1.12);
    }
    for row in &config.hud.contract.next_step_dynamic_rows {
        append_wrapped(
            &mut lines,
            &render_next_step_row(network, audio_activated, row),
            19.0,
            64,
            BODY_COLOR,
            0.0,
            1.12,
        );
    }
    lines
}

fn draw_block(
    game: &mut GoudGame,
    font_path: &str,
    x: f32,
    start_y: f32,
    lines: &[HudLine],
    advance: fn(f32) -> f32,
) {
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
        y += advance(line.size);
    }
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

fn append_wrapped(
    output: &mut Vec<HudLine>,
    text: &str,
    size: f32,
    max_chars: usize,
    color: (f32, f32, f32, f32),
    max_width: f32,
    line_spacing: f32,
) {
    for line in wrap_for_hud(text, max_chars) {
        output.push(HudLine {
            text: line,
            size,
            max_width,
            line_spacing,
            color,
        });
    }
}

fn wrap_for_hud(text: &str, max_chars: usize) -> Vec<String> {
    if text.trim().is_empty() {
        return Vec::new();
    }

    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate_len = if line.is_empty() {
            word.len()
        } else {
            line.len() + 1 + word.len()
        };
        if candidate_len > max_chars && !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
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

fn overview_advance(size: f32) -> f32 {
    if size >= 38.0 {
        44.0
    } else if size >= 24.0 {
        30.0
    } else {
        24.0
    }
}

fn status_advance(size: f32) -> f32 {
    if size >= 30.0 {
        38.0
    } else {
        24.0
    }
}

fn next_advance(size: f32) -> f32 {
    if size >= 32.0 {
        38.0
    } else {
        25.0
    }
}
