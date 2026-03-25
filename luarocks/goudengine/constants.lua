--- Engine constants for standalone Lua usage.
-- When using the embedded runtime, these are registered automatically
-- as global tables (key, mouse_button, etc.).
-- @module goudengine.constants

local M = {}

-- Key codes (mirrors Key enum)
M.key = {
    unknown = -1,
    space = 32,
    apostrophe = 39,
    comma = 44,
    minus = 45,
    period = 46,
    slash = 47,
    digit0 = 48,
    digit1 = 49,
    digit2 = 50,
    digit3 = 51,
    digit4 = 52,
    digit5 = 53,
    digit6 = 54,
    digit7 = 55,
    digit8 = 56,
    digit9 = 57,
    semicolon = 59,
    equal = 61,
    a = 65,
    b = 66,
    c = 67,
    d = 68,
    e = 69,
    f = 70,
    g = 71,
    h = 72,
    i = 73,
    j = 74,
    k = 75,
    l = 76,
    m = 77,
    n = 78,
    o = 79,
    p = 80,
    q = 81,
    r = 82,
    s = 83,
    t = 84,
    u = 85,
    v = 86,
    w = 87,
    x = 88,
    y = 89,
    z = 90,
    left_bracket = 91,
    backslash = 92,
    right_bracket = 93,
    grave_accent = 96,
    escape = 256,
    enter = 257,
    tab = 258,
    backspace = 259,
    insert = 260,
    delete = 261,
    right = 262,
    left = 263,
    down = 264,
    up = 265,
    page_up = 266,
    page_down = 267,
    home = 268,
    ["end"] = 269,
    caps_lock = 280,
    scroll_lock = 281,
    num_lock = 282,
    print_screen = 283,
    pause = 284,
    f1 = 290,
    f2 = 291,
    f3 = 292,
    f4 = 293,
    f5 = 294,
    f6 = 295,
    f7 = 296,
    f8 = 297,
    f9 = 298,
    f10 = 299,
    f11 = 300,
    f12 = 301,
    numpad0 = 320,
    numpad1 = 321,
    numpad2 = 322,
    numpad3 = 323,
    numpad4 = 324,
    numpad5 = 325,
    numpad6 = 326,
    numpad7 = 327,
    numpad8 = 328,
    numpad9 = 329,
    numpad_decimal = 330,
    numpad_divide = 331,
    numpad_multiply = 332,
    numpad_subtract = 333,
    numpad_add = 334,
    numpad_enter = 335,
    left_shift = 340,
    left_control = 341,
    left_alt = 342,
    left_super = 343,
    right_shift = 344,
    right_control = 345,
    right_alt = 346,
    right_super = 347,
}

-- Mouse buttons
M.mouse_button = {
    left = 0,
    right = 1,
    middle = 2,
    button4 = 3,
    button5 = 4,
    button6 = 5,
    button7 = 6,
    button8 = 7,
}

-- Overlay corner positions
M.overlay_corner = {
    top_left = 0,
    top_right = 1,
    bottom_left = 2,
    bottom_right = 3,
}

-- Debugger step kind
M.debugger_step_kind = {
    frame = 0,
    tick = 1,
}

-- Playback mode
M.playback_mode = {
    loop = 0,
    one_shot = 1,
}

-- Physics body type
M.body_type = {
    dynamic = 0,
    static = 1,
    kinematic = 2,
}

-- Collider shape type
M.shape_type = {
    box = 0,
    circle = 1,
}

-- Physics backend (2D)
M.physics_backend2_d = {
    default = 0,
    rapier = 1,
    simple = 2,
}

-- Render backend kind
M.render_backend_kind = {
    wgpu = 0,
    open_gl_legacy = 1,
}

-- Window backend kind
M.window_backend_kind = {
    winit = 0,
    glfw_legacy = 1,
}

-- Easing type
M.easing_type = {
    linear = 0,
    ease_in_quad = 1,
    ease_out_quad = 2,
    ease_in_out_quad = 3,
}

-- Network protocol
M.network_protocol = {
    udp = 0,
    web_socket = 1,
    tcp = 2,
}

-- Scene transition type
M.transition_type = {
    instant = 0,
    fade = 1,
    custom = 2,
}

-- Text alignment
M.text_alignment = {
    left = 0,
    center = 1,
    right = 2,
}

-- Text direction
M.text_direction = {
    auto = 0,
    ltr = 1,
    rtl = 2,
}

-- Blend mode
M.blend_mode = {
    override = 0,
    additive = 1,
}

-- Event payload type
M.event_payload_type = {
    none = 0,
    int = 1,
    float = 2,
    string = 3,
}

return M
