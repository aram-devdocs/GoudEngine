fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .input_extern_file("src/game.rs")
        .input_extern_file("src/types.rs")
        .input_extern_file("src/game/platform/graphics/window.rs")
        .input_extern_file("src/game/platform/graphics/window/input_handler.rs")
        .input_extern_file("src/game/platform/graphics/gl_wrapper.rs")
        .csharp_dll_name("libgoud_engine")
        .csharp_class_accessibility("public")
        .generate_csharp_file("../flappy_goud/NativeMethods.g.cs")
        .unwrap();

}
