import { GoudContext, GoudGame, NetworkManager, NetworkProtocol, UiManager } from 'goudengine/node';
import { DesktopNetworkState, SandboxApp, loadSandboxManifest } from './sandbox.js';

function smokeSeconds(): number {
  const raw = process.env.GOUD_SANDBOX_SMOKE_SECONDS;
  if (!raw) return 0;
  const value = Number(raw);
  return Number.isFinite(value) && value > 0 ? value : 0;
}

function envFlag(name: string): boolean {
  return ['1', 'true', 'yes', 'on'].includes((process.env[name] ?? '').trim().toLowerCase());
}

function envRole(name: string): 'auto' | 'host' | 'client' {
  const value = (process.env[name] ?? 'auto').trim().toLowerCase();
  return value === 'host' || value === 'client' ? value : 'auto';
}

function envPort(name: string, fallback: number): number {
  const value = Number(process.env[name] ?? '');
  return Number.isFinite(value) && value > 0 ? value : fallback;
}

const game = new GoudGame({ width: 1280, height: 720, title: 'GoudEngine Sandbox - TypeScript' });
const ui = new UiManager();
const root = ui.createPanel();
const title = ui.createLabel('Sandbox Widgets');
const button = ui.createButton(true);
ui.setParent(title, root);
ui.setParent(button, root);
ui.setLabelText(title, 'Sandbox Widgets');
ui.setButtonEnabled(button, true);

async function main(): Promise<void> {
  const config = await loadSandboxManifest('desktop');
  const network = new DesktopNetworkState(new GoudContext(), NetworkManager, NetworkProtocol, {
    port: envPort('GOUD_SANDBOX_NETWORK_PORT', config.networkPort),
    preferredRole: envRole('GOUD_SANDBOX_NETWORK_ROLE'),
    exitOnPeer: envFlag('GOUD_SANDBOX_EXIT_ON_PEER'),
    expectPeer: envFlag('GOUD_SANDBOX_EXPECT_PEER'),
  });
  const app = await SandboxApp.create(game, ui, 'desktop', config, network, { maxRuntimeSec: smokeSeconds() });

  while (!game.shouldClose()) {
    game.beginFrame(0.07, 0.10, 0.14, 1.0);
    app.update(game.deltaTime || 0.016);
    ui.update();
    ui.render();
    game.endFrame();
    if (app.shouldQuit()) {
      game.close();
    }
  }

  const networkError = app.finalize();
  network.destroy();
  game.destroy();
  if (networkError) {
    throw new Error(networkError);
  }
}

main().catch((error) => {
  const message = error instanceof Error ? error.stack ?? error.message : String(error);
  console.error(message);
  process.exit(1);
});
