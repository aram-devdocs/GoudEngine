/**
 * Shared Flappy Bird game logic for GoudEngine TypeScript SDK.
 *
 * Platform-agnostic: receives an IGoudGame instance and uses its
 * drawSprite/loadTexture/input APIs directly. Matches the C#/Python
 * Flappy Bird implementations exactly (same constants, physics, rendering).
 *
 * Both desktop (napi-rs/GLFW) and web (wasm) entry points
 * import this module and pass their platform's GoudGame.
 */

import type { IGoudGame } from 'goudengine';

// ---------------------------------------------------------------------------
// Constants (matching GameConstants.cs exactly)
// ---------------------------------------------------------------------------

export const SCREEN_WIDTH = 288;
export const SCREEN_HEIGHT = 512;
export const BASE_HEIGHT = 112;

const TARGET_FPS = 120;
const GRAVITY = 9.8;
const JUMP_STRENGTH = -3.5;
const JUMP_COOLDOWN = 0.30;
const PIPE_SPEED = 1.0;
const PIPE_SPAWN_INTERVAL = 1.5;
const PIPE_COLLISION_WIDTH = 60; // GameConstants.PipeWidth — off-screen/scoring
const PIPE_GAP = 100;

const BIRD_WIDTH = 34;
const BIRD_HEIGHT = 24;
const PIPE_IMG_WIDTH = 52;  // Sprite width — rendering + collision AABB
const PIPE_IMG_HEIGHT = 320;
const ROTATION_SMOOTHING = 0.03;

const BG_WIDTH = 288;
const BG_HEIGHT = 512;
const BASE_WIDTH = 336;

const DIGIT_WIDTH = 24;
const DIGIT_HEIGHT = 36;
const DIGIT_SPACING = 30;

const KEY_SPACE = 32;
const KEY_ESCAPE = 256;
const KEY_R = 82;
// Canonical decimal fixture bytes are documented in examples/shared/flappy_audio_fixture.txt.
const FLAP_WAV_BYTES = new Uint8Array([
  82, 73, 70, 70, 116, 0, 0, 0, 87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0, 1, 0, 64, 31,
  0, 0, 64, 31, 0, 0, 1, 0, 8, 0, 100, 97, 116, 97, 80, 0, 0, 0, 127, 182, 191, 147, 87, 61, 88, 146,
  186, 177, 127, 78, 70, 108, 160, 183, 159, 110, 75, 83, 126, 168, 175, 142, 98, 79, 99, 141, 169, 162,
  127, 92, 87, 114, 150, 165, 149, 115, 92, 98, 126, 154, 158, 136, 108, 96, 109, 135, 153, 148, 126, 106,
  104, 119, 140, 148, 138, 120, 109, 112, 126, 139, 141, 131, 119, 114, 120, 130, 136, 134, 126, 121, 121,
  125, 129, 130, 128, 126, 126, 127,
]);

const RESET_WAV_BYTES = new Uint8Array([
  82, 73, 70, 70, 156, 0, 0, 0, 87, 65, 86, 69, 102, 109, 116, 32, 16, 0, 0, 0, 1, 0, 1, 0, 64, 31,
  0, 0, 64, 31, 0, 0, 1, 0, 8, 0, 100, 97, 116, 97, 120, 0, 0, 0, 127, 143, 158, 171, 181, 188, 192, 192,
  189, 182, 172, 160, 146, 131, 117, 103, 91, 81, 74, 69, 68, 70, 76, 84, 94, 105, 118, 131, 143, 154, 164,
  171, 175, 177, 176, 172, 166, 158, 148, 137, 127, 116, 106, 97, 91, 86, 84, 84, 87, 91, 98, 106, 114, 123,
  132, 141, 148, 154, 158, 161, 161, 160, 156, 152, 146, 139, 131, 124, 117, 111, 106, 102, 100, 100, 100, 103,
  106, 110, 116, 121, 126, 132, 136, 140, 143, 145, 146, 145, 144, 142, 139, 135, 131, 128, 124, 121, 119, 117,
  115, 115, 115, 116, 118, 119, 121, 123, 125, 127, 128, 130, 130, 131, 130, 130, 129, 129, 128, 127, 127, 127,
]);

export function flappyBirdAssetManifest(assetBase: string): string[] {
  const digits = Array.from({ length: 10 }, (_, index) => `${assetBase}/${index}.png`);
  return [
    `${assetBase}/background-day.png`,
    `${assetBase}/base.png`,
    `${assetBase}/pipe-green.png`,
    `${assetBase}/bluebird-downflap.png`,
    `${assetBase}/bluebird-midflap.png`,
    `${assetBase}/bluebird-upflap.png`,
    ...digits,
  ];
}

// ---------------------------------------------------------------------------
// Movement — gravity, jump with cooldown, rotation smoothing
// ---------------------------------------------------------------------------

class Movement {
  velocity = 0;
  rotation = 0;
  private jumpCooldownTimer = 0;

  constructor(
    private readonly gravity: number,
    private readonly jumpStrength: number,
  ) {}

  applyGravity(dt: number): void {
    this.velocity += this.gravity * dt * TARGET_FPS;
    this.jumpCooldownTimer -= Math.max(0, dt);
  }

  tryJump(): boolean {
    if (this.jumpCooldownTimer <= 0) {
      this.velocity = 0;
      this.velocity = this.jumpStrength * TARGET_FPS;
      this.jumpCooldownTimer = JUMP_COOLDOWN;
      return true;
    }
    return false;
  }

  updatePosition(positionY: number, dt: number): number {
    positionY += this.velocity * dt;
    const targetRotation = Math.max(-45, Math.min(this.velocity * 3, 45));
    this.rotation += (targetRotation - this.rotation) * ROTATION_SMOOTHING;
    return positionY;
  }

  reset(): void {
    this.velocity = 0;
    this.rotation = 0;
    this.jumpCooldownTimer = 0;
  }
}

// ---------------------------------------------------------------------------
// BirdAnimator — 3-frame sprite animation at 0.1s/frame
// ---------------------------------------------------------------------------

class BirdAnimator {
  private frames: number[] = [];
  private currentFrame = 0;
  private animationTime = 0;
  private readonly frameDuration = 0.1;
  private drawX = 0;
  private drawY = 0;
  private drawRotation = 0;

  async init(game: IGoudGame, assetBase: string): Promise<void> {
    this.frames = [
      await game.loadTexture(`${assetBase}/bluebird-downflap.png`),
      await game.loadTexture(`${assetBase}/bluebird-midflap.png`),
      await game.loadTexture(`${assetBase}/bluebird-upflap.png`),
    ];
  }

  update(dt: number, x: number, y: number, rotation: number): void {
    this.animationTime += dt;
    if (this.animationTime >= this.frameDuration) {
      this.currentFrame = (this.currentFrame + 1) % this.frames.length;
      this.animationTime = 0;
    }
    this.drawX = x;
    this.drawY = y;
    this.drawRotation = rotation;
  }

  draw(game: IGoudGame): void {
    if (this.frames.length === 0) return;
    game.drawSprite(
      this.frames[this.currentFrame],
      this.drawX + BIRD_WIDTH / 2,
      this.drawY + BIRD_HEIGHT / 2,
      BIRD_WIDTH,
      BIRD_HEIGHT,
      this.drawRotation * Math.PI / 180, // degrees to radians
    );
  }

  reset(): void {
    this.currentFrame = 0;
    this.animationTime = 0;
  }
}

// ---------------------------------------------------------------------------
// Bird — owns Movement + BirdAnimator, position at (72, 256)
// ---------------------------------------------------------------------------

class Bird {
  x = SCREEN_WIDTH / 4;   // 72
  y = SCREEN_HEIGHT / 2;  // 256
  private movement = new Movement(GRAVITY, JUMP_STRENGTH);
  private animator = new BirdAnimator();

  async init(game: IGoudGame, assetBase: string): Promise<void> {
    await this.animator.init(game, assetBase);
  }

  update(_game: IGoudGame, dt: number, jumpPressed: boolean): boolean {
    const didFlap = jumpPressed ? this.movement.tryJump() : false;
    this.movement.applyGravity(dt);
    this.y = this.movement.updatePosition(this.y, dt);
    this.animator.update(dt, this.x, this.y, this.movement.rotation);
    return didFlap;
  }

  draw(game: IGoudGame): void {
    this.animator.draw(game);
  }

  reset(): void {
    this.x = SCREEN_WIDTH / 4;
    this.y = SCREEN_HEIGHT / 2;
    this.movement.reset();
    this.animator.reset();
  }
}

// ---------------------------------------------------------------------------
// Pipe — gap randomization, AABB collision, off-screen detection
// ---------------------------------------------------------------------------

class Pipe {
  x = SCREEN_WIDTH;
  readonly gapY: number;

  constructor() {
    this.gapY = PIPE_GAP + Math.random() * (SCREEN_HEIGHT - 2 * PIPE_GAP);
  }

  get topPipeY(): number {
    return this.gapY - PIPE_GAP - PIPE_IMG_HEIGHT;
  }

  get bottomPipeY(): number {
    return this.gapY + PIPE_GAP;
  }

  update(dt: number): void {
    this.x -= PIPE_SPEED * dt * TARGET_FPS;
  }

  draw(game: IGoudGame, pipeTexture: number): void {
    // Top pipe (rotated pi radians)
    game.drawSprite(
      pipeTexture,
      this.x + PIPE_IMG_WIDTH / 2,
      this.topPipeY + PIPE_IMG_HEIGHT / 2,
      PIPE_IMG_WIDTH, PIPE_IMG_HEIGHT,
      Math.PI,
    );
    // Bottom pipe (no rotation)
    game.drawSprite(
      pipeTexture,
      this.x + PIPE_IMG_WIDTH / 2,
      this.bottomPipeY + PIPE_IMG_HEIGHT / 2,
      PIPE_IMG_WIDTH, PIPE_IMG_HEIGHT,
      0,
    );
  }

  isOffScreen(): boolean {
    return this.x + PIPE_COLLISION_WIDTH < 0;
  }

  collidesWithBird(birdX: number, birdY: number): boolean {
    return (
      aabb(birdX, birdY, BIRD_WIDTH, BIRD_HEIGHT,
        this.x, this.topPipeY, PIPE_IMG_WIDTH, PIPE_IMG_HEIGHT) ||
      aabb(birdX, birdY, BIRD_WIDTH, BIRD_HEIGHT,
        this.x, this.bottomPipeY, PIPE_IMG_WIDTH, PIPE_IMG_HEIGHT)
    );
  }
}

// ---------------------------------------------------------------------------
// ScoreCounter — digit sprites at xOffset=SCREEN_WIDTH/2-30, yOffset=50
// ---------------------------------------------------------------------------

class ScoreCounter {
  score = 0;
  private digits: number[] = [];
  private readonly xOffset = SCREEN_WIDTH / 2 - 30;
  private readonly yOffset = 50;

  async init(game: IGoudGame, assetBase: string): Promise<void> {
    for (let i = 0; i < 10; i++) {
      this.digits.push(await game.loadTexture(`${assetBase}/${i}.png`));
    }
  }

  increment(): void {
    this.score++;
  }

  resetScore(): void {
    this.score = 0;
  }

  draw(game: IGoudGame): void {
    const s = this.score.toString();
    for (let i = 0; i < s.length; i++) {
      const d = parseInt(s[i], 10);
      game.drawSprite(
        this.digits[d],
        this.xOffset + i * DIGIT_SPACING + DIGIT_WIDTH / 2,
        this.yOffset + DIGIT_HEIGHT / 2,
        DIGIT_WIDTH, DIGIT_HEIGHT,
      );
    }
  }
}

// ---------------------------------------------------------------------------
// FlappyBirdGame — exported class matching C# GameManager
// ---------------------------------------------------------------------------

export class FlappyBirdGame {
  private bird = new Bird();
  private pipes: Pipe[] = [];
  private pipeSpawnTimer = 0;
  private scoreCounter = new ScoreCounter();

  private bgTexture = 0;
  private baseTexture = 0;
  private pipeTexture = 0;
  private audioActivated = false;

  async init(game: IGoudGame, assetBase: string): Promise<void> {
    await game.preload(flappyBirdAssetManifest(assetBase));
    this.bgTexture = await game.loadTexture(`${assetBase}/background-day.png`);
    this.baseTexture = await game.loadTexture(`${assetBase}/base.png`);
    this.pipeTexture = await game.loadTexture(`${assetBase}/pipe-green.png`);
    await this.bird.init(game, assetBase);
    await this.scoreCounter.init(game, assetBase);
  }

  update(game: IGoudGame, dt: number): void {
    const flapInput = game.isKeyPressed(KEY_SPACE) || game.isMouseButtonPressed(0);

    if (game.isKeyPressed(KEY_ESCAPE)) {
      game.close();
      return;
    }
    if (game.isKeyPressed(KEY_R)) {
      this.ensureAudioActivated(game);
      this.resetGame(game);
      return;
    }

    if (flapInput) {
      this.ensureAudioActivated(game);
    }

    const didFlap = this.bird.update(game, dt, flapInput);
    if (didFlap) {
      game.audioPlay(FLAP_WAV_BYTES);
    }

    // Ground collision
    if (this.bird.y + BIRD_HEIGHT > SCREEN_HEIGHT) {
      this.resetGame(game);
      return;
    }
    // Ceiling collision
    if (this.bird.y < 0) {
      this.resetGame(game);
      return;
    }

    // Pipe updates and collision
    for (const pipe of this.pipes) {
      pipe.update(dt);
      if (pipe.collidesWithBird(this.bird.x, this.bird.y)) {
        this.resetGame(game);
        return;
      }
    }

    // Spawn pipes
    this.pipeSpawnTimer += dt;
    if (this.pipeSpawnTimer > PIPE_SPAWN_INTERVAL) {
      this.pipeSpawnTimer = 0;
      this.pipes.push(new Pipe());
    }

    // Remove off-screen pipes and increment score
    this.pipes = this.pipes.filter(pipe => {
      if (pipe.isOffScreen()) {
        this.scoreCounter.increment();
        return false;
      }
      return true;
    });

    this.draw(game);
  }

  private draw(game: IGoudGame): void {
    // Layer 0: Background
    game.drawSprite(this.bgTexture,
      BG_WIDTH / 2, BG_HEIGHT / 2, BG_WIDTH, BG_HEIGHT);

    // Layer 1: Score
    this.scoreCounter.draw(game);

    // Layer 2: Pipes
    for (const pipe of this.pipes) {
      pipe.draw(game, this.pipeTexture);
    }

    // Layer 3: Bird
    this.bird.draw(game);

    // Layer 4: Base/ground
    game.drawSprite(this.baseTexture,
      BASE_WIDTH / 2, SCREEN_HEIGHT + BASE_HEIGHT / 2,
      BASE_WIDTH, BASE_HEIGHT);
  }

  private ensureAudioActivated(game: IGoudGame): void {
    if (!this.audioActivated) {
      game.audioActivate();
      this.audioActivated = true;
    }
  }

  private resetGame(game: IGoudGame): void {
    if (this.audioActivated) {
      game.audioPlay(RESET_WAV_BYTES);
    }
    this.bird.reset();
    this.pipes = [];
    this.scoreCounter.resetScore();
    this.pipeSpawnTimer = 0;
  }
}

// ---------------------------------------------------------------------------
// AABB collision helper
// ---------------------------------------------------------------------------

function aabb(
  x1: number, y1: number, w1: number, h1: number,
  x2: number, y2: number, w2: number, h2: number,
): boolean {
  return x1 < x2 + w2 && x1 + w1 > x2 && y1 < y2 + h2 && y1 + h1 > y2;
}
