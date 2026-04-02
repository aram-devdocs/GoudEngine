from goudengine import GoudGame, MouseButton

game = GoudGame(800, 600, "My Game")

while not game.should_close():
    game.begin_frame()

    if game.is_mouse_button_just_pressed(MouseButton.LEFT):
        pos = game.get_mouse_position()
        print(f"Click at ({pos.x:.0f}, {pos.y:.0f})")

    game.end_frame()

game.destroy()
