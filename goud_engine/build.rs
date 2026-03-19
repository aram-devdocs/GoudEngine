//! Build script for the FFI manifest and canonical C header.

#[path = "build_support/mod.rs"]
mod build_support;

use std::env;
use std::path::Path;

use build_support::{generate_c_header, generate_ffi_manifest};

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

    let manifest_output = codegen_dir.join("ffi_manifest.json");
    let manifest_count = generate_ffi_manifest(&manifest_dir, &manifest_output);
    println!("cargo:rerun-if-changed=../codegen/");

    let header_count = match generate_c_header(&manifest_dir, &codegen_dir) {
        Ok(count) => count,
        Err(err) => panic!("failed to generate cbindgen header: {err}"),
    };

    // Auto-copy the generated C header to SDK include directories that consume it.
    let root_dir = Path::new(&manifest_dir).join("..");
    let canonical_header = codegen_dir.join("generated").join("goud_engine.h");
    let sdk_header_targets = [
        root_dir.join("sdks/swift/Sources/CGoudEngine/include/goud_engine.h"),
        root_dir.join("sdks/c/include/goud_engine.h"),
        root_dir.join("sdks/cpp/include/goud_engine.h"),
        root_dir.join("sdks/go/include/goud_engine.h"),
    ];

    if canonical_header.exists() {
        for target in &sdk_header_targets {
            if let Some(parent) = target.parent() {
                if parent.exists() {
                    if let Err(err) = std::fs::copy(&canonical_header, target) {
                        println!(
                            "cargo:warning=  Header copy failed: {} -> {}: {err}",
                            canonical_header.display(),
                            target.display()
                        );
                    }
                }
            }
        }
    }

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
