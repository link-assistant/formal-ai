// Exercise the real Git index and production wrapper; Cargo is a recording stub.
import assert from 'node:assert/strict';
import {execFileSync, spawnSync} from 'node:child_process';
import {mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join} from 'node:path';
import test from 'node:test';

const script = new URL('../../../scripts/check-rust-dependencies.sh', import.meta.url);
const primary = 'rust/Cargo.lock';
function observe({paths = [primary], missing = [], failures = {}, inventoryFailure = 0,
  inventoryBytes = null, quotePaths = true, untracked = [], stagedOnly = []} = {}) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-cargo-inventory-'));
  try {
    for (const folder of ['.cargo', 'bin']) mkdirSync(join(directory, folder));
    writeFileSync(join(directory, '.cargo/audit.toml'), '[advisories]\nignore = []\n');
    const git = (args) => execFileSync('git', args, {cwd:directory, encoding:'utf8', stdio:['ignore','pipe','pipe']});
    git(['init', '--quiet']); git(['config', 'core.quotePath', String(quotePaths)]);
    for (const path of [...paths, ...untracked]) {
      mkdirSync(dirname(join(directory, path)), {recursive:true});
      writeFileSync(join(directory, path), '# finite protocol fixture; no Rust resolution\n');
    }
    const committed = paths.filter(path => !stagedOnly.includes(path));
    if (committed.length) git(['add', '--', ...committed]);
    git(['-c', 'user.name=Formal AI test fixture', '-c', 'user.email=fixture@example.invalid',
      '-c', 'core.hooksPath=/dev/null', 'commit', '--quiet', '--allow-empty', '--no-gpg-sign',
      '-m', 'Commit finite lockfile fixtures']);
    if (stagedOnly.length) git(['add', '--', ...stagedOnly]);
    for (const path of missing) rmSync(join(directory, path));
    const trace = join(directory, 'cargo-calls.jsonl');
    writeFileSync(trace, '');
    writeFileSync(join(directory, 'bin/cargo'), `#!${process.execPath}\n` + String.raw`
const fs = require('node:fs'), args = process.argv.slice(2);
fs.appendFileSync(process.env.CARGO_TRACE, JSON.stringify(args) + '\n');
if (args[0] !== 'audit') process.exit(99);
const path = args[args.indexOf('--file') + 1];
process.exit(Number(JSON.parse(process.env.AUDIT_FAILURES)[path] ?? 0));
`, {mode:0o755});
    if (inventoryFailure || inventoryBytes !== null) {
      writeFileSync(join(directory, 'bin/git'), `#!${process.execPath}\n` +
        `process.stdout.write(Buffer.from(${JSON.stringify(Buffer.from(inventoryBytes ?? (primary + '\0')).toString('base64'))}, 'base64'));\n` +
        `process.exit(${inventoryFailure});\n`, {mode:0o755});
    }
    const result = spawnSync('bash', [script.pathname], {
      cwd:directory, encoding:'utf8', timeout:5000, maxBuffer:65536,
      env:{...process.env, PATH:join(directory,'bin')+':'+process.env.PATH,
        CARGO_TRACE:trace, AUDIT_FAILURES:JSON.stringify(failures)}
    });
    assert.ifError(result.error);
    return {...result, calls:readFileSync(trace,'utf8').split('\n').filter(Boolean).map(JSON.parse)};
  } finally { rmSync(directory, {recursive:true, force:true}); }
}
const expected = (path) => ['audit','--file',path,'--deny', ...(path === primary ? ['warnings'] : ['unsound','--deny','yanked'])];

for (const quotePaths of [true, false]) {
  test(`audit preserves actual tracked paths with core.quotePath=${quotePaths}`, {timeout:10000}, () => {
    const paths = [primary, 'space dir/Cargo.lock', 'line\nbreak/Cargo.lock',
      'tab\tpath/Cargo.lock', 'quote"path/Cargo.lock', 'back\\slash/Cargo.lock', 'échelle/Cargo.lock'];
    const result = observe({paths, quotePaths, untracked:['untracked/Cargo.lock']});
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.deepEqual(result.calls, [expected(primary), ...paths.slice(1).sort().map(expected)]);
  });
}
test('a primary finding preserves its exit status while every independent lock is still audited', {timeout:10000}, () => {
  const paths = [primary, 'a/Cargo.lock', 'z/Cargo.lock'];
  const result = observe({paths, failures:{[primary]:17, 'a/Cargo.lock':23}});
  assert.equal(result.status, 17, result.stdout + result.stderr);
  assert.deepEqual(result.calls, [expected(primary), expected(paths[1]), expected(paths[2])]);
});
test('an independent finding remains fatal after collecting later independent reports', {timeout:10000}, () => {
  const paths = [primary, 'a/Cargo.lock', 'z/Cargo.lock'];
  const result = observe({paths, failures:{'a/Cargo.lock':23}});
  assert.equal(result.status, 23, result.stdout + result.stderr);
  assert.deepEqual(result.calls, [expected(primary), expected(paths[1]), expected(paths[2])]);
});
test('a missing tracked lock fails with a single escaped annotation and preserves later audits', {timeout:10000}, () => {
  const missing = 'a%\n\rbroken/Cargo.lock', paths = [primary, missing, 'z/Cargo.lock'];
  const result = observe({paths, missing:[missing]});
  assert.equal(result.status, 1, result.stdout + result.stderr);
  assert.match(result.stdout, /^::error::a%25%0A%0Dbroken\/Cargo\.lock is committed but not checked out/mu);
  assert.deepEqual(result.calls, [expected(primary), expected(paths[2])]);
});
test('a failed partial NUL inventory never executes Cargo', {timeout:10000}, () => {
  const result = observe({inventoryFailure:29});
  assert.equal(result.status, 29, result.stdout + result.stderr);
  assert.deepEqual(result.calls, []);
});
for (const [name, bytes] of [['unterminated', primary], ['empty record', primary+'\0\0']]) {
  test(`a successful malformed ${name} inventory fails before Cargo`, {timeout:10000}, () => {
    const result = observe({inventoryBytes:bytes});
    assert.equal(result.status, 1, result.stdout + result.stderr);
    assert.match(result.stdout + result.stderr, /invalid Cargo lockfile inventory/u);
    assert.deepEqual(result.calls, []);
  });
}
test('an untracked primary binary lock cannot pass as the committed dependency inventory', {timeout:10000}, () => {
  const result = observe({paths:['extra/Cargo.lock'], untracked:[primary]});
  assert.equal(result.status, 1, result.stdout + result.stderr);
  assert.match(result.stdout + result.stderr, /rust\/Cargo\.lock is not in the committed lockfile inventory/u);
  assert.deepEqual(result.calls, []);
});

test('a missing primary lock remains fatal while independent locks are audited', {timeout:10000}, () => {
  const result = observe({paths:[primary, 'extra/Cargo.lock'], missing:[primary]});
  assert.equal(result.status, 1, result.stdout + result.stderr);
  assert.match(result.stdout, /rust\/Cargo\.lock is committed but not checked out/u);
  assert.deepEqual(result.calls, [expected('extra/Cargo.lock')]);
});

test('a staged-only primary binary lock cannot stand in for a committed HEAD lock', {timeout:10000}, () => {
  const result = observe({paths:[primary, 'extra/Cargo.lock'], stagedOnly:[primary]});
  assert.equal(result.status, 1, result.stdout + result.stderr);
  assert.match(result.stdout + result.stderr, /rust\/Cargo\.lock is not committed at HEAD/u);
  assert.deepEqual(result.calls, []);
});
