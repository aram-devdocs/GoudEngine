fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .input_extern_file("src/game.rs")
        .input_extern_file("src/types.rs")
        .input_extern_file("src/sdk.rs")
        .input_extern_file("src/ffi_privates.rs")

        .input_extern_file("src/libs/platform/graphics/window.rs")
        .input_extern_file("src/libs/platform/graphics/window/input_handler.rs")
        .input_extern_file("src/libs/platform/graphics/rendering/renderer/renderer2d.rs")
        .csharp_dll_name("libgoud_engine")
        .csharp_class_accessibility("public")
        .generate_csharp_file("../GoudEngine/NativeMethods.g.cs")
        .unwrap();
}
