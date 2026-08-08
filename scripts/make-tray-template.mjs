#!/usr/bin/env node
// Derive the macOS menu-bar *template* tray icons from the app icon.
//
// macOS menu-bar items are expected to be template images: a single-colour
// silhouette in the alpha channel that AppKit tints black on a light menu bar
// and white on a dark one. A full-colour icon (our green app mark) is the one
// thing that looks out of place up there, so macOS gets its own pair of assets:
//
//   icons/tray-macos-template.png         plain silhouette
//   icons/tray-macos-template-unread.png  silhouette + unread badge dot
//
// Both are 36x36 px = 18pt @2x, which is exactly the height tray-icon renders a
// status-item image at, so the bitmap lands 1:1 on a Retina menu bar.
//
// Input is `icons/icon.png` (the 512px render of icons/app-icon.svg), so the SVG
// stays the single source of truth for every icon in the repo. The mark is a
// white-stroked speech bubble with a white handset on a green rounded square;
// the silhouette we want is "the bubble, filled, with the handset knocked out".
// We recover that from the raster rather than the path data: flood-fill the
// non-white pixels inward from the border, and everything the fill cannot reach
// (it is walled off by the bubble's white stroke) is the bubble. The handset is
// then the white blob that is not the stroke ring itself.
//
// Zero dependencies — PNG in/out via node:zlib, matching the other scripts here.
import { readFileSync, writeFileSync } from "node:fs";
import { deflateSync, inflateSync } from "node:zlib";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SRC = join(ROOT, "src-tauri/icons/icon.png");
const OUT_NORMAL = join(ROOT, "src-tauri/icons/tray-macos-template.png");
const OUT_UNREAD = join(ROOT, "src-tauri/icons/tray-macos-template-unread.png");

const OUT_SIZE = 36; // 18pt @2x — tray-icon renders status-item images at 18pt tall
const CONTENT = 32; // glyph box inside OUT_SIZE, leaving a 2px breathing margin
const SS = 9; // supersampling factor for the badge / downscale grid

// --- minimal PNG codec (8-bit RGBA, non-interlaced) --------------------------

const CRC_TABLE = (() => {
  const t = new Int32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c;
  }
  return t;
})();

function crc32(buf) {
  let c = -1;
  for (let i = 0; i < buf.length; i++) c = CRC_TABLE[(c ^ buf[i]) & 0xff] ^ (c >>> 8);
  return (c ^ -1) >>> 0;
}

function decodePng(file) {
  const buf = readFileSync(file);
  const w = buf.readUInt32BE(16);
  const h = buf.readUInt32BE(20);
  const [depth, colorType, , , interlace] = [buf[24], buf[25], buf[26], buf[27], buf[28]];
  if (depth !== 8 || colorType !== 6 || interlace !== 0) {
    throw new Error(`${file}: expected 8-bit RGBA, non-interlaced (got depth=${depth} type=${colorType} interlace=${interlace})`);
  }

  const idat = [];
  for (let off = 8; off + 8 <= buf.length; ) {
    const len = buf.readUInt32BE(off);
    const type = buf.toString("ascii", off + 4, off + 8);
    if (type === "IDAT") idat.push(buf.subarray(off + 8, off + 8 + len));
    if (type === "IEND") break;
    off += 12 + len;
  }

  const raw = inflateSync(Buffer.concat(idat));
  const bpp = 4;
  const stride = w * bpp;
  const px = Buffer.alloc(h * stride);
  for (let y = 0; y < h; y++) {
    const filter = raw[y * (stride + 1)];
    const line = raw.subarray(y * (stride + 1) + 1, y * (stride + 1) + 1 + stride);
    for (let x = 0; x < stride; x++) {
      const a = x >= bpp ? px[y * stride + x - bpp] : 0;
      const b = y > 0 ? px[(y - 1) * stride + x] : 0;
      const c = x >= bpp && y > 0 ? px[(y - 1) * stride + x - bpp] : 0;
      let v;
      switch (filter) {
        case 0: v = line[x]; break;
        case 1: v = line[x] + a; break;
        case 2: v = line[x] + b; break;
        case 3: v = line[x] + ((a + b) >> 1); break;
        case 4: {
          const p = a + b - c;
          const pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
          v = line[x] + (pa <= pb && pa <= pc ? a : pb <= pc ? b : c);
          break;
        }
        default: throw new Error(`unsupported PNG filter ${filter}`);
      }
      px[y * stride + x] = v & 0xff;
    }
  }
  return { w, h, px };
}

function chunk(type, data) {
  const out = Buffer.alloc(12 + data.length);
  out.writeUInt32BE(data.length, 0);
  out.write(type, 4, "ascii");
  data.copy(out, 8);
  out.writeUInt32BE(crc32(out.subarray(4, 8 + data.length)), 8 + data.length);
  return out;
}

/** Encode an alpha-only (black) RGBA PNG from a Float32 coverage buffer in [0,1]. */
function encodeAlphaPng(size, alpha) {
  const stride = size * 4;
  const raw = Buffer.alloc(size * (stride + 1));
  for (let y = 0; y < size; y++) {
    raw[y * (stride + 1)] = 0; // filter: none
    for (let x = 0; x < size; x++) {
      const o = y * (stride + 1) + 1 + x * 4;
      raw[o] = raw[o + 1] = raw[o + 2] = 0; // black; AppKit repaints template pixels anyway
      raw[o + 3] = Math.max(0, Math.min(255, Math.round(alpha[y * size + x] * 255)));
    }
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(size, 0);
  ihdr.writeUInt32BE(size, 4);
  ihdr[8] = 8;  // bit depth
  ihdr[9] = 6;  // colour type: RGBA
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk("IHDR", ihdr),
    chunk("IDAT", deflateSync(raw, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

// --- silhouette extraction ---------------------------------------------------

const GREEN = [0x25, 0xd3, 0x66];

/** Per-pixel "how white is this" in [0,1], along the green -> white ramp. */
function whiteness(px, i) {
  const a = px[i + 3] / 255;
  if (a < 0.5) return 0; // transparent corner of the rounded square
  const t = (px[i] - GREEN[0]) / (255 - GREEN[0]); // red channel separates green from white
  return Math.max(0, Math.min(1, t));
}

function extractSilhouette({ w, h, px }) {
  const white = new Uint8Array(w * h);
  for (let i = 0, p = 0; p < w * h; p++, i += 4) white[p] = whiteness(px, i) >= 0.5 ? 1 : 0;

  // Flood the non-white pixels in from the border. This eats the transparent
  // corners and the whole green background plate, and stops dead at the white
  // stroke that outlines the bubble — so anything left is bubble.
  const outside = new Uint8Array(w * h);
  const stack = [];
  for (let x = 0; x < w; x++) { stack.push(x, (h - 1) * w + x); }
  for (let y = 0; y < h; y++) { stack.push(y * w, y * w + w - 1); }
  while (stack.length) {
    const p = stack.pop();
    if (outside[p] || white[p]) continue;
    outside[p] = 1;
    const x = p % w, y = (p - x) / w;
    if (x > 0) stack.push(p - 1);
    if (x < w - 1) stack.push(p + 1);
    if (y > 0) stack.push(p - w);
    if (y < h - 1) stack.push(p + w);
  }

  // Two white shapes exist: the stroke ring around the bubble and the handset
  // inside it. The ring is the one with the larger bounding box; every other
  // white blob is a knockout (the handset).
  const label = new Int32Array(w * h).fill(-1);
  const boxes = [];
  for (let p0 = 0; p0 < w * h; p0++) {
    if (!white[p0] || label[p0] >= 0) continue;
    const id = boxes.length;
    const box = { x0: w, y0: h, x1: -1, y1: -1, n: 0 };
    const q = [p0];
    label[p0] = id;
    while (q.length) {
      const p = q.pop();
      const x = p % w, y = (p - x) / w;
      box.n++;
      if (x < box.x0) box.x0 = x;
      if (x > box.x1) box.x1 = x;
      if (y < box.y0) box.y0 = y;
      if (y > box.y1) box.y1 = y;
      for (let dy = -1; dy <= 1; dy++) {
        for (let dx = -1; dx <= 1; dx++) {
          const nx = x + dx, ny = y + dy;
          if (nx < 0 || ny < 0 || nx >= w || ny >= h) continue;
          const np = ny * w + nx;
          if (white[np] && label[np] < 0) { label[np] = id; q.push(np); }
        }
      }
    }
    boxes.push(box);
  }
  if (boxes.length < 2) throw new Error(`expected a stroke ring plus a handset, found ${boxes.length} white shape(s)`);
  const area = (b) => (b.x1 - b.x0 + 1) * (b.y1 - b.y0 + 1);
  const ring = boxes.reduce((a, b) => (area(b) > area(a) ? b : a));
  const ringId = boxes.indexOf(ring);

  // Solid bubble, handset punched back out.
  const mask = new Uint8Array(w * h);
  for (let p = 0; p < w * h; p++) {
    mask[p] = !outside[p] && !(white[p] && label[p] !== ringId) ? 1 : 0;
  }
  return { mask, box: ring };
}

// --- resampling --------------------------------------------------------------

/** Box-filter a binary mask region down to size x size floats in [0,1]. */
function downsample(mask, w, box, size) {
  const side = Math.max(box.x1 - box.x0 + 1, box.y1 - box.y0 + 1);
  const cx = (box.x0 + box.x1 + 1) / 2, cy = (box.y0 + box.y1 + 1) / 2;
  const left = cx - side / 2, top = cy - side / 2; // square crop, glyph centred
  const step = side / size;
  const out = new Float32Array(size * size);
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      let sum = 0, n = 0;
      const sx0 = Math.round(left + x * step), sx1 = Math.round(left + (x + 1) * step);
      const sy0 = Math.round(top + y * step), sy1 = Math.round(top + (y + 1) * step);
      for (let sy = sy0; sy < sy1; sy++) {
        for (let sx = sx0; sx < sx1; sx++, n++) {
          sum += mask[sy * w + sx] || 0;
        }
      }
      out[y * size + x] = n ? sum / n : 0;
    }
  }
  return out;
}

/** Unread badge: a filled dot in the top-right corner, ringed by a transparent
 *  gap so it still reads as a separate mark once AppKit flattens the image to a
 *  single colour (a template icon has no second colour to spend on it). Drawn on
 *  the supersampled grid, in units of the content box. */
const BADGE_SHRINK = 0.84; // glyph is scaled down to free up the corner
function stampBadge(buf, size) {
  const rDot = 0.165 * size, rGap = 0.225 * size;
  const cx = size - rDot, cy = rDot;
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const d = Math.hypot(x + 0.5 - cx, y + 0.5 - cy);
      if (d <= rDot) buf[y * size + x] = 1;
      else if (d <= rGap) buf[y * size + x] = 0;
    }
  }
}

/** Blit a smaller square grid into the bottom-left of a `size` x `size` grid. */
function anchorBottomLeft(small, n, size) {
  const out = new Float32Array(size * size);
  const dy = size - n;
  for (let y = 0; y < n; y++) {
    for (let x = 0; x < n; x++) out[(y + dy) * size + x] = small[y * n + x];
  }
  return out;
}

/** Average an SS x SS block grid down to its final resolution. */
function boxDown(buf, size, factor) {
  const out = new Float32Array((size / factor) * (size / factor));
  const n = size / factor;
  for (let y = 0; y < n; y++) {
    for (let x = 0; x < n; x++) {
      let sum = 0;
      for (let dy = 0; dy < factor; dy++) {
        for (let dx = 0; dx < factor; dx++) sum += buf[(y * factor + dy) * size + x * factor + dx];
      }
      out[y * n + x] = sum / (factor * factor);
    }
  }
  return out;
}

/** Centre a CONTENT x CONTENT glyph in an OUT_SIZE x OUT_SIZE canvas. */
function pad(content) {
  const out = new Float32Array(OUT_SIZE * OUT_SIZE);
  const off = (OUT_SIZE - CONTENT) / 2;
  for (let y = 0; y < CONTENT; y++) {
    for (let x = 0; x < CONTENT; x++) out[(y + off) * OUT_SIZE + x + off] = content[y * CONTENT + x];
  }
  return out;
}

// --- main --------------------------------------------------------------------

const src = decodePng(SRC);
const { mask, box } = extractSilhouette(src);
const N = CONTENT * SS;

writeFileSync(OUT_NORMAL, encodeAlphaPng(OUT_SIZE, pad(boxDown(downsample(mask, src.w, box, N), N, SS))));

const small = Math.round((N * BADGE_SHRINK) / SS) * SS; // keep the SS grid aligned
const badged = anchorBottomLeft(downsample(mask, src.w, box, small), small, N);
stampBadge(badged, N);
writeFileSync(OUT_UNREAD, encodeAlphaPng(OUT_SIZE, pad(boxDown(badged, N, SS))));

console.log(`Wrote ${OUT_NORMAL} and ${OUT_UNREAD} (${OUT_SIZE}x${OUT_SIZE}, template)`);
