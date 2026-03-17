import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import libCoverage from 'istanbul-lib-coverage';
import libReport from 'istanbul-lib-report';
import reports from 'istanbul-reports';
import v8toIstanbul from 'v8-to-istanbul';

import { runWebNetworkingRuntimeSmoke } from '../test/web-runtime-smoke-lib.mjs';

const { createCoverageMap } = libCoverage;
const { createContext } = libReport;
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const sdkRoot = path.resolve(__dirname, '..');
const reportDir = path.join(sdkRoot, 'coverage', 'web');
const lineThreshold = 80;

function resolveCoveredFile(entryUrl) {
  const url = new URL(entryUrl);
  const relativePath = decodeURIComponent(url.pathname).replace(/^\/+/, '');
  const filePath = path.normalize(path.join(sdkRoot, relativePath));

  if (!filePath.startsWith(sdkRoot)) {
    return null;
  }
  if (!filePath.endsWith('.js')) {
    return null;
  }
  const isWebDistFile = filePath.includes(`${path.sep}dist${path.sep}web${path.sep}`);
  const isWasmGlueFile = filePath.includes(`${path.sep}wasm${path.sep}goud_engine.js`);
  if (!isWebDistFile && !isWasmGlueFile) {
    return null;
  }

  return filePath;
}

async function buildCoverageMap() {
  const { coverageEntries } = await runWebNetworkingRuntimeSmoke({ collectCoverage: true });
  const coverageMap = createCoverageMap({});
  let coveredFiles = 0;

  for (const entry of coverageEntries) {
    const filePath = resolveCoveredFile(entry.url);
    if (!filePath) {
      continue;
    }

    const converter = v8toIstanbul(filePath, 0, {
      source: await fs.readFile(filePath, 'utf8'),
    });
    await converter.load();
    converter.applyCoverage(entry.functions);
    coverageMap.merge(converter.toIstanbul());
    coveredFiles += 1;
  }

  if (coveredFiles === 0) {
    throw new Error('Web runtime smoke did not produce any SDK coverage entries.');
  }

  return coverageMap;
}

async function main() {
  const coverageMap = await buildCoverageMap();
  const summary = coverageMap.getCoverageSummary().toJSON();

  await fs.rm(reportDir, { force: true, recursive: true });
  await fs.mkdir(reportDir, { recursive: true });

  const context = createContext({
    coverageMap,
    dir: reportDir,
  });

  reports.create('text-summary').execute(context);
  reports.create('json-summary').execute(context);
  reports.create('cobertura').execute(context);

  if (summary.lines.pct < lineThreshold) {
    throw new Error(
      `Web/WASM line coverage ${summary.lines.pct}% is below the ${lineThreshold}% threshold.`,
    );
  }

  console.log(`Web/WASM line coverage: ${summary.lines.pct}%`);
  console.log(`Cobertura report: ${path.join(reportDir, 'cobertura-coverage.xml')}`);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
