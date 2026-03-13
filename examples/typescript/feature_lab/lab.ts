export type FeatureLabMode = 'desktop' | 'web';

export interface FeatureLabOptions {
  mode: FeatureLabMode;
  log?: (line: string) => void;
  maxRuntimeSec?: number;
}

type ProbeStatus = 'ok' | 'unsupported' | 'error';

interface ProbeResult {
  label: string;
  status: ProbeStatus;
  detail: string;
}

export interface FeatureLab {
  update(dt: number): void;
  shouldQuit(): boolean;
  getResults(): ProbeResult[];
}

export interface FeatureLabGameContext {
  getRenderCapabilities(): unknown;
  getPhysicsCapabilities(): unknown;
  getAudioCapabilities(): unknown;
  getInputCapabilities(): unknown;
  getNetworkCapabilities(): unknown;
  loadScene?(name: string, json: string): number;
  setActiveScene?(sceneId: number, active: boolean): boolean;
  unloadScene?(name: string): boolean;
  spawnEmpty(): unknown;
  addName(entity: unknown, name: string): void;
  addTransform2d(entity: unknown, transform: {
    positionX: number;
    positionY: number;
    rotation: number;
    scaleX: number;
    scaleY: number;
  }): void;
  getTransform2d(entity: unknown): {
    positionX: number;
    positionY: number;
    rotation: number;
    scaleX: number;
    scaleY: number;
  } | null;
  setTransform2d(entity: unknown, transform: {
    positionX: number;
    positionY: number;
    rotation: number;
    scaleX: number;
    scaleY: number;
  }): void;
  drawQuad(x: number, y: number, width: number, height: number, color?: {
    r: number;
    g: number;
    b: number;
    a: number;
  }): void;
}

function summarize(value: unknown): string {
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

function classifyError(error: unknown): ProbeStatus {
  const message = error instanceof Error ? error.message : String(error);
  if (message.includes('Not supported in WASM mode')) {
    return 'unsupported';
  }
  return 'error';
}

function safeProbe(label: string, fn: () => unknown): ProbeResult {
  try {
    const value = fn();
    return { label, status: 'ok', detail: summarize(value) };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    return { label, status: classifyError(error), detail: message };
  }
}

export function createFeatureLab(game: FeatureLabGameContext, options: FeatureLabOptions): FeatureLab {
  const log = options.log ?? (() => {});
  const probes: ProbeResult[] = [];

  probes.push(safeProbe('cap.render', () => game.getRenderCapabilities()));
  probes.push(safeProbe('cap.physics', () => game.getPhysicsCapabilities()));
  probes.push(safeProbe('cap.audio', () => game.getAudioCapabilities()));
  probes.push(safeProbe('cap.input', () => game.getInputCapabilities()));
  probes.push(safeProbe('cap.network', () => game.getNetworkCapabilities()));

  const sceneJson = JSON.stringify({
    version: 1,
    entities: [],
    metadata: { source: 'feature_lab_alpha_001' },
  });

  const sceneResult = safeProbe('scene.load', () => {
    if (!game.loadScene) {
      throw new Error('scene API not available');
    }
    return game.loadScene('alpha-001-feature-lab', sceneJson);
  });
  probes.push(sceneResult);

  const loadedSceneId = sceneResult.status === 'ok' && typeof Number(sceneResult.detail) === 'number'
    ? Number(sceneResult.detail)
    : null;

  if (loadedSceneId !== null && Number.isFinite(loadedSceneId) && loadedSceneId >= 0) {
    probes.push(safeProbe('scene.set_active', () => {
      if (!game.setActiveScene) {
        throw new Error('scene API not available');
      }
      return game.setActiveScene(loadedSceneId, true);
    }));
    probes.push(safeProbe('scene.unload', () => {
      if (!game.unloadScene) {
        throw new Error('scene API not available');
      }
      return game.unloadScene('alpha-001-feature-lab');
    }));
  }

  for (const probe of probes) {
    log(`[feature-lab:${options.mode}] ${probe.label} -> ${probe.status} (${probe.detail})`);
  }

  const marker = game.spawnEmpty();
  const nativeGame = (game as any).native as Record<string, unknown>;
  const supportsTransformComponent =
    typeof nativeGame.addTransform2d === 'function'
    && typeof nativeGame.getTransform2d === 'function'
    && typeof nativeGame.setTransform2d === 'function';
  game.addName(marker, `feature_lab_${options.mode}`);
  if (supportsTransformComponent) {
    game.addTransform2d(marker, {
      positionX: 64,
      positionY: 64,
      rotation: 0,
      scaleX: 1,
      scaleY: 1,
    });
  }

  let elapsed = 0;
  let pulse = 0;

  return {
    update(dt: number): void {
      elapsed += dt;
      pulse += dt;

      const x = 60 + Math.sin(pulse * 2.0) * 40;
      const y = 80 + Math.cos(pulse * 1.5) * 30;

      game.drawQuad(120, 90, 220, 120, { r: 0.07, g: 0.12, b: 0.19, a: 0.95 });
      game.drawQuad(x, y, 24, 24, { r: 0.9, g: 0.25, b: 0.2, a: 1.0 });

      if (supportsTransformComponent) {
        const transform = game.getTransform2d(marker);
        if (transform) {
          transform.rotation += dt;
          game.setTransform2d(marker, transform);
        }
      }
    },

    shouldQuit(): boolean {
      const maxRuntimeSec = options.maxRuntimeSec ?? 0;
      return maxRuntimeSec > 0 && elapsed >= maxRuntimeSec;
    },

    getResults(): ProbeResult[] {
      return probes;
    },
  };
}
