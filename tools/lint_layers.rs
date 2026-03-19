//! lint-layers: Enforces the GoudEngine 5-layer architecture by scanning for import violations.
//!
//! Layer mapping (ordered from lowest to highest):
//!   - Layer 1 (Foundation): core/
//!   - Layer 2 (Libs):       libs/
//!   - Layer 3 (Services):   ecs/, assets/
//!   - Layer 4 (Engine):     sdk/
//!   - Layer 5 (FFI):        ffi/, wasm/
//!
//! Rules:
//!   - Dependencies flow DOWN only (higher layers may import from lower layers)
//!   - Foundation MUST NOT import from any other layer
//!   - Libs MUST NOT import from Services, Engine, or FFI
//!   - Services MUST NOT import from Engine or FFI
//!   - Engine MUST NOT import from FFI
//!   - FFI MAY import from all other layers

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

/// Which architectural layer a source file belongs to.
///
/// Variants are ordered from lowest to highest layer. A layer may only
/// import from layers with a *lower* ordinal (i.e. declared earlier).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Layer {
    Foundation, // Layer 1: core/
    Libs,       // Layer 2: libs/
    Services,   // Layer 3: ecs/, assets/
    Engine,     // Layer 4: sdk/
    Ffi,        // Layer 5: ffi/, wasm/
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Layer::Foundation => write!(f, "Layer 1 (Foundation)"),
            Layer::Libs => write!(f, "Layer 2 (Libs)"),
            Layer::Services => write!(f, "Layer 3 (Services)"),
            Layer::Engine => write!(f, "Layer 4 (Engine/SDK)"),
            Layer::Ffi => write!(f, "Layer 5 (FFI/WASM)"),
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
    if relative_path.starts_with("core/") {
        Some(Layer::Foundation)
    } else if relative_path.starts_with("libs/") {
        Some(Layer::Libs)
    } else if relative_path.starts_with("ecs/") || relative_path.starts_with("assets/") {
        Some(Layer::Services)
    } else if relative_path.starts_with("sdk/")
        || relative_path.starts_with("rendering/")
        || relative_path.starts_with("component_ops/")
        || relative_path.starts_with("context_registry/")
    {
        Some(Layer::Engine)
    } else if relative_path.starts_with("ffi/") || relative_path.starts_with("wasm/") {
        Some(Layer::Ffi)
    } else {
        None
    }
}

/// Determines which layer a `use crate::` import targets.
fn classify_import(import_path: &str) -> Option<Layer> {
    // Check more specific prefixes first to avoid false matches.
    if import_path.contains("crate::ffi") || import_path.contains("crate::wasm") {
        Some(Layer::Ffi)
    } else if import_path.contains("crate::sdk")
        || import_path.contains("crate::rendering")
        || import_path.contains("crate::component_ops")
        || import_path.contains("crate::context_registry")
    {
        Some(Layer::Engine)
    } else if import_path.contains("crate::ecs") || import_path.contains("crate::assets") {
        Some(Layer::Services)
    } else if import_path.contains("crate::libs") {
        Some(Layer::Libs)
    } else if import_path.contains("crate::core") {
        Some(Layer::Foundation)
    } else {
        None
    }
}

/// Returns true if an import from `from` layer to `to` layer is a violation.
///
/// A lower layer must not import from a higher layer. Since the enum variants
/// are ordered from lowest to highest, `from < to` means an upward import.
fn is_violation(from: Layer, to: Layer) -> bool {
    from < to
}

/// Files that are exempt from upward-import checks because they are
/// in-process Lua bindings that intentionally import from the FFI layer.
const LUA_EXEMPT_PREFIXES: &[&str] = &[
    "sdk/game/instance/lua_bridge",
    "sdk/game/instance/lua_bindings/tools.g",
    "sdk/game/instance/lua_integration",
    "sdk/lua_runner",
];

/// Returns true if a file is exempt from layer violation checks.
fn is_exempt(relative_path: &str) -> bool {
    LUA_EXEMPT_PREFIXES
        .iter()
        .any(|prefix| relative_path.starts_with(prefix))
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
        let relative = match file_path.strip_prefix(&src_dir) {
            Ok(r) => r.to_string_lossy().to_string(),
            Err(_) => continue,
        };

        // Exempt Lua in-process binding files from the Engine->FFI check.
        if is_exempt(&relative) {
            continue;
        }

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
            "Layer rules: Foundation(1) <- Libs(2) <- Services(3) <- Engine(4) <- FFI(5). \
             Dependencies flow DOWN only."
        );
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── classify_file ──

    #[test]
    fn classify_file_foundation() {
        assert_eq!(classify_file("core/error.rs"), Some(Layer::Foundation));
        assert_eq!(classify_file("core/math/mod.rs"), Some(Layer::Foundation));
    }

    #[test]
    fn classify_file_libs() {
        assert_eq!(classify_file("libs/graphics/mod.rs"), Some(Layer::Libs));
        assert_eq!(classify_file("libs/platform/glfw.rs"), Some(Layer::Libs));
    }

    #[test]
    fn classify_file_services() {
        assert_eq!(classify_file("ecs/world.rs"), Some(Layer::Services));
        assert_eq!(classify_file("assets/server.rs"), Some(Layer::Services));
    }

    #[test]
    fn classify_file_engine() {
        assert_eq!(classify_file("sdk/game.rs"), Some(Layer::Engine));
    }

    #[test]
    fn classify_file_ffi() {
        assert_eq!(classify_file("ffi/renderer.rs"), Some(Layer::Ffi));
        assert_eq!(classify_file("wasm/bindings.rs"), Some(Layer::Ffi));
    }

    #[test]
    fn classify_file_unclassified() {
        assert_eq!(classify_file("lib.rs"), None);
    }

    // ── classify_import ──

    #[test]
    fn classify_import_foundation() {
        assert_eq!(
            classify_import("use crate::core::math::Vec2;"),
            Some(Layer::Foundation)
        );
    }

    #[test]
    fn classify_import_libs() {
        assert_eq!(
            classify_import("use crate::libs::graphics::backend::RenderBackend;"),
            Some(Layer::Libs)
        );
    }

    #[test]
    fn classify_import_services() {
        assert_eq!(
            classify_import("use crate::ecs::World;"),
            Some(Layer::Services)
        );
        assert_eq!(
            classify_import("use crate::assets::AssetServer;"),
            Some(Layer::Services)
        );
    }

    #[test]
    fn classify_import_engine() {
        assert_eq!(
            classify_import("use crate::sdk::GoudGame;"),
            Some(Layer::Engine)
        );
    }

    #[test]
    fn classify_import_ffi() {
        assert_eq!(
            classify_import("use crate::ffi::renderer;"),
            Some(Layer::Ffi)
        );
        assert_eq!(
            classify_import("use crate::wasm::bindings;"),
            Some(Layer::Ffi)
        );
    }

    #[test]
    fn classify_import_unknown() {
        assert_eq!(classify_import("use std::collections::HashMap;"), None);
    }

    // ── is_violation ──

    #[test]
    fn violation_foundation_imports_higher() {
        assert!(is_violation(Layer::Foundation, Layer::Libs));
        assert!(is_violation(Layer::Foundation, Layer::Services));
        assert!(is_violation(Layer::Foundation, Layer::Engine));
        assert!(is_violation(Layer::Foundation, Layer::Ffi));
    }

    #[test]
    fn violation_libs_imports_higher() {
        assert!(is_violation(Layer::Libs, Layer::Services));
        assert!(is_violation(Layer::Libs, Layer::Engine));
        assert!(is_violation(Layer::Libs, Layer::Ffi));
    }

    #[test]
    fn violation_services_imports_higher() {
        assert!(is_violation(Layer::Services, Layer::Engine));
        assert!(is_violation(Layer::Services, Layer::Ffi));
    }

    #[test]
    fn violation_engine_imports_ffi() {
        assert!(is_violation(Layer::Engine, Layer::Ffi));
    }

    #[test]
    fn allowed_same_layer() {
        assert!(!is_violation(Layer::Foundation, Layer::Foundation));
        assert!(!is_violation(Layer::Libs, Layer::Libs));
        assert!(!is_violation(Layer::Services, Layer::Services));
    }

    #[test]
    fn allowed_downward_imports() {
        assert!(!is_violation(Layer::Ffi, Layer::Foundation));
        assert!(!is_violation(Layer::Engine, Layer::Services));
        assert!(!is_violation(Layer::Services, Layer::Libs));
        assert!(!is_violation(Layer::Libs, Layer::Foundation));
    }
}
