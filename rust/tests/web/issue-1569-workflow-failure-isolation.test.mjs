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
import {chmodSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {test} from 'node:test';
import {fileURLToPath} from 'node:url';
import YAML from 'yaml';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const workflow = name => YAML.parse(readFileSync(join(ROOT, '.github/workflows', name), 'utf8'));
const ALL = ['humaneval', 'mbpp', 'gsm8k', 'math', 'object_counting', 'coedit', 'egg_math', 'editeval', 'ascent_transitive_closure', 'swebench_lite'];

function benchmarkStep(suite, failing) {
  const steps = Object.values(workflow('external-benchmarks.yml').jobs).flatMap(job => job.steps ?? []);
  const step = steps.find(candidate => candidate.name === 'Run the bounded upstream slice');
  assert.ok(step, 'the benchmark step exists');
  const root = mkdtempSync(join(tmpdir(), 'issue-1569-benchmarks-'));
  try {
    mkdirSync(join(root, 'rust/target/release'), {recursive: true});
    const binary = join(root, 'rust/target/release/formal-ai');
    // The stub reports like the real runner and fails the listed suites.
    writeFileSync(binary, `#!/bin/bash\nsuite="$4"\necho "suite=$suite passed=1 failed=0 total=1"\ncase " ${failing.join(' ')} " in *" $suite "*) exit 1;; esac\n`);
    chmodSync(binary, 0o755);
    const summary = join(root, 'summary.md');
    const run = spawnSync('bash', ['-e', '-c', step.run], {cwd: root, encoding: 'utf8', timeout: 30000,
      env: {PATH: process.env.PATH, GITHUB_STEP_SUMMARY: summary, BENCHMARK_SLICE: '20', SWE_BENCH_SLICE: '23', BENCHMARK_SUITE: suite}});
    return {status: run.status, stdout: run.stdout, summary: readFileSync(summary, 'utf8')};
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

test('ladder composites run only on validated leaf evidence', () => {
  const {composites, compare} = workflow('issue-1028-agent-ladder.yml').jobs;
  assert.deepEqual(composites.needs, ['plan', 'leaf-evidence']);
  assert.match(composites.if, /needs\.leaf-evidence\.result == 'success'/u);
  // The verdict still counts the skipped composite legs as missing measurements.
  assert.ok(compare.needs.includes('composites') && compare.needs.includes('leaf-evidence'));
  assert.doesNotMatch(compare.if, /leaf-evidence/u);
});
