/**
 * Desktop Flappy Bird — Node.js + GoudEngine napi-rs SDK.
 *
 * Uses the real engine: GLFW window, OpenGL rendering, native input.
 * The game logic in game.ts calls game.drawSprite() and
 * game.isKeyPressed() directly — no terminal hacks, no adapter.
 *
 * Run:
 *   cd examples/typescript/flappy_bird
 *   npm install && npm run desktop
 *
 * Prerequisites: build the native addon —
 *   cd sdks/typescript && npm run build:native
 */

import { GoudGame } from 'goudengine';
import { FlappyBirdGame, SCREEN_WIDTH, SCREEN_HEIGHT, BASE_HEIGHT } from './game.js';

// ---------------------------------------------------------------------------
// Create engine + game
// ---------------------------------------------------------------------------

const game: InstanceType<typeof GoudGame> = new GoudGame({
  width: SCREEN_WIDTH,
  height: SCREEN_HEIGHT + BASE_HEIGHT,
  title: 'Flappy Goud',
});

const flappy = new FlappyBirdGame();
await flappy.init(game, '../../csharp/flappy_goud/assets/sprites');

// ---------------------------------------------------------------------------
// Game loop — blocking, runs until window is closed or Escape is pressed
// ---------------------------------------------------------------------------

while (!game.shouldClose()) {
  game.beginFrame(0.4, 0.7, 0.9, 1.0); // Sky blue background
  flappy.update(game, game.deltaTime);
  game.endFrame();
}

game.destroy();
