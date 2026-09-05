const fs = require('node:fs');
const path = require('node:path');
const cp = require('node:child_process');

module.exports = async function afterPack(context) {
  if (context.packager.platform.name !== 'mac') return;

  const appPath = path.join(
    context.appOutDir,
    `${context.packager.appInfo.productFilename}.app`,
  );
  const frameworksDir = path.join(appPath, 'Contents', 'Frameworks');
  const mainExecutable = path.join(appPath, 'Contents', 'MacOS', context.packager.appInfo.productFilename);

  if (!fs.existsSync(mainExecutable) || !fs.existsSync(frameworksDir)) return;

  let strings;
  try {
    strings = cp.execFileSync('/usr/bin/strings', [mainExecutable], {
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    });
  } catch {
    return;
  }

  // Electron's macOS launcher historically contains a hard-coded helper bundle
  // name. electron-builder can rename those bundles to the product name without
  // patching the main executable. Only apply the compatibility rename when the
  // executable still references the original Electron helper names.
  if (!strings.includes('Electron Helper.app')) return;

  const product = context.packager.appInfo.productFilename;
  const suffixes = ['', ' (GPU)', ' (Plugin)', ' (Renderer)'];

  for (const suffix of suffixes) {
    const renamedApp = path.join(frameworksDir, `${product} Helper${suffix}.app`);
    const electronApp = path.join(frameworksDir, `Electron Helper${suffix}.app`);

    if (!fs.existsSync(renamedApp) || fs.existsSync(electronApp)) continue;

    fs.renameSync(renamedApp, electronApp);

    const renamedBinary = path.join(
      electronApp,
      'Contents',
      'MacOS',
      `${product} Helper${suffix}`,
    );
    const electronBinary = path.join(
      electronApp,
      'Contents',
      'MacOS',
      `Electron Helper${suffix}`,
    );

    if (fs.existsSync(renamedBinary) && !fs.existsSync(electronBinary)) {
      fs.renameSync(renamedBinary, electronBinary);
    }
  }
};
