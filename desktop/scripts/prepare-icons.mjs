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
const source = pngSource;
const buildDir = join(desktopRoot, 'build');
const iconPng = join(buildDir, 'icon.png');
const iconIco = join(buildDir, 'icon.ico');
const iconSet = join(buildDir, 'Akron.iconset');
const iconIcns = join(buildDir, 'icon.icns');

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

let actualSource = source;
try {
  await access(actualSource, constants.F_OK);
} catch {
  actualSource = jpegFallback;
  await access(actualSource, constants.F_OK);
}

await mkdir(buildDir, { recursive: true });
await rm(iconSet, { recursive: true, force: true });
await mkdir(iconSet, { recursive: true });

// Keep the repository artwork untouched. Generate the platform assets from
// resources/icon.png when available, with the original JPEG as a safe fallback.
await sharp(actualSource).resize(1024, 1024, { fit: 'cover' }).png().toFile(iconPng);

const icoSizes = [16, 24, 32, 48, 64, 128, 256];
const icoPngs = [];
for (const size of icoSizes) {
  const output = join(buildDir, `icon-${size}.png`);
  await sharp(actualSource).resize(size, size, { fit: 'cover' }).png().toFile(output);
  icoPngs.push(output);
}
await writeFile(iconIco, await pngToIco(icoPngs));

if (process.platform === 'darwin') {
  const icnsSizes = [16, 32, 128, 256, 512];
  for (const size of icnsSizes) {
    await command('/usr/bin/sips', ['-z', String(size), String(size), iconPng, '--out', join(iconSet, `${size}x${size}.png`)]);
    const doubled = size * 2;
    await command('/usr/bin/sips', ['-z', String(doubled), String(doubled), iconPng, '--out', join(iconSet, `${size}x${size}@2x.png`)]);
  }
  await command('/usr/bin/iconutil', ['-c', 'icns', iconSet, '-o', iconIcns]);
  await access(iconIcns, constants.F_OK);
}

console.log(`Prepared Akron icons from ${actualSource}`);
console.log(`PNG: ${iconPng}`);
console.log(`ICO: ${iconIco}`);
if (process.platform === 'darwin') console.log(`ICNS: ${iconIcns}`);
