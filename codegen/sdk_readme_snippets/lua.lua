function on_init()
    tex = goud_game.texture_load("assets/player.png")
    x, y = 400, 300
end

function on_update(dt)
    if goud_game.input_key_just_pressed(key.escape) then
        goud_game.close()
    end
end

function on_draw()
    goud_game.window_clear(0.2, 0.2, 0.3, 1.0)
    goud_game.draw_sprite(tex, x, y, 64, 64, 0, 1, 1, 1, 1)
end
