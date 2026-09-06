const fs = require('node:fs');
const path = require('node:path');

module.exports = async function afterPack(context) {
  if (context.packager.platform.name !== 'mac') return;

  const appPath = path.join(
    context.appOutDir,
    `${context.packager.appInfo.productFilename}.app`,
  );
  const frameworksDir = path.join(appPath, 'Contents', 'Frameworks');

  if (!fs.existsSync(frameworksDir)) return;

  // macOS 26 + arm64 + electron-builder 26.x can rename Electron's helper
  // bundles to the product name without patching Electron's launcher binary.
  // Electron's launcher still resolves the canonical helper names, so restore
  // only the bundle and executable filenames. Do not rewrite nested plists.
  const product = context.packager.appInfo.productFilename;
  const suffixes = ['', ' (GPU)', ' (Plugin)', ' (Renderer)'];

  for (const suffix of suffixes) {
    const sourceApp = path.join(frameworksDir, `${product} Helper${suffix}.app`);
    const targetApp = path.join(frameworksDir, `Electron Helper${suffix}.app`);

    if (!fs.existsSync(sourceApp) || fs.existsSync(targetApp)) continue;

    fs.renameSync(sourceApp, targetApp);

    const macOSDir = path.join(targetApp, 'Contents', 'MacOS');
    const sourceBinary = path.join(macOSDir, `${product} Helper${suffix}`);
    const targetBinary = path.join(macOSDir, `Electron Helper${suffix}`);

    if (fs.existsSync(sourceBinary) && !fs.existsSync(targetBinary)) {
      fs.renameSync(sourceBinary, targetBinary);
    }
  }
};
