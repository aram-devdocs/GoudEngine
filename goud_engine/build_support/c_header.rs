use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

const HEADER_VERSION_PLACEHOLDER: &str = "@@GOUD_ENGINE_VERSION@@";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum HeaderSection {
    Ecs,
    Assets,
    Renderer,
    Input,
    Audio,
    Transform2d,
    Sprite,
    SpriteAnimator,
    Text,
    Animation,
    Physics,
    Collision,
    Network,
    Ui,
    Debugger,
    Providers,
    Plugin,
    Window,
    EngineConfig,
    Error,
    Other,
}

impl HeaderSection {
    fn title(self) -> &'static str {
        match self {
            Self::Ecs => "ECS",
            Self::Assets => "Assets",
            Self::Renderer => "Renderer",
            Self::Input => "Input",
            Self::Audio => "Audio",
            Self::Transform2d => "Transform2D",
            Self::Sprite => "Sprite",
            Self::SpriteAnimator => "Sprite Animator",
            Self::Text => "Text",
            Self::Animation => "Animation",
            Self::Physics => "Physics",
            Self::Collision => "Collision",
            Self::Network => "Network",
            Self::Ui => "UI",
            Self::Debugger => "Debugger",
            Self::Providers => "Providers",
            Self::Plugin => "Plugin",
            Self::Window => "Window",
            Self::EngineConfig => "Engine Config",
            Self::Error => "Error",
            Self::Other => "Other Exports",
        }
    }
}

const HEADER_SECTION_ORDER: &[HeaderSection] = &[
    HeaderSection::Ecs,
    HeaderSection::Assets,
    HeaderSection::Renderer,
    HeaderSection::Input,
    HeaderSection::Audio,
    HeaderSection::Transform2d,
    HeaderSection::Sprite,
    HeaderSection::SpriteAnimator,
    HeaderSection::Text,
    HeaderSection::Animation,
    HeaderSection::Physics,
    HeaderSection::Collision,
    HeaderSection::Network,
    HeaderSection::Ui,
    HeaderSection::Debugger,
    HeaderSection::Providers,
    HeaderSection::Plugin,
    HeaderSection::Window,
    HeaderSection::EngineConfig,
    HeaderSection::Error,
    HeaderSection::Other,
];

pub fn generate_c_header(manifest_dir: &str, codegen_dir: &Path) -> Result<usize, String> {
    let config_path = Path::new(manifest_dir).join("cbindgen.toml");
    let output_path = codegen_dir.join("generated").join("goud_engine.h");
    let version =
        env::var("CARGO_PKG_VERSION").map_err(|err| format!("missing CARGO_PKG_VERSION: {err}"))?;

    let config = cbindgen::Config::from_file(&config_path)
        .map_err(|err| format!("unable to load {}: {err}", config_path.display()))?;

    let bindings = cbindgen::Builder::new()
        .with_crate(manifest_dir)
        .with_config(config)
        .generate()
        .map_err(|err| err.to_string())?;
    let mut raw_header = Vec::new();
    bindings.write(&mut raw_header);
    let raw_header = String::from_utf8(raw_header)
        .map_err(|err| format!("cbindgen emitted non-utf8 header: {err}"))?
        .replace(HEADER_VERSION_PLACEHOLDER, &version);

    let organized = wrap_header_body(&organize_header_sections(&raw_header), &version);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("unable to create {}: {err}", parent.display()))?;
    }

    fs::write(&output_path, organized)
        .map_err(|err| format!("unable to write {}: {err}", output_path.display()))?;

    Ok(HEADER_SECTION_ORDER.len() + 1)
}

fn organize_header_sections(raw_header: &str) -> String {
    let normalized = raw_header.replace("\r\n", "\n");
    let blocks = split_blocks(&normalized);
    let mut preamble = Vec::new();
    let mut body_blocks = Vec::new();
    let mut has_extern_open = false;
    let mut in_body = false;
    let mut in_footer = false;

    for block in blocks {
        if !in_body {
            if block.contains("extern \"C\" {") {
                has_extern_open = true;
                in_body = true;
            } else {
                preamble.push(block);
            }
            continue;
        }

        if !in_footer && is_extern_close_block(&block) {
            in_footer = true;
            continue;
        }

        if !in_footer {
            body_blocks.push(block);
        }
    }

    if !has_extern_open {
        body_blocks = preamble;
        preamble = Vec::new();
    }

    let mut leading_preamble = Vec::new();
    let mut declaration_blocks = Vec::new();
    for block in preamble {
        if is_include_block(&block) {
            leading_preamble.push(block);
        } else {
            declaration_blocks.push(block);
        }
    }
    declaration_blocks.extend(body_blocks);
    body_blocks = declaration_blocks;

    let mut common_blocks = Vec::new();
    let mut section_blocks: BTreeMap<HeaderSection, Vec<String>> = BTreeMap::new();

    for block in body_blocks {
        if let Some(name) = extract_exported_function_name(&block) {
            section_blocks
                .entry(classify_export(&name))
                .or_default()
                .push(block);
        } else if let Some(section) = classify_declaration_block(&block) {
            section_blocks.entry(section).or_default().push(block);
        } else {
            common_blocks.push(block);
        }
    }

    let mut ordered = Vec::new();
    ordered.extend(leading_preamble);
    if !common_blocks.is_empty() {
        ordered.push(section_banner("Common Types and Constants"));
        ordered.extend(common_blocks);
    }

    for section in HEADER_SECTION_ORDER {
        if let Some(blocks) = section_blocks.get(section) {
            if blocks.is_empty() {
                continue;
            }
            ordered.push(section_banner(section.title()));
            ordered.extend(blocks.iter().cloned());
        }
    }

    ordered.join("\n\n") + "\n"
}

fn wrap_header_body(body: &str, version: &str) -> String {
    let (includes, rest) = split_include_block(body);
    let mut header = String::new();
    header.push_str("#ifndef GOUD_ENGINE_H\n");
    header.push_str("#define GOUD_ENGINE_H\n\n");
    header.push_str("/* This file is auto-generated by GoudEngine. Do not edit it by hand. */\n\n");
    if !includes.is_empty() {
        header.push_str(&includes);
        header.push('\n');
        header.push('\n');
    }
    header.push_str(&header_macro_block(version));
    header.push_str("\n\n#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n");
    header.push_str(rest.trim_start());
    header.push_str("\n\n#ifdef __cplusplus\n} /* extern \"C\" */\n#endif\n\n");
    header.push_str("#endif /* GOUD_ENGINE_H */\n");
    header
}

fn split_include_block(body: &str) -> (String, String) {
    let lines: Vec<&str> = body.lines().collect();
    let mut include_end = 0usize;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("#include") {
            include_end = idx + 1;
            continue;
        }
        break;
    }

    let includes = lines[..include_end].join("\n").trim().to_string();
    let rest = lines[include_end..].join("\n");
    (includes, rest)
}

fn split_blocks(text: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();

    for line in text.lines() {
        if line.trim().is_empty() {
            if !current.is_empty() {
                blocks.push(current.join("\n"));
                current.clear();
            }
            continue;
        }
        current.push(line.to_string());
    }

    if !current.is_empty() {
        blocks.push(current.join("\n"));
    }

    blocks
}

fn is_include_block(block: &str) -> bool {
    block.lines().all(|line| {
        let trimmed = line.trim();
        trimmed.is_empty() || trimmed.starts_with("#include")
    })
}

fn is_extern_close_block(block: &str) -> bool {
    let trimmed = block.trim();
    trimmed.starts_with('}')
        || (block.contains("extern \"C\"")
            && block.lines().any(|line| line.trim().starts_with('}')))
}

fn classify_declaration_block(block: &str) -> Option<HeaderSection> {
    if block.contains("typedef int32_t GoudKeyCode;")
        || block.contains("typedef int32_t GoudMouseButton;")
        || block.contains("#define KEY_")
        || block.contains("#define MOUSE_BUTTON_")
    {
        return Some(HeaderSection::Input);
    }

    None
}

fn section_banner(title: &str) -> String {
    format!("/* === {title} === */")
}

fn header_macro_block(version: &str) -> String {
    format!(
        "#define GOUD_ENGINE_VERSION \"{version}\"\n\n#ifndef GOUD_DEPRECATED\n#  if defined(_MSC_VER)\n#    define GOUD_DEPRECATED __declspec(deprecated)\n#    define GOUD_DEPRECATED_MSG(msg) __declspec(deprecated(msg))\n#  elif defined(__GNUC__) || defined(__clang__)\n#    define GOUD_DEPRECATED __attribute__((deprecated))\n#    define GOUD_DEPRECATED_MSG(msg) __attribute__((deprecated(msg)))\n#  else\n#    define GOUD_DEPRECATED\n#    define GOUD_DEPRECATED_MSG(msg)\n#  endif\n#endif"
    )
}

fn extract_exported_function_name(block: &str) -> Option<String> {
    let mut search = block;
    while let Some(pos) = search.find("goud_") {
        let rest = &search[pos..];
        let end = rest
            .char_indices()
            .take_while(|(_, ch)| ch.is_ascii_alphanumeric() || *ch == '_')
            .last()
            .map(|(idx, ch)| idx + ch.len_utf8())?;
        let name = &rest[..end];
        let after = rest[end..].trim_start();
        if after.starts_with('(') {
            return Some(name.to_string());
        }
        search = &rest[end..];
    }
    None
}

fn classify_export(name: &str) -> HeaderSection {
    if name == "goud_scene_load" || name == "goud_scene_unload" {
        return HeaderSection::Assets;
    }
    if name == "goud_draw_text"
        || name.starts_with("goud_renderer_")
        || name.starts_with("goud_renderer3d_")
    {
        return HeaderSection::Renderer;
    }
    if name.starts_with("goud_context_")
        || name.starts_with("goud_entity_")
        || name.starts_with("goud_component_")
        || name.starts_with("goud_scene_")
    {
        return HeaderSection::Ecs;
    }
    if name.starts_with("goud_texture_") || name.starts_with("goud_font_") {
        return HeaderSection::Assets;
    }
    if name.starts_with("goud_input_") {
        return HeaderSection::Input;
    }
    if name.starts_with("goud_audio_") {
        return HeaderSection::Audio;
    }
    if name.starts_with("goud_transform2d_") {
        return HeaderSection::Transform2d;
    }
    if name.starts_with("goud_sprite_animator_") || name.starts_with("goud_animation_clip_") {
        return HeaderSection::SpriteAnimator;
    }
    if name.starts_with("goud_sprite_") {
        return HeaderSection::Sprite;
    }
    if name.starts_with("goud_text_") {
        return HeaderSection::Text;
    }
    if name.starts_with("goud_animation_") {
        return HeaderSection::Animation;
    }
    if name.starts_with("goud_physics_")
        || name.starts_with("goud_physics2d_")
        || name.starts_with("goud_physics3d_")
    {
        return HeaderSection::Physics;
    }
    if name.starts_with("goud_collision_") {
        return HeaderSection::Collision;
    }
    if name.starts_with("goud_network_") {
        return HeaderSection::Network;
    }
    if name.starts_with("goud_ui_") {
        return HeaderSection::Ui;
    }
    if name.starts_with("goud_debugger_") {
        return HeaderSection::Debugger;
    }
    if name.starts_with("goud_provider_") {
        return HeaderSection::Providers;
    }
    if name.starts_with("goud_plugin_") {
        return HeaderSection::Plugin;
    }
    if name.starts_with("goud_window_") {
        return HeaderSection::Window;
    }
    if name.starts_with("goud_engine_config_") {
        return HeaderSection::EngineConfig;
    }
    if name.starts_with("goud_error_")
        || name.starts_with("goud_get_last_error")
        || name.starts_with("goud_clear_last_error")
    {
        return HeaderSection::Error;
    }
    HeaderSection::Other
}
