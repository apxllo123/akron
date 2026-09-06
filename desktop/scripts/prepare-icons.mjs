import { access, mkdir, rm, writeFile } from 'node:fs/promises';
import { constants } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import sharp from 'sharp';
import pngToIco from 'png-to-ico';
import { spawn } from 'node:child_process';

const desktopRoot = resolve(fileURLToPath(new URL('..', import.meta.url)));
const repoRoot = resolve(desktopRoot, '..');
const pngSource = join(repoRoot, 'resources', 'icon.png');
const jpegFallback = join(repoRoot, 'resources', 'icon.jpeg');
const buildDir = join(desktopRoot, 'build');
const iconPng = join(buildDir, 'icon.png');
const iconIco = join(buildDir, 'icon.ico');
const iconSet = join(buildDir, 'Akron.iconset');
const iconIcns = join(buildDir, 'icon.icns');
const CANVAS_SIZE = 1024;
const CORNER_RADIUS = 150;

async function command(command, args) {
  await new Promise((resolvePromise, reject) => {
    const child = spawn(command, args, { stdio: 'inherit' });
    child.once('error', reject);
    child.once('close', (code) => {
      if (code === 0) resolvePromise();
      else reject(new Error(`${command} failed with exit code ${code ?? 'unknown'}`));
    });
  });
}

let actualSource;
try {
  await access(pngSource, constants.F_OK);
  actualSource = pngSource;
} catch {
  await access(jpegFallback, constants.F_OK);
  actualSource = jpegFallback;
}

await mkdir(buildDir, { recursive: true });
await rm(iconSet, { recursive: true, force: true });
await mkdir(iconSet, { recursive: true });

// Never crop or stretch the source artwork. The complete image is fitted inside
// a square transparent canvas, then clipped with a real alpha rounded-corner mask.
const roundedMask = Buffer.from(
  `<svg width="${CANVAS_SIZE}" height="${CANVAS_SIZE}" xmlns="http://www.w3.org/2000/svg"><rect width="${CANVAS_SIZE}" height="${CANVAS_SIZE}" rx="${CORNER_RADIUS}" ry="${CORNER_RADIUS}" fill="white"/></svg>`,
);

await sharp(actualSource)
  .resize(CANVAS_SIZE, CANVAS_SIZE, {
    fit: 'contain',
    background: { r: 0, g: 0, b: 0, alpha: 0 },
  })
  .composite([{ input: roundedMask, blend: 'dest-in' }])
  .png()
  .toFile(iconPng);

const icoSizes = [16, 24, 32, 48, 64, 128, 256];
const icoPngs = [];
for (const size of icoSizes) {
  const output = join(buildDir, `icon-${size}.png`);
  await sharp(iconPng).resize(size, size).png().toFile(output);
  icoPngs.push(output);
}
await writeFile(iconIco, await pngToIco(icoPngs));

if (process.platform === 'darwin') {
  const icnsSizes = [16, 32, 128, 256, 512];
  for (const size of icnsSizes) {
    await command('/usr/bin/sips', [
      '-z', String(size), String(size), iconPng, '--out',
      join(iconSet, `icon_${size}x${size}.png`),
    ]);
    const doubled = size * 2;
    await command('/usr/bin/sips', [
      '-z', String(doubled), String(doubled), iconPng, '--out',
      join(iconSet, `icon_${size}x${size}@2x.png`),
    ]);
  }
  await command('/usr/bin/iconutil', ['-c', 'icns', iconSet, '-o', iconIcns]);
  await access(iconIcns, constants.F_OK);
}

console.log(`Prepared Akron icons from ${actualSource}`);
console.log(`PNG: ${iconPng}`);
console.log(`ICO: ${iconIco}`);
if (process.platform === 'darwin') console.log(`ICNS: ${iconIcns}`);
