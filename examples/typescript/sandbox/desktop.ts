import { GoudContext, GoudGame, NetworkManager, NetworkProtocol, UiManager } from 'goudengine/node';
import { DesktopNetworkState, SandboxApp, loadSandboxAssets } from './sandbox.js';

function smokeSeconds(): number {
  const raw = process.env.GOUD_SANDBOX_SMOKE_SECONDS;
  if (!raw) return 0;
  const value = Number(raw);
  return Number.isFinite(value) && value > 0 ? value : 0;
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
  const assets = await loadSandboxAssets('desktop');
  const network = new DesktopNetworkState(new GoudContext(), NetworkManager, NetworkProtocol, 38491);
  const app = await SandboxApp.create(game, ui, 'desktop', assets, network, { maxRuntimeSec: smokeSeconds() });

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

  network.destroy();
  game.destroy();
}

main().catch((error) => {
  const message = error instanceof Error ? error.stack ?? error.message : String(error);
  console.error(message);
  process.exit(1);
});
