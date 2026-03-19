package = "goudengine"
version = "scm-1"
source = {
    url = "git://github.com/aram-devdocs/GoudEngine.git"
}
description = {
    summary = "Lua bindings for GoudEngine game engine",
    detailed = [[
        GoudEngine is a cross-platform game engine with an Entity Component System,
        2D/3D rendering, physics, audio, and more. This package provides Lua bindings
        via the embedded mlua runtime.
    ]],
    homepage = "https://github.com/aram-devdocs/GoudEngine",
    license = "MIT"
}
dependencies = {
    "lua >= 5.4"
}
build = {
    type = "make",
    build_variables = {
        CFLAGS = "$(CFLAGS)",
    },
    install_variables = {
        INST_LIBDIR = "$(LIBDIR)",
        INST_LUADIR = "$(LUADIR)",
    },
}
