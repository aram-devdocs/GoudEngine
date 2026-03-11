from goud_engine import GoudGame, Key

game = GoudGame(800, 600, "My Game")

while not game.should_close():
    game.begin_frame()

    if game.is_key_just_pressed(Key.ESCAPE):
        game.close()

    game.end_frame()

game.destroy()
