import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';

const require = createRequire(import.meta.url);
const entry = fileURLToPath(new URL('../../../packages/picomatch-compat/index.js', import.meta.url));
const picomatch = require(entry);
const posix = require('../../../packages/picomatch-compat/posix.js');
const risky = ['+(*)', '*(*a)', '+(*a)', '+(a*)', '+(*a|b)', '+(@(*a))', '+([[]*a)', '+([*a)', '+([[:alpha:]]*a)'];

for (const pattern of risky) {
  test('literalizes unsafe repeated wildcard: ' + pattern, {timeout:5000}, () => {
    assert.equal(picomatch.isMatch(pattern, pattern), true);
    assert.equal(picomatch.isMatch('aaaa', pattern), false);
    assert.equal(picomatch.isMatch('[aaa', pattern), false);
    const literal = pattern.replace(/[|\\{}()[\]^$+*?.]/g, '\\$&');
    assert.equal(picomatch.makeRe(pattern).source, '^(?:' + literal + ')$');
    assert.equal(posix.isMatch(pattern, pattern), true);
  });
}

for (const pattern of ['+(*)', '+(*a)']) {
  test('bounded hostile match terminates: ' + pattern, {timeout:5000}, () => {
    const script = 'const pm=require(' + JSON.stringify(entry) + ');' +
      'if(pm.isMatch("a".repeat(40)+"/!",' + JSON.stringify(pattern) + '))process.exit(2);' +
      'if(pm.isMatch("a".repeat(40)+"!",' + JSON.stringify(pattern) + '))process.exit(3);';
    const child = spawnSync(process.execPath,
      ['--max-old-space-size=64', '--stack-size=256', '-e', script],
      {timeout:1250, killSignal:'SIGKILL', maxBuffer:4096});
    assert.equal(child.error, undefined, String(child.error));
    assert.equal(child.signal, null);
    assert.equal(child.status, 0, String(child.stderr));
  });
}

test('ordinary glob paths, ignore, dotfiles and callbacks remain supported', () => {
  const matched = [];
  const match = picomatch('src/**/*.js', {
    dot:true, ignore:'**/vendor/**', onMatch:result => matched.push(result.input),
  });
  assert.equal(match('src/a.js'), true);
  assert.equal(match('src/.hidden/a.js'), true);
  assert.equal(match('src/vendor/a.js'), false);
  assert.equal(match('src/a.txt'), false);
  assert.deepEqual(matched, ['src/a.js', 'src/.hidden/a.js']);
});

test('literal and flattened repeated extglobs retain their meaning', () => {
  assert.equal(picomatch.isMatch('abab', '+(ab)'), true);
  assert.equal(picomatch.isMatch('abac', '+(ab)'), false);
  assert.equal(picomatch.isMatch('abba', '+(*(a)|*(b))'), true);
  assert.equal(picomatch.isMatch('***', '+(\\*)'), true);
  assert.equal(picomatch.isMatch('***', '+([*])'), true);
  assert.equal(picomatch.isMatch('a*a*', '+("a*")'), true);
  assert.equal(picomatch.isMatch('ab', '+([[:alpha:]])'), true);
  assert.equal(picomatch.isMatch('+(*a)', '+(*a)', {noextglob:true}), true);
});

test('public scan, parse, regex and platform entrypoints remain usable', () => {
  assert.equal(picomatch.scan('src/**/*.js').base, 'src');
  assert.equal(typeof picomatch.parse('*.js').output, 'string');
  assert.equal(picomatch.makeRe('*.js').test('a.js'), true);
  assert.equal(picomatch.makeRe('*.js').test('a.txt'), false);
  assert.equal(picomatch.isMatch('SRC/A.JS', 'src/*.js', {nocase:true}), true);
  assert.equal(posix.isMatch('src/a.js', 'src/**/*.js'), true);
});

test('disabled bracket parsing cannot hide an active wildcard', () => {
  const pattern = '+([*])';
  assert.equal(picomatch.isMatch('[aaa]', pattern, {nobracket:true}), false);
  assert.equal(picomatch.isMatch(pattern, pattern, {nobracket:true}), true);
});

test('the retained upstream license and sources match the recorded provenance', () => {
  const root = new URL('../../../packages/picomatch-compat/', import.meta.url);
  const provenance = JSON.parse(readFileSync(new URL('upstream.json', root), 'utf8'));
  assert.deepEqual(provenance.modifiedFiles, ['lib/parse.js']);
  assert.equal(provenance.files.length, 8);
  assert.equal(provenance.addedFiles.length, 1);
  for (const file of provenance.addedFiles) {
    const hash = createHash('sha256').update(readFileSync(new URL(file.path, root))).digest('hex');
    assert.equal(hash, file.sha256, file.path);
  }
  for (const file of provenance.files) {
    const hash = createHash('sha256').update(readFileSync(new URL(file.path, root))).digest('hex');
    assert.equal(hash, file.localSha256, file.path);
    if (file.path !== 'lib/parse.js') assert.equal(hash, file.originalSha256, file.path);
    else assert.notEqual(hash, file.originalSha256);
  }
});

for (const application of ['desktop', 'vscode']) {
  test(application + ' resolves every locked picomatch to the reviewed local source', () => {
    const root = new URL('../../../' + application + '/', import.meta.url);
    const manifest = JSON.parse(readFileSync(new URL('package.json', root), 'utf8'));
    const lock = JSON.parse(readFileSync(new URL('package-lock.json', root), 'utf8'));
    assert.equal(manifest.devDependencies.picomatch, 'file:../packages/picomatch-compat');
    assert.equal(manifest.overrides.picomatch, '$picomatch');
    const entries = Object.entries(lock.packages).filter(([path]) => path.endsWith('/picomatch'));
    assert.equal(entries.length, 1);
    assert.equal(entries[0][1].name, '@formal-ai/picomatch-compat');
    assert.equal(entries[0][1].resolved, 'file:../packages/picomatch-compat');
    assert.equal(entries[0][1].version, '4.0.7-formal-ai.1');
  });
}

test('POSIX negation retains literal stars after an initial literal closing bracket', () => {
  assert.equal(picomatch.isMatch('aa', '+([!]*])', {posix:true}), true);
  assert.equal(picomatch.isMatch('***', '+([!]*])', {posix:true}), false);
});

test('unescape cannot expose a star quantifier under unbounded repetition', () => {
  const pattern = '+(a\\*)';
  assert.equal(picomatch.isMatch('aaaa', pattern, {unescape:true}), false);
  assert.equal(picomatch.isMatch(pattern, pattern, {unescape:true}), true);
});
