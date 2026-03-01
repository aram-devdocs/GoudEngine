//! # Build Script: C# Bindings + FFI Manifest
//!
//! This build script generates:
//!
//! - **C# Bindings** (`NativeMethods.g.cs`) via csbindgen
//! - **FFI Manifest** (`codegen/ffi_manifest.json`) via regex-free source parsing
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
//! #   - ../sdks/csharp/NativeMethods.g.cs (C# bindings)
//! #   - ../codegen/ffi_manifest.json      (auto-extracted FFI surface)
//! ```

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        return;
    }

    println!("cargo:rerun-if-changed=src/ffi/");
    println!("cargo:rerun-if-changed=src/core/math.rs");
    println!("cargo:rerun-if-changed=src/core/error.rs");
    println!("cargo:rerun-if-changed=src/core/types.rs");
    println!("cargo:rerun-if-changed=src/core/context_registry.rs");
    println!("cargo:rerun-if-changed=src/core/component_ops.rs");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let sdks_dir = Path::new(&manifest_dir).join("..").join("sdks");
    let codegen_dir = Path::new(&manifest_dir).join("..").join("codegen");

    // =========================================================================
    // Generate C# Bindings (csbindgen)
    // =========================================================================
    let csharp_output = sdks_dir.join("csharp").join("NativeMethods.g.cs");
    let csharp_result = generate_csharp_bindings(&csharp_output);

    // =========================================================================
    // Generate FFI Manifest (codegen/ffi_manifest.json)
    // =========================================================================
    let manifest_output = codegen_dir.join("ffi_manifest.json");
    let manifest_count = generate_ffi_manifest(&manifest_dir, &manifest_output);
    println!("cargo:rerun-if-changed=../codegen/");

    let status = if csharp_result { "OK" } else { "FAILED" };
    let short_path = csharp_output.display().to_string();
    let short_path = short_path
        .split("sdks/")
        .last()
        .unwrap_or("NativeMethods.g.cs");

    println!("cargo:warning=");
    println!("cargo:warning=GoudEngine SDK Binding Generation");
    println!("cargo:warning=  C# Bindings [{status}]: {short_path}");
    println!(
        "cargo:warning=  FFI Manifest [OK]: codegen/ffi_manifest.json ({manifest_count} functions)"
    );
    println!("cargo:warning=");
}

/// Generates C# bindings using csbindgen.
fn generate_csharp_bindings(output_path: &Path) -> bool {
    let result = csbindgen::Builder::default()
        // FFI type definitions
        .input_extern_file("src/ffi/types.rs")
        .input_extern_file("src/core/math.rs")
        .input_extern_file("src/core/error.rs")
        .input_extern_file("src/core/types.rs")
        .input_extern_file("src/core/context_registry.rs")
        .input_extern_file("src/core/component_ops.rs")
        // FFI entry points
        .input_extern_file("src/ffi/context.rs")
        .input_extern_file("src/ffi/entity.rs")
        .input_extern_file("src/ffi/component.rs")
        .input_extern_file("src/ffi/component_transform2d.rs")
        .input_extern_file("src/ffi/component_sprite.rs")
        .input_extern_file("src/ffi/window.rs")
        .input_extern_file("src/ffi/renderer.rs")
        .input_extern_file("src/ffi/renderer3d.rs")
        .input_extern_file("src/ffi/input.rs")
        .input_extern_file("src/ffi/collision.rs")
        // Configuration
        .csharp_dll_name("libgoud_engine")
        .csharp_class_accessibility("public")
        .generate_csharp_file(output_path);

    match result {
        Ok(_) => {
            println!(
                "cargo:warning=  Generated C# bindings: {}",
                output_path.display()
            );
            true
        }
        Err(e) => {
            println!("cargo:warning=  Failed to generate C# bindings: {}", e);
            false
        }
    }
}

// =============================================================================
// FFI Manifest Generation
// =============================================================================

/// Files to scan for FFI functions (same set csbindgen processes).
const FFI_SOURCE_FILES: &[&str] = &[
    "src/ffi/types.rs",
    "src/core/math.rs",
    "src/core/error.rs",
    "src/core/types.rs",
    "src/core/context_registry.rs",
    "src/core/component_ops.rs",
    "src/ffi/context.rs",
    "src/ffi/entity.rs",
    "src/ffi/component.rs",
    "src/ffi/component_transform2d.rs",
    "src/ffi/component_sprite.rs",
    "src/ffi/window.rs",
    "src/ffi/renderer.rs",
    "src/ffi/renderer3d.rs",
    "src/ffi/input.rs",
    "src/ffi/collision.rs",
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
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string());
    let generated_at = build_timestamp();

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
    let json = build_manifest_json(&version, &generated_at, &functions, total_count);

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
    version: &str,
    generated_at: &str,
    functions: &BTreeMap<String, FfiFunction>,
    total_count: usize,
) -> String {
    let mut json = String::with_capacity(64 * 1024);
    json.push_str("{\n");
    json.push_str(&format!("  \"version\": \"{version}\",\n"));
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

/// Returns an ISO 8601 timestamp string for the current build time.
///
/// Uses a simple approach: reads from environment or falls back to a
/// placeholder that still sorts correctly.
fn build_timestamp() -> String {
    // Try SOURCE_DATE_EPOCH for reproducible builds, otherwise use a
    // compile-time-constant placeholder (build.rs runs at build time,
    // but std::time is available).
    use std::time::{SystemTime, UNIX_EPOCH};

    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Convert epoch seconds to ISO 8601 manually (no chrono dep)
    let (year, month, day, hour, min, sec) = epoch_to_datetime(secs);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Converts Unix epoch seconds to (year, month, day, hour, minute, second).
/// Simplified algorithm -- correct for dates 1970-2099.
fn epoch_to_datetime(epoch: u64) -> (u64, u64, u64, u64, u64, u64) {
    let sec = epoch % 60;
    let min = (epoch / 60) % 60;
    let hour = (epoch / 3600) % 24;
    let mut days = epoch / 86400;

    let mut year = 1970u64;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let leap = is_leap_year(year);
    let month_days: [u64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 0u64;
    for (i, &md) in month_days.iter().enumerate() {
        if days < md {
            month = i as u64 + 1;
            break;
        }
        days -= md;
    }

    let day = days + 1;
    (year, month, day, hour, min, sec)
}

fn is_leap_year(y: u64) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}
