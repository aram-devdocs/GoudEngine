fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .input_extern_file("src/game.rs")
        .input_extern_file("src/game/platform/graphics/window.rs")
        .input_extern_file("src/game/platform/graphics/window/input_handler.rs")

        .input_extern_file("src/game/platform/graphics/gl_wrapper.rs")
        .csharp_dll_name("libgame")
        // .csharp_file_header("using System;\nusing System.Runtime.InteropServices;\n")
        .generate_csharp_file("../sample_net_app/NativeMethods.g.cs")
        .unwrap();

    // cbindgen::Builder::new()
    //     .with_crate(".")
    //     .generate()
    //     .expect("Unable to generate bindings")
    //     .write_to_file("bindings.h");
}
