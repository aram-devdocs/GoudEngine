export type SandboxMode = '2D' | '3D' | 'Hybrid';
export type SandboxTarget = 'desktop' | 'web';

export interface SandboxGame {
  shouldClose(): boolean;
  close(): void;
  deltaTime: number;
  drawSprite(texture: number, x: number, y: number, width: number, height: number, rotation?: number): void;
  drawQuad(x: number, y: number, width: number, height: number, color?: { r: number; g: number; b: number; a: number }): void;
  drawText(fontHandle: number, text: string, x: number, y: number, fontSize?: number, alignment?: number, maxWidth?: number, lineSpacing?: number, direction?: number, color?: { r: number; g: number; b: number; a: number }): boolean;
  loadTexture(path: string): Promise<number>;
  loadFont(path: string): Promise<number>;
  getMousePosition(): { x: number; y: number };
  isKeyPressed(key: number): boolean;
  isKeyJustPressed(key: number): boolean;
  getRenderCapabilities(): any;
  getPhysicsCapabilities(): any;
  getAudioCapabilities(): any;
  getInputCapabilities(): any;
  getNetworkCapabilities(): any;
  createCube(textureId: number, width: number, height: number, depth: number): number;
  createPlane(textureId: number, width: number, depth: number): number;
  setObjectPosition(objectId: number, x: number, y: number, z: number): boolean;
  setObjectRotation(objectId: number, x: number, y: number, z: number): boolean;
  setCameraPosition3D(x: number, y: number, z: number): boolean;
  setCameraRotation3D(pitch: number, yaw: number, roll: number): boolean;
  configureGrid(enabled: boolean, size: number, divisions: number): boolean;
  render3D(): boolean;
  audioActivate?(): number;
  audioPlay?(data: Uint8Array): number;
}

export interface SandboxUi {
  update(): void;
  render(): void;
  nodeCount(): number;
}

export interface NetworkStateLike {
  role: string;
  peerCount: number;
  detail: string;
  update(dt: number): void;
  destroy(): void;
}

const KEY_ESCAPE = 256;
const KEY_SPACE = 32;
const KEY_LEFT = 263;
const KEY_RIGHT = 262;
const KEY_UP = 265;
const KEY_DOWN = 264;
const KEY_A = 65;
const KEY_D = 68;
const KEY_W = 87;
const KEY_S = 83;
const KEY_1 = 49;
const KEY_2 = 50;
const KEY_3 = 51;

const WINDOW_WIDTH = 1280;
const WINDOW_HEIGHT = 720;
const MOVE_SPEED = 220;

export interface SandboxAssets {
  background: string;
  sprite: string;
  accentSprite: string;
  texture3d: string;
  font: string;
  audio: Uint8Array;
}

export async function loadSandboxAssets(target: SandboxTarget): Promise<SandboxAssets> {
  const root = target === 'web' ? '/examples/shared/sandbox' : '../../shared/sandbox';
  const audioPath = `${root}/audio/sandbox-tone.wav`;
  let audio: Uint8Array;
  if (target === 'web') {
    audio = new Uint8Array(await (await fetch(audioPath)).arrayBuffer());
  } else {
    const { readFile } = await import('node:fs/promises');
    audio = new Uint8Array(await readFile(audioPath));
  }
  return {
    background: `${root}/sprites/background-day.png`,
    sprite: `${root}/sprites/yellowbird-midflap.png`,
    accentSprite: `${root}/sprites/pipe-green.png`,
    texture3d: `${root}/textures/default_grey.png`,
    font: `${root}/fonts/test_font.ttf`,
    audio,
  };
}

export class DesktopNetworkState implements NetworkStateLike {
  role = 'offline';
  peerCount = 0;
  detail = 'No network provider';
  private endpoint: any = null;
  private knownPeerId: number | null = null;
  private heartbeatTimer = 0;
  private readonly context: any;

  constructor(context: any, managerCtor: any, protocol: any, port: number) {
    this.context = context;
    try {
      this.endpoint = new managerCtor(context).host(protocol.Tcp, port);
      this.role = 'host';
      this.detail = `Hosting localhost:${port}`;
    } catch {
      try {
        this.endpoint = new managerCtor(context).connect(protocol.Tcp, '127.0.0.1', port);
        this.role = 'client';
        this.detail = `Connected to localhost:${port}`;
      } catch (error) {
        this.detail = error instanceof Error ? error.message : String(error);
      }
    }
  }

  update(dt: number): void {
    if (!this.endpoint) return;
    this.endpoint.poll();
    this.peerCount = this.endpoint.peerCount();
    const packet = this.endpoint.receive();
    if (packet) {
      this.knownPeerId = packet.peerId;
      this.detail = `Received ${packet.data.length} bytes from peer ${packet.peerId}`;
    }
    this.heartbeatTimer += dt;
    if (this.heartbeatTimer < 1) return;
    this.heartbeatTimer = 0;
    const payload = new TextEncoder().encode(`sandbox:${this.role}:${this.peerCount}`);
    try {
      if (this.endpoint.defaultPeerId != null) {
        this.endpoint.send(payload);
      } else if (this.knownPeerId != null) {
        this.endpoint.sendTo(this.knownPeerId, payload);
      }
    } catch (error) {
      this.detail = error instanceof Error ? error.message : String(error);
    }
  }

  destroy(): void {
    try {
      this.endpoint?.disconnect();
    } catch {}
    this.context.destroy();
  }
}

export class DisabledNetworkState implements NetworkStateLike {
  role = 'capability-gated';
  peerCount = 0;
  detail = 'Web keeps the networking panel visible but host/client runtime parity is not faked here.';
  update(): void {}
  destroy(): void {}
}

export class SandboxApp {
  private modeIndex = 0;
  private readonly modes: SandboxMode[] = ['2D', '3D', 'Hybrid'];
  private playerX = 250;
  private playerY = 300;
  private angle = 0;
  private elapsed = 0;
  private readonly background: number;
  private readonly sprite: number;
  private readonly accentSprite: number;
  private readonly font: number;
  private readonly cube: number;
  private readonly canRender3d: boolean;
  private audioActivated = false;

  private constructor(
    private readonly game: SandboxGame,
    private readonly ui: SandboxUi,
    private readonly target: SandboxTarget,
    private readonly assets: SandboxAssets,
    private readonly network: NetworkStateLike,
    private readonly maxRuntimeSec: number,
    handles: { background: number; sprite: number; accentSprite: number; font: number; cube: number },
  ) {
    this.background = handles.background;
    this.sprite = handles.sprite;
    this.accentSprite = handles.accentSprite;
    this.font = handles.font;
    this.cube = handles.cube;
    this.canRender3d = this.target === 'desktop';
  }

  static async create(
    game: SandboxGame,
    ui: SandboxUi,
    target: SandboxTarget,
    assets: SandboxAssets,
    network: NetworkStateLike,
    options?: { maxRuntimeSec?: number },
  ): Promise<SandboxApp> {
    const background = await game.loadTexture(assets.background);
    const sprite = await game.loadTexture(assets.sprite);
    const accentSprite = await game.loadTexture(assets.accentSprite);
    const font = await game.loadFont(assets.font);
    let cube = 0;
    if (target === 'desktop') {
      const texture3d = await game.loadTexture(assets.texture3d);
      game.configureGrid(true, 12, 12);
      const plane = game.createPlane(texture3d, 8, 8);
      game.setObjectPosition(plane, 0, -1, 0);
      cube = game.createCube(texture3d, 1.2, 1.2, 1.2);
      game.setObjectPosition(cube, 0, 1, 0);
    }
    return new SandboxApp(game, ui, target, assets, network, options?.maxRuntimeSec ?? 0, {
      background,
      sprite,
      accentSprite,
      font,
      cube,
    });
  }

  update(dt: number): void {
    this.elapsed += dt;
    this.angle += dt;
    if (this.game.isKeyJustPressed(KEY_ESCAPE)) this.game.close();
    if (this.game.isKeyJustPressed(KEY_1)) this.modeIndex = 0;
    if (this.game.isKeyJustPressed(KEY_2)) this.modeIndex = 1;
    if (this.game.isKeyJustPressed(KEY_3)) this.modeIndex = 2;
    if (this.game.isKeyPressed(KEY_A) || this.game.isKeyPressed(KEY_LEFT)) this.playerX -= MOVE_SPEED * dt;
    if (this.game.isKeyPressed(KEY_D) || this.game.isKeyPressed(KEY_RIGHT)) this.playerX += MOVE_SPEED * dt;
    if (this.game.isKeyPressed(KEY_W) || this.game.isKeyPressed(KEY_UP)) this.playerY -= MOVE_SPEED * dt;
    if (this.game.isKeyPressed(KEY_S) || this.game.isKeyPressed(KEY_DOWN)) this.playerY += MOVE_SPEED * dt;
    if (this.game.isKeyJustPressed(KEY_SPACE) && this.game.audioPlay) {
      if (!this.audioActivated && this.game.audioActivate) {
        this.game.audioActivate();
        this.audioActivated = true;
      }
      this.game.audioPlay(this.assets.audio);
    }

    const mode = this.modes[this.modeIndex];
    const mouse = this.game.getMousePosition();
    this.network.update(dt);

    this.game.drawSprite(this.background, WINDOW_WIDTH / 2, WINDOW_HEIGHT / 2, WINDOW_WIDTH, WINDOW_HEIGHT);
    this.game.drawQuad(210, 110, 320, 110, { r: 0.05, g: 0.08, b: 0.12, a: 0.88 });
    this.game.drawQuad(620, 110, 560, 110, { r: 0.08, g: 0.12, b: 0.18, a: 0.88 });
    this.game.drawQuad(620, 630, 560, 120, { r: 0.05, g: 0.08, b: 0.12, a: 0.90 });
    this.game.drawQuad(mouse.x, mouse.y, 14, 14, { r: 0.95, g: 0.85, b: 0.2, a: 0.95 });

    if (mode !== '3D') {
      this.game.drawQuad(920, 260, 180, 40, { r: 0.20, g: 0.55, b: 0.95, a: 0.80 });
      this.game.drawSprite(this.sprite, this.playerX, this.playerY, 64, 64, this.angle * 0.25);
      this.game.drawSprite(this.accentSprite, 1040, 420, 72, 240, 0);
    }

    if (mode !== '2D' && this.canRender3d) {
      this.game.setCameraPosition3D(0, 3, -9.5);
      this.game.setCameraRotation3D(-10, this.angle * 20, 0);
      this.game.setObjectPosition(this.cube, 0, 1 + 0.35 * Math.sin(this.angle * 2), 0);
      this.game.setObjectRotation(this.cube, 0, this.angle * 55, 0);
      this.game.render3D();
    }

    const renderCaps = this.game.getRenderCapabilities();
    const physicsCaps = this.game.getPhysicsCapabilities();
    const audioCaps = this.game.getAudioCapabilities();
    const networkCaps = this.safeNetworkCaps();
    const lines = [
      'GoudEngine Sandbox',
      `Mode: ${mode}  (1/2/3 to switch)`,
      'Movement: WASD / Arrows',
      'Audio: SPACE',
      `Mouse marker: (${mouse.x.toFixed(0)}, ${mouse.y.toFixed(0)})`,
      `Render caps: tex=${renderCaps.maxTextureSize} instancing=${String(renderCaps.supportsInstancing)}`,
      `Physics caps: joints=${String(physicsCaps.supportsJoints)} maxBodies=${physicsCaps.maxBodies}`,
      `Audio caps: spatial=${String(audioCaps.supportsSpatial)} channels=${audioCaps.maxChannels}`,
      `UI nodes: ${this.ui.nodeCount()}`,
      `Target: ${this.target}${this.canRender3d ? '' : ' (3D capability-gated)'}`,
      `Network role: ${this.network.role} peers=${this.network.peerCount}`,
      `Network detail: ${this.network.detail}`,
      `Network caps: ${networkCaps?.maxConnections ?? 'unsupported'}`,
    ];

    lines.forEach((line, index) => {
      this.game.drawText(this.font, line, 40, 40 + index * 22, 18, 0, 0, 1, 0, { r: 1, g: 1, b: 1, a: 1 });
    });
  }

  shouldQuit(): boolean {
    return this.maxRuntimeSec > 0 && this.elapsed >= this.maxRuntimeSec;
  }

  private safeNetworkCaps(): any | null {
    try {
      return this.game.getNetworkCapabilities();
    } catch {
      return null;
    }
  }
}
