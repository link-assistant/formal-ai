import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const desktop = fileURLToPath(new URL('../../../desktop/', import.meta.url));
const require = createRequire(path.join(desktop, 'package.json'));
const {assertPackagePaths, resourcesDirectory} = require('./scripts/package-content.cjs');
const manifest = require('./package.json');
const fixture = JSON.parse(readFileSync(new URL('../fixtures/desktop-package-source-boundary.json', import.meta.url)));
const retained = ['node_modules/node-pty/prebuilds/win32-x64/conpty/OpenConsole.exe',
  'node_modules/@link-assistant/web-capture/package.json', 'node_modules/playwright/cli.js',
  'dist-web/app.js', 'browser-runtime/chrome-win64/chrome.exe', 'bin/formal-ai.exe'];

test('captured nested source and build outputs fail before installer creation', () => {
  for (const file of fixture.unexpectedPaths) {
    for (const variant of [file, '/' + file, 'C:\\app\\' + file.replaceAll('/', '\\')]) {
      assert.throws(() => assertPackagePaths([variant]), /repository source dependency/);
    }
  }
  assert.throws(() => assertPackagePaths(['node_modules/another/node_modules/formal-ai-web-bundle/package.json']), /repository source dependency/);
});

test('browser, native engine and real runtime dependencies remain allowed', () => {
  assert.doesNotThrow(() => assertPackagePaths(retained));
  assert.doesNotThrow(() => assertPackagePaths(['node_modules/formal-ai-web-bundle-tools/index.js']));
});

test('content audit is configured before installers and locates both resource layouts', () => {
  assert.equal(manifest.build.afterPack, './scripts/package-content.cjs');
  const context = {appOutDir: '/fixture/output', packager: {appInfo: {productFilename: 'formal-ai Desktop'}}};
  assert.equal(resourcesDirectory({...context, electronPlatformName: 'win32'}), path.join(context.appOutDir, 'resources'));
  assert.equal(resourcesDirectory({...context, electronPlatformName: 'darwin'}), path.join(context.appOutDir, 'formal-ai Desktop.app/Contents/Resources'));
});

let builder;
try { builder = require.resolve('app-builder-lib/package.json'); } catch { /* Installed only in hosted desktop packaging. */ }
test('installed electron-builder rejects captured paths while retaining runtime files', {
  skip: !builder && 'actual desktop packager is checked in hosted packaging',
}, () => {
  const upstream = createRequire(builder)('./out/fileMatcher.js');
  const matcher = upstream.getNodeModuleFileMatcher(desktop, path.join(desktop, 'fixture-output'), value => value, {}, {
    config: {...manifest.build, fileOptions: {files: manifest.build.files}}, debugLogger: {isEnabled: false},
  });
  const filter = matcher.createFilter();
  for (const relative of fixture.unexpectedPaths) {
    assert.equal(filter(path.join(desktop, relative), {moduleFullFilePath: relative, isDirectory: () => false}), false, relative);
  }
  for (const relative of retained.filter(file => file.startsWith('node_modules/'))) {
    assert.equal(filter(path.join(desktop, relative), {moduleFullFilePath: relative, isDirectory: () => false}), true, relative);
  }
});
