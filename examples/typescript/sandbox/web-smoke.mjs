import http from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import fs from 'node:fs/promises';

import { chromium } from 'playwright';

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

async function main() {
  const { server, port } = await startStaticServer(repoRoot);
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  const messages = [];
  page.on('console', (msg) => messages.push(`console:${msg.type()}: ${msg.text()}`));
  page.on('pageerror', (err) => messages.push(`pageerror: ${err.message}`));

  try {
    const url = `http://127.0.0.1:${port}/examples/typescript/sandbox/web/index.html?smokeSeconds=1`;
    await page.goto(url, { waitUntil: 'networkidle' });
    await page.waitForFunction(
      () => window.__goudSandboxSmoke && window.__goudSandboxSmoke.status !== 'booting',
      undefined,
      { timeout: 12000 },
    );
    const result = await page.evaluate(() => window.__goudSandboxSmoke);
    if (!result || result.status !== 'passed') {
      throw new Error(`unexpected smoke result: ${JSON.stringify(result)}`);
    }
    console.log(JSON.stringify({ ok: true, result, messages }, null, 2));
  } finally {
    await browser.close();
    await new Promise((resolve) => server.close(resolve));
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
