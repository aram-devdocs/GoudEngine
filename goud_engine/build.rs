fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .input_extern_file("src/game.rs")
        .input_extern_file("src/types.rs")
        .input_extern_file("src/sdk.rs")
        .input_extern_file("src/ffi_privates.rs")

        .input_extern_file("src/libs/platform/window/mod.rs")
        .input_extern_file("src/libs/platform/window/input_handler.rs")
        .input_extern_file("src/libs/graphics/renderer2d.rs")
        .input_extern_file("src/libs/graphics/renderer3d.rs")
        .input_extern_file("src/libs/graphics/renderer.rs")


        .csharp_dll_name("libgoud_engine")
        .csharp_class_accessibility("public")
        .generate_csharp_file("../sdks/GoudEngine/NativeMethods.g.cs")
        .unwrap();
}
