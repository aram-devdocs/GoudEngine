export type SandboxMode = '2D' | '3D' | 'Hybrid';
export type SandboxTarget = 'desktop' | 'web';

export interface SandboxScene {
  key: string;
  mode: SandboxMode;
  label: string;
}

export interface SandboxHud {
  overviewTitle: string;
  statusTitle: string;
  nextStepsTitle: string;
  tagline: string;
  overview: string[];
  nextSteps: string[];
}

export interface SandboxContract {
  panels: string[];
  overviewItems: string[];
  statusRows: string[];
  nextStepItems: string[];
  nextStepDynamicRows: string[];
  layout: HudLayout;
  typography: HudTypography;
  webBlockers: {
    networking: string;
    renderer: string;
  };
}

export interface SandboxCapabilityGates {
  webNetworking: string;
  webRenderer: string;
}

export interface SandboxConfig {
  title: string;
  background: string;
  sprite: string;
  accentSprite: string;
  texture3d: string;
  font: string;
  audio: Uint8Array;
  networkPort: number;
  packetVersion: string;
  hud: SandboxHud;
  contract: SandboxContract;
  scenes: SandboxScene[];
  capabilityGates: SandboxCapabilityGates;
}

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
  enableDepthTest(): void;
  disableDepthTest(): void;
  render3D(): boolean;
  addLight(lightType: number, x: number, y: number, z: number, dirX: number, dirY: number, dirZ: number, colorR: number, colorG: number, colorB: number, intensity: number, range: number, angle: number): number;
  audioActivate?(): number;
  audioPlay?(data: Uint8Array): number;
}

export interface SandboxUi {
  update(): void;
  render(): void;
  nodeCount(): number;
}

export interface NetworkUpdateState {
  mode: SandboxMode;
  x: number;
  y: number;
  packetVersion: string;
}

export interface NetworkStateLike {
  role: string;
  label: string;
  peerCount: number;
  detail: string;
  hasRemoteState: boolean;
  remoteX: number;
  remoteY: number;
  remoteMode: SandboxMode;
  remoteLabel: string;
  exitRequested: boolean;
  update(dt: number, state: NetworkUpdateState): void;
  finalize?(): string | null;
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

interface HudRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface OverviewTextLayout {
  x: number;
  titleY: number;
  taglineY: number;
  maxWidth: number;
}

interface StatusTextLayout {
  x: number;
  titleY: number;
  maxWidth: number;
}

interface NextTextLayout {
  x: number;
  titleY: number;
  maxWidth: number;
}

interface SceneLabelLayout {
  x: number;
  y: number;
  maxWidth: number;
}

interface HudLayout {
  overviewPanel: HudRect;
  statusPanel: HudRect;
  nextPanel: HudRect;
  sceneBadge: HudRect;
  overviewText: OverviewTextLayout;
  statusText: StatusTextLayout;
  nextText: NextTextLayout;
  sceneLabel: SceneLabelLayout;
}

interface OverviewLineAdvances {
  title: number;
  tagline: number;
  body: number;
}

interface StatusLineAdvances {
  title: number;
  body: number;
}

interface NextLineAdvances {
  title: number;
  body: number;
}

interface OverviewTypography {
  titleSize: number;
  taglineSize: number;
  bodySize: number;
  lineSpacing: number;
  lineAdvances: OverviewLineAdvances;
}

interface StatusTypography {
  titleSize: number;
  bodySize: number;
  lineSpacing: number;
  lineAdvances: StatusLineAdvances;
}

interface NextTypography {
  titleSize: number;
  bodySize: number;
  lineSpacing: number;
  lineAdvances: NextLineAdvances;
}

interface SceneLabelTypography {
  size: number;
  lineSpacing: number;
}

interface HudTypography {
  overview: OverviewTypography;
  status: StatusTypography;
  next: NextTypography;
  sceneLabel: SceneLabelTypography;
}

interface ManifestJson {
  title: string;
  contract?: string;
  network?: { port?: number; packet_version?: string };
  network_port?: number;
  assets: {
    background: string;
    sprite: string;
    accent_sprite: string;
    texture3d: string;
    font: string;
    audio: string;
  };
  hud: {
    overview_title: string;
    status_title: string;
    next_steps_title: string;
    tagline: string;
    overview: string[];
    next_steps: string[];
  };
  scenes: Array<{ key: string; mode: SandboxMode; label: string }>;
  capability_gates: {
    web_networking: string;
    web_renderer: string;
  };
}

interface ContractJson {
  panels: string[];
  overview_items: string[];
  status_rows: string[];
  next_step_items: string[];
  next_step_dynamic_rows: string[];
  layout: {
    overview_panel: HudRect;
    status_panel: HudRect;
    next_panel: HudRect;
    scene_badge: HudRect;
    overview_text: { x: number; title_y: number; tagline_y: number; max_width: number };
    status_text: { x: number; title_y: number; max_width: number };
    next_text: { x: number; title_y: number; max_width: number };
    scene_label: { x: number; y: number; max_width: number };
  };
  typography: {
    overview: {
      title_size: number;
      tagline_size: number;
      body_size: number;
      line_spacing: number;
      line_advances: { title: number; tagline: number; body: number };
    };
    status: {
      title_size: number;
      body_size: number;
      line_spacing: number;
      line_advances: { title: number; body: number };
    };
    next: {
      title_size: number;
      body_size: number;
      line_spacing: number;
      line_advances: { title: number; body: number };
    };
    scene_label: {
      size: number;
      line_spacing: number;
    };
  };
  web_blockers: {
    networking: string;
    renderer: string;
  };
}

function assetPath(target: SandboxTarget, manifestPath: string): string {
  const relativePath = manifestPath.replace(/^examples\/shared\/sandbox\//, '');
  const root = target === 'web' ? '/examples/shared/sandbox' : '../../shared/sandbox';
  return `${root}/${relativePath}`;
}

async function loadJson<T>(target: SandboxTarget, path: string): Promise<T> {
  if (target === 'web') {
    return (await (await fetch(path)).json()) as T;
  }
  const { readFile } = await import('node:fs/promises');
  return JSON.parse(await readFile(path, 'utf8')) as T;
}

export async function loadSandboxManifest(target: SandboxTarget): Promise<SandboxConfig> {
  const manifestPath = target === 'web' ? '/examples/shared/sandbox/manifest.json' : '../../shared/sandbox/manifest.json';
  const manifest = await loadJson<ManifestJson>(target, manifestPath);
  const contractPath = assetPath(target, manifest.contract ?? 'examples/shared/sandbox/contract.json');
  const contract = await loadJson<ContractJson>(target, contractPath);

  const audioPath = assetPath(target, manifest.assets.audio);
  let audio: Uint8Array;
  if (target === 'web') {
    audio = new Uint8Array(await (await fetch(audioPath)).arrayBuffer());
  } else {
    const { readFile } = await import('node:fs/promises');
    audio = new Uint8Array(await readFile(audioPath));
  }

  return {
    title: manifest.title,
    background: assetPath(target, manifest.assets.background),
    sprite: assetPath(target, manifest.assets.sprite),
    accentSprite: assetPath(target, manifest.assets.accent_sprite),
    texture3d: assetPath(target, manifest.assets.texture3d),
    font: assetPath(target, manifest.assets.font),
    audio,
    networkPort: Number(manifest.network?.port ?? manifest.network_port ?? 38491),
    packetVersion: manifest.network?.packet_version ?? 'v1',
    hud: {
      overviewTitle: manifest.hud.overview_title,
      statusTitle: manifest.hud.status_title,
      nextStepsTitle: manifest.hud.next_steps_title,
      tagline: manifest.hud.tagline,
      overview: manifest.hud.overview,
      nextSteps: manifest.hud.next_steps,
    },
    contract: {
      panels: contract.panels,
      overviewItems: contract.overview_items,
      statusRows: contract.status_rows,
      nextStepItems: contract.next_step_items,
      nextStepDynamicRows: contract.next_step_dynamic_rows,
      layout: {
        overviewPanel: contract.layout.overview_panel,
        statusPanel: contract.layout.status_panel,
        nextPanel: contract.layout.next_panel,
        sceneBadge: contract.layout.scene_badge,
        overviewText: {
          x: contract.layout.overview_text.x,
          titleY: contract.layout.overview_text.title_y,
          taglineY: contract.layout.overview_text.tagline_y,
          maxWidth: contract.layout.overview_text.max_width,
        },
        statusText: {
          x: contract.layout.status_text.x,
          titleY: contract.layout.status_text.title_y,
          maxWidth: contract.layout.status_text.max_width,
        },
        nextText: {
          x: contract.layout.next_text.x,
          titleY: contract.layout.next_text.title_y,
          maxWidth: contract.layout.next_text.max_width,
        },
        sceneLabel: {
          x: contract.layout.scene_label.x,
          y: contract.layout.scene_label.y,
          maxWidth: contract.layout.scene_label.max_width,
        },
      },
      typography: {
        overview: {
          titleSize: contract.typography.overview.title_size,
          taglineSize: contract.typography.overview.tagline_size,
          bodySize: contract.typography.overview.body_size,
          lineSpacing: contract.typography.overview.line_spacing,
          lineAdvances: contract.typography.overview.line_advances,
        },
        status: {
          titleSize: contract.typography.status.title_size,
          bodySize: contract.typography.status.body_size,
          lineSpacing: contract.typography.status.line_spacing,
          lineAdvances: contract.typography.status.line_advances,
        },
        next: {
          titleSize: contract.typography.next.title_size,
          bodySize: contract.typography.next.body_size,
          lineSpacing: contract.typography.next.line_spacing,
          lineAdvances: contract.typography.next.line_advances,
        },
        sceneLabel: {
          size: contract.typography.scene_label.size,
          lineSpacing: contract.typography.scene_label.line_spacing,
        },
      },
      webBlockers: contract.web_blockers,
    },
    scenes: manifest.scenes,
    capabilityGates: {
      webNetworking: manifest.capability_gates.web_networking,
      webRenderer: manifest.capability_gates.web_renderer,
    },
  };
}

function parseSandboxPayload(payload: Uint8Array): { x: number; y: number; mode: SandboxMode; label: string } | null {
  const text = new TextDecoder().decode(payload);
  const parts = text.split('|');
  if (parts.length === 7 && parts[0] === 'sandbox' && parts[1] === 'v1') {
    const x = Number(parts[4]);
    const y = Number(parts[5]);
    if (!Number.isFinite(x) || !Number.isFinite(y)) return null;
    return { x, y, mode: parts[3] as SandboxMode, label: parts[6] };
  }
  if (parts.length === 5 && parts[0] === 'sandbox') {
    const x = Number(parts[2]);
    const y = Number(parts[3]);
    if (!Number.isFinite(x) || !Number.isFinite(y)) return null;
    return { x, y, mode: parts[4] as SandboxMode, label: 'connected' };
  }
  return null;
}

function drawTextLines(
  game: SandboxGame,
  font: number,
  lines: string[],
  x: number,
  y: number,
  sizes: number[],
  kinds: string[],
  maxWidth: number,
  lineSpacing: number,
  advances: Record<string, number>,
  color: { r: number; g: number; b: number; a: number },
): void {
  let currentY = y;
  lines.forEach((line, index) => {
    const size = sizes[index] ?? sizes[sizes.length - 1];
    const kind = kinds[index] ?? 'body';
    game.drawText(font, line, x, currentY, size, 0, maxWidth, lineSpacing, 0, color);
    const baseAdvance = advances[kind] ?? advances.body ?? 24;
    currentY += baseAdvance * estimateWrappedLineCount(line, size, maxWidth);
  });
}

function estimateWrappedLineCount(text: string, fontSize: number, maxWidth: number): number {
  if (!text.trim() || maxWidth <= 0) return 1;
  const approxGlyphWidth = Math.max(fontSize * 0.52, 1);
  const maxChars = Math.max(1, Math.floor(maxWidth / approxGlyphWidth));
  let total = 0;
  for (const rawLine of text.split('\n')) {
    const words = rawLine.trim().split(/\s+/).filter(Boolean);
    if (words.length === 0) {
      total += 1;
      continue;
    }
    let wrapped = 1;
    let current = 0;
    for (const word of words) {
      const length = Array.from(word).length;
      if (current === 0) {
        current = length;
        continue;
      }
      if (current + 1 + length <= maxChars) {
        current += 1 + length;
      } else {
        wrapped += 1;
        current = length;
      }
    }
    total += wrapped;
  }
  return Math.max(1, total);
}

export interface DesktopNetworkOptions {
  port: number;
  preferredRole?: 'auto' | 'host' | 'client';
  exitOnPeer?: boolean;
  expectPeer?: boolean;
}

export class DesktopNetworkState implements NetworkStateLike {
  role = 'offline';
  label = 'solo';
  peerCount = 0;
  detail = 'No network provider';
  hasRemoteState = false;
  remoteX = 0;
  remoteY = 0;
  remoteMode: SandboxMode = '2D';
  remoteLabel = 'waiting';
  exitRequested = false;
  private endpoint: any = null;
  private knownPeerId: number | null = null;
  private heartbeatTimer = 0;
  private readonly context: any;
  private readonly exitOnPeer: boolean;
  private readonly expectPeer: boolean;

  constructor(context: any, managerCtor: any, protocol: any, options: DesktopNetworkOptions) {
    this.context = context;
    this.exitOnPeer = options.exitOnPeer ?? false;
    this.expectPeer = options.expectPeer ?? false;
    const preferredRole = options.preferredRole ?? 'auto';
    try {
      if (preferredRole === 'host') {
        this.host(managerCtor, protocol, options.port);
      } else if (preferredRole === 'client') {
        this.connect(managerCtor, protocol, options.port);
      } else {
        try {
          this.host(managerCtor, protocol, options.port);
        } catch {
          this.connect(managerCtor, protocol, options.port);
        }
      }
    } catch (error) {
      this.detail = error instanceof Error ? error.message : String(error);
    }
  }

  private host(managerCtor: any, protocol: any, port: number): void {
    this.endpoint = new managerCtor(this.context).host(protocol.Tcp, port);
    this.role = 'host';
    this.label = 'waiting';
    this.detail = `Hosting localhost:${port}`;
  }

  private connect(managerCtor: any, protocol: any, port: number): void {
    this.endpoint = new managerCtor(this.context).connect(protocol.Tcp, '127.0.0.1', port);
    this.role = 'client';
    this.label = 'connected';
    this.detail = `Connected to localhost:${port}`;
  }

  update(dt: number, state: NetworkUpdateState): void {
    if (!this.endpoint) return;
    this.endpoint.poll();
    this.peerCount = this.endpoint.peerCount();
    this.label = this.role === 'host' && this.peerCount <= 0 ? 'waiting' : 'connected';
    const packet = this.endpoint.receive();
    if (packet) {
      this.knownPeerId = packet.peerId;
      const parsed = parseSandboxPayload(packet.data);
      if (parsed) {
        this.hasRemoteState = true;
        this.remoteX = parsed.x;
        this.remoteY = parsed.y;
        this.remoteMode = parsed.mode;
        this.remoteLabel = parsed.label;
        this.detail = `Peer ${packet.peerId} synced in ${parsed.mode} mode`;
      } else {
        this.detail = `Received ${packet.data.length} bytes from peer ${packet.peerId}`;
      }
    }
    this.heartbeatTimer += dt;
    if (this.heartbeatTimer >= 1) {
      this.heartbeatTimer = 0;
      const payload = new TextEncoder().encode(
        `sandbox|${state.packetVersion}|${this.role}|${state.mode}|${state.x.toFixed(1)}|${state.y.toFixed(1)}|${this.label}`,
      );
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
    if (this.exitOnPeer && this.hasRemoteState) {
      this.exitRequested = true;
    }
  }

  finalize(): string | null {
    if (this.expectPeer && !this.hasRemoteState) {
      return 'Expected peer discovery before exit, but no remote peer state arrived.';
    }
    return null;
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
  label = 'gated';
  peerCount = 0;
  detail: string;
  hasRemoteState = false;
  remoteX = 0;
  remoteY = 0;
  remoteMode: SandboxMode = '2D';
  remoteLabel = 'gated';
  exitRequested = false;

  constructor(detail: string) {
    this.detail = detail;
  }

  update(): void {}
  finalize(): string | null {
    return null;
  }
  destroy(): void {}
}

export class WebSocketNetworkState implements NetworkStateLike {
  role = 'websocket-client';
  label = 'connecting';
  peerCount = 0;
  detail: string;
  hasRemoteState = false;
  remoteX = 0;
  remoteY = 0;
  remoteMode: SandboxMode = '2D';
  remoteLabel = 'waiting';
  exitRequested = false;
  private readonly endpoint: any;
  private heartbeatTimer = 0;
  private connected = false;

  constructor(game: any, managerCtor: any, protocol: any, wsUrl: string) {
    this.detail = `Connecting to ${wsUrl}`;
    this.endpoint = new managerCtor(game).connect(protocol.WebSocket, wsUrl, 0);
  }

  update(dt: number, state: NetworkUpdateState): void {
    this.endpoint.poll();
    this.peerCount = this.endpoint.peerCount();
    if (this.peerCount > 0) {
      this.connected = true;
      this.label = 'connected';
    }
    const packet = this.endpoint.receive();
    if (packet) {
      const parsed = parseSandboxPayload(packet.data);
      if (parsed) {
        this.hasRemoteState = true;
        this.remoteX = parsed.x;
        this.remoteY = parsed.y;
        this.remoteMode = parsed.mode;
        this.remoteLabel = parsed.label;
        this.detail = `Peer synced in ${parsed.mode} mode`;
      } else {
        this.detail = `Received ${packet.data.length} bytes from websocket peer`;
      }
    }
    this.heartbeatTimer += dt;
    if (this.connected && this.heartbeatTimer >= 1) {
      this.heartbeatTimer = 0;
      const payload = new TextEncoder().encode(
        `sandbox|${state.packetVersion}|${this.role}|${state.mode}|${state.x.toFixed(1)}|${state.y.toFixed(1)}|${this.label}`,
      );
      try {
        this.endpoint.send(payload);
      } catch (error) {
        this.detail = error instanceof Error ? error.message : String(error);
      }
    }
  }

  destroy(): void {
    try {
      this.endpoint?.disconnect();
    } catch {}
  }
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
  private cube = 0;
  private plane = 0;
  private canRender3d = false;
  private tried3dSetup = false;
  private setup3dPromise: Promise<void> | null = null;
  private readonly sceneLookup: Record<SandboxMode, SandboxScene>;
  private audioActivated = false;

  private constructor(
    private readonly game: SandboxGame,
    private readonly ui: SandboxUi,
    private readonly target: SandboxTarget,
    private readonly config: SandboxConfig,
    private readonly network: NetworkStateLike,
    private readonly maxRuntimeSec: number,
    handles: { background: number; sprite: number; accentSprite: number; font: number },
  ) {
    this.background = handles.background;
    this.sprite = handles.sprite;
    this.accentSprite = handles.accentSprite;
    this.font = handles.font;
    this.sceneLookup = Object.fromEntries(
      this.config.scenes.map((scene) => [scene.mode, scene]),
    ) as Record<SandboxMode, SandboxScene>;
  }

  static async create(
    game: SandboxGame,
    ui: SandboxUi,
    target: SandboxTarget,
    config: SandboxConfig,
    network: NetworkStateLike,
    options?: { maxRuntimeSec?: number },
  ): Promise<SandboxApp> {
    const background = await game.loadTexture(config.background);
    const sprite = await game.loadTexture(config.sprite);
    const accentSprite = await game.loadTexture(config.accentSprite);
    const font = await game.loadFont(config.font);
    return new SandboxApp(game, ui, target, config, network, options?.maxRuntimeSec ?? 0, {
      background,
      sprite,
      accentSprite,
      font,
    });
  }

  private ensure3dSetup(): void {
    if (this.canRender3d || this.tried3dSetup || this.setup3dPromise) {
      return;
    }
    this.setup3dPromise = (async () => {
      this.tried3dSetup = true;
      try {
        const texture3d = await this.game.loadTexture(this.config.texture3d);
        this.game.configureGrid(true, 12, 12);
        this.plane = this.game.createPlane(texture3d, 8, 8);
        this.game.setObjectPosition(this.plane, 0, -1.2, 2.5);
        this.cube = this.game.createCube(texture3d, 1.2, 1.2, 1.2);
        this.game.setObjectPosition(this.cube, 0.85, 1.2, 2.1);
        this.game.addLight(0, 4, 6, -4, 0, -1, 0, 1, 0.95, 0.80, 5, 28, 0);
        this.game.addLight(0, -3.5, 3.5, -2, 0, -0.65, 0.35, 0.70, 0.85, 1, 2.5, 18, 0);
        this.game.addLight(0, 0, 2.4, 7, 0, -0.25, -1, 0.55, 0.65, 0.90, 1.8, 20, 0);
        this.canRender3d = this.cube !== 0 && this.plane !== 0;
      } catch {
        this.canRender3d = false;
      } finally {
        this.setup3dPromise = null;
      }
    })();
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
      this.game.audioPlay(this.config.audio);
    }

    const mode = this.modes[this.modeIndex];
    const mouse = this.game.getMousePosition();
    this.network.update(dt, {
      mode,
      x: this.playerX,
      y: this.playerY,
      packetVersion: this.config.packetVersion,
    });

    if (mode !== '2D') {
      this.ensure3dSetup();
    }

    if (mode !== '2D' && this.canRender3d) {
      this.game.enableDepthTest();
      this.game.setCameraPosition3D(0, 2.2, mode === '3D' ? -7.0 : -7.8);
      this.game.setCameraRotation3D(-7, mode === '3D' ? 0 : 8, 0);
      this.game.setObjectPosition(this.plane, 0, -1.2, 2.5);
      this.game.setObjectPosition(this.cube, 0.85, 1.2 + 0.26 * Math.sin(this.angle * 2), 2.1);
      this.game.setObjectRotation(this.cube, 20, this.angle * 46, 0);
      this.game.render3D();
      this.game.disableDepthTest();
    }

    if (mode === '2D') {
      this.game.drawSprite(this.background, WINDOW_WIDTH / 2, WINDOW_HEIGHT / 2, WINDOW_WIDTH, WINDOW_HEIGHT);
      this.game.drawSprite(this.sprite, this.playerX, this.playerY, 64, 64, this.angle * 0.25);
      this.game.drawSprite(this.accentSprite, 1040, 420, 72, 240, 0);
      this.game.drawQuad(920, 260, 180, 40, { r: 0.20, g: 0.55, b: 0.95, a: 0.80 });
    }

    if (mode === 'Hybrid') {
      this.game.drawQuad(640, 360, 1280, 720, { r: 0.08, g: 0.17, b: 0.24, a: 0.10 });
      this.game.drawQuad(640, 654, 1280, 132, { r: 0.03, g: 0.10, b: 0.12, a: 0.18 });
      this.game.drawSprite(this.sprite, this.playerX, this.playerY, 72, 72, this.angle * 0.25);
      this.game.drawSprite(this.accentSprite, 1044, 420, 78, 250, 0);
      this.game.drawQuad(920, 260, 180, 40, { r: 0.20, g: 0.55, b: 0.95, a: 0.62 });
    }

    const is3DFamilyMode = mode !== '2D';
    const panelAlpha = is3DFamilyMode ? 0.48 : 0.72;
    const bottomAlpha = is3DFamilyMode ? 0.55 : 0.78;
    const layout = this.config.contract.layout;
    this.game.drawQuad(layout.overviewPanel.x, layout.overviewPanel.y, layout.overviewPanel.width, layout.overviewPanel.height, { r: 0.05, g: 0.08, b: 0.12, a: panelAlpha });
    this.game.drawQuad(layout.statusPanel.x, layout.statusPanel.y, layout.statusPanel.width, layout.statusPanel.height, { r: 0.08, g: 0.12, b: 0.18, a: panelAlpha });
    this.game.drawQuad(layout.nextPanel.x, layout.nextPanel.y, layout.nextPanel.width, layout.nextPanel.height, { r: 0.05, g: 0.08, b: 0.12, a: bottomAlpha });
    this.game.drawQuad(layout.sceneBadge.x, layout.sceneBadge.y, layout.sceneBadge.width, layout.sceneBadge.height, { r: 0.20, g: 0.55, b: 0.95, a: 0.84 });
    this.game.drawQuad(mouse.x, mouse.y, 14, 14, { r: 0.95, g: 0.85, b: 0.2, a: 0.95 });

    const renderCaps = this.game.getRenderCapabilities();
    const physicsCaps = this.game.getPhysicsCapabilities();
    const audioCaps = this.game.getAudioCapabilities();
    const networkCaps = this.safeNetworkCaps();
    const overviewLines = [
      this.config.hud.overviewTitle,
      this.config.hud.tagline,
      ...this.config.contract.overviewItems,
    ];
    const statusLines = [
      this.config.hud.statusTitle,
      ...this.config.contract.statusRows.map((row) => this.renderStatusRow(row, mode, mouse, renderCaps, physicsCaps, audioCaps, networkCaps)),
    ];
    const nextStepLines = [
      this.config.hud.nextStepsTitle,
      ...this.config.contract.nextStepItems,
      ...this.config.contract.nextStepDynamicRows.map((row) => this.renderNextStepRow(row)),
    ];
    const typography = this.config.contract.typography;
    const overviewSizes = [
      typography.overview.titleSize,
      typography.overview.taglineSize,
      ...this.config.contract.overviewItems.map(() => typography.overview.bodySize),
    ];
    const overviewKinds = [
      'title',
      'tagline',
      ...this.config.contract.overviewItems.map(() => 'body'),
    ];
    const statusSizes = [
      typography.status.titleSize,
      ...this.config.contract.statusRows.map(() => typography.status.bodySize),
    ];
    const statusKinds = [
      'title',
      ...this.config.contract.statusRows.map(() => 'body'),
    ];
    const nextBodyCount =
      this.config.contract.nextStepItems.length + this.config.contract.nextStepDynamicRows.length;
    const nextSizes = [
      typography.next.titleSize,
      ...Array.from({ length: nextBodyCount }, () => typography.next.bodySize),
    ];
    const nextKinds = [
      'title',
      ...Array.from({ length: nextBodyCount }, () => 'body'),
    ];

    drawTextLines(
      this.game,
      this.font,
      overviewLines,
      layout.overviewText.x,
      layout.overviewText.titleY,
      overviewSizes,
      overviewKinds,
      layout.overviewText.maxWidth,
      typography.overview.lineSpacing,
      {
        title: typography.overview.lineAdvances.title,
        tagline: typography.overview.lineAdvances.tagline,
        body: typography.overview.lineAdvances.body,
      },
      { r: 1, g: 1, b: 1, a: 1 },
    );
    drawTextLines(
      this.game,
      this.font,
      statusLines,
      layout.statusText.x,
      layout.statusText.titleY,
      statusSizes,
      statusKinds,
      layout.statusText.maxWidth,
      typography.status.lineSpacing,
      {
        title: typography.status.lineAdvances.title,
        body: typography.status.lineAdvances.body,
      },
      { r: 0.94, g: 0.97, b: 1, a: 1 },
    );
    drawTextLines(
      this.game,
      this.font,
      nextStepLines,
      layout.nextText.x,
      layout.nextText.titleY,
      nextSizes,
      nextKinds,
      layout.nextText.maxWidth,
      typography.next.lineSpacing,
      {
        title: typography.next.lineAdvances.title,
        body: typography.next.lineAdvances.body,
      },
      { r: 0.94, g: 0.97, b: 1, a: 1 },
    );
    this.game.drawText(
      this.font,
      this.sceneLookup[mode].label,
      layout.sceneLabel.x,
      layout.sceneLabel.y,
      typography.sceneLabel.size,
      0,
      layout.sceneLabel.maxWidth,
      typography.sceneLabel.lineSpacing,
      0,
      { r: 1, g: 1, b: 1, a: 1 },
    );
    if (mode !== '3D' && this.network.hasRemoteState) {
      this.game.drawQuad(this.network.remoteX, this.network.remoteY - 48, 108, 20, { r: 0.96, g: 0.70, b: 0.20, a: 0.92 });
      this.game.drawText(this.font, `Peer ${this.network.remoteMode}`, this.network.remoteX - 40, this.network.remoteY - 54, 14, 0, 0, 1, 0, { r: 0.04, g: 0.05, b: 0.08, a: 1 });
      this.game.drawSprite(this.sprite, this.network.remoteX, this.network.remoteY, 52, 52, -this.angle * 0.18);
    }
  }

  shouldQuit(): boolean {
    return this.network.exitRequested || (this.maxRuntimeSec > 0 && this.elapsed >= this.maxRuntimeSec);
  }

  finalize(): string | null {
    return this.network.finalize?.() ?? null;
  }

  private safeNetworkCaps(): any | null {
    try {
      return this.game.getNetworkCapabilities();
    } catch {
      return null;
    }
  }

  private renderStatusRow(
    row: string,
    mode: SandboxMode,
    mouse: { x: number; y: number },
    renderCaps: any,
    physicsCaps: any,
    audioCaps: any,
    networkCaps: any | null,
  ): string {
    switch (row) {
      case 'scene':
        return `Scene: ${this.sceneLookup[mode].label} (${this.sceneLookup[mode].key} to switch)`;
      case 'mouse':
        return `Mouse marker: (${mouse.x.toFixed(0)}, ${mouse.y.toFixed(0)})`;
      case 'render_caps':
        return `Render caps: tex=${renderCaps.maxTextureSize} instancing=${String(renderCaps.supportsInstancing)}`;
      case 'physics_caps':
        return `Physics caps: joints=${String(physicsCaps.supportsJoints)} maxBodies=${physicsCaps.maxBodies}`;
      case 'audio_caps':
        return `Audio caps: spatial=${String(audioCaps.supportsSpatial)} channels=${audioCaps.maxChannels}`;
      case 'scene_count':
        return `Scene count: ${this.config.scenes.length} active mode=${mode}`;
      case 'target':
        return `Target: ${this.target}${this.canRender3d ? '' : ' (renderer gated by browser backend)'}`;
      case 'network_role':
        return `Network role: ${this.network.role} peers=${this.network.peerCount} label=${this.network.label}`;
      case 'network_detail':
        return `Network detail: ${this.network.detail}${networkCaps ? ` (cap=${networkCaps.maxConnections})` : ''}`;
      default:
        return row;
    }
  }

  private renderNextStepRow(row: string): string {
    switch (row) {
      case 'audio_status':
        return `Audio status: ${this.audioActivated ? 'active' : 'press SPACE to activate'}`;
      case 'network_probe':
        return this.network.hasRemoteState
          ? `Peer sprite live at (${this.network.remoteX.toFixed(0)}, ${this.network.remoteY.toFixed(0)})`
          : this.target === 'web'
            ? this.config.contract.webBlockers.networking
            : 'Networking: open a second native sandbox to confirm peer sync.';
      default:
        return row;
    }
  }
}
