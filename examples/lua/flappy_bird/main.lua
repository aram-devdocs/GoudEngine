-- main.lua
-- Flappy Bird clone for the GoudEngine Lua SDK.
--
-- Defines three callbacks consumed by the Lua runner:
--   on_init()      -- load textures, reset state
--   on_update(dt)  -- game logic
--   on_draw()      -- rendering

-- Asset base path (runner is launched from the repository root).
local ASSET_BASE = "examples/lua/flappy_bird/assets/sprites/"

-- Textures (populated in on_init).
local tex_bg   = 0
local tex_base = 0
local tex_pipe = 0
local tex_bird = {} -- three animation frames
local tex_digits = {} -- 0-9 digit textures for score

-- Bird state
local bird_x = BIRD_START_X
local bird_y = BIRD_START_Y
local bird_velocity = 0
local bird_rotation = 0
local jump_timer = 0
local bird_frame = 1
local bird_anim_timer = 0
local ANIM_FRAME_DURATION = 0.1
local ROTATION_SMOOTHING = 0.03

-- Pipe state
local pipes = {}
local pipe_spawn_timer = 0

-- Score
local score = 0

-- Simple PRNG seed from system time
math.randomseed(os.time())

--------------------------------------------------------------------
-- Helpers
--------------------------------------------------------------------

local function reset_game()
    bird_x = BIRD_START_X
    bird_y = BIRD_START_Y
    bird_velocity = 0
    bird_rotation = 0
    jump_timer = 0
    bird_frame = 1
    bird_anim_timer = 0
    pipes = {}
    pipe_spawn_timer = 0
    score = 0
end

local function aabb_overlap(ax, ay, aw, ah, bx, by, bw, bh)
    return ax < bx + bw
       and ax + aw > bx
       and ay < by + bh
       and ay + ah > by
end

local function spawn_pipe()
    local gap_y = math.random(PIPE_GAP, SCREEN_HEIGHT - PIPE_GAP)
    table.insert(pipes, { x = SCREEN_WIDTH, gap_y = gap_y })
end

--------------------------------------------------------------------
-- Score rendering (digit sprites).
--------------------------------------------------------------------

local function draw_score()
    local s = tostring(score)
    local total_width = #s * SCORE_DIGIT_SPACING
    local start_x = (SCREEN_WIDTH - total_width) / 2
    local y = 50

    for i = 1, #s do
        local digit = tonumber(s:sub(i, i))
        local x = start_x + (i - 1) * SCORE_DIGIT_SPACING + SCORE_DIGIT_WIDTH / 2
        goud_game.draw_sprite(tex_digits[digit],
            x, y,
            SCORE_DIGIT_WIDTH, SCORE_DIGIT_HEIGHT,
            0, 1, 1, 1, 1)
    end
end

--------------------------------------------------------------------
-- Callbacks
--------------------------------------------------------------------

function on_init()
    tex_bg   = goud_game.texture_load(ASSET_BASE .. "background-day.png")
    tex_base = goud_game.texture_load(ASSET_BASE .. "base.png")
    tex_pipe = goud_game.texture_load(ASSET_BASE .. "pipe-green.png")

    tex_bird[1] = goud_game.texture_load(ASSET_BASE .. "yellowbird-downflap.png")
    tex_bird[2] = goud_game.texture_load(ASSET_BASE .. "yellowbird-midflap.png")
    tex_bird[3] = goud_game.texture_load(ASSET_BASE .. "yellowbird-upflap.png")

    for i = 0, 9 do
        tex_digits[i] = goud_game.texture_load(ASSET_BASE .. tostring(i) .. ".png")
    end

    reset_game()
end

function on_update(dt)
    if goud_game.input_key_just_pressed(key.escape) then
        goud_game.close()
        return
    end

    if goud_game.input_key_just_pressed(key.r) then
        reset_game()
        return
    end

    jump_timer = jump_timer + dt
    if goud_game.input_key_just_pressed(key.space) then
        if jump_timer >= JUMP_COOLDOWN then
            bird_velocity = JUMP_STRENGTH * TARGET_FPS
            jump_timer = 0
        end
    end

    bird_velocity = bird_velocity + GRAVITY * dt * TARGET_FPS
    bird_y = bird_y + bird_velocity * dt

    local target_rotation
    if bird_velocity < 0 then
        target_rotation = -0.4
    else
        target_rotation = math.min(bird_velocity / (TARGET_FPS * 2), 1.2)
    end
    bird_rotation = bird_rotation
        + (target_rotation - bird_rotation) * ROTATION_SMOOTHING * TARGET_FPS * dt

    bird_anim_timer = bird_anim_timer + dt
    if bird_anim_timer >= ANIM_FRAME_DURATION then
        bird_anim_timer = bird_anim_timer - ANIM_FRAME_DURATION
        bird_frame = (bird_frame % 3) + 1
    end

    if bird_y + BIRD_HEIGHT > SCREEN_HEIGHT then
        reset_game()
        return
    end

    if bird_y < 0 then
        reset_game()
        return
    end

    pipe_spawn_timer = pipe_spawn_timer + dt
    if pipe_spawn_timer >= PIPE_SPAWN_INTERVAL then
        pipe_spawn_timer = 0
        spawn_pipe()
    end

    local i = 1
    while i <= #pipes do
        local p = pipes[i]
        p.x = p.x - PIPE_SPEED * dt * TARGET_FPS

        local top_y = p.gap_y - PIPE_GAP - PIPE_IMAGE_HEIGHT
        if aabb_overlap(bird_x, bird_y, BIRD_WIDTH, BIRD_HEIGHT,
                        p.x, top_y, PIPE_IMAGE_WIDTH, PIPE_IMAGE_HEIGHT) then
            reset_game()
            return
        end

        local bot_y = p.gap_y + PIPE_GAP
        if aabb_overlap(bird_x, bird_y, BIRD_WIDTH, BIRD_HEIGHT,
                        p.x, bot_y, PIPE_IMAGE_WIDTH, PIPE_IMAGE_HEIGHT) then
            reset_game()
            return
        end

        if p.x + PIPE_COLLISION_WIDTH < 0 then
            table.remove(pipes, i)
            score = score + 1
        else
            i = i + 1
        end
    end
end

function on_draw()
    goud_game.draw_sprite(tex_bg,
        BACKGROUND_WIDTH / 2, BACKGROUND_HEIGHT / 2,
        BACKGROUND_WIDTH, BACKGROUND_HEIGHT,
        0, 1, 1, 1, 1)

    draw_score()

    for _, p in ipairs(pipes) do
        local top_y = p.gap_y - PIPE_GAP - PIPE_IMAGE_HEIGHT
        goud_game.draw_sprite(tex_pipe,
            p.x + PIPE_IMAGE_WIDTH / 2,
            top_y + PIPE_IMAGE_HEIGHT / 2,
            PIPE_IMAGE_WIDTH, PIPE_IMAGE_HEIGHT,
            math.pi, 1, 1, 1, 1)

        local bot_y = p.gap_y + PIPE_GAP
        goud_game.draw_sprite(tex_pipe,
            p.x + PIPE_IMAGE_WIDTH / 2,
            bot_y + PIPE_IMAGE_HEIGHT / 2,
            PIPE_IMAGE_WIDTH, PIPE_IMAGE_HEIGHT,
            0, 1, 1, 1, 1)
    end

    goud_game.draw_sprite(tex_bird[bird_frame],
        bird_x + BIRD_WIDTH / 2,
        bird_y + BIRD_HEIGHT / 2,
        BIRD_WIDTH, BIRD_HEIGHT,
        bird_rotation, 1, 1, 1, 1)

    goud_game.draw_sprite(tex_base,
        BASE_SPRITE_WIDTH / 2,
        SCREEN_HEIGHT + BASE_HEIGHT / 2,
        BASE_SPRITE_WIDTH, BASE_HEIGHT,
        0, 1, 1, 1, 1)
end
