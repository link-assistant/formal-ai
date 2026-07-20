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

  // Must be a single function, never an array: @electron/osx-sign's
  // validateOptsIgnore() drops arrays (see adhoc-sign-mac.cjs).
  assert.equal(typeof ignore, 'function');
  assert.equal(ignore(path.join(appPath, 'Contents', 'MacOS', 'existing-helper')), true);
  assert.equal(
    ignore(path.join(appPath, 'Contents', 'Resources', 'browser-runtime', 'chrome')),
    true,
  );
  assert.equal(ignore(path.join(appPath, 'Contents', 'MacOS', 'formal-ai Desktop')), false);
});

// Issue #808: guard against the upstream quirk that caused the failure. If a
// future @electron/osx-sign release starts honouring arrays this test still
// passes; if it keeps dropping them, our single-function contract stays required.
test('an array of ignore rules is discarded by @electron/osx-sign', () => {
  const signPath = require.resolve('@electron/osx-sign/dist/cjs/sign.js');
  const source = require('node:fs').readFileSync(signPath, 'utf8');
  const arraysDropped = /function validateOptsIgnore\(ignore\) \{\s*if \(ignore && !\(ignore instanceof Array\)\) \{\s*return \[ignore\];\s*\}\s*\}/.test(
    source,
  );

  if (arraysDropped) {
    const appPath = path.join('/tmp', 'formal-ai Desktop.app');
    assert.equal(typeof adhocSignMac.signingIgnoreRules({ app: appPath }), 'function');
  }
});

// Issue #808: run 29724500254 failed signing
// Contents/Resources/browser-runtime/.../Google Chrome for Testing Framework.framework
// ("unsealed contents present in the root directory of an embedded framework")
// even though the hook above already excluded it -- the hook produced no output
// at all, so the exclusion never reached @electron/osx-sign. The exclusion is
// therefore also declared in electron-builder configuration, where
// MacTargetHelper.buildSignOptions() applies it regardless of the sign hook.
test('electron-builder config excludes the bundled browser runtime from signing', () => {
  const { signIgnore } = require('../package.json').build.mac;

  assert.ok(Array.isArray(signIgnore) && signIgnore.length > 0, 'mac.signIgnore must be configured');

  const matches = (filePath) => signIgnore.some((pattern) => new RegExp(pattern).test(filePath));
  const app = '/Users/runner/work/formal-ai/formal-ai/desktop/release/mac/formal-ai Desktop.app';

  assert.equal(
    matches(
      `${app}/Contents/Resources/browser-runtime/Frameworks/Google Chrome for Testing Framework.framework`,
    ),
    true,
  );
  assert.equal(matches(`${app}/Contents/Resources/browser-runtime/chrome`), true);
  assert.equal(matches(`${app}/Contents/MacOS/formal-ai Desktop`), false);
  assert.equal(matches(app), false);
});
