from goud_engine import GoudGame, Key

game = GoudGame(800, 600, "My Game")

x = 400.0

while not game.should_close():
    game.begin_frame()
    dt = game.delta_time

    if game.is_key_just_pressed(Key.ESCAPE):
        game.close()

    if game.is_key_pressed(Key.LEFT):
        x -= 200 * dt
    if game.is_key_pressed(Key.RIGHT):
        x += 200 * dt

    game.end_frame()

game.destroy()
