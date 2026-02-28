/**
 * Desktop Flappy Bird — Node.js + GoudEngine napi-rs SDK.
 *
 * Uses the real engine: GLFW window, OpenGL rendering, native input.
 * The game logic in game.ts calls game.drawQuad() and
 * game.isKeyJustPressed() directly — no terminal hacks, no adapter.
 *
 * Run:
 *   cd examples/typescript/flappy_bird
 *   npm install && npm run desktop
 *
 * Prerequisites: build the native addon —
 *   cd sdks/typescript && npm run build:native
 */

import { GoudGame } from '@goudengine/sdk';
import { FlappyBirdGame, SCREEN_W, SCREEN_H } from './game.js';

// ---------------------------------------------------------------------------
// Create engine + game
// ---------------------------------------------------------------------------

const game: InstanceType<typeof GoudGame> = new GoudGame({
  width: SCREEN_W,
  height: SCREEN_H,
  title: 'Flappy Goud',
});

const flappy = new FlappyBirdGame();
await flappy.init(game);

// ---------------------------------------------------------------------------
// Game loop — blocking, runs until window is closed or Escape is pressed
// ---------------------------------------------------------------------------

while (!game.shouldClose()) {
  game.beginFrame();
  flappy.update(game, game.deltaTime);
  game.endFrame();
}

game.destroy();
