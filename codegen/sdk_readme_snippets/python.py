from goudengine import GoudGame, Key

game = GoudGame(800, 600, "My Game")
player_tex = game.load_texture("assets/player.png")

while not game.should_close():
    game.begin_frame()

    if game.is_key_just_pressed(Key.ESCAPE):
        game.close()

    game.draw_sprite(player_tex, 400, 300, 64, 64)
    game.end_frame()

game.destroy()
