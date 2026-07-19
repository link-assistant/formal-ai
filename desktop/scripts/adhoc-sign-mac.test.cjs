'use strict';

const assert = require('node:assert/strict');
const Module = require('node:module');
const path = require('node:path');
const test = require('node:test');

const originalLoad = Module._load;
Module._load = function loadWithSigningStub(request, parent, isMain) {
  if (request === '@electron/osx-sign') {
    return { signAsync: async () => {} };
  }
  return originalLoad(request, parent, isMain);
};
const adhocSignMac = require('./adhoc-sign-mac.cjs');
Module._load = originalLoad;

test('bundled browser runtime is excluded from Electron child signing', () => {
  const appPath = path.join('/tmp', 'formal-ai Desktop.app');
  const browserRuntime = path.join(appPath, 'Contents', 'Resources', 'browser-runtime');

  assert.equal(adhocSignMac.isBundledBrowserRuntime(browserRuntime, appPath), true);
  assert.equal(
    adhocSignMac.isBundledBrowserRuntime(
      path.join(
        browserRuntime,
        'Frameworks',
        'Google Chrome for Testing Framework.framework',
        'Versions',
        'Current',
        'Resources',
        'Info.plist',
      ),
      appPath,
    ),
    true,
  );
  assert.equal(
    adhocSignMac.isBundledBrowserRuntime(
      path.join(appPath, 'Contents', 'Frameworks', 'Electron Framework.framework'),
      appPath,
    ),
    false,
  );
  assert.equal(adhocSignMac.isBundledBrowserRuntime(appPath, appPath), false);
});

test('browser exclusion composes with electron-builder ignore rules', () => {
  const appPath = path.join('/tmp', 'formal-ai Desktop.app');
  const upstreamIgnore = /existing-helper/;
  const ignore = adhocSignMac.signingIgnoreRules({
    app: appPath,
    ignore: upstreamIgnore,
  });

  assert.equal(ignore[0], upstreamIgnore);
  assert.equal(ignore.length, 2);
  assert.equal(
    ignore[1](path.join(appPath, 'Contents', 'Resources', 'browser-runtime', 'chrome')),
    true,
  );
  assert.equal(ignore[1](path.join(appPath, 'Contents', 'MacOS', 'formal-ai Desktop')), false);
});
