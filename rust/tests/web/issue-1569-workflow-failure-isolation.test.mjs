// Issue #1569: two main-only workflows turned one real failure into many
// missing or misleading results.
//
// - External Benchmarks ran its suites in one `set -e` loop, so the first
//   failing suite (humaneval, 12/20, every weekly run from 2026-10-26) ended it
//   and the nine suites after it never reported.
// - The Agent CLI ladder started every composite leg after leaf-evidence had
//   failed (run 42472083551); all 16 failed on the leaf artifact that was never
//   uploaded, burying the one real failure.
//
// The benchmark step is executed here as written, against a stub binary.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {chmodSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {test} from 'node:test';
import {fileURLToPath} from 'node:url';
import YAML from 'yaml';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const workflow = name => YAML.parse(readFileSync(join(ROOT, '.github/workflows', name), 'utf8'));
const source = path => readFileSync(join(ROOT, path), 'utf8');
const ALL = ['humaneval', 'mbpp', 'gsm8k', 'math', 'object_counting', 'coedit', 'egg_math', 'editeval', 'ascent_transitive_closure', 'swebench_lite'];

function benchmarkStep(suite, failing, {sharded = false, readerFails = false} = {}) {
  const job = workflow('external-benchmarks.yml').jobs[sharded ? 'upstream-shard' : 'external-benchmarks'];
  const step = job.steps.find(candidate => candidate.name === (sharded ? 'Run the window' : 'Run the bounded upstream slice'));
  assert.ok(step, 'the benchmark step exists');
  const root = mkdtempSync(join(tmpdir(), 'issue-1569-benchmarks-'));
  try {
    mkdirSync(join(root, 'rust/target/release'), {recursive: true});
    const binary = join(root, 'rust/target/release/formal-ai');
    const calls = join(root, 'calls');
    const bin = join(root, 'bin');
    mkdirSync(bin);
    mkdirSync(join(root, 'scripts'));
    const reader = 'scripts/install-swebench-parquet-reader.sh';
    copyFileSync(join(ROOT, reader), join(root, reader));
    writeFileSync(join(bin, 'python3'), `#!/bin/bash
printf 'python3 %s\\n' "$*" >> "$TEST_CALLS"
[[ "$TEST_READER_FAILS" == 'false' ]]
`, {mode: 0o755});
    // The stub reports like the real runner and fails the listed suites.
    writeFileSync(binary, `#!/bin/bash
suite="$4"
printf 'run %s\\n' "$suite" >> "$TEST_CALLS"
echo "suite=$suite passed=1 failed=0 total=1"
case " ${failing.join(' ')} " in *" $suite "*) exit 1;; esac
`);
    chmodSync(binary, 0o755);
    const summary = join(root, 'summary.md');
    const run = spawnSync('bash', ['-e', '-c', step.run], {cwd: root, encoding: 'utf8', timeout: 30000,
      env: {PATH: `${bin}:${process.env.PATH}`, TEST_CALLS: calls, TEST_READER_FAILS: String(readerFails),
        GITHUB_STEP_SUMMARY: summary, BENCHMARK_SLICE: '20', SWE_BENCH_SLICE: '23', BENCHMARK_SUITE: suite,
        SUITE: suite, OFFSET: '5', WINDOW: '2'}});
    return {status: run.status, stdout: run.stdout,
      summary: existsSync(summary) ? readFileSync(summary, 'utf8') : '',
      calls: existsSync(calls) ? readFileSync(calls, 'utf8').trim().split('\n') : []};
  } finally {
    rmSync(root, {recursive: true, force: true});
  }
}

test('a failing benchmark suite no longer stops the suites after it', () => {
  const run = benchmarkStep('all', ['humaneval', 'math']);
  assert.equal(run.status, 1, 'the step still fails');
  assert.deepEqual(run.summary.trim().split('\n').map(line => line.split(' ')[0]), ALL.map(suite => 'suite=' + suite));
  assert.deepEqual(run.stdout.split('\n').filter(line => line.startsWith('::error')).map(line => /title=External benchmark (\S+) failed/u.exec(line)[1]),
    ['humaneval', 'math']);
  assert.match(run.stdout, /^failed suites: humaneval math$/mu);
});

test('passing and single-suite selections keep their exit status', () => {
  assert.equal(benchmarkStep('all', []).status, 0);
  const single = benchmarkStep('swebench_lite', []);
  assert.equal(single.status, 0);
  assert.equal(single.summary.trim(), 'suite=swebench_lite passed=1 failed=0 total=1');
  assert.equal(benchmarkStep('gsm8k', ['gsm8k']).status, 1);
});

test('ratchet rejection still uploads review-gated diagnostics', () => {
  const job = workflow('external-benchmarks.yml').jobs['external-benchmarks'];
  const run = job.steps.find(step => step.name === 'Run the bounded upstream slice');
  const upload = job.steps.find(step => step.name === 'Upload run log');
  assert.match(run.run, /--learning-report "target\/formal-ai-benchmarks\/learning\/\$1\.lino"/u);
  assert.equal(upload.if, 'always()');
  assert.ok(upload.with.path.split('\n').includes('target/formal-ai-benchmarks/learning/*.lino'));

  const native = source('scripts/ci-native-regressions.sh');
  assert.match(native, /interfaces\) family_arguments=\(--test interface_contracts\)/u);
  assert.match(source('rust/tests/integration/interface_contracts.rs'), /^pub mod formal_ai_cli;/mu);
  assert.match(source('rust/tests/integration/formal_ai_cli.rs'), /^mod benchmark_report;/mu);
  assert.match(source('rust/tests/integration/formal_ai_cli/benchmark_report.rs'),
    /^fn benchmark_report_survives_a_rejected_score_without_publishing_it\(\)/mu);
});

// Replays main run 45197910485, which could not decode the SWE-bench dataset.
test('SWE-bench installs the locked dataset reader before evaluation in both workflows', () => {
  const reader = [
    'python3 -m pip install --require-hashes --only-binary=:all: --force-reinstall -r .github/benchmarks/swebench-requirements.txt',
    'python3 -c import pyarrow.parquet',
  ];
  const swebench = [...reader, 'run swebench_lite'];
  assert.deepEqual(benchmarkStep('all', []).calls, [...ALL.slice(0, -1).map(suite => 'run ' + suite), ...swebench]);
  for (const sharded of [false, true]) {
    const sweep = benchmarkStep('swebench_lite', [], {sharded});
    assert.equal(sweep.status, 0);
    assert.deepEqual(sweep.calls, swebench);
    assert.deepEqual(benchmarkStep('gsm8k', [], {sharded}).calls, ['run gsm8k']);
  }
  const failed = benchmarkStep('all', [], {readerFails: true});
  assert.equal(failed.status, 1);
  assert.match(failed.stdout, /^failed suites: swebench_lite$/mu);
  assert.deepEqual(failed.calls, [...ALL.slice(0, -1).map(suite => 'run ' + suite), reader[0]]);
  const failedShard = benchmarkStep('swebench_lite', [], {sharded: true, readerFails: true});
  assert.equal(failedShard.status, 1);
  assert.deepEqual(failedShard.calls, [reader[0]]);
});

test('ladder composites run only on validated leaf evidence', () => {
  const {composites, compare} = workflow('issue-1028-agent-ladder.yml').jobs;
  assert.deepEqual(composites.needs, ['plan', 'leaf-evidence']);
  assert.match(composites.if, /needs\.leaf-evidence\.result == 'success'/u);
  // The verdict still counts the skipped composite legs as missing measurements.
  assert.ok(compare.needs.includes('composites') && compare.needs.includes('leaf-evidence'));
  assert.doesNotMatch(compare.if, /leaf-evidence/u);
});
