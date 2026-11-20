// Exercise the real runner with a finite Cargo stub; no Rust build is performed.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync, readFileSync, writeFileSync, rmSync} from 'node:fs';
import {join, resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import YAML from 'yaml';
const groups = {
  core: ['calendar_currency', 'openai_forced_functions', 'meta_executor_parity', 'synthesis_contracts',
    'cnl_output', 'cnl_parity', 'unknown_reasoning', 'data_files', 'scheduling_contracts',
    'operational_contracts', 'workflow_contracts'],
  coding: ['coding_contracts'], reasoning: ['reasoning_contracts'], hive: ['reasoning_contracts'],
  benchmarks: ['reasoning_contracts'], interfaces: ['interface_contracts'],
};
function observe(args, status = 0) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-native-group-'));
  try {
    writeFileSync(join(directory, 'cargo'), `#!/usr/bin/env node
import fs from 'node:fs';
fs.appendFileSync(process.env.NATIVE_GROUP_CALLS, JSON.stringify(process.argv.slice(2)) + '\\n');
process.exit(Number(process.env.NATIVE_GROUP_STATUS));
`, {mode: 0o755});
    const calls = join(directory, 'calls.jsonl');
    const result = spawnSync('bash', [resolve('scripts/ci-native-regressions.sh'), ...args], {
      cwd: directory, encoding: 'utf8', timeout: 5000, maxBuffer: 65536,
      env: {...process.env, PATH: directory + ':' + process.env.PATH, RUNNER_TEMP: directory,
        NATIVE_GROUP_CALLS: calls, NATIVE_GROUP_STATUS: String(status)},
    });
    assert.equal(result.error, undefined, String(result.error));
    let invocations = [];
    try { invocations = readFileSync(calls, 'utf8').trim().split('\n').map(JSON.parse); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
    return {result, invocations};
  } finally { rmSync(directory, {recursive: true, force: true}); }
}
for (const [group, targets] of Object.entries(groups)) {
  test('native group ' + group + ' selects only its complete contract family', {timeout: 15000}, () => {
    const {result, invocations} = observe([group]);
    assert.equal(result.status, 0, result.stderr);
    assert.equal(invocations.length, 1, 'each matrix job runs one Cargo test invocation');
    const args = invocations[0];
    assert.deepEqual(args.slice(0, 5), ['test', '--manifest-path', 'rust/Cargo.toml', '--locked', '--no-fail-fast']);
    assert.deepEqual(args.flatMap((arg, index) => arg === '--test' ? [args[index + 1]] : []), targets);
    assert.deepEqual(args.slice(args.indexOf('--') + 1), group === 'reasoning'
      ? ['--skip', 'benchmarks::', '--skip', 'issue_1133_hive_mind_three_runs::', '--test-threads=1', '--nocapture']
      : ['--test-threads=1', '--nocapture']);
    const filter = group === 'hive' ? 'issue_1133_hive_mind_three_runs::' : group === 'benchmarks' ? 'benchmarks::' : null;
    assert.deepEqual(args.slice(args.indexOf('--test') + 2, args.indexOf('--')), targets.length === 1 && filter ? [filter] : targets.slice(1).flatMap(target => ['--test', target]));
  });
  test('native group ' + group + ' propagates a Cargo failure', {timeout: 15000}, () => {
    const {result, invocations} = observe([group], 17);
    assert.equal(result.status, 17, result.stderr);
    assert.equal(invocations.length, 1);
  });
}
for (const args of [['unknown'], ['core', 'extra'], ['all', 'extra']]) {
  test('native groups reject invalid arguments before Cargo: ' + args.join(' '), {timeout: 15000}, () => {
    const {result, invocations} = observe(args);
    assert.equal(result.status, 2);
    assert.match(result.stderr, /Usage:/u);
    assert.deepEqual(invocations, []);
  });
}
test('native workflow gives every expensive family its own bounded independent job', () => {
  const workflow = YAML.parse(readFileSync('.github/workflows/native-type-check.yml', 'utf8'));
  const matrix = workflow.jobs['contract-family'];
  assert(matrix);
  assert.deepEqual(matrix.strategy.matrix.group, Object.keys(groups).filter(group => group !== 'core'));
  assert.equal(matrix.strategy['fail-fast'], false);
  assert.equal(matrix.strategy['max-parallel'], 2);
  assert.equal(matrix.needs, undefined, 'a failed core job must not suppress independent families');
  assert.equal(matrix.if, '${{ inputs.compile-inputs-changed }}');
  assert.equal(matrix['runs-on'], 'ubuntu-24.04');
  assert.equal(matrix['timeout-minutes'], 20);
  assert.deepEqual(matrix.permissions, {contents: 'read'});
  assert.deepEqual(matrix.env, {CARGO_BUILD_JOBS: '2', FORMAL_AI_OFFLINE: '1'});
  assert.equal(matrix['continue-on-error'], undefined);
  assert.equal(matrix.steps.at(-1).run, 'bash scripts/ci-native-regressions.sh "${{ matrix.group }}"');
  assert(matrix.steps.some(step => step.uses === 'actions/setup-node@v7' && step.with['node-version'] === 22));
  assert.deepEqual(matrix.steps.find(step => step.uses === './.github/actions/cache-cargo-registry').with, {'restore-only': 'true'});
  const core = workflow.jobs.clippy;
  assert.equal(core.name, 'Rust compile inputs changed');
  assert.equal(core['timeout-minutes'], 20);
  assert.equal(core.steps.find(step => step.name === 'Verify isolated native behavior regressions').run,
    'bash scripts/ci-native-regressions.sh core');
  assert.equal(new Set(Object.values(groups).flat()).size, 14, 'all original Cargo targets remain covered');
});
