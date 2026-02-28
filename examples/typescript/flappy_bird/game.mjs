/**
 * Shared Flappy Bird game logic for GoudEngine TypeScript SDK.
 *
 * Platform-agnostic: receives an IGoudGame instance and uses its
 * rendering (drawQuad/drawSprite) and input APIs directly.
 * No Canvas 2D, no DOM, no adapter pattern.
 *
 * Both desktop (napi-rs/GLFW) and web (wasm/wgpu) entry points
 * import this module and pass their platform's GoudGame.
 */

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

export const SCREEN_W = 288;
export const SCREEN_H = 512;
export const GROUND_H = 80;
export const PLAY_H   = SCREEN_H - GROUND_H;

const BIRD_X     = 72;
const BIRD_SIZE  = 24;
const PIPE_W     = 52;
const PIPE_GAP   = 110;
const PIPE_CAP_H = 20;
const PIPE_CAP_W = PIPE_W + 6;

const GRAVITY    = 1176;
const JUMP_VEL   = -420;
const MAX_VEL    = 600;
const PIPE_SPEED = 120;
const PIPE_SPAWN = 1.5;
const PIPE_MIN_Y = 60;

// GLFW key codes (matching the Key enum in the TypeScript SDK)
const KEY_SPACE  = 32;
const KEY_ESCAPE = 256;

const IDLE = 0;
const PLAY = 1;
const DEAD = 2;

// Colors (IColor-compatible objects)
const SKY        = { r: 0.31, g: 0.75, b: 0.93, a: 1 };
const BIRD_COLOR = { r: 0.97, g: 0.86, b: 0.43, a: 1 };
const BIRD_EYE   = { r: 1, g: 1, b: 1, a: 1 };
const BIRD_PUPIL = { r: 0.1, g: 0.1, b: 0.1, a: 1 };
const BIRD_BEAK  = { r: 0.91, g: 0.30, b: 0.24, a: 1 };
const PIPE_BODY  = { r: 0.42, g: 0.75, b: 0.19, a: 1 };
const PIPE_CAP   = { r: 0.35, g: 0.62, b: 0.12, a: 1 };
const GROUND_TOP = { r: 0.13, g: 0.55, b: 0.13, a: 1 };
const GROUND_COL = { r: 0.87, g: 0.72, b: 0.53, a: 1 };
const OVERLAY    = { r: 0, g: 0, b: 0, a: 0.3 };
const WHITE_DIM  = { r: 1, g: 1, b: 1, a: 0.7 };
const RED_DIM    = { r: 1, g: 0.3, b: 0.3, a: 0.8 };
const SCORE_DOT  = { r: 1, g: 1, b: 1, a: 0.9 };

// ---------------------------------------------------------------------------
// FlappyBirdGame
// ---------------------------------------------------------------------------

export class FlappyBirdGame {
  constructor() {
    this.birdY     = PLAY_H / 2;
    this.velocity  = 0;
    this.rotation  = 0;
    this.state     = IDLE;
    this.score     = 0;
    this.best      = 0;
    this.pipes     = [];
    this.pipeTimer = 0;
    this.bobTimer  = 0;
  }

  /** Load textures or other assets. Currently a no-op (quad-only rendering). */
  async init(_game) {}

  /**
   * Per-frame update: handle input, advance physics, draw everything.
   * @param {IGoudGame} game  The engine instance (rendering + input).
   * @param {number}    dt    Seconds since last frame.
   */
  update(game, dt) {
    dt = Math.min(dt, 0.05);

    const jump =
      game.isKeyJustPressed(KEY_SPACE) ||
      game.isMouseButtonJustPressed(0);

    // ---- State machine ----

    if (this.state === IDLE) {
      this.bobTimer += dt;
      this.birdY = PLAY_H / 2 + Math.sin(this.bobTimer * 3) * 8;
      this.rotation = 0;
      if (jump) {
        this.state = PLAY;
        this.velocity = JUMP_VEL;
      }
    } else if (this.state === PLAY) {
      this._updatePlay(dt, jump);
    } else {
      this._updateDead(dt, jump);
    }

    if (game.isKeyJustPressed(KEY_ESCAPE)) game.close();

    this._draw(game);
  }

  // -- internal: play state -------------------------------------------------

  _updatePlay(dt, jump) {
    if (jump) this.velocity = JUMP_VEL;

    this.velocity = Math.min(this.velocity + GRAVITY * dt, MAX_VEL);
    this.birdY += this.velocity * dt;

    const target = this.velocity > 0
      ? Math.min(this.velocity / MAX_VEL * 1.57, 0.78)
      : Math.max(this.velocity / 400 * 0.52, -0.52);
    this.rotation += (target - this.rotation) * 0.12;

    this.pipeTimer += dt;
    if (this.pipeTimer >= PIPE_SPAWN) {
      this._spawnPipe();
      this.pipeTimer -= PIPE_SPAWN;
    }

    for (let i = this.pipes.length - 1; i >= 0; i--) {
      const p = this.pipes[i];
      p.x -= PIPE_SPEED * dt;
      if (!p.scored && p.x + PIPE_W < BIRD_X) {
        p.scored = true;
        this.score++;
      }
      if (p.x + PIPE_W < -10) this.pipes.splice(i, 1);
    }

    if (this.birdY + BIRD_SIZE / 2 >= PLAY_H || this.birdY - BIRD_SIZE / 2 <= 0) {
      this._die();
      return;
    }

    const bx = BIRD_X - BIRD_SIZE / 2;
    const by = this.birdY - BIRD_SIZE / 2;
    for (const p of this.pipes) {
      if (
        _aabb(bx, by, BIRD_SIZE, BIRD_SIZE, p.x, 0, PIPE_W, p.gapTop) ||
        _aabb(bx, by, BIRD_SIZE, BIRD_SIZE, p.x, p.gapTop + PIPE_GAP, PIPE_W, PLAY_H)
      ) {
        this._die();
        return;
      }
    }
  }

  // -- internal: dead state -------------------------------------------------

  _updateDead(dt, jump) {
    if (this.birdY + BIRD_SIZE / 2 < PLAY_H) {
      this.velocity = Math.min(this.velocity + GRAVITY * dt, MAX_VEL);
      this.birdY += this.velocity * dt;
      this.rotation = Math.min(this.rotation + dt * 8, 1.57);
    }
    if (jump) this._restart();
  }

  // -- internal: drawing ----------------------------------------------------

  _draw(game) {
    // Sky
    game.drawQuad(0, 0, SCREEN_W, PLAY_H, SKY);

    // Pipes
    for (const p of this.pipes) {
      const capX = p.x - 3;
      game.drawQuad(p.x, 0, PIPE_W, Math.max(0, p.gapTop - PIPE_CAP_H), PIPE_BODY);
      game.drawQuad(capX, p.gapTop - PIPE_CAP_H, PIPE_CAP_W, PIPE_CAP_H, PIPE_CAP);
      const botY = p.gapTop + PIPE_GAP;
      game.drawQuad(capX, botY, PIPE_CAP_W, PIPE_CAP_H, PIPE_CAP);
      game.drawQuad(p.x, botY + PIPE_CAP_H, PIPE_W, PLAY_H - botY - PIPE_CAP_H, PIPE_BODY);
    }

    // Bird body
    const bx = BIRD_X - BIRD_SIZE / 2;
    const by = this.birdY - BIRD_SIZE / 2;
    game.drawQuad(bx, by, BIRD_SIZE, BIRD_SIZE, BIRD_COLOR);
    // Eye
    game.drawQuad(bx + 14, by + 4, 6, 6, BIRD_EYE);
    game.drawQuad(bx + 16, by + 5, 3, 4, BIRD_PUPIL);
    // Beak
    game.drawQuad(bx + BIRD_SIZE - 2, by + 10, 8, 6, BIRD_BEAK);

    // Ground
    game.drawQuad(0, PLAY_H, SCREEN_W, 4, GROUND_TOP);
    game.drawQuad(0, PLAY_H + 4, SCREEN_W, GROUND_H - 4, GROUND_COL);

    // Score dots (one white dot per point, up to 30)
    const dots = Math.min(this.score, 30);
    for (let i = 0; i < dots; i++) {
      game.drawQuad(8 + i * 9, 8, 7, 7, SCORE_DOT);
    }

    // State overlays
    if (this.state === IDLE) {
      game.drawQuad(0, 0, SCREEN_W, PLAY_H, OVERLAY);
      game.drawQuad(SCREEN_W / 2 - 50, PLAY_H / 2 - 6, 100, 12, WHITE_DIM);
    }
    if (this.state === DEAD) {
      game.drawQuad(0, 0, SCREEN_W, PLAY_H, OVERLAY);
      game.drawQuad(SCREEN_W / 2 - 50, PLAY_H / 2 - 6, 100, 12, RED_DIM);
    }
  }

  // -- internal helpers -----------------------------------------------------

  _spawnPipe() {
    const min = PIPE_MIN_Y;
    const max = PLAY_H - PIPE_GAP - PIPE_MIN_Y;
    const gapTop = min + Math.random() * (max - min);
    this.pipes.push({ x: SCREEN_W, gapTop, scored: false });
  }

  _die() {
    this.state = DEAD;
    if (this.score > this.best) this.best = this.score;
  }

  _restart() {
    this.pipes     = [];
    this.velocity  = JUMP_VEL;
    this.rotation  = 0;
    this.score     = 0;
    this.pipeTimer = 0;
    this.bobTimer  = 0;
    this.state     = PLAY;
    this.birdY     = PLAY_H / 2;
  }
}

function _aabb(ax, ay, aw, ah, bx, by, bw, bh) {
  return ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by;
}
