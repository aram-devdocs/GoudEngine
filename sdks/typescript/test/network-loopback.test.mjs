import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'node:net';
import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '..');
const require = createRequire(import.meta.url);
const {
  GoudContext,
  NetworkManager,
  NetworkProtocol,
} = require(path.join(repoRoot, 'dist', 'node', 'index.js'));

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function reservePort() {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      if (!address || typeof address === 'string') {
        reject(new Error('failed to reserve loopback port'));
        return;
      }

      server.close((err) => {
        if (err) {
          reject(err);
          return;
        }
        resolve(address.port);
      });
    });
  });
}

async function waitForPacket(target, peers, message) {
  const deadline = Date.now() + 5000;
  while (Date.now() < deadline) {
    for (const peer of peers) {
      peer.poll();
    }

    const packet = target.receive();
    if (packet) {
      return packet;
    }

    await delay(10);
  }

  throw new Error(message);
}

async function waitForPeerCounts(hostEndpoint, clientEndpoint, message) {
  const deadline = Date.now() + 5000;
  while (Date.now() < deadline) {
    hostEndpoint.poll();
    clientEndpoint.poll();
    if (hostEndpoint.peerCount() > 0 && clientEndpoint.peerCount() > 0) {
      return;
    }
    await delay(10);
  }

  throw new Error(message);
}

describe('NetworkManager loopback', () => {
  it('exchanges TCP packets through goudengine/node wrappers', async () => {
    const port = await reservePort();
    const hostContext = new GoudContext();
    const clientContext = new GoudContext();
    let hostEndpoint = null;
    let clientEndpoint = null;

    try {
      hostEndpoint = new NetworkManager(hostContext).host(NetworkProtocol.Tcp, port);
      clientEndpoint = new NetworkManager(clientContext).connect(
        NetworkProtocol.Tcp,
        '127.0.0.1',
        port,
      );

      assert.notEqual(clientEndpoint.defaultPeerId, null);
      assert.notEqual(clientEndpoint.defaultPeerId, 0);
      await waitForPeerCounts(
        hostEndpoint,
        clientEndpoint,
        'host/client did not report connected peers in time',
      );

      const ping = Buffer.from('ts-ping');
      const pong = Buffer.from('ts-pong');

      assert.equal(clientEndpoint.send(ping), 0);

      const hostPacket = await waitForPacket(
        hostEndpoint,
        [hostEndpoint, clientEndpoint],
        'host should receive client payload',
      );
      assert.deepEqual(Buffer.from(hostPacket.data), ping);
      assert.notEqual(hostPacket.peerId, 0);

      assert.equal(hostEndpoint.sendTo(hostPacket.peerId, pong), 0);

      const clientPacket = await waitForPacket(
        clientEndpoint,
        [hostEndpoint, clientEndpoint],
        'client should receive host reply',
      );
      assert.deepEqual(Buffer.from(clientPacket.data), pong);
      assert.equal(clientPacket.peerId, clientEndpoint.defaultPeerId);

      for (let i = 0; i < 10; i += 1) {
        hostEndpoint.poll();
        clientEndpoint.poll();
        await delay(10);
      }

      assert.ok(hostEndpoint.peerCount() > 0);
      assert.ok(clientEndpoint.peerCount() > 0);

      const hostStats = hostEndpoint.getStats();
      const clientStats = clientEndpoint.getStats();
      assert.ok(hostStats.bytesReceived > 0);
      assert.ok(clientStats.bytesSent > 0);
      assert.ok(clientStats.bytesReceived > 0);
    } finally {
      if (clientEndpoint) {
        try {
          clientEndpoint.disconnect();
        } catch {}
      }
      if (hostEndpoint) {
        try {
          hostEndpoint.disconnect();
        } catch {}
      }
      clientContext.destroy();
      hostContext.destroy();
    }
  });
});
