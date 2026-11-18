'use strict';

const path = require('node:path');
const {createRequire} = require('node:module');
const {pathToFileURL} = require('node:url');
const repositoryPackage = require('../../package.json').name;

// The renderer is already copied into dist-web. Including its private parent
// package as a runtime dependency also copies desktop/release back into itself.
function assertPackagePaths(files) {
  for (const file of files) {
    const parts = file.replaceAll('\\', '/').split('/');
    if (parts.some((part, index) => part === 'node_modules' && parts[index + 1] === repositoryPackage)) {
      throw new Error(`Desktop package contains repository source dependency: ${file}`);
    }
  }
}

function resourcesDirectory(context) {
  return context.electronPlatformName === 'darwin'
    ? path.join(context.appOutDir, context.packager.appInfo.productFilename + '.app', 'Contents', 'Resources')
    : path.join(context.appOutDir, 'resources');
}

async function afterPack(context) {
  // Read only ASAR metadata, before NSIS/DMG/other installer creation. Its list
  // includes unpacked entries too, so native modules cannot bypass this check.
  const builderRequire = createRequire(require.resolve('app-builder-lib/package.json'));
  const asar = await import(pathToFileURL(builderRequire.resolve('@electron/asar')).href);
  const files = asar.listPackage(path.join(resourcesDirectory(context), 'app.asar'));
  assertPackagePaths(files);
  console.log('Desktop package source boundary verified before installer creation');
}

module.exports = afterPack;
module.exports.assertPackagePaths = assertPackagePaths;
module.exports.resourcesDirectory = resourcesDirectory;
