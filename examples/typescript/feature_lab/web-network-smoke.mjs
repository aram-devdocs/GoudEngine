import http from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import fs from 'node:fs/promises';

import { chromium } from 'playwright';
import { WebSocketServer } from 'ws';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '../../..');

const MIME_TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.wasm': 'application/wasm',
  '.ts': 'text/plain; charset=utf-8',
};

function contentTypeFor(filePath) {
  return MIME_TYPES[path.extname(filePath).toLowerCase()] ?? 'application/octet-stream';
}

async function startStaticServer(rootDir) {
  const server = http.createServer(async (req, res) => {
    const url = new URL(req.url ?? '/', 'http://127.0.0.1');
    const decodedPath = decodeURIComponent(url.pathname);
    const targetPath = path.join(rootDir, decodedPath);

    if (!targetPath.startsWith(rootDir)) {
      res.writeHead(403).end('forbidden');
      return;
    }

    try {
      const stat = await fs.stat(targetPath);
      const filePath = stat.isDirectory() ? path.join(targetPath, 'index.html') : targetPath;
      const body = await fs.readFile(filePath);
      res.writeHead(200, { 'content-type': contentTypeFor(filePath) });
      res.end(body);
    } catch {
      res.writeHead(404).end('not found');
    }
  });

  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address();
  return { server, port };
}

async function startWsServer() {
  const server = http.createServer();
  const wss = new WebSocketServer({ server });

  wss.on('connection', (socket) => {
    socket.on('message', (data, isBinary) => {
      if (isBinary) {
        socket.send(Buffer.from('pong'));
        return;
      }

      const text = data.toString();
      socket.send(text === 'ping' ? 'pong' : `echo:${text}`);
    });
  });

  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address();
  return { server, wss, port };
}

async function main() {
  const { server: staticServer, port: staticPort } = await startStaticServer(repoRoot);
  const { server: wsServer, wss, port: wsPort } = await startWsServer();
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();

  const messages = [];
  page.on('console', (msg) => messages.push(`console:${msg.type()}: ${msg.text()}`));
  page.on('pageerror', (err) => messages.push(`pageerror: ${err.message}`));

  const smokeUrl =
    `http://127.0.0.1:${staticPort}/examples/typescript/feature_lab/web/network-smoke.html?` +
    `ws=${encodeURIComponent(`ws://127.0.0.1:${wsPort}`)}`;

  try {
    await page.goto(smokeUrl, { waitUntil: 'networkidle' });
    await page.waitForFunction(
      () => window.__goudNetworkSmoke && window.__goudNetworkSmoke.status !== 'booting',
      undefined,
      { timeout: 12000 },
    );
    const result = await page.evaluate(() => window.__goudNetworkSmoke);
    if (!result || result.status !== 'passed') {
      throw new Error(`unexpected smoke result: ${JSON.stringify(result)}`);
    }
    console.log(JSON.stringify({ ok: true, result, messages }, null, 2));
  } finally {
    await browser.close();
    await new Promise((resolve) => wss.close(resolve));
    await new Promise((resolve) => wsServer.close(resolve));
    await new Promise((resolve) => staticServer.close(resolve));
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
