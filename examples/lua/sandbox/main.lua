-- main.lua
-- GoudEngine Sandbox for the Lua SDK.
--
-- Demonstrates:
--   - Window configuration (resize)
--   - Texture loading and sprite drawing
--   - Keyboard input (WASD/arrows to move, Escape to quit)
--   - Mode switching (1/2/3 keys)
--   - Basic game loop callbacks
--
-- Run with: cargo run -p lua-runner -- examples/lua/sandbox/main.lua
-- Or:       ./dev.sh --sdk lua --game sandbox

-- Asset base path (runner is launched from the repository root).
local ASSET_BASE = "examples/csharp/flappy_goud/assets/sprites/"

-- Window dimensions
local SANDBOX_WIDTH = 1280
local SANDBOX_HEIGHT = 720
local MOVE_SPEED = 220.0

-- Textures (populated in on_init)
local tex_bg = 0
local tex_sprite = 0

-- State
local player_x = 250.0
local player_y = 300.0
local angle = 0.0
local current_mode = 1 -- 1=2D, 2=3D placeholder, 3=Hybrid
local mode_names = { "2D", "3D", "Hybrid" }

--------------------------------------------------------------------
-- Callbacks
--------------------------------------------------------------------

function on_init()
    -- Resize the window from the default Lua runner size (288x512)
    -- to the sandbox dimensions.
    goud_game.set_window_size(SANDBOX_WIDTH, SANDBOX_HEIGHT)

    tex_bg = goud_game.texture_load(ASSET_BASE .. "background-day.png")
    tex_sprite = goud_game.texture_load(ASSET_BASE .. "bluebird-midflap.png")

    print("Sandbox initialized.")
    print("  WASD/Arrows: move sprite")
    print("  1/2/3: switch mode (2D / 3D / Hybrid)")
    print("  Escape: quit")
end

function on_update(dt)
    angle = angle + dt

    -- Quit
    if goud_game.input_key_just_pressed(key.escape) then
        goud_game.close()
        return
    end

    -- Mode switching
    if goud_game.input_key_just_pressed(key.digit1) then
        current_mode = 1
        print("Mode: " .. mode_names[current_mode])
    end
    if goud_game.input_key_just_pressed(key.digit2) then
        current_mode = 2
        print("Mode: " .. mode_names[current_mode])
    end
    if goud_game.input_key_just_pressed(key.digit3) then
        current_mode = 3
        print("Mode: " .. mode_names[current_mode])
    end

    -- Movement
    if goud_game.input_key_pressed(key.a) or goud_game.input_key_pressed(key.left) then
        player_x = player_x - MOVE_SPEED * dt
    end
    if goud_game.input_key_pressed(key.d) or goud_game.input_key_pressed(key.right) then
        player_x = player_x + MOVE_SPEED * dt
    end
    if goud_game.input_key_pressed(key.w) or goud_game.input_key_pressed(key.up) then
        player_y = player_y - MOVE_SPEED * dt
    end
    if goud_game.input_key_pressed(key.s) or goud_game.input_key_pressed(key.down) then
        player_y = player_y + MOVE_SPEED * dt
    end
end

function on_draw()
    if current_mode == 1 then
        -- 2D mode: background + sprite
        goud_game.window_clear(0.07, 0.10, 0.14, 1.0)
        goud_game.draw_sprite(tex_bg,
            SANDBOX_WIDTH / 2, SANDBOX_HEIGHT / 2,
            SANDBOX_WIDTH, SANDBOX_HEIGHT,
            0, 1, 1, 1, 1)
        goud_game.draw_sprite(tex_sprite,
            player_x, player_y,
            64, 64,
            angle * 0.25, 1, 1, 1, 1)

    elseif current_mode == 2 then
        -- 3D placeholder: dark background
        goud_game.window_clear(0.05, 0.08, 0.12, 1.0)
        -- No draw_quad available in Lua yet; just show sprite
        goud_game.draw_sprite(tex_sprite,
            SANDBOX_WIDTH / 2, SANDBOX_HEIGHT / 2,
            128, 128,
            angle * 0.5, 0.20, 0.55, 0.95, 0.80)

    elseif current_mode == 3 then
        -- Hybrid: dark overlay + sprite
        goud_game.window_clear(0.08, 0.17, 0.24, 1.0)
        goud_game.draw_sprite(tex_sprite,
            player_x, player_y,
            72, 72,
            angle * 0.25, 1, 1, 1, 1)
    end
end
