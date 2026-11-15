// Issue #1569: gate check-wasm-worker-current rebuilds the browser worker and
// diffs it against the commit, but only on main. Three merged commits
// (c50eedf30, 0bb8317e0, 29a8be67e) edited rust/src/language.rs and its seed,
// passed their pull requests, and turned main red; separately, rustc 1.100.0
// compiled the unchanged sources to different bytes and kept the daily
// dependencies-latest job red from 2026-11-13. data/meta/wasm-worker-inputs.lino
// records the inputs, the binary and the compiler, so a pull request catches
// the first cause without a compiler and a main failure names which one moved.
import assert from 'node:assert/strict';
import {mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join} from 'node:path';
import {test} from 'node:test';
import {fileURLToPath} from 'node:url';

import {compareInputs, compilerIdentity, explainDrift, main, MANIFEST, parseManifest, renderManifest, WORKER}
  from '../../../scripts/wasm-worker-inputs.mjs';
import {createHash} from 'node:crypto';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const digest = text => createHash('sha256').update(text).digest('hex');
const RUSTC = commit => `rustc 1.100.0 (${commit.slice(0, 9)} 2026-11-09)\nbinary: rustc\ncommit-hash: ${commit}\nhost: x86_64-unknown-linux-gnu\n`;
const BUILT = '0ef1bc386059596dada8fb65b50425aab2ea637a';
const NEWER = '1234567890abcdef1234567890abcdef12345678';

function fixture() {
  const root = mkdtempSync(join(tmpdir(), 'issue-1569-wasm-'));
  const files = {'js/wasm-worker/src/lib.rs': 'pub fn f() {}\n', 'rust/src/language.rs': 'pub const L: u8 = 1;\n', [WORKER]: 'wasm bytes'};
  for (const [path, text] of Object.entries(files)) {
    mkdirSync(dirname(join(root, path)), {recursive: true});
    writeFileSync(join(root, path), text);
  }
  const record = {worker: {bytes: files[WORKER].length, sha256: digest(files[WORKER])}, compiler: compilerIdentity(RUSTC(BUILT)),
    inputs: ['js/wasm-worker/src/lib.rs', 'rust/src/language.rs'].map(path => ({path, sha256: digest(files[path])}))};
  mkdirSync(join(root, 'data/meta'), {recursive: true});
  writeFileSync(join(root, MANIFEST), renderManifest(record));
  return root;
}

test('the committed worker matches the inputs it records', () => {
  const lines = [];
  assert.equal(main(['--check', '--repository', ROOT], line => lines.push(line)), 0, lines.join('\n'));
  const manifest = parseManifest(readFileSync(join(ROOT, MANIFEST), 'utf8'));
  for (const path of ['js/wasm-worker/src/lib.rs', 'rust/src/language.rs', 'rust/embedded/data/seed/language-detection.lino'])
    assert.ok(manifest.inputs.some(input => input.path === path), path + ' is a recorded worker input');
  assert.ok(manifest.inputs.every(input => !input.path.includes('..')), 'paths are normalized');
});

test('the manifest round-trips and refuses an input path it cannot quote', () => {
  const root = fixture();
  try {
    const text = readFileSync(join(root, MANIFEST), 'utf8');
    assert.equal(renderManifest(parseManifest(text)), text);
    assert.throws(() => renderManifest({...parseManifest(text), inputs: [{path: 'rust/src/a b.rs', sha256: '0'}]}), /quoting rule/u);
  } finally {
    rmSync(root, {recursive: true, force: true});
  }
});

test('a pull request that edits a worker input without rebuilding fails --check', () => {
  const root = fixture();
  try {
    let lines = [];
    assert.equal(main(['--check', '--repository', root], line => lines.push(line)), 0);
    writeFileSync(join(root, 'rust/src/language.rs'), 'pub const L: u8 = 2;\n');
    lines = [];
    assert.equal(main(['--check', '--repository', root], line => lines.push(line)), 1);
    assert.equal(lines[0], 'rust/src/language.rs changed since the worker was built');
    assert.match(lines.at(-1), /^::error title=WASM worker is stale::1 difference\(s\).*node scripts\/wasm-worker-inputs\.mjs --write/u);
    rmSync(join(root, 'js/wasm-worker/src/lib.rs'));
    writeFileSync(join(root, WORKER), 'other bytes');
    assert.deepEqual(compareInputs(root, parseManifest(readFileSync(join(root, MANIFEST), 'utf8'))).map(finding => finding.path + ' ' + finding.detail), [
      'js/wasm-worker/src/lib.rs is missing', 'rust/src/language.rs changed since the worker was built', WORKER + ' is not the binary the manifest records']);
  } finally {
    rmSync(root, {recursive: true, force: true});
  }
});

test('a main failure names the cause that moved: inputs, compiler, both, or neither', () => {
  const root = fixture();
  try {
    const manifest = parseManifest(readFileSync(join(root, MANIFEST), 'utf8'));
    const titles = lines => lines.filter(line => line.startsWith('::error')).map(line => /title=([^:]+)::/u.exec(line)[1]);
    assert.deepEqual(titles(explainDrift(root, manifest, RUSTC(NEWER))), ['WASM worker compiler changed']);
    assert.match(explainDrift(root, manifest, RUSTC(NEWER)).at(-1), /built by rustc 1\.100\.0 \(0ef1bc386 2026-11-09\) and this runner has rustc 1\.100\.0 \(123456789/u);
    assert.deepEqual(titles(explainDrift(root, manifest, RUSTC(BUILT))), ['WASM worker differs']);
    writeFileSync(join(root, 'rust/src/language.rs'), 'pub const L: u8 = 2;\n');
    assert.deepEqual(titles(explainDrift(root, manifest, RUSTC(BUILT))), ['WASM worker inputs changed']);
    assert.deepEqual(titles(explainDrift(root, manifest, RUSTC(NEWER))), ['WASM worker inputs changed', 'WASM worker compiler changed']);
  } finally {
    rmSync(root, {recursive: true, force: true});
  }
});

test('the main gate explains its failure and pull requests run this guard', () => {
  const gate = readFileSync(join(ROOT, 'data/meta/ci-gates/check-wasm-worker-current.lino'), 'utf8');
  assert.match(gate, /run "git diff --exit-code -- js\/formal_ai_worker\.wasm \|\| \{ node scripts\/wasm-worker-inputs\.mjs --explain; exit 1; \}"/u);
  const inventory = JSON.parse(readFileSync(join(ROOT, 'data/benchmarks/fast-pull-request-javascript-tests.json'), 'utf8'));
  assert.ok(JSON.stringify(inventory).includes('rust/tests/web/issue-1569-wasm-worker-inputs.test.mjs'), 'pull requests run this guard');
});
