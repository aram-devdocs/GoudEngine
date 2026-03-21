use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

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
    "src/ffi/component_sprite/layering.rs",
    "src/ffi/component_sprite/properties.rs",
    "src/ffi/component_sprite/texture.rs",
    // window module
    "src/ffi/window/lifecycle.rs",
    "src/ffi/window/properties.rs",
    // renderer module
    "src/ffi/renderer/lifecycle.rs",
    "src/ffi/renderer/draw/batch.rs",
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
    "src/ffi/renderer3d/materials.rs",
    "src/ffi/renderer3d/postprocess.rs",
    "src/ffi/renderer3d/primitives.rs",
    "src/ffi/renderer3d/skinned.rs",
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
    "src/ffi/network/p2p.rs",
    "src/ffi/network/rollback.rs",
    "src/ffi/network/rpc.rs",
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
pub fn generate_ffi_manifest(manifest_dir: &str, output_path: &Path) -> usize {
    let generated_at = "build-time".to_string();

    let mut functions: BTreeMap<String, FfiFunction> = BTreeMap::new();

    for relative_path in FFI_SOURCE_FILES {
        let full_path = Path::new(manifest_dir).join(relative_path);
        let source = match fs::read_to_string(&full_path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let display_path = relative_path.strip_prefix("src/").unwrap_or(relative_path);
        extract_ffi_functions(&source, display_path, &mut functions);
    }

    let total_count = functions.len();
    let json = build_manifest_json(&generated_at, &functions, total_count);

    if let Some(parent) = output_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Err(e) = fs::write(output_path, json) {
        println!("cargo:warning=  Failed to write FFI manifest: {e}");
    }

    total_count
}

fn extract_ffi_functions(
    source: &str,
    display_path: &str,
    functions: &mut BTreeMap<String, FfiFunction>,
) {
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line == "#[no_mangle]" {
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
                if let Some(name) = extract_fn_name(&sig) {
                    functions.insert(name, func);
                }
            }

            i = j + 1;
        } else {
            i += 1;
        }
    }
}

fn parse_ffi_signature(sig: &str, display_path: &str) -> Option<FfiFunction> {
    if !sig.contains("extern \"C\" fn") {
        return None;
    }

    let is_unsafe = sig.contains("pub unsafe extern");
    let fn_keyword_pos = sig.find("extern \"C\" fn")?;
    let after_fn = &sig[fn_keyword_pos..];

    let paren_open = after_fn.find('(')?;
    let paren_close = find_matching_paren(after_fn, paren_open)?;
    let params = parse_params(&after_fn[paren_open + 1..paren_close]);

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

fn extract_fn_name(sig: &str) -> Option<String> {
    let marker = "extern \"C\" fn ";
    let pos = sig.find(marker)?;
    let after = &sig[pos + marker.len()..];
    let end = after.find('(')?;
    Some(after[..end].trim().to_string())
}

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

fn parse_params(params_str: &str) -> Vec<String> {
    let trimmed = params_str.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

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

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\x20' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}
