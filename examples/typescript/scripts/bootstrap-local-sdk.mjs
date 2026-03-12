import { existsSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

const VALID_TARGETS = new Set(['node', 'web']);

function run(command, args, cwd) {
  const result = spawnSync(command, args, {
    cwd,
    stdio: 'inherit',
    shell: process.platform === 'win32',
  });

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const repoRoot = path.resolve(__dirname, '..', '..', '..');
const sdkDir = path.join(repoRoot, 'sdks', 'typescript');

const target = process.argv[2] ?? 'node';
if (!VALID_TARGETS.has(target)) {
  console.error(`Unsupported bootstrap target: ${target}`);
  process.exit(1);
}

const nativeTypeOutput = path.join(sdkDir, 'dist', 'node', 'index.d.ts');
const nativeJsOutput = path.join(sdkDir, 'dist', 'node', 'index.js');
const packageRootTypeOutput = path.join(sdkDir, 'dist', 'index.d.ts');
const packageRootJsOutput = path.join(sdkDir, 'dist', 'index.js');
const webTypeOutput = path.join(sdkDir, 'dist', 'web', 'index.d.ts');
const webJsOutput = path.join(sdkDir, 'dist', 'web', 'index.js');
const wasmOutput = path.join(sdkDir, 'wasm', 'goud_engine.js');
const hasNativeAddon = readdirSync(sdkDir, { withFileTypes: true })
  .some((entry) => entry.isFile() && entry.name.endsWith('.node'));

const needsNodeBuild = !existsSync(packageRootTypeOutput)
  || !existsSync(packageRootJsOutput)
  || !existsSync(nativeTypeOutput)
  || !existsSync(nativeJsOutput)
  || !hasNativeAddon;
const needsWebBuild = !existsSync(packageRootTypeOutput)
  || !existsSync(packageRootJsOutput)
  || !existsSync(webTypeOutput)
  || !existsSync(webJsOutput)
  || !existsSync(wasmOutput);

if (target === 'node' && !needsNodeBuild) {
  process.exit(0);
}

if (target === 'web' && !needsWebBuild) {
  process.exit(0);
}

run('npm', ['ci'], sdkDir);

if (target === 'node') {
  run('npm', ['run', 'build:native:debug'], sdkDir);
  run('npm', ['run', 'build:ts'], sdkDir);
} else {
  run('npm', ['run', 'build:native:debug'], sdkDir);
  run('npm', ['run', 'build:ts'], sdkDir);
  run('npm', ['run', 'build:web'], sdkDir);
}
