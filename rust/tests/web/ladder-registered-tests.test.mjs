import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {registeredRustTestFilters} from '../../../scripts/lib/ladder-test-projection.mjs';

function fixture(imported = 'use formal_ai::answer::value;') {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'ladder-projection-'));
  const files = {'rust/Cargo.toml': '[package]\nname = "formal-ai"\nversion = "0.0.0"\n[[test]]\nname = "unit"\npath = "tests/unit/mod.rs"\n', 'rust/src/lib.rs': 'pub mod answer;', 'rust/src/answer.rs': 'pub fn value() {}',
    'rust/tests/unit/mod.rs': '#[path = "renamed.rs"]\nmod independent;',
    'rust/tests/unit/renamed.rs': imported + '\n#[test]\nfn retained() {}'};
  for (const [relative, source] of Object.entries(files)) {
    fs.mkdirSync(path.dirname(path.join(root, relative)), {recursive: true});
    fs.writeFileSync(path.join(root, relative), source);
  }
  return root;
}
test('registered paths and parsed imports replace basename guesses', async () => {
  const root = fixture();
  try {
    const observed = await registeredRustTestFilters(root, 'rust/src/answer.rs');
    assert.deepEqual(observed.filters, ['independent::retained']);
    assert.deepEqual(observed.sourceModule, ['answer']);
    assert.equal(observed.runtimeSelection, 'Pending');
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});
test('comments and literals cannot supply source import ownership', async () => {
  for (const source of ['// use formal_ai::answer::value;', 'const FAKE: &str = "use formal_ai::answer::value;";']) {
    const root = fixture(source);
    try { await assert.rejects(registeredRustTestFilters(root, 'rust/src/answer.rs'), /Missing registered test projection/); }
    finally { fs.rmSync(root, {recursive: true, force: true}); }
  }
});
test('source path escapes, unknown seed projection and symlinks refuse', async () => {
  const root = fixture();
  try {
    await assert.rejects(registeredRustTestFilters(root, 'rust/src/../src/answer.rs'), /Missing source-owned/);
    await assert.rejects(registeredRustTestFilters(root, 'data/seed/meanings-answer.lino'), /Missing source-owned/);
    fs.renameSync(path.join(root, 'rust/src/answer.rs'), path.join(root, 'rust/src/original.rs'));
    fs.symlinkSync('original.rs', path.join(root, 'rust/src/answer.rs'));
    await assert.rejects(registeredRustTestFilters(root, 'rust/src/answer.rs'), /symlink refused/);
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});
test('the unchanged Cargo operation cannot pass on zero or absent tests', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'ladder-summary-'));
  try {
    const source = fs.readFileSync(path.resolve('experiments/issue_1028_agent_cli_ladder/verify-node.sh'), 'utf8');
    const begin = source.indexOf('cargo_test() {'), end = source.indexOf('\nrustfmt_parses() {', begin);
    assert.ok(begin >= 0 && end > begin);
    const body = source.slice(begin, end);
    for (const [passed, status, expected] of [['0', 0, 1], ['2', 0, 0], ['', 0, 1], ['2', 1, 1]]) {
      const workspace = path.join(root, 'case-' + passed + '-' + status);
      fs.mkdirSync(path.join(workspace, 'rust'), {recursive: true});
      fs.mkdirSync(path.join(workspace, '.agent-ladder'));
      fs.writeFileSync(path.join(workspace, 'rust/Cargo.toml'), '');
      const bash = 'workspace="$1"; node=fixture; record() { :; }; target_dir() { printf "%s" "$workspace/target"; };\n'
        + 'cargo() { [[ "$*" == "test --test unit --quiet -- --exact independent::retained" ]] || return 9; '
        + (passed ? 'printf "test result: ok. ' + passed + ' passed; 0 failed; 0 ignored;\\n"; ' : '')
        + 'return ' + status + '; };\n' + body + '\ncargo_test independent::retained';
      const observed = spawnSync('bash', ['-c', bash, 'controlled-summary', workspace], {encoding: 'utf8', timeout: 5000});
      assert.equal(observed.signal, null);
      assert.equal(observed.status, expected);
    }
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});

test('source-declared library cases carry their actual Cargo target', async () => {
  const root = fixture('// no external imports');
  try {
    fs.writeFileSync(path.join(root, 'rust/src/answer.rs'), '#[cfg(test)] mod tests { #[test] fn local_case() {} }');
    const observed = await registeredRustTestFilters(root, 'rust/src/answer.rs');
    assert.deepEqual(observed.jobs, [{target: 'lib', filter: 'answer::tests::local_case'}]);
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});

test('crate and custom library root identities come from Cargo source', async () => {
  const root = fixture('use observed_crate::answer::value;');
  try {
    fs.writeFileSync(path.join(root, 'rust/Cargo.toml'), '[package]\nname = "observed-crate"\nversion = "0.0.0"\n[lib]\npath = "src/entry.rs"\n[[test]]\nname = "unit"\npath = "tests/unit/mod.rs"\n');
    fs.renameSync(path.join(root, 'rust/src/lib.rs'), path.join(root, 'rust/src/entry.rs'));
    const observed = await registeredRustTestFilters(root, 'rust/src/answer.rs');
    assert.equal(observed.crateName, 'observed_crate');
    assert.equal(observed.libraryRoot, 'rust/src/entry.rs');
    assert.deepEqual(observed.jobs, [{target: 'unit', filter: 'independent::retained'}]);
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});

test('registered module cycles refuse before proposing execution', async () => {
  for (const relative of ['rust/src/answer.rs', 'rust/tests/unit/mod.rs']) {
    const root = fixture();
    try {
      fs.writeFileSync(path.join(root, relative), '#[path = "' + path.basename(relative) + '"] mod cycle;');
      await assert.rejects(registeredRustTestFilters(root, 'rust/src/answer.rs'), /Cyclic registered/);
    } finally { fs.rmSync(root, {recursive: true, force: true}); }
  }
});

test('conditional module path authority is not guessed from the default file', async () => {
  const root = fixture();
  try {
    fs.writeFileSync(path.join(root, 'rust/src/lib.rs'), '#[cfg_attr(feature = "other", path = "foreign.rs")] pub mod answer;');
    await assert.rejects(registeredRustTestFilters(root, 'rust/src/answer.rs'), /Missing conditional Rust module path projection/);
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});
