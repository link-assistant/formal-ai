// Actual maintained upstream matching, exercised without installing an application.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import legacy from '../../../packages/minimatch-compat/index.cjs';
import modern, {Minimatch, minimatch, GLOBSTAR} from '../../../packages/minimatch-compat/index.mjs';
const require = createRequire(new URL('../../../packages/minimatch-compat/index.cjs', import.meta.url));

test('old callable CommonJS and modern ESM consumers use the same patched matcher', () => {
  assert.equal(typeof legacy, 'function');
  assert.equal(modern, minimatch);
  const commonjs = require('minimatch-modern');
  assert.equal(legacy.minimatch, commonjs.minimatch);
  assert.equal(legacy.Minimatch, commonjs.Minimatch);
  assert.equal(legacy.GLOBSTAR, commonjs.GLOBSTAR);
  const cases = [
    ['src/index.js', '**/*.js', true], ['src/index.ts', '**/*.js', false],
    ['image.png', '*.{png,jpg}', true], ['.hidden', '*', false],
  ];
  for (const [path, pattern, expected] of cases) {
    assert.equal(legacy(path, pattern), expected);
    assert.equal(minimatch(path, pattern), expected);
    assert.equal(new legacy.Minimatch(pattern).match(path), expected);
    assert.equal(new Minimatch(pattern).match(path), expected);
  }
  assert.deepEqual(legacy.match(['a.js','b.ts'], '*.js'), ['a.js']);
  assert.deepEqual(['a.js','b.ts'].filter(legacy.filter('*.js')), ['a.js']);
  assert.equal(legacy.defaults({nocase:true})('README.MD','*.md'), true);
  assert.equal(legacy.defaults({nocase:true}).defaults({dot:true})('.HIDDEN','.hidden'), true);
  assert.equal(new legacy.Minimatch('**/a').set[0][0], legacy.GLOBSTAR);
  assert.equal(new Minimatch('**/a').set[0][0], GLOBSTAR);
});
test('every lock uses the audited implementation and preserves legacy API resolution', () => {
  const upstream = JSON.parse(readFileSync(require.resolve('minimatch-modern/package.json')));
  assert.equal(upstream.name, 'minimatch');
  assert.equal(upstream.version, '11.0.1');
  const facade = JSON.parse(readFileSync('packages/minimatch-compat/package.json'));
  assert.equal(facade.version, upstream.version, 'facade version must track its actual matcher');
  for (const folder of ['desktop','vscode','rust/tests/e2e']) {
    const manifest = JSON.parse(readFileSync(folder+'/package.json'));
    const lock = JSON.parse(readFileSync(folder+'/package-lock.json'));
    assert.equal(manifest.overrides.minimatch, '$minimatch');
    assert.equal(manifest.overrides['glob@>=9'], '^14.0.2');
    assert.equal(lock.packages['node_modules/minimatch'].name, '@formal-ai/minimatch-compat');
    assert.equal(lock.packages['node_modules/minimatch'].version, upstream.version);
    assert.equal(lock.packages['node_modules/minimatch-modern'].version, upstream.version);
    for (const [path, entry] of Object.entries(lock.packages)) {
      if (path.endsWith('/minimatch')) assert.equal(entry.name, '@formal-ai/minimatch-compat');
      if (path.endsWith('/glob') && Number(entry.version.split('.')[0]) >= 9) {
        const [major,minor,patch] = entry.version.split('.').map(Number);
        assert(major > 14 || (major === 14 && (minor > 0 || patch >= 2)));
      }
    }
  }
});
