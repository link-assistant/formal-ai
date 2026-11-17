// A small installed-consumer probe; never installs desktop or VS Code packages.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const baseline = process.argv[2];
const workspace = mkdtempSync(join(tmpdir(), 'formal-picomatch-consumers-'));
const log = join(tmpdir(), 'issue-1580-picomatch-consumer-install.log');
try {
  const manifest = {
    name:'formal-picomatch-consumer-probe', version:'1.0.0', private:true,
    dependencies:{
      picomatch:'file:' + (baseline || join(root, 'packages/picomatch-compat')),
      micromatch:'4.0.8', tinyglobby:'0.2.17', fdir:'6.5.0',
    },
    overrides:{picomatch:'$picomatch'},
  };
  writeFileSync(join(workspace, 'package.json'), JSON.stringify(manifest, null, 2));
  const installed = spawnSync('npm', [
    'install', '--install-links', '--ignore-scripts', '--no-audit', '--no-fund',
  ], {cwd:workspace, encoding:'utf8', maxBuffer:64 * 1024,
    env:{...process.env, NODE_OPTIONS:'--max-old-space-size=256'}});
  writeFileSync(log, String(installed.stdout) + String(installed.stderr));
  assert.equal(installed.error, undefined, String(installed.error));
  assert.equal(installed.status, 0, readFileSync(log, 'utf8'));
  const require = createRequire(join(workspace, 'package.json'));
  const mm = require('micromatch');
  const pm = require('picomatch');
  const {globSync} = require('tinyglobby');
  const {fdir} = require('fdir');
  mkdirSync(join(workspace, 'src/vendor'), {recursive:true});
  for (const path of ['src/a.js', 'src/b.txt', 'src/.hidden.js', 'src/vendor/a.js']) {
    writeFileSync(join(workspace, path), 'fixture');
  }
  assert.deepEqual(mm(['src/a.js', 'src/b.txt', 'src/vendor/a.js'],
    ['src/**/*.js', '!**/vendor/**']), ['src/a.js']);
  assert.equal(mm.matcher('**/*.js')('src/a.js'), true);
  assert.deepEqual(mm.capture('src/*.js', 'src/a.js'), ['a']);
  assert.equal(mm.contains('src/a.js', '*.js'), true);
  assert.deepEqual(mm.not(['a.js', 'b.txt'], '*.js'), ['b.txt']);
  assert.equal(mm.some(['a.js', 'b.txt'], '*.js'), true);
  assert.equal(mm.every(['a.js', 'b.js'], '*.js'), true);
  assert.equal(typeof mm.parse('*.js')[0].output, 'string');
  assert.equal(mm.scan('src/**/*.js').base, 'src');
  assert.deepEqual(mm.braces('a.{js,ts}', {expand:true}), ['a.js', 'a.ts']);
  assert.equal(mm.makeRe('*.js').test('a.js'), true);
  if (!baseline) assert.equal(mm.isMatch('aaaa', '+(*a)'), false);
  assert.deepEqual(globSync('src/**/*.js', {
    cwd:workspace, dot:true, ignore:['**/vendor/**'],
  }).sort(), ['src/.hidden.js', 'src/a.js']);
  assert.deepEqual(new fdir().withRelativePaths().glob('**/*.js')
    .crawl(join(workspace, 'src')).sync().sort(), ['.hidden.js', 'a.js', 'vendor/a.js']);
  if (!baseline) assert.equal(pm.isMatch('[aaa', '+([[]*a)'), false);
  const lock = JSON.parse(readFileSync(join(workspace, 'package-lock.json'), 'utf8'));
  const entries = Object.entries(lock.packages).filter(([key]) => key.endsWith('/picomatch'));
  assert.equal(entries.length, 1);
  if (!baseline) assert.equal(entries[0][1].name, '@formal-ai/picomatch-compat');
  console.log(JSON.stringify({
    passed:true, mode:baseline ? 'upstream-control' : 'local-backport', consumers:['micromatch@4.0.8', 'tinyglobby@0.2.17', 'fdir@6.5.0'],
    installedPicomatch:entries[0][1], npmInstallLog:log, nativeExecuted:false,
    autonomousAuthorshipCredit:0,
  }, null, 2));
} finally {
  rmSync(workspace, {recursive:true, force:true});
}
