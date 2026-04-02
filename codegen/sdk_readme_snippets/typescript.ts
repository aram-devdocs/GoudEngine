import { GoudGame } from "goudengine";

const game = new GoudGame(800, 600, "My Game");

while (!game.shouldClose()) {
  game.beginFrame(0.2, 0.2, 0.2, 1.0);
  // game logic here
  game.endFrame();
}

game.destroy();
