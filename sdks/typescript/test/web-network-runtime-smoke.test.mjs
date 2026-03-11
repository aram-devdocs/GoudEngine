import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { createServer as createHttpServer } from 'node:http';
import net from 'node:net';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { chromium } from 'playwright';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const sdkRoot = path.resolve(__dirname, '..');

function websocketAcceptValue(key) {
  return createHash('sha1')
    .update(`${key}258EAFA5-E914-47DA-95CA-C5AB0DC85B11`)
    .digest('base64');
}

function parseWebSocketFrame(buffer) {
  if (buffer.length < 2) return null;

  const first = buffer[0];
  const second = buffer[1];
  const opcode = first & 0x0f;
  const masked = (second & 0x80) !== 0;
  let payloadLength = second & 0x7f;
  let offset = 2;

  if (payloadLength === 126) {
    if (buffer.length < offset + 2) return null;
    payloadLength = buffer.readUInt16BE(offset);
    offset += 2;
  } else if (payloadLength === 127) {
    if (buffer.length < offset + 8) return null;
    const value = Number(buffer.readBigUInt64BE(offset));
    if (!Number.isSafeInteger(value)) {
      throw new Error(`Unsupported frame size: ${value}`);
    }
    payloadLength = value;
    offset += 8;
  }

  if (!masked) {
    throw new Error('Expected masked client frame');
  }

  if (buffer.length < offset + 4 + payloadLength) return null;

  const mask = buffer.subarray(offset, offset + 4);
  offset += 4;
  const payload = Buffer.alloc(payloadLength);

  for (let i = 0; i < payloadLength; i += 1) {
    payload[i] = buffer[offset + i] ^ mask[i % 4];
  }

  const consumed = offset + payloadLength;
  return {
    frame: { opcode, payload },
    remaining: buffer.subarray(consumed),
  };
}

function buildServerFrame(payload) {
  const len = payload.length;
  if (len <= 125) {
    return Buffer.concat([Buffer.from([0x82, len]), payload]);
  }
  if (len <= 65535) {
    const header = Buffer.alloc(4);
    header[0] = 0x82;
    header[1] = 126;
    header.writeUInt16BE(len, 2);
    return Buffer.concat([header, payload]);
  }

  const header = Buffer.alloc(10);
  header[0] = 0x82;
  header[1] = 127;
  header.writeBigUInt64BE(BigInt(len), 2);
  return Buffer.concat([header, payload]);
}

async function withWebSocketEchoServer(fn) {
  const server = net.createServer((socket) => {
    let handshakeBuffer = Buffer.alloc(0);
    let frameBuffer = Buffer.alloc(0);
    let upgraded = false;

    socket.on('data', (chunk) => {
      if (!upgraded) {
        handshakeBuffer = Buffer.concat([handshakeBuffer, chunk]);
        const headerEnd = handshakeBuffer.indexOf('\r\n\r\n');
        if (headerEnd < 0) return;

        const headerText = handshakeBuffer.subarray(0, headerEnd).toString('utf8');
        const match = headerText.match(/Sec-WebSocket-Key:\s*(.+)\r\n/i);
        if (!match) {
          socket.destroy();
          return;
        }

        const accept = websocketAcceptValue(match[1].trim());
        const response = [
          'HTTP/1.1 101 Switching Protocols',
          'Upgrade: websocket',
          'Connection: Upgrade',
          `Sec-WebSocket-Accept: ${accept}`,
          '',
          '',
        ].join('\r\n');
        socket.write(response);
        upgraded = true;

        const rest = handshakeBuffer.subarray(headerEnd + 4);
        handshakeBuffer = Buffer.alloc(0);
        if (rest.length > 0) {
          frameBuffer = Buffer.concat([frameBuffer, rest]);
        }
      } else {
        frameBuffer = Buffer.concat([frameBuffer, chunk]);
      }

      while (frameBuffer.length > 0) {
        const parsed = parseWebSocketFrame(frameBuffer);
        if (!parsed) break;
        frameBuffer = parsed.remaining;
        const { opcode, payload } = parsed.frame;

        if (opcode === 0x8) {
          socket.end();
          return;
        }
        if (opcode === 0x9) {
          socket.write(Buffer.concat([Buffer.from([0x8a, payload.length]), payload]));
          continue;
        }
        if (opcode === 0x1 || opcode === 0x2) {
          socket.write(buildServerFrame(payload));
        }
      }
    });
  });

  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });

  const address = server.address();
  if (!address || typeof address === 'string') {
    throw new Error('Failed to resolve WS server address');
  }

  try {
    return await fn({ wsUrl: `ws://127.0.0.1:${address.port}` });
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
}

function contentTypeFor(filePath) {
  if (filePath.endsWith('.html')) return 'text/html; charset=utf-8';
  if (filePath.endsWith('.js')) return 'application/javascript; charset=utf-8';
  if (filePath.endsWith('.wasm')) return 'application/wasm';
  if (filePath.endsWith('.map')) return 'application/json; charset=utf-8';
  return 'application/octet-stream';
}

async function withStaticServer(fn) {
  const server = createHttpServer(async (req, res) => {
    const rawPath = (req.url ?? '/').split('?')[0];
    if (rawPath === '/' || rawPath === '/index.html') {
      res.writeHead(200, { 'Content-Type': 'text/html; charset=utf-8' });
      res.end('<!doctype html><html><body><div id="app"></div></body></html>');
      return;
    }

    const safePath = path.normalize(rawPath).replace(/^(\.\.[/\\])+/, '');
    const filePath = path.join(sdkRoot, safePath);
    if (!filePath.startsWith(sdkRoot)) {
      res.writeHead(403).end();
      return;
    }

    try {
      const contents = await readFile(filePath);
      res.writeHead(200, { 'Content-Type': contentTypeFor(filePath) });
      res.end(contents);
    } catch {
      res.writeHead(404).end();
    }
  });

  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });

  const address = server.address();
  if (!address || typeof address === 'string') {
    throw new Error('Failed to resolve static server address');
  }

  try {
    return await fn({ origin: `http://127.0.0.1:${address.port}` });
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
}

describe('web networking runtime smoke (browser + wasm)', () => {
  it(
    'connects to a live websocket host and round-trips one payload',
    async () => {
      const requiredArtifacts = [
        path.join(sdkRoot, 'dist', 'web', 'web', 'index.js'),
        path.join(sdkRoot, 'wasm', 'goud_engine.js'),
        path.join(sdkRoot, 'wasm', 'goud_engine_bg.wasm'),
      ];
      for (const artifact of requiredArtifacts) {
        try {
          await readFile(artifact);
        } catch {
          throw new Error(
            `Missing runtime artifact: ${artifact}. Run 'npm run build:web' before test:web-runtime.`,
          );
        }
      }

      await withWebSocketEchoServer(async ({ wsUrl }) => {
        await withStaticServer(async ({ origin }) => {
          const browser = await chromium.launch({ headless: true });
          const context = await browser.newContext();
          const page = await context.newPage();

          try {
            await page.goto(`${origin}/`, { waitUntil: 'domcontentloaded' });
            const result = await page.evaluate(async ({ importUrl, wasmUrl, wsUrl: socketUrl }) => {
              const sdk = await import(importUrl);
              const { GoudGame, NetworkManager, NetworkProtocol } = sdk;
              const canvas = document.createElement('canvas');
              canvas.width = 64;
              canvas.height = 64;
              document.body.appendChild(canvas);

              const game = await GoudGame.create({
                width: 64,
                height: 64,
                title: 'network-runtime-smoke',
                canvas,
                wasmUrl,
              });

              const manager = new NetworkManager(game);
              const endpoint = manager.connect(NetworkProtocol.WebSocket, socketUrl, 0);

              for (let i = 0; i < 80; i += 1) {
                endpoint.poll();
                if (endpoint.peerCount() > 0) break;
                await new Promise((resolve) => setTimeout(resolve, 25));
              }
              if (endpoint.peerCount() <= 0) {
                throw new Error('peerCount did not become positive; connection never became active');
              }

              const payloadText = `runtime-smoke-${Date.now()}`;
              const payload = new TextEncoder().encode(payloadText);
              const sendStatus = endpoint.send(payload, 0);
              if (sendStatus !== 0) {
                throw new Error(`send failed with status ${sendStatus}`);
              }

              let packet = null;
              for (let i = 0; i < 120; i += 1) {
                endpoint.poll();
                packet = endpoint.receive();
                if (packet) break;
                await new Promise((resolve) => setTimeout(resolve, 25));
              }
              if (!packet) {
                throw new Error('no packet received from live host');
              }

              const receivedText = new TextDecoder().decode(packet.data);
              if (receivedText !== payloadText) {
                throw new Error(`payload mismatch: expected '${payloadText}', got '${receivedText}'`);
              }

              const stats = endpoint.getStats();
              if (stats.bytesSent < payload.length || stats.packetsSent < 1 || stats.packetsReceived < 1) {
                throw new Error(
                  `unexpected stats: ${JSON.stringify(stats)} for payload length ${payload.length}`,
                );
              }

              const disconnectStatus = endpoint.disconnect();
              game.destroy();

              return {
                handle: endpoint.handle,
                peerId: packet.peerId,
                sendStatus,
                disconnectStatus,
                bytesSent: stats.bytesSent,
                bytesReceived: stats.bytesReceived,
                packetsSent: stats.packetsSent,
                packetsReceived: stats.packetsReceived,
              };
            }, {
              importUrl: `${origin}/dist/web/web/index.js`,
              wasmUrl: `${origin}/wasm/goud_engine_bg.wasm`,
              wsUrl,
            });

            assert.equal(result.sendStatus, 0);
            assert.equal(result.disconnectStatus, 0);
            assert.ok(result.handle > 0);
            assert.ok(result.peerId >= 0);
            assert.ok(result.bytesSent > 0);
            assert.ok(result.bytesReceived > 0);
            assert.ok(result.packetsSent >= 1);
            assert.ok(result.packetsReceived >= 1);
          } finally {
            await page.close();
            await context.close();
            await browser.close();
          }
        });
      });
    },
    { timeout: 120_000 },
  );
});
