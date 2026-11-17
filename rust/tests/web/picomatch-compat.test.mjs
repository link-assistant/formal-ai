import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import test from 'node:test';

const root = new URL('../../../', import.meta.url);
const packageRoot = new URL('packages/picomatch-compat/', root);
const readJson = url => JSON.parse(readFileSync(url, 'utf8'));
const manifest = readJson(new URL('package.json', packageRoot));
const provenance = readJson(new URL('upstream.json', packageRoot));

test('the local picomatch sources match their recorded hashes', () => {
  const files = [
    ...provenance.files.map(file => [file.path, file.localSha256]),
    ...provenance.addedFiles.map(file => [file.path, file.sha256]),
  ];
  for (const [path, expected] of files) {
    const hash = createHash('sha256').update(readFileSync(new URL(path, packageRoot)));
    assert.equal(hash.digest('hex'), expected, path);
  }
});

for (const consumer of ['desktop', 'vscode']) {
  test(`${consumer} locks every picomatch dependency to the local backport`, () => {
    const consumerRoot = new URL(`${consumer}/`, root);
    const packageJson = readJson(new URL('package.json', consumerRoot));
    const lock = readJson(new URL('package-lock.json', consumerRoot));
    const localPath = 'file:../packages/picomatch-compat';
    assert.equal(packageJson.devDependencies.picomatch, localPath);
    assert.equal(packageJson.overrides.picomatch, '$picomatch');
    assert.equal(lock.packages[''].devDependencies.picomatch, localPath);
    const entries = Object.entries(lock.packages).filter(([path]) =>
      path.endsWith('node_modules/picomatch'));
    assert.equal(entries.length, 1);
    const [[, entry]] = entries;
    assert.equal(entry.resolved, localPath);
    assert.equal(entry.name, manifest.name);
    assert.equal(entry.version, manifest.version);
  });
}

test('the local picomatch package preserves ordinary project file matching', () => {
  const require = createRequire(packageRoot);
  const picomatch = require('./index.js');
  const matches = picomatch('src/**/*.{js,ts}', { ignore: 'src/generated/**' });
  assert.equal(matches('src/main.js'), true);
  assert.equal(matches('src/lib/settings.ts'), true);
  assert.equal(matches('src/generated/settings.ts'), false);
  assert.equal(matches('docs/usage.md'), false);
  assert.equal(picomatch.scan('src/**/*.js').isGlob, true);
});
