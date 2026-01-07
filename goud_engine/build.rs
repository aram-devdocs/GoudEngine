fn main() {
    csbindgen::Builder::default()
        // FFI type definitions
        .input_extern_file("src/ffi/types.rs")
        .input_extern_file("src/core/math.rs")
        .input_extern_file("src/core/error.rs")
        // New FFI entry points
        .input_extern_file("src/ffi/context.rs")
        .input_extern_file("src/ffi/entity.rs")
        .input_extern_file("src/ffi/component.rs")
        // Configuration
        .csharp_dll_name("libgoud_engine")
        .csharp_class_accessibility("public")
        .generate_csharp_file("../sdks/GoudEngine/NativeMethods.g.cs")
        .unwrap();
}
