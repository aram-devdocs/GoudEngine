#!/usr/bin/env node

import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { chromium } from 'playwright';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..', '..', '..');
const manifestPath = path.join(repoRoot, 'examples', 'showcase.manifest.json');
const mediaDir = path.join(repoRoot, 'docs', 'src', 'generated', 'media');

const manifest = JSON.parse(await fs.readFile(manifestPath, 'utf8'));
const packageJson = JSON.parse(
  await fs.readFile(path.join(repoRoot, 'sdks', 'typescript', 'package.json'), 'utf8')
);
const version = packageJson.version;

function slugify(value) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

function previewTheme(entry, section) {
  const key = `${section.id}:${entry.sdk}:${entry.target}:${entry.name}`;
  const palettes = [
    ['#f97316', '#431407'],
    ['#4ade80', '#052e16'],
    ['#38bdf8', '#082f49'],
    ['#f472b6', '#4a044e'],
    ['#facc15', '#422006'],
    ['#a78bfa', '#2e1065'],
    ['#2dd4bf', '#134e4a'],
    ['#fb7185', '#4c0519']
  ];
  let hash = 0;
  for (const char of key) {
    hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
  }
  return palettes[hash % palettes.length];
}

function buildPreviewHtml(section, entry) {
  const [accent, background] = previewTheme(entry, section);
  const tags = [entry.sdk, entry.target, ...entry.description.split(' ').slice(0, 3)]
    .map((value) => `<span class="tag">${value}</span>`)
    .join('');

  return `<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <style>
      :root {
        color-scheme: dark;
      }
      * { box-sizing: border-box; }
      body {
        margin: 0;
        width: 1280px;
        height: 720px;
        overflow: hidden;
        background:
          radial-gradient(circle at top right, ${accent}55, transparent 28%),
          linear-gradient(145deg, ${background}, #020617 70%);
        color: #f8fafc;
        font-family: "Avenir Next", "Segoe UI", sans-serif;
      }
      main {
        display: grid;
        grid-template-columns: 1.35fr 0.8fr;
        gap: 28px;
        width: 100%;
        height: 100%;
        padding: 52px;
      }
      .panel {
        border-radius: 28px;
        border: 1px solid rgba(255, 255, 255, 0.12);
        background: rgba(2, 6, 23, 0.58);
        box-shadow: 0 28px 80px rgba(0, 0, 0, 0.35);
        backdrop-filter: blur(14px);
      }
      .hero {
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        padding: 48px;
      }
      .eyebrow {
        color: ${accent};
        font-size: 18px;
        font-weight: 800;
        letter-spacing: 0.18em;
        text-transform: uppercase;
      }
      h1 {
        margin: 18px 0 12px;
        font-size: 72px;
        line-height: 0.96;
      }
      .runtime {
        margin: 0;
        font-size: 24px;
        color: #cbd5e1;
      }
      .description {
        margin-top: 22px;
        max-width: 700px;
        font-size: 30px;
        line-height: 1.36;
        color: #e2e8f0;
      }
      .footer {
        display: flex;
        justify-content: space-between;
        align-items: center;
        color: #cbd5e1;
        font-size: 18px;
      }
      .sidebar {
        padding: 32px;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
      }
      .label {
        color: #94a3b8;
        text-transform: uppercase;
        letter-spacing: 0.16em;
        font-size: 13px;
      }
      pre {
        margin: 14px 0 0;
        padding: 18px 20px;
        border-radius: 18px;
        background: rgba(15, 23, 42, 0.92);
        border: 1px solid rgba(255, 255, 255, 0.08);
        white-space: pre-wrap;
        font-size: 22px;
        line-height: 1.45;
        font-family: "SFMono-Regular", "Menlo", monospace;
      }
      .tags {
        display: flex;
        flex-wrap: wrap;
        gap: 12px;
      }
      .tag {
        padding: 8px 14px;
        border-radius: 999px;
        background: ${accent}22;
        border: 1px solid ${accent}55;
        font-size: 18px;
      }
      .path {
        font-size: 22px;
        line-height: 1.5;
        color: #e2e8f0;
      }
    </style>
  </head>
  <body>
    <main>
      <section class="panel hero">
        <div>
          <div class="eyebrow">${section.title}</div>
          <h1>${entry.name}</h1>
          <p class="runtime">${entry.sdk} · ${entry.target}</p>
          <p class="description">${entry.description}</p>
        </div>
        <div class="footer">
          <span>${entry.path}</span>
          <span>GoudEngine ${version}</span>
        </div>
      </section>
      <aside class="panel sidebar">
        <div>
          <div class="label">Run command</div>
          <pre>${entry.run}</pre>
        </div>
        <div>
          <div class="label">Tags</div>
          <div class="tags">${tags}</div>
        </div>
        <div>
          <div class="label">Source path</div>
          <div class="path">${entry.path}</div>
        </div>
      </aside>
    </main>
  </body>
</html>`;
}

async function ensureDir(dirPath) {
  await fs.mkdir(dirPath, { recursive: true });
}

async function generatePreviewImages(browser) {
  for (const section of manifest.sections) {
    for (const entry of section.entries) {
      if (!entry.media?.path) {
        continue;
      }
      const outputPath = path.join(repoRoot, entry.media.path);
      await ensureDir(path.dirname(outputPath));
      const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });
      await page.setContent(buildPreviewHtml(section, entry), { waitUntil: 'load' });
      await page.screenshot({ path: outputPath, type: 'png' });
      await page.close();
      console.log(`Generated ${path.relative(repoRoot, outputPath)}`);
    }
  }
}

async function generateTutorialVideo(browser) {
  await ensureDir(mediaDir);
  const slides = [
    {
      title: 'Getting Started: TypeScript Web',
      subtitle: `GoudEngine ${version}`,
      body: 'This repo-hosted recording matches the current TypeScript web docs path.',
      accent: '#22d3ee'
    },
    {
      title: '1. Install once',
      subtitle: 'Repo root',
      body: 'cd sdks/typescript\\nnpm ci',
      accent: '#38bdf8'
    },
    {
      title: '2. Run the baseline',
      subtitle: 'Flappy Bird (web)',
      body: './dev.sh --sdk typescript --game flappy_bird_web',
      accent: '#4ade80'
    },
    {
      title: '3. Run broader smoke',
      subtitle: 'Sandbox parity app',
      body: './dev.sh --sdk typescript --game sandbox_web\\ncd examples/typescript/sandbox && npm run build:web',
      accent: '#f59e0b'
    },
    {
      title: 'Reference',
      subtitle: 'Docs + examples',
      body: 'Use the TypeScript getting-started page, Build Your First Game, and Example Showcase together.',
      accent: '#a78bfa'
    }
  ];
  const slideDurationMs = 2200;
  const totalDurationMs = slides.length * slideDurationMs;
  const videoPath = path.join(mediaDir, 'getting-started-typescript-web.webm');
  const context = await browser.newContext({
    viewport: { width: 1280, height: 720 },
    recordVideo: {
      dir: mediaDir,
      size: { width: 1280, height: 720 }
    }
  });
  const page = await context.newPage();
  const video = page.video();
  await page.setContent(`<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <style>
      body {
        margin: 0;
        overflow: hidden;
        background: #020617;
        color: #f8fafc;
        font-family: "Avenir Next", "Segoe UI", sans-serif;
      }
      main {
        position: relative;
        width: 1280px;
        height: 720px;
      }
      .slide {
        position: absolute;
        inset: 0;
        padding: 72px;
        display: grid;
        grid-template-columns: 1.3fr 0.9fr;
        gap: 28px;
        opacity: 0;
        transition: opacity 350ms ease;
      }
      .slide.active {
        opacity: 1;
      }
      .panel {
        border-radius: 28px;
        border: 1px solid rgba(255, 255, 255, 0.12);
        background: rgba(2, 6, 23, 0.62);
        box-shadow: 0 28px 80px rgba(0, 0, 0, 0.35);
      }
      .hero {
        padding: 48px;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
      }
      .eyebrow {
        color: var(--accent);
        font-size: 18px;
        font-weight: 800;
        letter-spacing: 0.18em;
        text-transform: uppercase;
      }
      h1 {
        margin: 18px 0 10px;
        font-size: 72px;
        line-height: 0.95;
      }
      h2 {
        margin: 0;
        color: #cbd5e1;
        font-size: 28px;
      }
      .body {
        margin-top: 24px;
        color: #e2e8f0;
        font-size: 30px;
        line-height: 1.4;
        white-space: pre-wrap;
      }
      .command {
        padding: 28px;
        font-size: 28px;
        line-height: 1.5;
        font-family: "SFMono-Regular", "Menlo", monospace;
      }
      .footer {
        display: flex;
        justify-content: space-between;
        color: #94a3b8;
        font-size: 20px;
      }
    </style>
  </head>
  <body>
    <main id="app"></main>
    <script>
      const slides = ${JSON.stringify(slides)};
      const app = document.getElementById('app');
      app.innerHTML = slides.map((slide, index) => \`
        <section class="slide \${index === 0 ? 'active' : ''}" style="background: linear-gradient(145deg, #020617, \${slide.accent}); --accent: \${slide.accent};">
          <div class="panel hero">
            <div>
              <div class="eyebrow">GoudEngine alpha web tutorial</div>
              <h1>\${slide.title}</h1>
              <h2>\${slide.subtitle}</h2>
              <div class="body">\${slide.body.replaceAll('\\n', '<br />')}</div>
            </div>
            <div class="footer">
              <span>Docs + examples generated from repo source</span>
              <span>GoudEngine ${version}</span>
            </div>
          </div>
          <div class="panel command">\${slide.body.replaceAll('\\n', '<br />')}</div>
        </section>
      \`).join('');

      let index = 0;
      setInterval(() => {
        const nodes = Array.from(document.querySelectorAll('.slide'));
        nodes[index].classList.remove('active');
        index = Math.min(nodes.length - 1, index + 1);
        nodes[index].classList.add('active');
      }, ${slideDurationMs});
    </script>
  </body>
</html>`, { waitUntil: 'load' });
  await page.waitForTimeout(totalDurationMs + 500);
  await context.close();

  const tempVideoPath = await video.path();
  await fs.rm(videoPath, { force: true });
  await fs.rename(tempVideoPath, videoPath);

  const captions = `WEBVTT

00:00.000 --> 00:02.200
Getting Started: TypeScript Web. This recording matches the current TypeScript web docs path.

00:02.200 --> 00:04.400
Step 1. Install once. Run cd sdks/typescript and npm ci.

00:04.400 --> 00:06.600
Step 2. Run the baseline. Use ./dev.sh --sdk typescript --game flappy_bird_web.

00:06.600 --> 00:08.800
Step 3. Run broader parity. Use ./dev.sh --sdk typescript --game sandbox_web and npm run build:web.

00:08.800 --> 00:11.000
Reference. Use the TypeScript getting-started page, Build Your First Game, and Example Showcase together.
`;
  const captionsPath = path.join(mediaDir, 'getting-started-typescript-web.vtt');
  await fs.writeFile(captionsPath, captions);
  console.log(`Generated ${path.relative(repoRoot, videoPath)}`);
  console.log(`Generated ${path.relative(repoRoot, captionsPath)}`);
}

async function main() {
  const browser = await chromium.launch({ headless: true });
  try {
    await generatePreviewImages(browser);
    await generateTutorialVideo(browser);
  } finally {
    await browser.close();
  }
}

await main();
