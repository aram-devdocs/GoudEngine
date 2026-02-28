//! # C# Binding Generator
//!
//! This build script generates C# SDK bindings via csbindgen:
//!
//! - **C# Bindings** (`NativeMethods.g.cs`) via csbindgen
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
//! ```

use std::env;
use std::path::Path;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        return;
    }

    println!("cargo:rerun-if-changed=src/ffi/");
    println!("cargo:rerun-if-changed=src/core/math.rs");
    println!("cargo:rerun-if-changed=src/core/error.rs");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let sdks_dir = Path::new(&manifest_dir).join("..").join("sdks");

    // =========================================================================
    // Generate C# Bindings (csbindgen)
    // =========================================================================
    let csharp_output = sdks_dir.join("csharp").join("NativeMethods.g.cs");
    let csharp_result = generate_csharp_bindings(&csharp_output);

    let status = if csharp_result { "OK" } else { "FAILED" };
    let short_path = csharp_output.display().to_string();
    let short_path = short_path
        .split("sdks/")
        .last()
        .unwrap_or("NativeMethods.g.cs");

    println!("cargo:warning=");
    println!("cargo:warning=GoudEngine SDK Binding Generation");
    println!("cargo:warning=  C# Bindings [{status}]: {short_path}");
    println!("cargo:warning=");
}

/// Generates C# bindings using csbindgen.
fn generate_csharp_bindings(output_path: &Path) -> bool {
    let result = csbindgen::Builder::default()
        // FFI type definitions
        .input_extern_file("src/ffi/types.rs")
        .input_extern_file("src/core/math.rs")
        .input_extern_file("src/core/error.rs")
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
