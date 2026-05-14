import sharp from 'sharp';
import { writeFileSync, mkdirSync } from 'node:fs';
import { resolve } from 'node:path';

const OUT_DIR = resolve('src-tauri/icons');
mkdirSync(OUT_DIR, { recursive: true });

// Squircle SVG with a cyan oklch gradient and the 5-bar WaveGlyph centered.
// Reproduces design/v2/project/voxtide.jsx → AppIconSquircle.
const SIZE = 1024;
const RADIUS = Math.round(SIZE * 0.22);
const BAR_COUNT = 5;
const HEIGHTS = [0.4, 0.7, 1.0, 0.7, 0.4];
const GLYPH_SIZE = Math.round(SIZE * 0.55);
const BAR_W = Math.max(1, Math.round(GLYPH_SIZE / (BAR_COUNT * 2.5)));
const GAP = Math.max(1, Math.round(GLYPH_SIZE / (BAR_COUNT * 4)));
const TOTAL_W = BAR_COUNT * BAR_W + (BAR_COUNT - 1) * GAP;

const cx = SIZE / 2;
const cy = SIZE / 2;
const startX = cx - TOTAL_W / 2;

const bars = HEIGHTS.map((h, i) => {
  const bh = h * GLYPH_SIZE * 0.7;
  const x = startX + i * (BAR_W + GAP);
  const y = cy - bh / 2;
  const rx = Math.max(0.5, BAR_W * 0.3);
  return `<rect x="${x}" y="${y}" width="${BAR_W}" height="${bh}" rx="${rx}" fill="white"/>`;
}).join('');

const svg = `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${SIZE}" height="${SIZE}" viewBox="0 0 ${SIZE} ${SIZE}">
  <defs>
    <linearGradient id="vt-grad" x1="0" y1="0" x2="1" y2="1" gradientUnits="objectBoundingBox">
      <!-- oklch(0.78 0.14 200) and oklch(0.50 0.16 220) converted to sRGB hex
           because sharp/libvips does not support CSS Color 4 oklch in SVG stop-color -->
      <stop offset="0%"  stop-color="#00d1da"/>
      <stop offset="100%" stop-color="#0074a1"/>
    </linearGradient>
    <filter id="vt-inner-hi" x="0" y="0" width="100%" height="100%">
      <feFlood flood-color="white" flood-opacity="0.25"/>
      <feComposite in2="SourceGraphic" operator="in"/>
      <feGaussianBlur stdDeviation="6"/>
    </filter>
  </defs>
  <rect x="0" y="0" width="${SIZE}" height="${SIZE}" rx="${RADIUS}" ry="${RADIUS}" fill="url(#vt-grad)"/>
  <rect x="0" y="0" width="${SIZE}" height="${RADIUS}" rx="${RADIUS}" ry="${RADIUS}" fill="white" opacity="0.06"/>
  <g>${bars}</g>
</svg>`;

writeFileSync(resolve(OUT_DIR, 'icon.svg'), svg);
await sharp(Buffer.from(svg)).png().toFile(resolve(OUT_DIR, 'icon.png'));

// 22 px monochrome menu-bar template (V silhouette).
const TPL_SIZE = 22;
const tplSvg = `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${TPL_SIZE * 2}" height="${TPL_SIZE * 2}" viewBox="0 0 22 22">
  <path d="M4 6 L11 17 L18 6 M11 17 L11 6" stroke="black" stroke-width="2.4" fill="none"
        stroke-linecap="round" stroke-linejoin="round" />
</svg>`;
await sharp(Buffer.from(tplSvg)).png().toFile(resolve(OUT_DIR, 'menu-bar-template.png'));

console.log('wrote src-tauri/icons/icon.png + icon.svg + menu-bar-template.png');
