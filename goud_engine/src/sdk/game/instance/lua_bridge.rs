//! Hand-written Lua bridge functions for FFI methods that accept string
//! parameters (`*const c_char`).  The auto-generated `tools.g.rs` skips
//! these because mlua's string type cannot be directly cast to a C pointer.
//!
//! Each bridge function receives a Lua `String`, converts it to a `CString`,
//! and calls the corresponding FFI export via its crate-internal path.

use mlua::prelude::*;

use crate::core::context_id::GoudContextId;
use crate::ffi::input::{goud_input_key_just_pressed, goud_input_key_pressed};
use crate::ffi::renderer::{
    goud_font_load, goud_renderer_begin, goud_renderer_draw_sprite, goud_renderer_draw_text,
    goud_renderer_end, goud_texture_load,
};
use crate::ffi::window::{
    goud_window_clear, goud_window_get_delta_time, goud_window_poll_events,
    goud_window_set_should_close, goud_window_should_close, goud_window_swap_buffers,
};

/// Registers hand-written Lua bridge functions on the `goud_game` global table.
///
/// These supplement the auto-generated tool methods by providing wrappers for
/// FFI functions that accept string/pointer parameters.
pub(crate) fn register_lua_bridge(lua: &Lua, ctx_id: u64) -> LuaResult<()> {
    let ctx = GoudContextId::from_raw(ctx_id);
    let globals = lua.globals();
    let tbl = globals
        .get::<LuaTable>("goud_game")
        .or_else(|_| lua.create_table())?;

    // -- texture_load(path: string) -> texture_handle (u64) -------------------
    tbl.set(
        "texture_load",
        lua.create_function(move |_, path: mlua::String| {
            let c_path = std::ffi::CString::new(path.as_bytes().to_vec())
                .map_err(|e| mlua::Error::runtime(format!("invalid path string: {e}")))?;
            // SAFETY: c_path is a valid null-terminated C string.
            let handle = unsafe { goud_texture_load(ctx, c_path.as_ptr()) };
            Ok(handle)
        })?,
    )?;

    // -- font_load(path: string) -> font_handle (u64) -------------------------
    tbl.set(
        "font_load",
        lua.create_function(move |_, path: mlua::String| {
            let c_path = std::ffi::CString::new(path.as_bytes().to_vec())
                .map_err(|e| mlua::Error::runtime(format!("invalid font path: {e}")))?;
            // SAFETY: c_path is a valid null-terminated C string.
            let handle = unsafe { goud_font_load(ctx, c_path.as_ptr()) };
            Ok(handle)
        })?,
    )?;

    // -- draw_text(font, text, x, y, size, r, g, b, a) -----------------------
    // Wraps goud_renderer_draw_text with sensible defaults for alignment,
    // max_width, line_spacing, and direction.
    #[allow(clippy::too_many_arguments)]
    tbl.set(
        "draw_text",
        lua.create_function(
            move |_,
                  (font, text, x, y, size, r, g, b, a): (
                i64,
                mlua::String,
                f64,
                f64,
                f64,
                f64,
                f64,
                f64,
                f64,
            )| {
                let c_text = std::ffi::CString::new(text.as_bytes().to_vec())
                    .map_err(|e| mlua::Error::runtime(format!("invalid text string: {e}")))?;
                // SAFETY: c_text is a valid null-terminated C string.
                let result = unsafe {
                    goud_renderer_draw_text(
                        ctx,
                        font as u64,
                        c_text.as_ptr(),
                        x as f32,
                        y as f32,
                        size as f32,
                        0,   // alignment: left
                        0.0, // max_width: unlimited
                        1.2, // line_spacing: default
                        0,   // direction: left-to-right
                        r as f32,
                        g as f32,
                        b as f32,
                        a as f32,
                    )
                };
                Ok(result)
            },
        )?,
    )?;

    // -- delta_time() -> f32 --------------------------------------------------
    tbl.set(
        "delta_time",
        lua.create_function(move |_, _: ()| Ok(goud_window_get_delta_time(ctx) as f64))?,
    )?;

    // -- draw_sprite(texture, x, y, w, h, rotation, r, g, b, a) -> bool ------
    #[allow(clippy::too_many_arguments)]
    tbl.set(
        "draw_sprite",
        lua.create_function(
            move |_,
                  (texture, x, y, w, h, rotation, r, g, b, a): (
                i64,
                f64,
                f64,
                f64,
                f64,
                f64,
                f64,
                f64,
                f64,
                f64,
            )| {
                Ok(goud_renderer_draw_sprite(
                    ctx,
                    texture as u64,
                    x as f32,
                    y as f32,
                    w as f32,
                    h as f32,
                    rotation as f32,
                    r as f32,
                    g as f32,
                    b as f32,
                    a as f32,
                ))
            },
        )?,
    )?;

    // -- input_key_pressed(key_code) -> bool ----------------------------------
    tbl.set(
        "input_key_pressed",
        lua.create_function(move |_, key: i64| Ok(goud_input_key_pressed(ctx, key as i32)))?,
    )?;

    // -- input_key_just_pressed(key_code) -> bool -----------------------------
    tbl.set(
        "input_key_just_pressed",
        lua.create_function(move |_, key: i64| Ok(goud_input_key_just_pressed(ctx, key as i32)))?,
    )?;

    // -- renderer_begin() -> bool ---------------------------------------------
    tbl.set(
        "renderer_begin",
        lua.create_function(move |_, _: ()| Ok(goud_renderer_begin(ctx)))?,
    )?;

    // -- renderer_end() -> bool -----------------------------------------------
    tbl.set(
        "renderer_end",
        lua.create_function(move |_, _: ()| Ok(goud_renderer_end(ctx)))?,
    )?;

    // -- window_poll_events() -> dt (f64) -------------------------------------
    tbl.set(
        "window_poll_events",
        lua.create_function(move |_, _: ()| Ok(goud_window_poll_events(ctx) as f64))?,
    )?;

    // -- window_clear(r, g, b, a) ---------------------------------------------
    tbl.set(
        "window_clear",
        lua.create_function(move |_, (r, g, b, a): (f64, f64, f64, f64)| {
            goud_window_clear(ctx, r as f32, g as f32, b as f32, a as f32);
            Ok(())
        })?,
    )?;

    // -- window_swap_buffers() ------------------------------------------------
    tbl.set(
        "window_swap_buffers",
        lua.create_function(move |_, _: ()| {
            goud_window_swap_buffers(ctx);
            Ok(())
        })?,
    )?;

    // -- window_should_close() -> bool ----------------------------------------
    // (Also in tools.g.rs as should_close, but added here for naming consistency)
    tbl.set(
        "window_should_close",
        lua.create_function(move |_, _: ()| Ok(goud_window_should_close(ctx)))?,
    )?;

    // -- set_should_close(flag) -----------------------------------------------
    tbl.set(
        "set_should_close",
        lua.create_function(move |_, flag: bool| {
            goud_window_set_should_close(ctx, flag);
            Ok(())
        })?,
    )?;

    // -- close() -- convenience: sets should_close to true --------------------
    tbl.set(
        "close",
        lua.create_function(move |_, _: ()| {
            goud_window_set_should_close(ctx, true);
            Ok(())
        })?,
    )?;

    globals.set("goud_game", tbl)?;
    Ok(())
}
