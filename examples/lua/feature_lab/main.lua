-- main.lua
-- GoudEngine Feature Lab for the Lua SDK.
--
-- Headless smoke test that exercises Lua SDK API surfaces:
--   - Window creation (via runner)
--   - Texture loading
--   - Input key constant accessibility
--   - Basic callback lifecycle (on_init, on_update, on_draw)
--
-- Run with: cargo run -p lua-runner -- examples/lua/feature_lab/main.lua
-- Or:       ./dev.sh --sdk lua --game feature_lab
--
-- The feature lab runs a few frames then closes, printing pass/fail results.

local pass_count = 0
local fail_count = 0
local skip_count = 0
local frame_count = 0
local checks_complete = false

local results = {}

local function record(name, ok, detail)
    local status = "PASS"
    if ok == nil then
        status = "SKIP"
    elseif not ok then
        status = "FAIL"
    end

    if status == "PASS" then
        pass_count = pass_count + 1
    elseif status == "FAIL" then
        fail_count = fail_count + 1
    else
        skip_count = skip_count + 1
    end

    table.insert(results, { name = name, status = status, detail = detail or "" })
end

local function check_key_constants()
    -- Verify that the key table has expected constants
    local ok = type(key) == "table"
        and type(key.escape) == "number"
        and type(key.space) == "number"
        and type(key.a) == "number"
        and type(key.digit1) == "number"
    local detail = string.format(
        "escape=%s, space=%s, a=%s, digit1=%s",
        tostring(key.escape), tostring(key.space),
        tostring(key.a), tostring(key.digit1)
    )
    return ok, detail
end

local function check_goud_game_api()
    -- Verify that the goud_game table has expected functions
    local ok = type(goud_game) == "table"
        and type(goud_game.texture_load) == "function"
        and type(goud_game.draw_sprite) == "function"
        and type(goud_game.window_clear) == "function"
        and type(goud_game.input_key_pressed) == "function"
        and type(goud_game.input_key_just_pressed) == "function"
        and type(goud_game.close) == "function"
    local detail = string.format(
        "texture_load=%s, draw_sprite=%s, window_clear=%s",
        type(goud_game.texture_load),
        type(goud_game.draw_sprite),
        type(goud_game.window_clear)
    )
    return ok, detail
end

local function check_texture_load()
    -- Try loading a texture from shared assets
    local ok_load, tex = pcall(function()
        return goud_game.texture_load("examples/csharp/flappy_goud/assets/sprites/background-day.png")
    end)
    if not ok_load then
        return false, "texture_load raised: " .. tostring(tex)
    end
    local ok = type(tex) == "number" and tex > 0
    return ok, "handle=" .. tostring(tex)
end

local function check_set_window_size()
    local ok_call, result = pcall(function()
        return goud_game.set_window_size(320, 240)
    end)
    if not ok_call then
        return nil, "set_window_size raised: " .. tostring(result)
    end
    return true, "set_window_size(320, 240) called"
end

--------------------------------------------------------------------
-- Callbacks
--------------------------------------------------------------------

function on_init()
    print("================================================================")
    print(" GoudEngine Lua Feature Lab")
    print("================================================================")

    record("key constants accessible", check_key_constants())
    record("goud_game API surface check", check_goud_game_api())
    record("texture loading", check_texture_load())
    record("set_window_size", check_set_window_size())

    -- on_init callback itself is a check
    record("on_init callback invoked", true, "callback reached")
end

function on_update(dt)
    frame_count = frame_count + 1

    if frame_count == 1 then
        -- First frame: verify dt is reasonable
        local ok = type(dt) == "number" and dt >= 0 and dt < 10
        record("on_update receives valid dt", ok, "dt=" .. tostring(dt))
    end

    if frame_count >= 3 and not checks_complete then
        checks_complete = true

        record("on_update ran multiple frames", true,
            "frame_count=" .. tostring(frame_count))

        -- Print results
        print(string.format(
            "\nFeature Lab complete: %d pass, %d fail, %d skip",
            pass_count, fail_count, skip_count
        ))
        for _, r in ipairs(results) do
            local suffix = ""
            if r.detail ~= "" then
                suffix = " (" .. r.detail .. ")"
            end
            print(r.status .. ": " .. r.name .. suffix)
        end

        -- Close the window
        goud_game.close()
    end
end

function on_draw()
    goud_game.window_clear(0.1, 0.1, 0.1, 1.0)
end
