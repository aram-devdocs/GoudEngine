fn main() {
    csbindgen::Builder::default()
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
