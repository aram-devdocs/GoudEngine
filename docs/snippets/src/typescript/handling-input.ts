while (!game.shouldClose()) {
  game.beginFrame(0.1, 0.1, 0.1, 1.0);

  if (game.isKeyPressed(256)) {
    break;
  }
  if (game.isKeyPressed(32) || game.isMouseButtonPressed(0)) {
  }
  if (game.isKeyPressed(263)) {
  }
  if (game.isKeyPressed(262)) {
  }

  game.endFrame();
}
