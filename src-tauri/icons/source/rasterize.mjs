// Rasterizes the canonical icon SVG into a 1024×1024 PNG that
// `tauri icon` can then expand into every platform variant. Run via:
//
//     npm run icon:rebuild
//
// The script is intentionally tiny — sharp does the heavy lifting and
// keeps SVG→PNG fidelity high enough for a 1024 source.

import { readFile, writeFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import sharp from 'sharp';

const here = dirname(fileURLToPath(import.meta.url));
const SRC_SVG = resolve(here, 'icon.svg');
const OUT_PNG = resolve(here, 'icon-1024.png');

const svg = await readFile(SRC_SVG);
const png = await sharp(svg, { density: 384 })
  .resize(1024, 1024, { fit: 'contain' })
  .png({ compressionLevel: 9 })
  .toBuffer();
await writeFile(OUT_PNG, png);

console.log(`Wrote ${OUT_PNG} (${png.length} bytes)`);
