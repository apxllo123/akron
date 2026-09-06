const fs = require('node:fs');
const path = require('node:path');
const plist = require('node:child_process');

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

    if (fs.existsSync(sourceApp) && !fs.existsSync(targetApp)) {
      fs.renameSync(sourceApp, targetApp);
    }

    if (!fs.existsSync(targetApp)) continue;

    const macOSDir = path.join(targetApp, 'Contents', 'MacOS');
    const sourceBinary = path.join(macOSDir, `${product} Helper${suffix}`);
    const targetBinary = path.join(macOSDir, `Electron Helper${suffix}`);

    if (fs.existsSync(sourceBinary) && !fs.existsSync(targetBinary)) {
      fs.renameSync(sourceBinary, targetBinary);
    }

    // Keep the nested helper bundle's metadata consistent with its actual
    // bundle/executable name. A renamed bundle with a stale CFBundleExecutable
    // can fail to launch through Launch Services/Finder even when the main
    // Electron process itself starts successfully.
    const infoPlist = path.join(targetApp, 'Contents', 'Info.plist');
    if (fs.existsSync(infoPlist)) {
      try {
        plist.execFileSync('/usr/bin/plutil', [
          '-replace',
          'CFBundleExecutable',
          '-string',
          `Electron Helper${suffix}`,
          infoPlist,
        ]);
        plist.execFileSync('/usr/bin/plutil', [
          '-replace',
          'CFBundleName',
          '-string',
          `Electron Helper${suffix}`,
          infoPlist,
        ]);
        plist.execFileSync('/usr/bin/plutil', [
          '-replace',
          'CFBundleDisplayName',
          '-string',
          `Electron Helper${suffix}`,
          infoPlist,
        ]);
      } catch (error) {
        throw new Error(
          `Failed to normalize Electron helper metadata in ${infoPlist}: ${error instanceof Error ? error.message : String(error)}`,
        );
      }
    }
  }
};
