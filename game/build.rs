fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .input_extern_file("src/game.rs")
        .input_extern_file("src/game/platform/graphics/window.rs")
        .input_extern_file("src/game/platform/graphics/gl_wrapper.rs")

        .csharp_dll_name("libgame")
        .generate_csharp_file("../sample_net_app/NativeMethods.g.cs")
        .unwrap();
}
