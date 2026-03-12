/**
 * Feature Lab (ALPHA-001) -- Desktop / Node.js
 *
 * Exercises:
 * - Capability queries
 * - Scene wrapper calls
 * - UI manager wrapper calls
 * - Networking wrapper access (headless loopback via GoudContext)
 * - Safe fallback/error logging
 */

import { createServer } from 'node:net';
import {
  GoudContext,
  GoudGame,
  Key,
  NetworkManager,
  NetworkProtocol,
  UiManager,
} from 'goudengine/node';
import { createFeatureLab } from './lab.js';

const SCREEN_WIDTH = 960;
const SCREEN_HEIGHT = 540;

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function reservePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      if (!address || typeof address === 'string') {
        reject(new Error('failed to reserve loopback port'));
        return;
      }
      server.close((error) => {
        if (error) {
          reject(error);
          return;
        }
        resolve(address.port);
      });
    });
  });
}

async function runNetworkProbe(log: (line: string) => void): Promise<void> {
  const hostContext = new GoudContext();
  const clientContext = new GoudContext();
  let hostEndpoint: ReturnType<NetworkManager['host']> | null = null;
  let clientEndpoint: ReturnType<NetworkManager['connect']> | null = null;

  try {
    const port = await reservePort();
    hostEndpoint = new NetworkManager(hostContext).host(NetworkProtocol.Tcp, port);
    clientEndpoint = new NetworkManager(clientContext).connect(NetworkProtocol.Tcp, '127.0.0.1', port);

    const payload = new TextEncoder().encode('alpha-001-feature-lab');
    const sendStatus = clientEndpoint.send(payload);
    log(`[feature-lab:desktop] network.send -> ${sendStatus}`);

    const deadline = Date.now() + 2500;
    let received = false;

    while (Date.now() < deadline) {
      hostEndpoint.poll();
      clientEndpoint.poll();
      const packet = hostEndpoint.receive();
      if (packet) {
        received = true;
        log(`[feature-lab:desktop] network.receive -> ${new TextDecoder().decode(packet.data)} from peer ${packet.peerId}`);
        break;
      }
      await delay(10);
    }

    if (!received) {
      log('[feature-lab:desktop] network.receive -> timeout (no packet)');
    }

    const stats = hostEndpoint.getStats();
    log(`[feature-lab:desktop] network.stats -> ${JSON.stringify(stats)}`);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    log(`[feature-lab:desktop] network.error -> ${message}`);
  } finally {
    if (clientEndpoint) {
      try {
        clientEndpoint.disconnect();
      } catch {
        // ignored in example cleanup
      }
    }
    if (hostEndpoint) {
      try {
        hostEndpoint.disconnect();
      } catch {
        // ignored in example cleanup
      }
    }
    clientContext.destroy();
    hostContext.destroy();
  }
}

async function main(): Promise<void> {
  const game = new GoudGame({
    width: SCREEN_WIDTH,
    height: SCREEN_HEIGHT,
    title: 'Feature Lab (ALPHA-001) -- Desktop',
  });

  const log = (line: string): void => console.log(line);

  const ui = new UiManager();
  const root = ui.createPanel();
  const header = ui.createLabel('Feature Lab ALPHA-001');
  const button = ui.createButton(true);
  ui.setParent(header, root);
  ui.setParent(button, root);
  ui.setLabelText(header, 'Feature Lab ALPHA-001 (Node)');
  ui.setButtonEnabled(button, true);
  ui.update();
  log(`[feature-lab:desktop] ui.node_count -> ${ui.nodeCount()}`);
  log(`[feature-lab:desktop] ui.event_count -> ${ui.eventCount()}`);

  await runNetworkProbe(log);

  const lab = createFeatureLab(game, {
    mode: 'desktop',
    maxRuntimeSec: 10,
    log,
  });

  while (!game.shouldClose()) {
    if (game.isKeyJustPressed(Key.Escape)) {
      game.close();
      break;
    }

    game.beginFrame(0.08, 0.11, 0.14, 1.0);
    lab.update(game.deltaTime);
    ui.update();
    ui.render();
    game.endFrame();

    if (lab.shouldQuit()) {
      game.close();
      break;
    }
  }

  game.destroy();
}

main().catch((error) => {
  const message = error instanceof Error ? error.stack ?? error.message : String(error);
  console.error(message);
  process.exit(1);
});
