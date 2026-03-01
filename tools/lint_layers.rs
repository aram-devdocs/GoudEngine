//! lint-layers: Enforces the GoudEngine layer architecture by scanning for import violations.
//!
//! Layer mapping:
//!   - Layer 1 (Core):   libs/, core/, ecs/, assets/
//!   - Layer 2 (Engine): src/sdk/
//!   - Layer 3 (FFI):    src/ffi/, src/wasm/
//!
//! Rules:
//!   - Layer 2 MUST NOT import from Layer 3 (sdk/ must not use ffi:: or wasm::)
//!   - Layer 1 MUST NOT import from Layer 2 or Layer 3
//!   - Layer 3 MAY import from Layer 2 and Layer 1 (that is fine)

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

/// Which architectural layer a source file belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layer {
    Core,   // Layer 1: libs/, core/, ecs/, assets/
    Engine, // Layer 2: sdk/
    Ffi,    // Layer 3: ffi/, wasm/
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Layer::Core => write!(f, "Layer 1 (Core)"),
            Layer::Engine => write!(f, "Layer 2 (Engine/SDK)"),
            Layer::Ffi => write!(f, "Layer 3 (FFI/WASM)"),
        }
    }
}

/// A single detected violation.
struct Violation {
    file: PathBuf,
    line_number: usize,
    line_content: String,
    from_layer: Layer,
    to_layer: Layer,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "  {}:{}: {} -> {} import\n    {}",
            self.file.display(),
            self.line_number,
            self.from_layer,
            self.to_layer,
            self.line_content.trim(),
        )
    }
}

/// Determines which layer a file belongs to based on its path relative to `goud_engine/src/`.
fn classify_file(relative_path: &str) -> Option<Layer> {
    if relative_path.starts_with("sdk/") {
        Some(Layer::Engine)
    } else if relative_path.starts_with("ffi/") || relative_path.starts_with("wasm/") {
        Some(Layer::Ffi)
    } else if relative_path.starts_with("core/")
        || relative_path.starts_with("ecs/")
        || relative_path.starts_with("assets/")
        || relative_path.starts_with("libs/")
    {
        Some(Layer::Core)
    } else {
        // Files directly in src/ (like lib.rs) are not layer-classified
        None
    }
}

/// Determines which layer a `use crate::` import targets.
fn classify_import(import_path: &str) -> Option<Layer> {
    // Extract the first module segment after `crate::` or `super::` resolution.
    // We look for known module prefixes.
    if import_path.contains("crate::ffi") || import_path.contains("crate::wasm") {
        Some(Layer::Ffi)
    } else if import_path.contains("crate::sdk") {
        Some(Layer::Engine)
    } else if import_path.contains("crate::core")
        || import_path.contains("crate::ecs")
        || import_path.contains("crate::assets")
        || import_path.contains("crate::libs")
    {
        Some(Layer::Core)
    } else {
        None
    }
}

/// Returns true if an import from `from` to `to` is a violation.
fn is_violation(from: Layer, to: Layer) -> bool {
    match (from, to) {
        // Layer 1 must not import Layer 2 or Layer 3
        (Layer::Core, Layer::Engine) | (Layer::Core, Layer::Ffi) => true,
        // Layer 2 must not import Layer 3
        (Layer::Engine, Layer::Ffi) => true,
        // Everything else is allowed
        _ => false,
    }
}

/// Recursively collects all `.rs` files under a directory.
fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

fn main() {
    let src_dir = PathBuf::from("goud_engine/src");

    if !src_dir.is_dir() {
        eprintln!(
            "Error: {} not found. Run this tool from the workspace root.",
            src_dir.display()
        );
        process::exit(2);
    }

    let mut rs_files = Vec::new();
    collect_rs_files(&src_dir, &mut rs_files);
    rs_files.sort();

    let mut violations = Vec::new();

    for file_path in &rs_files {
        // Get path relative to goud_engine/src/
        let relative = match file_path.strip_prefix(&src_dir) {
            Ok(r) => r.to_string_lossy().to_string(),
            Err(_) => continue,
        };

        let from_layer = match classify_file(&relative) {
            Some(l) => l,
            None => continue,
        };

        let contents = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: could not read {}: {}", file_path.display(), e);
                continue;
            }
        };

        for (line_idx, line) in contents.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }

            // Only look at `use crate::` statements
            if !trimmed.contains("use crate::") {
                continue;
            }

            if let Some(to_layer) = classify_import(trimmed) {
                if is_violation(from_layer, to_layer) {
                    violations.push(Violation {
                        file: file_path.clone(),
                        line_number: line_idx + 1,
                        line_content: line.to_string(),
                        from_layer,
                        to_layer,
                    });
                }
            }
        }
    }

    if violations.is_empty() {
        println!("lint-layers: No layer violations found.");
        process::exit(0);
    } else {
        eprintln!(
            "lint-layers: Found {} layer violation(s):\n",
            violations.len()
        );
        for v in &violations {
            eprintln!("{}\n", v);
        }
        eprintln!(
            "Layer rules: Core(1) <- Engine/SDK(2) <- FFI/WASM(3). Dependencies flow DOWN only."
        );
        process::exit(1);
    }
}
