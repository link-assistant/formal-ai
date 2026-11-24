import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {registeredRustTestFilters} from '../../../scripts/lib/ladder-test-projection.mjs';
import {ladderTestLogFilename} from '../../../scripts/lib/ladder-test-log-name.mjs';

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
test('out-of-line cases nested under inline test modules retain their Cargo identity', async () => {
  const root = fixture('// no external imports');
  try {
    fs.appendFileSync(path.join(root, 'rust/Cargo.toml'),
      '\n[[test]]\nname = "coding_contracts"\npath = "tests/unit/coding_contracts.rs"\n');
    fs.writeFileSync(path.join(root, 'rust/tests/unit/coding_contracts.rs'),
      'mod coding_discovery { mod composition; #[path = "source.rs"] mod renamed; }\n');
    const directory = path.join(root, 'rust/tests/unit/coding_discovery');
    fs.mkdirSync(directory);
    fs.writeFileSync(path.join(directory, 'composition.rs'),
      'use formal_ai::answer::value; #[test] fn runs() { value(); }');
    fs.writeFileSync(path.join(directory, 'source.rs'),
      'use formal_ai::answer::value; #[test] fn runs() { value(); }');
    const observed = await registeredRustTestFilters(root, 'rust/src/answer.rs');
    assert.deepEqual(observed.jobs, [
      {target: 'coding_contracts', filter: 'coding_discovery::composition::runs'},
      {target: 'coding_contracts', filter: 'coding_discovery::renamed::runs'},
    ]);
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});
test('a binary-only module selects its exact Cargo binary cases', async () => {
  const root = fixture('// no external imports');
  try {
    fs.appendFileSync(path.join(root, 'rust/Cargo.toml'),
      '\n[[bin]]\nname = "formal-ai"\npath = "src/main.rs"\n');
    fs.writeFileSync(path.join(root, 'rust/src/main.rs'), 'mod cli_report; fn main() {}');
    fs.writeFileSync(path.join(root, 'rust/src/cli_report.rs'),
      '#[cfg(test)] mod tests { #[test] fn trace_runs() {} #[test] #[ignore] fn optional() {} }');
    const observed = await registeredRustTestFilters(root, 'rust/src/cli_report.rs');
    assert.deepEqual(observed.sourceModule, ['cli_report']);
    assert.deepEqual(observed.jobs, [{target: 'bin:formal-ai', filter: 'cli_report::tests::trace_runs'}]);
    assert.deepEqual(observed.ignoredJobs, [
      {target: 'bin:formal-ai', filter: 'cli_report::tests::optional', source: 'rust/src/cli_report.rs'},
    ]);
    fs.appendFileSync(path.join(root, 'rust/Cargo.toml'), 'test = false\n');
    await assert.rejects(registeredRustTestFilters(root, 'rust/src/cli_report.rs'), /Missing registered changed Rust module/);
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});
// Replays main run 45018538750, where frontier-only cases used the leaf's native test budget.
test('the learning adoption leaf projects only tests that import the adoption ledger', async () => {
  const observed = await registeredRustTestFilters(path.resolve('.'), 'rust/src/learning_adoption_ledger.rs');
  const owner = 'issue_701_learning_adoption::';
  assert.deepEqual(observed.jobs
    .filter(job => job.target === 'unit' && job.filter.startsWith(owner))
    .map(job => job.filter.slice(owner.length)), [
    'committed_adoption_ledger_matches_a_fresh_run',
    'every_failure_to_adopt_is_preserved_as_a_durable_record',
    'the_adoption_ledger_records_a_real_capability_delta_in_every_language',
    'the_trends_corpus_unknown_rate_is_ratcheted_to_zero',
  ]);
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
test('Cargo cannot pass on zero or absent exact tests', () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'ladder-summary-'));
  try {
    const source = fs.readFileSync(path.resolve('experiments/issue_1028_agent_cli_ladder/verify-node.sh'), 'utf8');
    const begin = source.indexOf('cargo_test() {'), end = source.indexOf('\nrustfmt_parses() {', begin);
    assert.ok(begin >= 0 && end > begin);
    const body = source.slice(begin, end);
    const selections = [
      ['unit', '--test unit', 'independent::retained'],
      ['bin:formal-ai', '--bin formal-ai', 'cli_report::tests::trace_runs'],
    ];
    for (const [target, option, filter] of selections) {
      for (const [passed, status, expected] of [['0', 0, 1], ['1', 0, 0], ['', 0, 1], ['1', 1, 1]]) {
        const workspace = fs.mkdtempSync(path.join(root, 'case-'));
        fs.mkdirSync(path.join(workspace, 'rust'), {recursive: true});
        fs.mkdirSync(path.join(workspace, '.agent-ladder'));
        fs.writeFileSync(path.join(workspace, 'rust/Cargo.toml'), '');
        const bash = 'ROOT="$2"; workspace="$1"; verification_dir="$workspace/.agent-ladder"; node=fixture; record() { :; }; target_dir() { printf "%s" "$workspace/target"; };\n'
          + 'cargo() { [[ "$*" == "test ' + option + ' -- --exact --format pretty --color never --test-threads=1 ' + filter + '" ]] || return 9; '
          + (passed === '1' ? 'printf "test ' + filter + ' ... ok\\n"; ' : '')
          + (passed ? 'printf "test result: ok. ' + passed + ' passed; 0 failed; 0 ignored;\\n"; ' : '')
          + 'return ' + status + '; };\n' + body + '\ncargo_test ' + target + ' ' + filter;
        const observed = spawnSync('bash', ['-c', bash, 'controlled-summary', workspace, path.resolve('.')], {encoding: 'utf8', timeout: 5000});
        assert.equal(observed.signal, null);
        assert.equal(observed.status, expected);
      }
    }
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});

test('the verifier writes the compile log and exact test receipt to the producer directory', () => {
  const root = fixture();
  try {
    function git(...args) {
      const result = spawnSync('git', ['-C', root, ...args], {encoding: 'utf8'});
      assert.equal(result.status, 0, result.stderr);
    }
    git('init', '-q');
    git('add', 'rust');
    git('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid',
      '-c', 'commit.gpgsign=false', 'commit', '-qm', 'Record verifier fixture');
    fs.writeFileSync(path.join(root, 'rust/src/answer.rs'), 'pub fn updated_value() {}\n');
    const node = '1.1.1.1.1';
    const proof = path.join(root, 'proof.md');
    fs.writeFileSync(proof, 'node_path=' + node + '\nThe answer function now uses updated_value.\n');
    fs.mkdirSync(path.join(root, 'agent-ladder-effects'));
    fs.writeFileSync(path.join(root, 'agent-ladder-effects', 'node-' + node + '.lino'),
      'node_path=' + node + '\nnode_depth=5\nnode_kind=leaf\nresult=The function renamed to updated_value\n');
    const mockBin = path.join(root, 'mock-bin');
    fs.mkdirSync(mockBin);
    fs.writeFileSync(path.join(mockBin, 'cargo'), [
      '#!/usr/bin/env bash',
      'case "$*" in',
      '  "check --lib --quiet") printf "compile receipt\\n" ;;',
      '  "test --test unit -- --exact --format pretty --color never --test-threads=1 independent::retained")',
      '    printf "test independent::retained ... ok\\ntest result: ok. 1 passed; 0 failed; 0 ignored;\\n" ;;',
      '  *) exit 9 ;;',
      'esac',
    ].join('\n'), {mode: 0o700});
    const verifier = path.resolve('experiments/issue_1028_agent_cli_ladder/verify-node.sh');
    const logDir = path.join(root, 'producer-evidence');
    const result = spawnSync('bash', [verifier, root, proof, node, '5', '', '',
      'rust/src/answer.rs', 'updated_value', 'pub fn'], {
      encoding: 'utf8',
      env: {...process.env, PATH: mockBin + path.delimiter + process.env.PATH,
        LADDER_CARGO_CHECK: '1', LADDER_CARGO_TEST: '1', LADDER_VERIFY_LOG_DIR: 'producer-evidence'},
      cwd: root,
    });
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.equal(result.stdout.trim(), 'ok');
    const testLog = ladderTestLogFilename('unit', 'independent::retained');
    assert.match(fs.readFileSync(path.join(logDir, 'verify.tsv'), 'utf8'),
      new RegExp('test_log:unit:independent::retained\\t' + testLog + '\\ntests:independent::retained\\tpassed=1'));
    assert.equal(fs.readFileSync(path.join(logDir, 'cargo-check.log'), 'utf8'), 'compile receipt\n');
    assert.match(fs.readFileSync(path.join(logDir, testLog), 'utf8'), /test result: ok\. 1 passed/u);
    assert.equal(fs.existsSync(path.join(root, '.agent-ladder/verify.tsv')), false);
  } finally {
    fs.rmSync(root, {recursive: true, force: true});
  }
});

test('the verifier batches exact cases per Cargo target and requires each to pass', () => {
  const root = fixture();
  try {
    fs.appendFileSync(path.join(root, 'rust/tests/unit/renamed.rs'), '\n#[test] fn retained_other() {}');
    fs.appendFileSync(path.join(root, 'rust/src/answer.rs'), '\n#[test] fn source_case() {}');
    fs.mkdirSync(path.join(root, 'receipt'));
    const source = fs.readFileSync(path.resolve('experiments/issue_1028_agent_cli_ladder/verify-node.sh'), 'utf8');
    const begin = source.indexOf('module_filter() {'), end = source.indexOf('\nrustfmt_parses() {', begin);
    assert.ok(begin >= 0 && end > begin);
    const functions = source.slice(begin, end);
    const shell = [
      'workspace="$1"; ROOT="$2"; verification_dir="$workspace/receipt"; node=fixture',
      'target_dir() { printf "%s" "$workspace/target"; }',
      'record() { printf "%s\\t%s\\n" "$1" "$2"; }',
      'cargo() {',
      '  case "$*" in',
      '    "test --test unit -- --exact --format pretty --color never --test-threads=1 independent::retained independent::retained_other")',
      '      printf "%s\\n" "test independent::retained ... ok"',
      '      if [[ "$OMIT_CASE" == 0 ]]; then printf "%s\\n" "test independent::retained_other ... ok"; fi',
      '      printf "%s\\n" "test result: ok. 2 passed; 0 failed; 0 ignored;"',
      '      return "$BATCH_STATUS" ;;',
      '    "test --lib -- --exact --format pretty --color never --test-threads=1 answer::source_case")',
      '      printf "%s\\n" "test answer::source_case ... ok" "test result: ok. 1 passed; 0 failed; 0 ignored;" ;;',
      '    *) printf "unexpected cargo arguments: %s\\n" "$*" >&2; return 9 ;;',
      '  esac',
      '}',
      functions,
      'verify_source_tests rust/src/answer.rs',
    ].join('\n');
    for (const [omit, status, expected] of [['0', '0', 'passed=1'], ['1', '0', 'missing_selected_tests'], ['0', '1', 'failed']]) {
      const result = spawnSync('bash', ['-c', shell, 'controlled-batch', root, path.resolve('.')], {
        encoding: 'utf8',
        env: {...process.env, LADDER_CARGO_TEST: '1', OMIT_CASE: omit, BATCH_STATUS: status},
      });
      const picked = ['independent::retained', 'independent::retained_other'];
      assert.equal(result.status, expected === 'passed=1' ? 0 : 1, result.stdout + result.stderr);
      const logName = ladderTestLogFilename('unit', ...picked);
      for (const filter of picked) {
        assert.ok(result.stdout.includes('test_log:unit:' + filter + '\t' + logName + '\n'), result.stdout);
      }
      assert.ok(result.stdout.includes('tests:independent::retained_other\t' + expected + '\n'), result.stdout);
      assert.ok(fs.readFileSync(path.join(root, 'receipt', logName), 'utf8').includes('test independent::retained ... ok'));
      if (expected === 'passed=1') {
        assert.ok(result.stdout.includes('tests:answer::source_case\tpassed=1\n'), result.stdout);
      } else {
        assert.doesNotMatch(result.stdout, /answer::source_case/u);
      }
    }
  } finally {
    fs.rmSync(root, {recursive: true, force: true});
  }
});

test('a changed canonical seed reaches Cargo through a temporary package mirror', () => {
  const source = 'data/seed/example.lino';
  const mirror = 'rust/embedded/' + source;
  const original = 'meaning original\n';
  const revised = 'meaning revised\n';
  const node = '1.1.1.1.1';
  for (const kind of ['success', 'empty_native_selection', 'stale_mirror']) {
    const root = fixture();
    try {
      const mirrorBefore = kind === 'stale_mirror' ? 'meaning stale\n' : original;
      function write(relative, content, mode) {
        const fullPath = path.join(root, relative);
        fs.mkdirSync(path.dirname(fullPath), {recursive: true});
        fs.writeFileSync(fullPath, content, {mode});
      }
      function git(...args) {
        const result = spawnSync('git', ['-C', root, ...args], {encoding: 'utf8'});
        assert.equal(result.status, 0, result.stderr);
        return result.stdout.trim();
      }
      write(source, original);
      write(mirror, mirrorBefore);
      git('init', '-q');
      git('add', 'rust', 'data');
      git('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid',
        '-c', 'commit.gpgsign=false', 'commit', '-qm', 'Record canonical seed and mirror');
      write(source, revised);
      write('proof.md', 'node_path=' + node + '\nThe canonical seed now records the revised meaning.\n');
      write('agent-ladder-effects/node-' + node + '.lino',
        'node_path=' + node + '\nnode_depth=5\nnode_kind=leaf\nresult=The seed records revised meaning\n');
      const selected = kind === 'empty_native_selection' ? 0 : 1;
      write('mock-bin/cargo', [
        '#!/usr/bin/env bash',
        'set -eu',
        'cmp -s ../data/seed/example.lino embedded/data/seed/example.lino',
        'case "$*" in',
        '  "check --lib --quiet") printf "compile saw changed seed\\n" ;;',
        '  "test --test data_files -- --exact --format pretty --color never --test-threads=1 embedded_seed_matches_canonical_source")',
        '    [[ "$LADDER_CHANGED_SEED" == "data/seed/example.lino" ]]',
        ...(selected ? ['    printf "test embedded_seed_matches_canonical_source ... ok\\n"'] : []),
        '    printf "test result: ok. ' + selected + ' passed; 0 failed; 0 ignored;\\n" ;;',
        '  *) exit 9 ;;',
        'esac',
      ].join('\n'), 0o700);
      const result = spawnSync('bash', [
        path.resolve('experiments/issue_1028_agent_cli_ladder/verify-node.sh'),
        root, path.join(root, 'proof.md'), node, '5', '', '', source, 'revised', 'meaning',
      ], {
        encoding: 'utf8',
        env: {...process.env, PATH: path.join(root, 'mock-bin') + path.delimiter + process.env.PATH,
          LADDER_CARGO_CHECK: '1', LADDER_CARGO_TEST: '1', LADDER_VERIFY_LOG_DIR: path.join(root, 'receipt')},
        cwd: root,
      });
      const expected = {success: 'ok', empty_native_selection: 'failing_leaf_tests',
        stale_mirror: 'seed_projection_failed'}[kind];
      assert.equal(result.stdout.trim(), expected, result.stderr);
      assert.equal(result.status, kind === 'success' ? 0 : 1);
      assert.equal(fs.readFileSync(path.join(root, mirror), 'utf8'), mirrorBefore);
      assert.equal(git('status', '--porcelain=v1', '--untracked-files=no'), 'M ' + source);
      const receipt = fs.readFileSync(path.join(root, 'receipt', 'verify.tsv'), 'utf8');
      if (kind === 'stale_mirror') {
        assert.doesNotMatch(receipt, /^seed_projection:/mu);
        assert.equal(fs.existsSync(path.join(root, 'receipt', 'cargo-check.log')), false);
      } else {
        assert.ok(receipt.includes('seed_projection:' + source + '\t' + mirror + ' '
          + git('hash-object', source) + '\n'));
        assert.equal(fs.readFileSync(path.join(root, 'receipt', 'cargo-check.log'), 'utf8'),
          'compile saw changed seed\n');
        assert.match(receipt, selected ? /\tpassed=1\n/u : /\tmissing_selected_tests\n/u);
      }
    } finally {
      fs.rmSync(root, {recursive: true, force: true});
    }
  }
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

test('namespaced exact Cargo selections have bounded portable log filenames', () => {
  const filter = 'issue_1138_recovery_live::a_missing_held_out_compiler_is_discovered_installed_and_retried';
  const observed = ladderTestLogFilename('integration', filter);
  assert.match(observed, /^cargo-test-[0-9a-f]{64}\.log$/u);
  assert.equal(observed, ladderTestLogFilename('integration', filter));
  for (const [target, name] of [['unit', filter], ['integration', filter.replace('::', '__')], ['integration', filter + 'é']]) {
    assert.notEqual(observed, ladderTestLogFilename(target, name));
  }
  assert.throws(() => ladderTestLogFilename('../foreign', filter));
  assert.throws(() => ladderTestLogFilename('unit', 'line\nbreak'));
  assert.throws(() => ladderTestLogFilename('unit'));
  assert.notEqual(observed, ladderTestLogFilename('integration', filter, 'another_case'));
});
test('existing ignored network cases are retained as evidence and cannot supply zero-test witnesses', async () => {
  const root = fixture();
  try {
    const file = path.join(root, 'rust/tests/unit/renamed.rs');
    fs.appendFileSync(file, '\n#[test]\n#[ignore = "network"]\nfn unavailable_live_case() {}');
    const observed = await registeredRustTestFilters(root, 'rust/src/answer.rs');
    assert.deepEqual(observed.jobs, [{target: 'unit', filter: 'independent::retained'}]);
    assert.deepEqual(observed.ignoredJobs, [{target: 'unit', filter: 'independent::unavailable_live_case', source: 'rust/tests/unit/renamed.rs'}]);
    fs.writeFileSync(file, 'use formal_ai::answer::value;\n#[test]\n#[ignore]\nfn only_ignored() {}');
    await assert.rejects(registeredRustTestFilters(root, 'rust/src/answer.rs'), /Missing registered test projection/);
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});
test('unregistered production sources refuse with the exact changed path', async () => {
  const root = fixture();
  try {
    fs.writeFileSync(path.join(root, 'rust/src/not_registered.rs'), 'pub fn value() {}');
    await assert.rejects(registeredRustTestFilters(root, 'rust/src/not_registered.rs'), /Missing registered changed Rust module: rust\/src\/not_registered\.rs/);
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});

test('literal include module registries retain source ownership and public reexport aliases', async () => {
  const root = fixture('use formal_ai::public_value;');
  try {
    fs.writeFileSync(path.join(root, 'rust/src/lib.rs'), 'include!("declarations.rs");\npub use answer::value as public_value;');
    fs.writeFileSync(path.join(root, 'rust/src/declarations.rs'), 'pub mod answer;');
    const observed = await registeredRustTestFilters(root, 'rust/src/answer.rs');
    assert.deepEqual(observed.sourceModule, ['answer']);
    assert.deepEqual(observed.jobs, [{target: 'unit', filter: 'independent::retained'}]);
    assert(observed.sourceGuards.some(item => item.path === 'rust/src/declarations.rs'));
    assert(observed.ownershipRoutes.some(route => JSON.stringify(route) === '["public_value"]'));
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});
test('cyclic, computed, conditional and shadowed include registries refuse', async () => {
  for (const [source, extra, expected] of [
    ['include!("declarations.rs");', 'include!("lib.rs");', /Cyclic registered Rust include/],
    ['include!(concat!("declarations", ".rs"));', 'pub mod answer;', /Missing literal Rust include/],
    ['#[cfg(feature = "other")] include!("declarations.rs");', 'pub mod answer;', /Missing unambiguous unconditional include/],
    ['macro_rules! include { ($x:expr) => {}; } include!("declarations.rs");', 'pub mod answer;', /Missing unambiguous unconditional include/],
  ]) {
    const root = fixture();
    try {
      fs.writeFileSync(path.join(root, 'rust/src/lib.rs'), source);
      fs.writeFileSync(path.join(root, 'rust/src/declarations.rs'), extra);
      await assert.rejects(registeredRustTestFilters(root, 'rust/src/answer.rs'), expected);
    } finally { fs.rmSync(root, {recursive: true, force: true}); }
  }
});

test('absolute crate imports preserve source ownership without guessing prefixes', async () => {
  const root = fixture('use ::formal_ai::answer::value;');
  try {
    const observed = await registeredRustTestFilters(root, 'rust/src/answer.rs');
    assert.deepEqual(observed.jobs, [{target: 'unit', filter: 'independent::retained'}]);
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});

test('opaque generated test scopes supply no witness and do not erase independent source-observed cases', async () => {
  const root = fixture();
  try {
    fs.appendFileSync(path.join(root, 'rust/tests/unit/mod.rs'), '\nmod generated;');
    fs.writeFileSync(path.join(root, 'rust/tests/unit/generated.rs'),
      'use formal_ai::answer::value; #[test] fn unobserved() {} include!(concat!(env!("OUT_DIR"), "/generated.rs"));');
    const observed = await registeredRustTestFilters(root, 'rust/src/answer.rs');
    assert.deepEqual(observed.jobs, [{target: 'unit', filter: 'independent::retained'}]);
    assert.equal(observed.unresolvedTestScopes.length, 1);
    assert.equal(observed.unresolvedTestScopes[0].source, 'rust/tests/unit/generated.rs');
    assert(observed.sourceGuards.some(item => item.path === 'rust/tests/unit/generated.rs'));
    fs.writeFileSync(path.join(root, 'rust/tests/unit/renamed.rs'), '// no source-owned case');
    await assert.rejects(registeredRustTestFilters(root, 'rust/src/answer.rs'), /Missing registered test projection/);
  } finally { fs.rmSync(root, {recursive: true, force: true}); }
});
