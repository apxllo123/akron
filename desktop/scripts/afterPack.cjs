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

  // On macOS arm64, Electron's launcher expects the canonical helper bundle
  // names. electron-builder can rename those helper bundles to the product
  // name without patching the Electron launcher binary. Restore the canonical
  // names before the final signing step.
  const product = context.packager.appInfo.productFilename;
  const suffixes = ['', ' (GPU)', ' (Plugin)', ' (Renderer)'];

  for (const suffix of suffixes) {
    const sourceApp = path.join(frameworksDir, `${product} Helper${suffix}.app`);
    const targetApp = path.join(frameworksDir, `Electron Helper${suffix}.app`);

    if (fs.existsSync(targetApp) || !fs.existsSync(sourceApp)) continue;

    fs.renameSync(sourceApp, targetApp);

    const macOSDir = path.join(targetApp, 'Contents', 'MacOS');
    const sourceBinary = path.join(macOSDir, `${product} Helper${suffix}`);
    const targetBinary = path.join(macOSDir, `Electron Helper${suffix}`);

    if (fs.existsSync(sourceBinary) && !fs.existsSync(targetBinary)) {
      fs.renameSync(sourceBinary, targetBinary);
    }
  }
};
