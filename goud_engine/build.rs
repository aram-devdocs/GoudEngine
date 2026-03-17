//! # Build Script: FFI Manifest and C Header
//!
//! This build script generates:
//!
//! - **FFI Manifest** (`codegen/ffi_manifest.json`) via regex-free source parsing
//! - **C Header** (`codegen/generated/goud_engine.h`) via cbindgen
//!
//! ## Design Philosophy
//!
//! All logic lives in Rust. SDKs are thin wrappers that marshal data and call
//! FFI functions. This ensures consistent behavior across all language bindings.
//!
//! ## Usage
//!
//! ```bash
//! cargo build
//! # Outputs:
//! #   - ../codegen/ffi_manifest.json (auto-extracted FFI surface)
//! #   - ../codegen/generated/goud_engine.h (grouped C header)
//! ```

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

const HEADER_VERSION_PLACEHOLDER: &str = "@@GOUD_ENGINE_VERSION@@";

fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        return;
    }

    println!("cargo:rerun-if-changed=src/ffi/");
    println!("cargo:rerun-if-changed=src/core/math.rs");
    println!("cargo:rerun-if-changed=src/core/error.rs");
    println!("cargo:rerun-if-changed=src/core/types.rs");
    println!("cargo:rerun-if-changed=src/context_registry/");
    println!("cargo:rerun-if-changed=src/component_ops/");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let codegen_dir = Path::new(&manifest_dir).join("..").join("codegen");

    // =========================================================================
    // Generate FFI Manifest (codegen/ffi_manifest.json)
    // =========================================================================
    let manifest_output = codegen_dir.join("ffi_manifest.json");
    let manifest_count = generate_ffi_manifest(&manifest_dir, &manifest_output);
    println!("cargo:rerun-if-changed=../codegen/");
    let header_count = match generate_c_header(&manifest_dir, &codegen_dir) {
        Ok(count) => count,
        Err(err) => panic!("failed to generate cbindgen header: {err}"),
    };

    println!("cargo:warning=");
    println!("cargo:warning=GoudEngine Build");
    println!(
        "cargo:warning=  FFI Manifest [OK]: codegen/ffi_manifest.json ({manifest_count} functions)"
    );
    println!(
        "cargo:warning=  C Header [OK]: codegen/generated/goud_engine.h ({header_count} sections)"
    );
    println!("cargo:warning=");
}

// =============================================================================
// FFI Manifest Generation
// =============================================================================

/// Files to scan for FFI functions.
const FFI_SOURCE_FILES: &[&str] = &[
    "src/ffi/types.rs",
    // core types (split into directories)
    "src/core/math/vec2.rs",
    "src/core/math/vec3.rs",
    "src/core/math/vec4.rs",
    "src/core/math/color.rs",
    "src/core/math/rect.rs",
    "src/core/error/ffi_bridge.rs",
    "src/core/error/types.rs",
    "src/core/types/entity.rs",
    "src/core/types/math_types.rs",
    "src/core/types/result.rs",
    "src/core/types/sprite.rs",
    "src/core/types/transform.rs",
    "src/core/context_id.rs",
    "src/component_ops/storage.rs",
    "src/component_ops/helpers.rs",
    "src/component_ops/single_ops.rs",
    "src/component_ops/batch_ops.rs",
    // context module
    "src/ffi/context/lifecycle.rs",
    // entity module
    "src/ffi/entity/lifecycle.rs",
    "src/ffi/entity/queries.rs",
    // component module
    "src/ffi/component/ops.rs",
    "src/ffi/component/access.rs",
    "src/ffi/component/batch.rs",
    // component_transform2d module
    "src/ffi/component_transform2d/factory.rs",
    "src/ffi/component_transform2d/builder.rs",
    "src/ffi/component_transform2d/position.rs",
    "src/ffi/component_transform2d/rotation.rs",
    "src/ffi/component_transform2d/scale.rs",
    "src/ffi/component_transform2d/direction.rs",
    "src/ffi/component_transform2d/matrix_ops.rs",
    // component_sprite_animator module
    "src/ffi/component_sprite_animator/factory.rs",
    "src/ffi/component_sprite_animator/playback.rs",
    // component_text module
    "src/ffi/component_text/factory.rs",
    "src/ffi/component_text/properties.rs",
    // component_sprite module
    "src/ffi/component_sprite/factory.rs",
    "src/ffi/component_sprite/builder.rs",
    "src/ffi/component_sprite/color.rs",
    "src/ffi/component_sprite/properties.rs",
    "src/ffi/component_sprite/texture.rs",
    // window module
    "src/ffi/window/lifecycle.rs",
    "src/ffi/window/properties.rs",
    // renderer module
    "src/ffi/renderer/lifecycle.rs",
    "src/ffi/renderer/draw/ffi.rs",
    "src/ffi/renderer/draw/mod.rs",
    "src/ffi/renderer/draw/helpers.rs",
    "src/ffi/renderer/draw/debug.rs",
    "src/ffi/renderer/draw/internal.rs",
    "src/ffi/renderer/texture.rs",
    "src/ffi/renderer/handles.rs",
    "src/ffi/renderer/text.rs",
    // renderer3d module
    "src/ffi/renderer3d/camera.rs",
    "src/ffi/renderer3d/environment.rs",
    "src/ffi/renderer3d/lighting.rs",
    "src/ffi/renderer3d/primitives.rs",
    // input module
    "src/ffi/input/keyboard.rs",
    "src/ffi/input/mouse.rs",
    "src/ffi/input/actions.rs",
    // collision (not yet split)
    "src/ffi/collision.rs",
    // scene module
    "src/ffi/scene.rs",
    "src/ffi/scene_loading.rs",
    "src/ffi/scene_transition.rs",
    // debug overlay
    "src/ffi/debug.rs",
    "src/ffi/debug/debugger_control.rs",
    "src/ffi/debug/debugger_runtime.rs",
    // error query module
    "src/ffi/error.rs",
    // engine config module
    "src/ffi/engine_config.rs",
    "src/ffi/engine_config/native.rs",
    // physics module
    "src/ffi/physics/physics2d/mod.rs",
    "src/ffi/physics/physics2d/lifecycle.rs",
    "src/ffi/physics/physics2d/simulation.rs",
    "src/ffi/physics/physics2d/bodies.rs",
    "src/ffi/physics/physics2d_ex.rs",
    "src/ffi/physics/physics2d_events.rs",
    "src/ffi/physics/physics2d_material.rs",
    "src/ffi/physics/physics3d/mod.rs",
    "src/ffi/physics/physics3d/lifecycle.rs",
    "src/ffi/physics/physics3d/simulation.rs",
    "src/ffi/physics/physics3d/bodies.rs",
    "src/ffi/physics/physics3d/access.rs",
    "src/ffi/physics/physics3d_material.rs",
    // animation module
    "src/ffi/animation/control.rs",
    "src/ffi/animation/controller.rs",
    "src/ffi/animation/tween.rs",
    "src/ffi/animation/skeletal.rs",
    "src/ffi/animation/events.rs",
    "src/ffi/animation/layer.rs",
    // network module
    "src/ffi/network/mod.rs",
    "src/ffi/network/lifecycle.rs",
    "src/ffi/network/stats.rs",
    "src/ffi/network/controls.rs",
    // providers module
    "src/ffi/providers.rs",
    // plugin module
    "src/ffi/plugin.rs",
    // audio module
    "src/ffi/audio/playback.rs",
    "src/ffi/audio/controls.rs",
    "src/ffi/audio/spatial.rs",
    // ui module
    "src/ffi/ui/manager.rs",
    "src/ffi/ui/node.rs",
    "src/ffi/ui/events.rs",
    "src/ffi/ui/widget.rs",
];

/// A single extracted FFI function signature.
struct FfiFunction {
    source_file: String,
    params: Vec<String>,
    return_type: String,
    is_unsafe: bool,
}

/// Generates `codegen/ffi_manifest.json` by scanning source files for
/// `#[no_mangle] pub [unsafe] extern "C" fn` signatures.
///
/// Returns the total number of functions extracted.
fn generate_ffi_manifest(manifest_dir: &str, output_path: &Path) -> usize {
    let generated_at = "build-time".to_string();

    let mut functions: BTreeMap<String, FfiFunction> = BTreeMap::new();

    for relative_path in FFI_SOURCE_FILES {
        let full_path = Path::new(manifest_dir).join(relative_path);
        let source = match fs::read_to_string(&full_path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Derive the display path (relative to src/)
        let display_path = relative_path.strip_prefix("src/").unwrap_or(relative_path);

        extract_ffi_functions(&source, display_path, &mut functions);
    }

    let total_count = functions.len();

    // Build JSON manually to avoid adding serde as a build-dependency
    let json = build_manifest_json(&generated_at, &functions, total_count);

    // Ensure output directory exists
    if let Some(parent) = output_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Err(e) = fs::write(output_path, json) {
        println!("cargo:warning=  Failed to write FFI manifest: {e}");
    }

    total_count
}

/// Extracts all `#[no_mangle] pub [unsafe] extern "C" fn` signatures from
/// a single source file. Handles multi-line parameter lists.
fn extract_ffi_functions(
    source: &str,
    display_path: &str,
    functions: &mut BTreeMap<String, FfiFunction>,
) {
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Look for #[no_mangle] attribute
        if line == "#[no_mangle]" {
            // The next non-attribute, non-empty line should be the fn signature.
            // Skip any additional attributes (e.g., #[allow(...)]) between
            // #[no_mangle] and the fn declaration.
            let mut fn_start = i + 1;
            while fn_start < lines.len() {
                let next = lines[fn_start].trim();
                if next.is_empty() || next.starts_with("#[") || next.starts_with("//") {
                    fn_start += 1;
                } else {
                    break;
                }
            }

            if fn_start >= lines.len() {
                i += 1;
                continue;
            }

            // Collect the full signature up to the opening brace `{`
            let mut sig = String::new();
            let mut j = fn_start;
            while j < lines.len() {
                let l = lines[j].trim();
                if !sig.is_empty() {
                    sig.push(' ');
                }
                sig.push_str(l);
                if l.contains('{') {
                    break;
                }
                j += 1;
            }

            if let Some(func) = parse_ffi_signature(&sig, display_path) {
                let name = extract_fn_name(&sig);
                if let Some(name) = name {
                    functions.insert(name, func);
                }
            }

            i = j + 1;
        } else {
            i += 1;
        }
    }
}

/// Parses a collected signature line into an FfiFunction.
///
/// Expected formats:
///   `pub extern "C" fn name(params) -> ReturnType {`
///   `pub unsafe extern "C" fn name(params) -> ReturnType {`
///   `pub extern "C" fn name(params) {`  (void return)
fn parse_ffi_signature(sig: &str, display_path: &str) -> Option<FfiFunction> {
    // Must contain `extern "C" fn`
    if !sig.contains("extern \"C\" fn") {
        return None;
    }

    let is_unsafe = sig.contains("pub unsafe extern");

    // Extract params: everything between the first `(` after `fn name` and
    // the matching `)`.
    let fn_keyword_pos = sig.find("extern \"C\" fn")?;
    let after_fn = &sig[fn_keyword_pos..];

    let paren_open = after_fn.find('(')?;
    let paren_close = find_matching_paren(after_fn, paren_open)?;

    let params_str = &after_fn[paren_open + 1..paren_close];
    let params = parse_params(params_str);

    // Extract return type: everything between `) ->` and `{`
    let after_paren = &after_fn[paren_close + 1..];
    let return_type = if let Some(arrow_pos) = after_paren.find("->") {
        let after_arrow = &after_paren[arrow_pos + 2..];
        let brace_pos = after_arrow.find('{').unwrap_or(after_arrow.len());
        after_arrow[..brace_pos].trim().to_string()
    } else {
        "()".to_string()
    };

    Some(FfiFunction {
        source_file: display_path.to_string(),
        params,
        return_type,
        is_unsafe,
    })
}

/// Extracts the function name from a signature string.
fn extract_fn_name(sig: &str) -> Option<String> {
    let marker = "extern \"C\" fn ";
    let pos = sig.find(marker)?;
    let after = &sig[pos + marker.len()..];
    let end = after.find('(')?;
    Some(after[..end].trim().to_string())
}

/// Finds the position of the closing `)` matching the opening `(` at
/// `open_pos`, handling nested parentheses.
fn find_matching_paren(s: &str, open_pos: usize) -> Option<usize> {
    let mut depth = 0;
    for (i, ch) in s[open_pos..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open_pos + i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Splits a parameter string like `"context_id: GoudContextId, x: f32"`
/// into individual parameter strings, trimming whitespace.
fn parse_params(params_str: &str) -> Vec<String> {
    let trimmed = params_str.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    // Split on commas, but respect nested angle brackets and parens
    let mut params = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;

    for ch in trimmed.chars() {
        match ch {
            '<' | '(' => {
                depth += 1;
                current.push(ch);
            }
            '>' | ')' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => {
                let p = current.trim().to_string();
                if !p.is_empty() {
                    params.push(p);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    let p = current.trim().to_string();
    if !p.is_empty() {
        params.push(p);
    }

    params
}

/// Builds the manifest JSON string manually (no serde dependency).
fn build_manifest_json(
    generated_at: &str,
    functions: &BTreeMap<String, FfiFunction>,
    total_count: usize,
) -> String {
    let mut json = String::with_capacity(64 * 1024);
    json.push_str("{\n");
    json.push_str(&format!("  \"generated_at\": \"{generated_at}\",\n"));
    json.push_str("  \"functions\": {\n");

    let entries: Vec<_> = functions.iter().collect();
    for (idx, (name, func)) in entries.iter().enumerate() {
        json.push_str(&format!("    \"{name}\": {{\n"));
        json.push_str(&format!(
            "      \"source_file\": \"{}\",\n",
            func.source_file
        ));

        // params array
        json.push_str("      \"params\": [");
        if func.params.is_empty() {
            json.push(']');
        } else {
            json.push('\n');
            for (pi, param) in func.params.iter().enumerate() {
                let escaped = json_escape(param);
                json.push_str(&format!("        \"{escaped}\""));
                if pi + 1 < func.params.len() {
                    json.push(',');
                }
                json.push('\n');
            }
            json.push_str("      ]");
        }
        json.push_str(",\n");

        let escaped_ret = json_escape(&func.return_type);
        json.push_str(&format!("      \"return_type\": \"{escaped_ret}\",\n"));
        json.push_str(&format!("      \"is_unsafe\": {}\n", func.is_unsafe));

        json.push_str("    }");
        if idx + 1 < entries.len() {
            json.push(',');
        }
        json.push('\n');
    }

    json.push_str("  },\n");
    json.push_str(&format!("  \"total_count\": {total_count}\n"));
    json.push_str("}\n");

    json
}

/// Escapes a string for JSON output.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\x20' => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

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

fn generate_c_header(manifest_dir: &str, codegen_dir: &Path) -> Result<usize, String> {
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

    let preamble = leading_preamble;
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
    ordered.extend(preamble);
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
