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

async function assertRuntimeArtifacts() {
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
}

export async function runWebNetworkingRuntimeSmoke({ collectCoverage = false } = {}) {
  await assertRuntimeArtifacts();

  return withWebSocketEchoServer(async ({ wsUrl }) =>
    withStaticServer(async ({ origin }) => {
      const browser = await chromium.launch({ headless: true });
      const context = await browser.newContext();
      const page = await context.newPage();
      let coverageEntries = [];

      try {
        if (collectCoverage) {
          await page.coverage.startJSCoverage({
            reportAnonymousScripts: false,
            resetOnNavigation: false,
          });
        }

        await page.goto(`${origin}/`, { waitUntil: 'domcontentloaded' });
        const result = await page.evaluate(async ({ importUrl, wasmUrl, wsUrl: socketUrl, textureUrl, fontUrl }) => {
          const sdk = await import(importUrl);
          const {
            Color,
            EngineConfig,
            GoudGame,
            NetworkEndpoint,
            NetworkManager,
            NetworkProtocol,
            Rect,
            UiManager,
            Vec2,
            Vec3,
            parseDebuggerManifest,
            parseDebuggerSnapshot,
          } = sdk;
          const canvas = document.createElement('canvas');
          canvas.width = 64;
          canvas.height = 64;
          document.body.appendChild(canvas);
          const spaceKey = sdk.Key?.Space ?? 32;
          const leftMouse = sdk.MouseButton?.Left ?? 0;
          const stubCalls = [];

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

          game.setClearColor(0.1, 0.2, 0.3, 1);
          game.setWindowSize(96, 96);
          game.beginFrame(0.1, 0.2, 0.3, 1);
          game.endFrame();
          game.updateFrame(1 / 60);

          const entity = game.spawnEmpty();
          game.addTransform2d(entity, {
            positionX: 10,
            positionY: 20,
            rotation: 0.5,
            scaleX: 1,
            scaleY: 2,
          });
          game.getTransform2d(entity);
          game.setTransform2d(entity, {
            positionX: 30,
            positionY: 40,
            rotation: 1.5,
            scaleX: 2,
            scaleY: 3,
          });
          game.hasTransform2d(entity);
          game.addName(entity, 'web-runtime');
          game.getName(entity);
          game.hasName(entity);
          game.addSprite(entity, {
            textureHandle: 0,
            colorR: 1,
            colorG: 1,
            colorB: 1,
            colorA: 1,
            sourceRectX: 0,
            sourceRectY: 0,
            sourceRectWidth: 0,
            sourceRectHeight: 0,
            hasSourceRect: false,
            flipX: false,
            flipY: false,
            anchorX: 0.5,
            anchorY: 0.5,
            customSizeX: 0,
            customSizeY: 0,
            hasCustomSize: false,
          });
          game.getSprite(entity);
          game.hasSprite(entity);
          let clone = null;
          let recursiveClone = null;
          try {
            if (typeof game.cloneEntity === 'function') {
              clone = game.cloneEntity(entity);
            }
          } catch {}
          try {
            if (typeof game.cloneEntityRecursive === 'function') {
              recursiveClone = game.cloneEntityRecursive(entity);
            }
          } catch {}
          if (clone) game.isAlive(clone);
          if (recursiveClone) game.isAlive(recursiveClone);
          const batch = game.spawnBatch(2);
          game.despawnBatch(batch);
          game.entityCount();

          window.dispatchEvent(new KeyboardEvent('keydown', { code: 'Space', bubbles: true, cancelable: true }));
          canvas.dispatchEvent(new MouseEvent('mousedown', { button: leftMouse, clientX: 12, clientY: 18, bubbles: true, cancelable: true }));
          canvas.dispatchEvent(new MouseEvent('mousemove', { button: leftMouse, clientX: 22, clientY: 28, bubbles: true, cancelable: true }));
          canvas.dispatchEvent(new WheelEvent('wheel', { deltaX: 2, deltaY: -3, bubbles: true, cancelable: true }));
          canvas.dispatchEvent(new MouseEvent('mouseup', { button: leftMouse, clientX: 22, clientY: 28, bubbles: true, cancelable: true }));
          window.dispatchEvent(new KeyboardEvent('keyup', { code: 'Space', bubbles: true, cancelable: true }));
          game.isKeyPressed(spaceKey);
          game.isKeyJustPressed(spaceKey);
          game.isKeyJustReleased(spaceKey);
          game.isMouseButtonPressed(leftMouse);
          game.isMouseButtonJustPressed(leftMouse);
          game.isMouseButtonJustReleased(leftMouse);
          game.getMousePosition();
          game.getMouseDelta();
          game.getScrollDelta();
          game.mapActionKey('jump', spaceKey);
          game.isActionPressed('jump');
          game.isActionJustPressed('jump');
          game.isActionJustReleased('jump');

          const renderStats = game.getRenderStats();
          const fpsStats = game.getFpsStats();
          const memorySummary = game.getMemorySummary();
          const snapshot = parseDebuggerSnapshot(game);
          const manifest = parseDebuggerManifest({
            getDebuggerManifestJson() {
              return JSON.stringify({ route: 'local-web-runtime' });
            },
          });
          game.setDebuggerPaused(true);
          game.stepDebugger(0, 1);
          game.stepDebugger(1, 2);
          game.setDebuggerTimeScale(0.5);
          game.setDebuggerDebugDrawEnabled(true);
          game.injectDebuggerKeyEvent(spaceKey, true);
          game.injectDebuggerMouseButton(leftMouse, true);
          game.injectDebuggerMousePosition({ x: 4, y: 5 });
          game.injectDebuggerScroll({ x: 1, y: -1 });
          game.setDebuggerProfilingEnabled(true);
          game.setDebuggerSelectedEntity(Number(entity.toBits()));
          game.clearDebuggerSelectedEntity();

          const geometry = {
            aabb: game.collisionAabbAabb(0, 0, 5, 5, 3, 0, 5, 5),
            circle: game.collisionCircleCircle(0, 0, 5, 3, 0, 5),
            circleAabb: game.collisionCircleAabb(0, 0, 3, 2, 2, 4, 4),
            pointRect: game.pointInRect(1, 1, 0, 0, 5, 5),
            pointCircle: game.pointInCircle(1, 1, 0, 0, 3),
            aabbOverlap: game.aabbOverlap(0, 0, 5, 5, 1, 1, 6, 6),
            circleOverlap: game.circleOverlap(0, 0, 3, 2, 0, 3),
            distance: game.distance(0, 0, 3, 4),
            distanceSquared: game.distanceSquared(0, 0, 3, 4),
          };

          let sceneError = '';
          try {
            game.loadScene('unsupported', '{}');
          } catch (error) {
            sceneError = error instanceof Error ? error.message : String(error);
          }
          try {
            game.unloadScene('unsupported');
          } catch {}
          try {
            game.setActiveScene(1, true);
          } catch {}

          const stubContext = {
            networkHost() { stubCalls.push('host'); return 41; },
            networkConnectWithPeer() { stubCalls.push('connect'); return { handle: 42, peerId: 7 }; },
            networkReceivePacket() { stubCalls.push('receive'); return { peerId: 7, data: new Uint8Array([9, 8, 7]) }; },
            networkSend() { stubCalls.push('send'); return 0; },
            networkPoll() { stubCalls.push('poll'); return 0; },
            networkDisconnect() { stubCalls.push('disconnect'); return 0; },
            getNetworkStats() {
              stubCalls.push('stats');
              return {
                bytesSent: 1,
                bytesReceived: 2,
                packetsSent: 3,
                packetsReceived: 4,
                packetsLost: 0,
                rttMs: 0,
                sendBandwidthBytesPerSec: 0,
                receiveBandwidthBytesPerSec: 0,
                packetLossPercent: 0,
                jitterMs: 0,
              };
            },
            networkPeerCount() { stubCalls.push('peerCount'); return 1; },
            setNetworkSimulation() { stubCalls.push('setSimulation'); return 0; },
            clearNetworkSimulation() { stubCalls.push('clearSimulation'); return 0; },
            setNetworkOverlayHandle() { stubCalls.push('setOverlay'); return 0; },
            clearNetworkOverlayHandle() { stubCalls.push('clearOverlay'); return 0; },
          };
          const managerFromStub = new NetworkManager(stubContext);
          managerFromStub.host(NetworkProtocol.WebSocket, 0);
          managerFromStub.connect(NetworkProtocol.WebSocket, 'ws://example.invalid', 0);
          let missingPeerError = '';
          try {
            new NetworkEndpoint(stubContext, 50).send(new Uint8Array([1]));
          } catch (error) {
            missingPeerError = error instanceof Error ? error.message : String(error);
          }
          const directEndpoint = new NetworkEndpoint(stubContext, 51, 7);
          directEndpoint.send(new Uint8Array([1, 2, 3]), 0);
          directEndpoint.sendTo(9, new Uint8Array([4, 5]), 1);
          directEndpoint.receive();
          directEndpoint.poll();
          directEndpoint.disconnect();
          directEndpoint.getStats();
          directEndpoint.peerCount();
          directEndpoint.setSimulation({ oneWayLatencyMs: 4, jitterMs: 1, packetLossPercent: 2 });
          directEndpoint.clearSimulation();
          directEndpoint.setOverlayTarget();
          directEndpoint.clearOverlayTarget();

          stubCalls.push(String(game.createCube(0, 1, 1, 1)));
          stubCalls.push(String(game.createPlane(0, 1, 1)));
          stubCalls.push(String(game.createSphere(0, 1, 16)));
          stubCalls.push(String(game.createCylinder(0, 1, 2, 16)));
          stubCalls.push(String(game.setObjectPosition(1, 0, 0, 0)));
          stubCalls.push(String(game.setObjectRotation(1, 0, 0, 0)));
          stubCalls.push(String(game.setObjectScale(1, 1, 1, 1)));
          stubCalls.push(String(game.destroyObject(1)));
          stubCalls.push(String(game.addLight(0, 0, 0, 0, 0, -1, 0, 1, 1, 1, 1, 10, 0.5)));
          stubCalls.push(String(game.updateLight(1, 0, 0, 0, 0, 0, -1, 0, 1, 1, 1, 1, 10, 0.5)));
          stubCalls.push(String(game.removeLight(1)));
          stubCalls.push(String(game.setCameraPosition3D(0, 0, 5)));
          stubCalls.push(String(game.setCameraRotation3D(0, 0, 0)));
          stubCalls.push(String(game.configureGrid(true, 10, 10)));
          stubCalls.push(String(game.setGridEnabled(true)));
          stubCalls.push(String(game.configureSkybox(true, 0.1, 0.2, 0.3, 1)));
          stubCalls.push(String(game.configureFog(true, 0.4, 0.5, 0.6, 0.2)));
          stubCalls.push(String(game.setFogEnabled(true)));
          stubCalls.push(String(game.render3D()));
          game.setViewport(0, 0, 64, 64);
          game.enableDepthTest();
          game.disableDepthTest();
          game.clearDepth();
          game.disableBlending();
          game.setFpsOverlayEnabled(true);
          game.setFpsUpdateInterval(0.25);
          game.setFpsOverlayCorner(1);

          const disconnectStatus = endpoint.disconnect();
          const sendAfterDisconnectStatus = endpoint.send(payload, packet.peerId);
          game.removeSprite(entity);
          game.removeName(entity);
          game.removeTransform2d(entity);
          game.despawn(entity);
          if (clone) game.despawn(clone);
          if (recursiveClone) game.despawn(recursiveClone);
          game.close();
          game.destroy();

          const stubLog = [];
          const preloadProgress = [];
          let stubFrameCount = 0;
          let builderConfig = null;

          function wait(ms) {
            return new Promise((resolve) => setTimeout(resolve, ms));
          }

          function createStubHandle({
            withOptionalAudioFns = true,
            renderStats = { draw_calls: 1, triangles: 2, texture_binds: 3, shader_binds: 4 },
          } = {}) {
            let nextTextureHandle = 200;
            let nextFontHandle = 300;
            let nextEntityBits = 400n;
            const handle = {
              delta_time: 1 / 60,
              total_time: 2.5,
              fps: 60,
              title: 'stub-game',
              frame_count: 3n,
              window_width: 48,
              window_height: 48,
              _mouseX: 0,
              _mouseY: 0,
              _scrollX: 0,
              _scrollY: 0,
              free() { stubLog.push('free'); },
              begin_frame(dt) {
                stubLog.push(`begin:${dt}`);
                handle.delta_time = dt;
                handle.frame_count += 1n;
              },
              end_frame() { stubLog.push('end'); },
              set_clear_color(r, g, b, a) { stubLog.push(`clear:${r},${g},${b},${a}`); },
              set_canvas_size(w, h) {
                stubLog.push(`resize:${w}x${h}`);
                handle.window_width = w;
                handle.window_height = h;
              },
              has_renderer() { return true; },
              register_texture_from_bytes(data) {
                stubLog.push(`register-texture:${data.length}`);
                return nextTextureHandle += 1;
              },
              register_font_from_bytes(data) {
                stubLog.push(`register-font:${data.length}`);
                return nextFontHandle += 1;
              },
              destroy_texture(textureHandleValue) { stubLog.push(`destroy-texture:${textureHandleValue}`); },
              destroy_font(fontHandleValue) {
                stubLog.push(`destroy-font:${fontHandleValue}`);
                return true;
              },
              draw_text(fontHandleValue, text) {
                stubLog.push(`draw-text:${fontHandleValue}:${text}`);
                return true;
              },
              draw_sprite() { stubLog.push('draw-sprite'); },
              draw_sprite_rect() {
                stubLog.push('draw-sprite-rect');
                return true;
              },
              draw_quad() { stubLog.push('draw-quad'); },
              spawn_empty() { return nextEntityBits += 1n; },
              spawn_batch(count) {
                return BigUint64Array.from(
                  Array.from({ length: count }, () => (nextEntityBits += 1n)),
                );
              },
              despawn(bits) {
                stubLog.push(`despawn:${bits}`);
                return true;
              },
              despawn_batch(bits) {
                stubLog.push(`despawn-batch:${bits.length}`);
                return bits.length;
              },
              clone_entity(bits) { return bits + 10n; },
              clone_entity_recursive(bits) { return bits + 20n; },
              entity_count() { return 6; },
              is_alive(bits) { return bits !== 0n; },
              add_transform2d() {},
              get_transform2d() {
                return { position_x: 1, position_y: 2, rotation: 3, scale_x: 4, scale_y: 5 };
              },
              set_transform2d() {},
              has_transform2d() { return true; },
              remove_transform2d() { return true; },
              add_sprite() {},
              get_sprite() {
                return {
                  texture_handle: 7,
                  r: 1,
                  g: 0.5,
                  b: 0.25,
                  a: 1,
                  flip_x: false,
                  flip_y: true,
                  anchor_x: 0.5,
                  anchor_y: 0.25,
                };
              },
              set_sprite() {},
              has_sprite() { return true; },
              remove_sprite() { return true; },
              add_name() {},
              get_name() { return 'stub-entity'; },
              has_name() { return true; },
              remove_name() { return true; },
              press_key(kc) { stubLog.push(`press-key:${kc}`); },
              release_key(kc) { stubLog.push(`release-key:${kc}`); },
              press_mouse_button(button) { stubLog.push(`press-mouse:${button}`); },
              release_mouse_button(button) { stubLog.push(`release-mouse:${button}`); },
              set_mouse_position(x, y) {
                handle._mouseX = x;
                handle._mouseY = y;
              },
              add_scroll_delta(dx, dy) {
                handle._scrollX = dx;
                handle._scrollY = dy;
              },
              is_key_pressed() { return true; },
              is_key_just_pressed() { return true; },
              is_key_just_released() { return false; },
              is_mouse_button_pressed() { return true; },
              is_mouse_button_just_pressed() { return true; },
              is_mouse_button_just_released() { return false; },
              mouse_x() { return handle._mouseX; },
              mouse_y() { return handle._mouseY; },
              scroll_dx() { return handle._scrollX; },
              scroll_dy() { return handle._scrollY; },
              map_action_key() { return true; },
              is_action_pressed() { return true; },
              is_action_just_pressed() { return true; },
              is_action_just_released() { return false; },
              get_render_stats() { return renderStats; },
              collision_aabb_aabb() {
                return { point_x: 1, point_y: 2, normal_x: 0, normal_y: 1, penetration: 0.5 };
              },
              collision_circle_circle() {
                return { point_x: 2, point_y: 3, normal_x: 1, normal_y: 0, penetration: 0.25 };
              },
              collision_circle_aabb() {
                return { point_x: 3, point_y: 4, normal_x: -1, normal_y: 0, penetration: 0.75 };
              },
              point_in_rect() { return true; },
              point_in_circle() { return true; },
              aabb_overlap() { return true; },
              circle_overlap() { return true; },
              distance() { return 5; },
              distance_squared() { return 25; },
              audio_play() { return 11; },
              audio_stop() { return 0; },
              audio_pause() { return 0; },
              audio_resume() { return 0; },
              audio_stop_all() { return 0; },
              audio_set_player_volume() { return 0; },
              audio_set_player_speed() { return 0; },
              audio_play_spatial_3d() { return 12; },
              audio_update_spatial_volume_3d() { return 0; },
              audio_set_listener_position_3d() { return 0; },
              audio_set_source_position_3d() { return 0; },
              audio_crossfade() { return 0; },
              network_host() { return 13; },
              network_connect() { return 14; },
              network_connect_with_peer() { return { handle: 15, peer_id: 16n }; },
              network_disconnect() { return 0; },
              network_send() { return 0; },
              network_receive() { return new Uint8Array([6, 5, 4]); },
              network_receive_packet() { return { peer_id: 16n, data: new Uint8Array([3, 2, 1]) }; },
              network_poll() { return 0; },
              get_network_stats() {
                return {
                  bytes_sent: 1,
                  bytes_received: 2,
                  packets_sent: 3,
                  packets_received: 4,
                  packets_lost: 0,
                  rtt_ms: 5,
                  send_bandwidth_bytes_per_sec: 6,
                  receive_bandwidth_bytes_per_sec: 7,
                  packet_loss_percent: 0,
                  jitter_ms: 0,
                };
              },
              get_network_capabilities() {
                return {
                  supports_hosting: false,
                  max_connections: 8,
                  max_channels: 2,
                  max_message_size: 4096,
                };
              },
              network_peer_count() { return 1; },
              set_network_simulation() { return 0; },
              clear_network_simulation() { return 0; },
              set_network_overlay_handle() { return 0; },
              clear_network_overlay_handle() { return 0; },
              initDebugger(label) { stubLog.push(`init-debugger:${label}`); },
              dispatchDebuggerRequest(json) {
                const request = JSON.parse(json);
                if (request.verb === 'stop_recording') {
                  return JSON.stringify({ result: { manifest_json: '{"ok":true}', data: [1, 2, 3] } });
                }
                if (request.verb === 'get_replay_status') {
                  return JSON.stringify({ status: 'idle' });
                }
                if (request.verb === 'get_metrics_trace') {
                  return JSON.stringify({ traces: [] });
                }
                return JSON.stringify({ ok: true, request });
              },
              getDebuggerSnapshotJson() { return JSON.stringify({ snapshot: 'ok' }); },
            };
            if (withOptionalAudioFns) {
              handle.audio_play_on_channel = () => 21;
              handle.audio_play_with_settings = () => 22;
              handle.audio_set_global_volume = () => 0;
              handle.audio_get_global_volume = () => 0.75;
              handle.audio_set_channel_volume = () => 0;
              handle.audio_get_channel_volume = () => 0.5;
              handle.audio_is_playing = () => 1;
              handle.audio_active_count = () => 2;
              handle.audio_cleanup_finished = () => 0;
              handle.audio_crossfade_to = () => 23;
              handle.audio_mix_with = () => 24;
              handle.audio_update_crossfades = () => 0;
              handle.audio_active_crossfade_count = () => 1;
              handle.audio_activate = () => 0;
            }
            return handle;
          }

          function createStubGame(handle, canvasElement) {
            const stubGame = Object.create(GoudGame.prototype);
            stubGame.handle = handle;
            stubGame.canvas = canvasElement;
            stubGame.detachInput = null;
            stubGame.rafId = 0;
            stubGame.running = false;
            stubGame.lastTs = 0;
            stubGame._shouldClose = false;
            stubGame._updateFn = null;
            stubGame._audioGlobalVolume = 1;
            stubGame._audioChannelVolumes = new Map();
            stubGame._activeAudioPlayers = new Set();
            stubGame.preloadedTextures = new Map();
            stubGame.preloadedFonts = new Map();
            stubGame.texturePathByHandle = new Map();
            stubGame.fontPathByHandle = new Map();
            stubGame.preloadInFlight = false;
            return stubGame;
          }

          const stubCanvas = document.createElement('canvas');
          stubCanvas.width = 48;
          stubCanvas.height = 48;
          stubCanvas.getBoundingClientRect = () => ({
            left: 5,
            top: 7,
            right: 53,
            bottom: 55,
            width: 48,
            height: 48,
            x: 5,
            y: 7,
            toJSON() { return {}; },
          });
          document.body.appendChild(stubCanvas);

          const stubGame = createStubGame(createStubHandle({ withOptionalAudioFns: true }), stubCanvas);
          const fallbackAudioGame = createStubGame(
            createStubHandle({ withOptionalAudioFns: false, renderStats: undefined }),
            stubCanvas,
          );

          const warningMessages = [];
          const originalWarn = console.warn;
          console.warn = (...args) => warningMessages.push(args.join(' '));
          try {
            stubGame.preloadInFlight = true;
            let preloadGuardError = '';
            try {
              stubGame.run(() => {});
            } catch (error) {
              preloadGuardError = error instanceof Error ? error.message : String(error);
            }
            stubGame.preloadInFlight = false;

            let invalidPreloadError = '';
            try {
              await stubGame.preload(['coverage.invalid']);
            } catch (error) {
              invalidPreloadError = error instanceof Error ? error.message : String(error);
            }

            const preloadedHandles = await stubGame.preload(
              [textureUrl, { path: fontUrl, kind: 'font' }],
              {
                onProgress(update) {
                  preloadProgress.push(`${update.kind}:${update.path}`);
                },
              },
            );
            const textureHandle = await stubGame.loadTexture(textureUrl);
            const cachedTextureHandle = await stubGame.loadTexture(textureUrl);
            const fontHandle = await stubGame.loadFont(fontUrl);
            const cachedFontHandle = await stubGame.loadFont(fontUrl);
            stubGame.drawText(fontHandle, 'stub-text', 1, 2, 14, 0, 64, 1, 0, Color.cyan());
            stubGame.drawSprite(textureHandle, 2, 3, 10, 12, 0.25, Color.red());
            stubGame.drawSpriteRect(textureHandle, 4, 5, 12, 14, 0.5, 0, 0, 0.5, 0.5, Color.green());
            stubGame.drawQuad(6, 7, 8, 9, Color.blue());

            stubGame.deltaTime;
            stubGame.fps;
            stubGame.windowWidth;
            stubGame.windowHeight;
            stubGame.title;
            stubGame.totalTime;
            stubGame.frameCount;
            stubGame.shouldClose();
            stubGame.stop({ toBits() { return 1n; } });
            stubGame.resume();
            stubGame.run(async (dt) => {
              stubLog.push(`update:${dt}`);
              stubFrameCount += 1;
            });
            stubGame.run(() => {});
            await wait(30);
            window.dispatchEvent(new KeyboardEvent('keydown', { code: 'KeyA', bubbles: true, cancelable: true }));
            window.dispatchEvent(new KeyboardEvent('keyup', { code: 'KeyA', bubbles: true, cancelable: true }));
            stubCanvas.dispatchEvent(new MouseEvent('mousedown', { button: 0, clientX: 12, clientY: 16, bubbles: true, cancelable: true }));
            stubCanvas.dispatchEvent(new MouseEvent('mousemove', { clientX: 22, clientY: 26, bubbles: true, cancelable: true }));
            stubCanvas.dispatchEvent(new WheelEvent('wheel', { deltaX: 3, deltaY: -2, bubbles: true, cancelable: true }));
            const touchStart = new Event('touchstart', { bubbles: true, cancelable: true });
            Object.defineProperty(touchStart, 'touches', { value: [{ clientX: 18, clientY: 21 }] });
            stubCanvas.dispatchEvent(touchStart);
            const touchEnd = new Event('touchend', { bubbles: true, cancelable: true });
            Object.defineProperty(touchEnd, 'touches', { value: [] });
            stubCanvas.dispatchEvent(touchEnd);
            stubGame.pause();
            stubGame.resume();
            await wait(30);
            stubGame.stop();

            const stubEntity = stubGame.spawnEmpty();
            const stubBatch = stubGame.spawnBatch(3);
            const stubClone = stubGame.cloneEntity(stubEntity);
            const stubRecursiveClone = stubGame.cloneEntityRecursive(stubEntity);
            stubGame.isAlive(stubClone);
            stubGame.isAlive(stubRecursiveClone);
            stubGame.entityCount();
            stubGame.addTransform2d(stubEntity, {
              positionX: 1,
              positionY: 2,
              rotation: 3,
              scaleX: 4,
              scaleY: 5,
            });
            stubGame.getTransform2d(stubEntity);
            stubGame.setTransform2d(stubEntity, {
              positionX: 6,
              positionY: 7,
              rotation: 8,
              scaleX: 9,
              scaleY: 10,
            });
            stubGame.hasTransform2d(stubEntity);
            stubGame.addSprite(stubEntity, {
              textureHandle,
              colorR: 1,
              colorG: 1,
              colorB: 1,
              colorA: 1,
              sourceRectX: 0,
              sourceRectY: 0,
              sourceRectWidth: 0,
              sourceRectHeight: 0,
              hasSourceRect: false,
              flipX: false,
              flipY: false,
              anchorX: 0.5,
              anchorY: 0.5,
              customSizeX: 0,
              customSizeY: 0,
              hasCustomSize: false,
            });
            stubGame.getSprite(stubEntity);
            stubGame.setSprite(stubEntity, {
              textureHandle,
              colorR: 0.5,
              colorG: 0.5,
              colorB: 0.5,
              colorA: 1,
              sourceRectX: 0,
              sourceRectY: 0,
              sourceRectWidth: 0,
              sourceRectHeight: 0,
              hasSourceRect: false,
              flipX: true,
              flipY: false,
              anchorX: 0.25,
              anchorY: 0.75,
              customSizeX: 0,
              customSizeY: 0,
              hasCustomSize: false,
            });
            stubGame.hasSprite(stubEntity);
            stubGame.addName(stubEntity, 'stub');
            stubGame.getName(stubEntity);
            stubGame.hasName(stubEntity);
            stubGame.getRenderStats();
            fallbackAudioGame.getRenderStats();
            stubGame.audioPlay(new Uint8Array([1, 2, 3]));
            stubGame.audioPlayOnChannel(new Uint8Array([1, 2, 3]), 1);
            stubGame.audioPlayWithSettings(new Uint8Array([1, 2, 3]), 0.5, 1.25, false, 2);
            stubGame.audioPause(11);
            stubGame.audioResume(11);
            stubGame.audioSetGlobalVolume(0.75);
            stubGame.audioGetGlobalVolume();
            stubGame.audioSetChannelVolume(2, 0.5);
            stubGame.audioGetChannelVolume(2);
            stubGame.audioIsPlaying(11);
            stubGame.audioActiveCount();
            stubGame.audioCleanupFinished();
            stubGame.audioPlaySpatial3d(new Uint8Array([4, 5, 6]), 0, 0, 0, 1, 1, 1, 5, 0.5);
            stubGame.audioUpdateSpatial3d(12, 1, 2, 3, 4, 5, 6, 7, 0.2);
            stubGame.audioSetListenerPosition3d(1, 2, 3);
            stubGame.audioSetSourcePosition3d(12, 3, 2, 1, 5, 0.5);
            stubGame.audioSetPlayerVolume(12, 0.4);
            stubGame.audioSetPlayerSpeed(12, 1.1);
            stubGame.audioCrossfade(11, 12, 0.5);
            stubGame.audioCrossfadeTo(11, new Uint8Array([7, 8, 9]), 0.25, 1);
            stubGame.audioMixWith(11, new Uint8Array([9, 8, 7]), 0.3, 2);
            stubGame.audioUpdateCrossfades(1 / 60);
            stubGame.audioActiveCrossfadeCount();
            stubGame.audioActivate();
            stubGame.audioStop(11);
            stubGame.audioStopAll();

            fallbackAudioGame.audioPlayOnChannel(new Uint8Array([1]), 0);
            fallbackAudioGame.audioPlayWithSettings(new Uint8Array([1]), 0.2, 0.8, true, 0);
            fallbackAudioGame.audioSetGlobalVolume(0.25);
            fallbackAudioGame.audioGetGlobalVolume();
            fallbackAudioGame.audioSetChannelVolume(1, 0.4);
            fallbackAudioGame.audioGetChannelVolume(1);
            fallbackAudioGame.audioIsPlaying(11);
            fallbackAudioGame.audioActiveCount();
            fallbackAudioGame.audioCleanupFinished();
            fallbackAudioGame.audioCrossfadeTo(11, new Uint8Array([1, 2]), 0.5, 1);
            fallbackAudioGame.audioMixWith(11, new Uint8Array([3, 4]), 0.6, 2);
            fallbackAudioGame.audioUpdateCrossfades(0.1);
            fallbackAudioGame.audioActiveCrossfadeCount();
            fallbackAudioGame.audioActivate();

            stubGame.isKeyPressed(spaceKey);
            stubGame.isKeyJustPressed(spaceKey);
            stubGame.isKeyJustReleased(spaceKey);
            stubGame.isMouseButtonPressed(leftMouse);
            stubGame.isMouseButtonJustPressed(leftMouse);
            stubGame.isMouseButtonJustReleased(leftMouse);
            stubGame.getMousePosition();
            stubGame.getMouseDelta();
            stubGame.getScrollDelta();
            stubGame.mapActionKey('stub-action', spaceKey);
            stubGame.isActionPressed('stub-action');
            stubGame.isActionJustPressed('stub-action');
            stubGame.isActionJustReleased('stub-action');
            stubGame.networkHost(NetworkProtocol.WebSocket, 1);
            stubGame.networkConnect(NetworkProtocol.WebSocket, 'ws://stub', 2);
            const directConnect = stubGame.networkConnectWithPeer(NetworkProtocol.WebSocket, 'ws://stub', 3);
            stubGame.networkSend(directConnect.handle, directConnect.peerId, new Uint8Array([1]), 0);
            stubGame.networkReceive(directConnect.handle);
            stubGame.networkReceivePacket(directConnect.handle);
            stubGame.networkPoll(directConnect.handle);
            stubGame.getNetworkStats(directConnect.handle);
            stubGame.networkPeerCount(directConnect.handle);
            stubGame.setNetworkSimulation(directConnect.handle, { oneWayLatencyMs: 2, jitterMs: 1, packetLossPercent: 0 });
            stubGame.clearNetworkSimulation(directConnect.handle);
            stubGame.setNetworkOverlayHandle(directConnect.handle);
            stubGame.clearNetworkOverlayHandle();
            stubGame.networkDisconnect(directConnect.handle);
            stubGame.collisionAabbAabb(0, 0, 1, 1, 1, 1, 1, 1);
            stubGame.collisionCircleCircle(0, 0, 1, 1, 1, 1);
            stubGame.collisionCircleAabb(0, 0, 1, 1, 1, 1, 1);
            stubGame.pointInRect(0, 0, 0, 0, 1, 1);
            stubGame.pointInCircle(0, 0, 0, 0, 1);
            stubGame.aabbOverlap(0, 0, 1, 1, 0, 0, 1, 1);
            stubGame.circleOverlap(0, 0, 1, 1, 1, 1);
            stubGame.distance(0, 0, 3, 4);
            stubGame.distanceSquared(0, 0, 3, 4);
            stubGame.captureDebuggerFrame();
            stubGame.startDebuggerRecording();
            const replayArtifact = stubGame.stopDebuggerRecording();
            stubGame.startDebuggerReplay(replayArtifact.data);
            stubGame.stopDebuggerReplay();
            stubGame.getDebuggerReplayStatusJson();
            stubGame.getDebuggerMetricsTraceJson();
            stubGame.getDebuggerSnapshotJson();
            stubGame.getDebuggerManifestJson();
            stubGame.getMemorySummary();
            stubGame.getRenderCapabilities();
            stubGame.getPhysicsCapabilities();
            stubGame.getAudioCapabilities();
            stubGame.getInputCapabilities();
            stubGame.getNetworkCapabilities();
            stubGame.checkHotSwapShortcut = GoudGame.prototype.checkHotSwapShortcut;
            let hotSwapError = '';
            try {
              stubGame.checkHotSwapShortcut();
            } catch (error) {
              hotSwapError = error instanceof Error ? error.message : String(error);
            }
            let collisionCallbackError = '';
            try {
              stubGame.physicsSetCollisionCallback(1, 2);
            } catch (error) {
              collisionCallbackError = error instanceof Error ? error.message : String(error);
            }
            let collisionEventsError = '';
            try {
              stubGame.physicsCollisionEventsCount();
            } catch (error) {
              collisionEventsError = error instanceof Error ? error.message : String(error);
            }
            stubGame.physicsCollisionEventsRead(0);
            stubGame.physicsRaycastEx(0, 0, 1, 0, 10, 0xffff);
            stubGame.play(stubEntity);
            stubGame.setState(stubEntity, 'idle');
            stubGame.setParameterBool(stubEntity, 'enabled', true);
            stubGame.setParameterFloat(stubEntity, 'speed', 1.5);
            stubGame.animationLayerStackCreate(stubEntity);
            stubGame.animationLayerAdd(stubEntity, 'base', 0);
            stubGame.animationLayerSetWeight(stubEntity, 0, 1);
            stubGame.animationLayerPlay(stubEntity, 0);
            stubGame.animationLayerSetClip(stubEntity, 0, 2, 0.25, 0);
            stubGame.animationLayerAddFrame(stubEntity, 0, 0, 0, 8, 8);
            stubGame.animationLayerReset(stubEntity, 0);
            stubGame.animationClipAddEvent(stubEntity, 0, 'event', 0, 1, 1.5, 'payload');
            stubGame.animationEventsCount();
            stubGame.animationEventsRead(0);

            const originalWebSocket = window.WebSocket;
            const fakeSockets = [];
            class FakeWebSocket {
              constructor(url) {
                this.url = url;
                fakeSockets.push(this);
                setTimeout(() => this.onopen?.(), 0);
              }
              send(message) {
                stubLog.push(`debug-send:${message}`);
              }
              emit(payload) {
                this.onmessage?.({ data: JSON.stringify(payload) });
              }
            }
            window.WebSocket = FakeWebSocket;
            try {
              stubGame.connectDebugger('ws://debug.local');
              await wait(10);
              fakeSockets[0]?.emit({ type: 'registration_ack' });
              fakeSockets[0]?.emit({ verb: 'capture_frame' });
              fakeSockets[0]?.emit({ verb: 'get_metrics_trace' });
              await wait(10);
            } finally {
              window.WebSocket = originalWebSocket;
            }

            const stubUi = Object.create(UiManager.prototype);
            const uiEvents = [{ type: 'click', nodeId: 1n }];
            stubUi.handle = {
              free() { stubLog.push('ui-free'); },
              update() { stubLog.push('ui-update'); },
              render() { stubLog.push('ui-render'); },
              node_count() { return 4; },
              create_node(componentType) { return BigInt(componentType + 1); },
              remove_node() { return 0; },
              set_parent() { return 0; },
              get_parent() { return 1n; },
              get_child_count() { return 2; },
              get_child_at(_nodeId, index) { return BigInt(index + 2); },
              set_widget() { return 0; },
              set_style() { return 0; },
              set_label_text() { return 0; },
              set_button_enabled() { return 0; },
              set_image_texture_path() { return 0; },
              set_slider() { return 0; },
              event_count() { return uiEvents.length; },
              event_read(index) { return uiEvents[index]; },
            };
            const panelNode = stubUi.createPanel();
            const labelNode = stubUi.createLabel('stub-label');
            const buttonNode = stubUi.createButton(false);
            const imageNode = stubUi.createImage(textureUrl);
            const sliderNode = stubUi.createSlider(0, 10, 5, true);
            stubUi.nodeCount();
            stubUi.setParent(labelNode, panelNode);
            stubUi.getParent(labelNode);
            stubUi.getChildCount(panelNode);
            stubUi.getChildAt(panelNode, 0);
            stubUi.setWidget(panelNode, 0);
            stubUi.setStyle(panelNode, {
              backgroundColor: Color.rgba(0.1, 0.2, 0.3, 1),
              foregroundColor: Color.white(),
              borderColor: Color.black(),
              borderWidth: 1,
              fontFamily: 'stub-font',
              fontSize: 12,
              texturePath: textureUrl,
              widgetSpacing: 2,
            });
            stubUi.setLabelText(labelNode, 'updated');
            stubUi.setButtonEnabled(buttonNode, true);
            stubUi.setImageTexturePath(imageNode, textureUrl);
            stubUi.setSlider(sliderNode, 0, 20, 10, false);
            stubUi.eventCount();
            stubUi.eventRead(0);
            stubUi.update();
            stubUi.render();
            stubUi.removeNode(sliderNode);
            stubUi.destroy();

            const rect = new Rect(0, 0, 10, 10);
            const vec = new Vec2(3, 4);
            const vecMath = vec.add(Vec2.one()).sub(Vec2.left()).scale(0.5);
            const colorMath = Color.fromHex(0xff00ff).withAlpha(0.5).lerp(Color.green(), 0.25);
            const vec3 = Vec3.one();

            const engineConfig = new EngineConfig();
            engineConfig
              .setTitle('coverage-builder')
              .setSize(32, 32)
              .setFpsOverlay(true)
              .setPhysicsDebug(true)
              .setPhysicsBackend2D(sdk.PhysicsBackend2D?.None ?? 0)
              .setRenderBackend(sdk.RenderBackendKind?.Auto ?? 0)
              .setWindowBackend(sdk.WindowBackendKind?.Auto ?? 0)
              .setDebugger({ enabled: true, routeLabel: 'stub-builder' });
            const originalCreate = GoudGame.create;
            GoudGame.create = async (config) => {
              builderConfig = config;
              return stubGame;
            };
            try {
              await engineConfig.build();
            } finally {
              GoudGame.create = originalCreate;
            }
            engineConfig.destroy();

            stubGame.removeSprite(stubEntity);
            stubGame.removeName(stubEntity);
            stubGame.removeTransform2d(stubEntity);
            stubGame.despawn(stubClone);
            stubGame.despawn(stubRecursiveClone);
            stubGame.despawnBatch(stubBatch);
            stubGame.despawn(stubEntity);
            stubGame.destroyTexture(textureHandle);
            stubGame.destroyFont(fontHandle);
            stubGame.close();
            stubGame.destroy();

            if (!preloadGuardError || !invalidPreloadError || !hotSwapError || !collisionCallbackError || !collisionEventsError) {
              throw new Error('Stub coverage block missed an expected error path.');
            }
            if (cachedTextureHandle !== textureHandle || cachedFontHandle !== fontHandle) {
              throw new Error('Stub asset cache did not return stable handles.');
            }
            if (preloadedHandles[textureUrl] !== textureHandle || preloadedHandles[fontUrl] !== fontHandle) {
              throw new Error('Stub preload handles did not match explicit load handles.');
            }
            if (!builderConfig?.debugger?.enabled) {
              throw new Error('EngineConfig.build() did not forward debugger config.');
            }
            if (!rect.contains(new Vec2(1, 1)) || !rect.intersects(new Rect(5, 5, 2, 2))) {
              throw new Error('Math helpers returned unexpected results.');
            }
            if (vecMath.length() <= 0 || colorMath.a <= 0 || vec3.x !== 1) {
              throw new Error('Vector or color helpers returned unexpected results.');
            }
          } finally {
            console.warn = originalWarn;
          }

          return {
            handle: endpoint.handle,
            peerId: packet.peerId,
            sendStatus,
            disconnectStatus,
            sendAfterDisconnectStatus,
            bytesSent: stats.bytesSent,
            bytesReceived: stats.bytesReceived,
            packetsSent: stats.packetsSent,
            packetsReceived: stats.packetsReceived,
            renderStats,
            fpsStats,
            memorySummary,
            snapshotType: typeof snapshot,
            manifestRoute: manifest.route,
            geometry,
            sceneError,
            missingPeerError,
            stubCalls,
            preloadProgress,
            stubLogCount: stubLog.length,
            stubFrameCount,
            warningMessages,
            builderConfig,
          };
        }, {
          importUrl: `${origin}/dist/web/web/index.js`,
          wasmUrl: `${origin}/wasm/goud_engine_bg.wasm`,
          wsUrl,
          textureUrl: `${origin}/test/fixtures/test_bitmap.png`,
          fontUrl: `${origin}/test/fixtures/test_font.ttf`,
        });

        assert.equal(result.sendStatus, 0);
        assert.equal(result.disconnectStatus, 0);
        assert.notEqual(result.sendAfterDisconnectStatus, 0);
        assert.ok(result.handle > 0);
        assert.ok(result.peerId >= 0);
        assert.ok(result.bytesSent > 0);
        assert.ok(result.bytesReceived > 0);
        assert.ok(result.packetsSent >= 1);
        assert.ok(result.packetsReceived >= 1);
        assert.ok(result.renderStats);
        assert.ok(result.fpsStats);
        assert.ok(result.memorySummary);
        assert.equal(result.snapshotType, 'object');
        assert.equal(result.manifestRoute, 'local-web-runtime');
        assert.equal(result.geometry.pointRect, true);
        assert.equal(result.geometry.pointCircle, true);
        assert.equal(result.geometry.aabbOverlap, true);
        assert.equal(result.geometry.circleOverlap, true);
        assert.equal(result.geometry.distance, 5);
        assert.equal(result.geometry.distanceSquared, 25);
        assert.match(result.sceneError, /Not supported in WASM mode/);
        assert.match(result.missingPeerError, /no default peer ID/i);
        assert.ok(result.stubCalls.length >= 20);
        assert.equal(result.preloadProgress.length, 2);
        assert.ok(result.stubFrameCount >= 1);
        assert.ok(result.warningMessages.some((message) => /should be synchronous/i.test(message)));
        assert.equal(result.builderConfig?.debugger?.routeLabel, 'stub-builder');

        if (collectCoverage) {
          coverageEntries = await page.coverage.stopJSCoverage();
        }

        return { coverageEntries, result };
      } finally {
        if (collectCoverage && coverageEntries.length === 0) {
          try {
            coverageEntries = await page.coverage.stopJSCoverage();
          } catch {
            // The browser may already be closing after a failing smoke run.
          }
        }
        await page.close();
        await context.close();
        await browser.close();
      }
    }),
  );
}
