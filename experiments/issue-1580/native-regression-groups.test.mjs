// Exercise the real runner with a finite Cargo stub; no Rust build is performed.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync, mkdirSync, existsSync, readdirSync, readFileSync, writeFileSync, rmSync} from 'node:fs';
import {join, resolve, dirname} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import YAML from 'yaml';
import {gunzipSync} from 'node:zlib';
import {parseTestDurations} from '../../scripts/lib/ci-speed-durations.mjs';
import {planShards, durationLookup, partitionProblems} from '../../scripts/lib/ci-speed-shards.mjs';
import {admission} from '../../scripts/ci-pull-request-admission.mjs';
const groups = {
  core: ['calendar_currency', 'openai_forced_functions', 'meta_executor_parity', 'synthesis_contracts',
    'cnl_output', 'cnl_parity', 'unknown_reasoning', 'data_files', 'scheduling_contracts',
    'operational_contracts', 'workflow_contracts'],
  coding: ['coding_contracts'], reasoning: ['reasoning_contracts'], hive: ['reasoning_contracts'],
  'coding-1': ['coding_contracts'], 'coding-2': ['coding_contracts'],
  'reasoning-1': ['reasoning_contracts'], 'reasoning-2': ['reasoning_contracts'],
  benchmarks: ['reasoning_contracts'], interfaces: ['interface_contracts'],
  'hive-1': ['reasoning_contracts'], 'hive-2': ['reasoning_contracts'], 'hive-3': ['reasoning_contracts'],
};
const hiveNames = [
  'extended::a_workflow_requirement_adds_the_workflow_in_every_language',
  'extended::codex_plans_the_work_item_and_never_narrates_the_connector_placeholder',
  'extended::the_named_branch_is_pushed_to_in_every_language',
  'extended::the_named_branch_is_pushed_to_in_russian',
  'extended::a_future_contract', 'repository_workflows_include_runtime_setup_and_output_verification',
  'program_output_constraints_do_not_consume_filenames_or_change_case',
].map(name => 'issue_1133_hive_mind_three_runs::' + name);
function observe(args, status = 0, {listStatus = 0, names = hiveNames} = {}) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-native-group-'));
  try {
    writeFileSync(join(directory, 'cargo'), `#!/usr/bin/env node
import fs from 'node:fs';
fs.appendFileSync(process.env.NATIVE_GROUP_CALLS, JSON.stringify(process.argv.slice(2)) + '\\n');
if (process.argv.includes('--list')) {
  process.stdout.write(JSON.parse(process.env.NATIVE_GROUP_NAMES).map(name => name + ': test\\n').join(''));
  process.exit(Number(process.env.NATIVE_GROUP_LIST_STATUS));
}
process.exit(Number(process.env.NATIVE_GROUP_STATUS));
`, {mode: 0o755});
    const calls = join(directory, 'calls.jsonl');
    const result = spawnSync('bash', [resolve('scripts/ci-native-regressions.sh'), ...args], {
      cwd: directory, encoding: 'utf8', timeout: 5000, maxBuffer: 65536,
      env: {...process.env, PATH: directory + ':' + process.env.PATH, RUNNER_TEMP: directory,
        NATIVE_GROUP_CALLS: calls, NATIVE_GROUP_STATUS: String(status),
        NATIVE_GROUP_NAMES: JSON.stringify(names), NATIVE_GROUP_LIST_STATUS: String(listStatus)},
    });
    assert.equal(result.error, undefined, String(result.error));
    let invocations = [];
    try { invocations = readFileSync(calls, 'utf8').trim().split('\n').map(JSON.parse); }
    catch (error) { if (error.code !== 'ENOENT') throw error; }
    return {result, invocations};
  } finally { rmSync(directory, {recursive: true, force: true}); }
}
for (const [group, targets] of Object.entries(groups).filter(([name]) => !name.startsWith('hive-') && !name.startsWith('reasoning-') && !name.startsWith('coding-'))) {
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
  assert.deepEqual(matrix.strategy.matrix.group, Object.keys(groups).filter(group => !['core', 'hive', 'reasoning', 'coding'].includes(group)));
  assert.equal(matrix.strategy['fail-fast'], false);
  assert.equal(matrix.strategy['max-parallel'], 2);
  assert.equal(matrix.needs, 'contract-build', 'only failure to compile the actual shared targets suppresses consumers');
  assert.equal(matrix.if, '${{ inputs.compile-inputs-changed }}');
  assert.equal(matrix['runs-on'], 'ubuntu-24.04');
  assert.equal(matrix['timeout-minutes'], 20);
  assert.deepEqual(matrix.permissions, {contents: 'read'});
  assert.deepEqual(matrix.env, {CARGO_BUILD_JOBS: '2', FORMAL_AI_OFFLINE: '1'});
  assert.equal(matrix['continue-on-error'], undefined);
  const execute = matrix.steps.at(-1);
  assert.equal(execute.run, 'bash scripts/run-with-budget-warning.sh 900 "Native contracts ${{ matrix.group }}" bash scripts/ci-native-regressions.sh "${{ matrix.group }}"');
  assert.deepEqual(execute.env, {TEST_WARN_RATIO_PERCENT: '75', FORMAL_AI_NATIVE_CONTRACT_ARTIFACT: '${{ runner.temp }}/native-contracts'});
  assert(900 + 2 * 120 < matrix['timeout-minutes'] * 60,
    'execution and both bounded checkout attempts must expire before the job clock');
  assert(matrix.steps.some(step => step.uses === 'actions/setup-node@v7' && step.with['node-version'] === 22));
  const producer = workflow.jobs['contract-build'];
  assert.equal(producer.if, matrix.if);
  assert.equal(producer.needs, undefined, 'core behavior failures do not suppress independent contract families');
  assert.equal(producer['timeout-minutes'], 20);
  assert.deepEqual(producer.permissions, {contents: 'read'});
  assert.deepEqual(producer.steps.find(step => step.uses === './.github/actions/cache-cargo-registry').with, {'restore-only': 'true'});
  assert(!matrix.steps.some(step => /sccache|cache-cargo-registry/u.test(step.uses ?? '')), 'consumer jobs never compile');
  const upload = producer.steps.find(step => step.uses === 'actions/upload-artifact@v7');
  const download = matrix.steps.find(step => step.uses === 'actions/download-artifact@v8');
  assert(upload && download);
  assert.equal(upload.with.name, 'native-contracts-${{ github.sha }}');
  assert.equal(download.with.name, upload.with.name);
  assert.equal(download.with.path, upload.with.path);
  assert.equal(download.with['run-id'], undefined, 'download is restricted to this workflow run');
  assert.equal(upload.with['if-no-files-found'], 'error');
  assert.equal(upload.with['retention-days'], 1);
  assert.equal(producer.outputs, undefined, 'transport grants no release or semantic authority');
  assert.equal(producer.steps.find(step => step.name === 'Compile and bind the three focused test targets').run,
    'bash scripts/run-with-budget-warning.sh 900 "Compile native contracts" node scripts/native-contract-artifact.mjs build "${{ runner.temp }}/native-contracts"');
  for (const path of ['scripts/native-contract-artifact.mjs', 'scripts/native-release-source.mjs', 'scripts/native-release-trust.mjs'])
    assert.equal(admission([path])['native-type-check'], true, path);
  const core = workflow.jobs.clippy;
  assert.equal(core.name, 'Rust compile inputs changed');
  assert.equal(core['timeout-minutes'], 20);
  assert.equal(core.steps.find(step => step.name === 'Verify isolated native behavior regressions').run,
    'bash scripts/ci-native-regressions.sh core');
  assert.equal(new Set(Object.values(groups).flat()).size, 14, 'all original Cargo targets remain covered');
});

function executedShard(observed, family = 'hive') {
  assert.equal(observed.result.status, 0, observed.result.stderr);
  assert.equal(observed.invocations.length, 2, 'one exact listing and one test execution');
  const [list, run] = observed.invocations;
  assert.deepEqual(list.slice(-3), ['--list', '--format', 'terse']);
  if (family === 'hive') {
    assert.equal(list[list.indexOf('--test') + 2], 'issue_1133_hive_mind_three_runs::');
  } else {
    assert.deepEqual(list.slice(list.indexOf('--') + 1), ['--skip', 'benchmarks::',
      '--skip', 'issue_1133_hive_mind_three_runs::', '--list', '--format', 'terse']);
  }
  assert.deepEqual(run.slice(0, 7), ['test', '--manifest-path', 'rust/Cargo.toml', '--locked',
    '--no-fail-fast', '--test', 'reasoning_contracts']);
  assert.deepEqual(run.slice(7, 9), ['--', '--exact']);
  assert.deepEqual(run.slice(-2), ['--test-threads=1', '--nocapture']);
  const names = run.slice(9, -2);
  assert(names.length > 0, 'an empty shard must never run the entire target');
  return names;
}
test('the three native Hive shards cover the live listing exactly once without renaming cases', {timeout: 15000}, () => {
  const actual = [1, 2, 3].flatMap(index => executedShard(observe(['hive-' + index])));
  assert.deepEqual(actual.toSorted(), hiveNames.toSorted());
  assert.equal(new Set(actual).size, actual.length);
});
test('Hive shard selection is deterministic under a reordered native listing', {timeout: 15000}, () => {
  for (const index of [1, 2, 3]) {
    assert.deepEqual(executedShard(observe(['hive-' + index])),
      executedShard(observe(['hive-' + index], 0, {names: [...hiveNames].reverse()})));
  }
});
for (const index of [1, 2, 3]) {
  test(`Hive shard ${index} preserves listing and execution failures`, {timeout: 15000}, () => {
    const listing = observe(['hive-' + index], 0, {listStatus: 19});
    assert.equal(listing.result.status, 19, listing.result.stderr);
    assert.equal(listing.invocations.length, 1, 'a failed listing cannot start tests');
    const execution = observe(['hive-' + index], 17);
    assert.equal(execution.result.status, 17, execution.result.stderr);
    assert.equal(execution.invocations.length, 2);
  });
}
for (const names of [[], [hiveNames[0], hiveNames[0], hiveNames[1]], ['foreign::case'], [hiveNames[0]]]) {
  test('Hive refuses empty, duplicate, foreign or insufficient native inventories: ' + JSON.stringify(names),
    {timeout: 15000}, () => {
      const observed = observe(['hive-3'], 0, {names});
      assert.notEqual(observed.result.status, 0, observed.result.stderr);
      assert.equal(observed.invocations.length, 1);
    });
}


const reasoningNames = [
  'issue_1157_idempotent_commit::the_commit_names_its_files_and_a_repeated_run_is_a_clean_no_op',
  'issue_1160_regular_prompt::the_regular_prompt_delivers_a_ready_pull_request',
  'reasoning_loop::a_future_case', 'reasoning_loop::another_future_case',
  'tool_workspace::process_observation_tests::nonzero_exit_marks_failure_and_preserves_both_streams',
];
function executedReasoningShard(observed) {
  const names = executedShard(observed, 'reasoning');
  assert(names.every(name => !name.startsWith('benchmarks::') &&
    !name.startsWith('issue_1133_hive_mind_three_runs::')));
  return names;
}
test('the two native reasoning shards cover every live case exactly once', {timeout: 15000}, () => {
  const actual = [1, 2].flatMap(index => executedReasoningShard(
    observe(['reasoning-' + index], 0, {names: reasoningNames})));
  assert.deepEqual(actual.toSorted(), reasoningNames.toSorted());
  assert.equal(new Set(actual).size, actual.length);
});
test('reasoning shard selection is stable under a reordered listing', {timeout: 15000}, () => {
  for (const index of [1, 2]) {
    assert.deepEqual(executedReasoningShard(observe(['reasoning-' + index], 0, {names: reasoningNames})),
      executedReasoningShard(observe(['reasoning-' + index], 0, {names: [...reasoningNames].reverse()})));
  }
});
for (const index of [1, 2]) {
  test(`reasoning shard ${index} propagates listing and execution failures`, {timeout: 15000}, () => {
    const listing = observe(['reasoning-' + index], 0, {names: reasoningNames, listStatus: 19});
    assert.equal(listing.result.status, 19, listing.result.stderr);
    assert.equal(listing.invocations.length, 1);
    const execution = observe(['reasoning-' + index], 17, {names: reasoningNames});
    assert.equal(execution.result.status, 17, execution.result.stderr);
    assert.equal(execution.invocations.length, 2);
  });
}
for (const names of [[], [reasoningNames[0]], [reasoningNames[0], reasoningNames[0]],
  ['benchmarks::case', reasoningNames[0]], [hiveNames[0], reasoningNames[0]], ['invalid name', reasoningNames[0]]]) {
  test('reasoning rejects incomplete or foreign listings: ' + JSON.stringify(names), {timeout: 15000}, () => {
    const observed = observe(['reasoning-2'], 0, {names});
    assert.notEqual(observed.result.status, 0);
    assert.equal(observed.invocations.length, 1);
  });
}
test('the measured 148-case reasoning inventory remains a complete two-shard partition', {timeout: 15000}, () => {
  const names = [...readFileSync('scripts/ci-native-reasoning-durations.lino', 'utf8')
    .matchAll(/^  test "([^"]+)"$/gmu)].map(match => match[1]);
  assert.equal(names.length, 148);
  const actual = [1, 2].flatMap(index => executedReasoningShard(
    observe(['reasoning-' + index], 0, {names})));
  assert.deepEqual(actual.toSorted(), names.toSorted());
  assert.equal(new Set(actual).size, actual.length);
});
test('reasoning weights reproduce all 148 completed hosted observations', {timeout: 15000}, () => {
  const observed = spawnSync('python3', ['experiments/issue-1580/derive-native-reasoning-durations.py', '--check'],
    {encoding: 'utf8', timeout: 10000, maxBuffer: 65536});
  assert.equal(observed.status, 0, observed.stderr);
  assert.equal(admission(['scripts/ci-native-reasoning-durations.lino'])['native-type-check'], true);
});

test('native Hive duration weights reproduce completed hosted observations', {timeout: 15000}, () => {
  const observed = spawnSync('python3', ['experiments/issue-1580/derive-native-hive-durations.py', '--check'],
    {encoding: 'utf8', timeout: 10000, maxBuffer: 65536});
  assert.equal(observed.status, 0, observed.stderr);
});

test('completed Hive timings isolate the eleven-minute workflow case without losing coverage', () => {
  const weights = parseTestDurations(readFileSync('scripts/ci-native-hive-durations.lino', 'utf8'));
  const names = [...new Set([1, 2, 3].flatMap(index => {
    const jobs = [136489959548, 136489959657, 136489959672];
    const log = gunzipSync(readFileSync('experiments/issue-1580/evidence/ci/native-families-1d7ba479/job-' + jobs[index - 1] + '.stdout.gz')).toString();
    return [...log.matchAll(/test (issue_1133_hive_mind_three_runs::\S+) \.{3} ok/g)].map(match => match[1]);
  }))];
  assert.equal(names.length, 37);
  const slow = 'issue_1133_hive_mind_three_runs::repository_workflows_include_runtime_setup_and_output_verification';
  assert(weights.get(slow) > 672, 'the actual long case must not fall back to sixty seconds');
  assert(weights.get('issue_1133_hive_mind_three_runs::extended::a_workflow_requirement_adds_the_workflow_in_every_language') >= 160.854,
    'a faster fresh observation must preserve the earlier measured cost');
  const {shards, load} = planShards(names, 3, durationLookup(weights, 60));
  assert.deepEqual(partitionProblems(names, shards), {missing: [], repeated: [], extra: []});
  assert.deepEqual(shards.find(shard => shard.includes(slow)), [slow]);
  assert(Math.max(...load) < 700, 'the completed observations fit below twelve test minutes per shard');
});

test('the native lane admits changes to the complete Hive planner input closure', () => {
  for (const path of ['scripts/ci-native-hive-durations.lino', 'scripts/plan-test-shards.mjs',
    'scripts/lib/ci-speed-durations.mjs', 'scripts/lib/ci-speed-shards.mjs']) {
    assert.equal(admission([path])['native-type-check'], true, path);
  }
});

test('the native lane replays exclusive fragment creation when either production input changes', () => {
  for (const path of ['scripts/changelog-fragment-file.rs', 'scripts/create-changelog-fragment.rs']) {
    assert.equal(admission([path])['native-type-check'], true, path);
  }
});

test('the real JavaScript sparse checkout retains only the native weight evidence it consumes', {timeout: 15000}, () => {
  const workflow = YAML.parse(readFileSync('.github/workflows/layered-ci.yml', 'utf8'));
  const checkouts = Object.values(workflow.jobs).flatMap(job => job.steps ?? [])
    .filter(step => step.with?.['sparse-checkout']?.includes('!/experiments/*/evidence/**/*.gz'));
  assert.equal(checkouts.length, 1);
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-hive-checkout-'));
  const evidence = 'experiments/issue-1580/evidence/ci/native-families-7fce5c19/job-136011692528.stdout.gz';
  const reasoningEvidence = 'experiments/issue-1580/evidence/checks/native-reasoning-budget/job-136423862321.stdout.gz';
  const codingEvidence = [136874234881, 136890696854].map(job =>
    'experiments/issue-1580/evidence/checks/native-coding-budget/job-' + job + '.stdout.gz');
  const currentHiveEvidence = [136489959548, 136489959657, 136489959672]
    .map(job => 'experiments/issue-1580/evidence/ci/native-families-1d7ba479/job-' + job + '.stdout.gz');
  const unrelated = 'experiments/issue-1580/evidence/ci/unrelated-historical.log.gz';
  const run = (command, args) => {
    const result = spawnSync(command, args, {cwd: directory, encoding: 'utf8', timeout: 5000,
      maxBuffer: 65536, env: {...process.env, GIT_CONFIG_GLOBAL: '/dev/null', GIT_CONFIG_NOSYSTEM: '1'}});
    assert.equal(result.error, undefined, String(result.error));
    assert.equal(result.status, 0, result.stdout + result.stderr);
  };
  try {
    run('git', ['init', '--quiet']);
    for (const path of [evidence, ...currentHiveEvidence, ...codingEvidence, reasoningEvidence,
      'experiments/issue-1580/evidence/checks/native-coding-budget/identity.json',
      'scripts/ci-native-coding-durations.lino', 'experiments/issue-1580/derive-native-coding-durations.py', 'scripts/ci-native-hive-durations.lino',
      'scripts/ci-native-reasoning-durations.lino', 'experiments/issue-1580/derive-native-reasoning-durations.py',
      'experiments/issue-1580/derive-native-hive-durations.py']) {
      mkdirSync(dirname(join(directory, path)), {recursive: true});
      writeFileSync(join(directory, path), readFileSync(path));
    }
    writeFileSync(join(directory, unrelated), 'excluded archive');
    run('git', ['add', '.']);
    run('git', ['-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid',
      'commit', '--quiet', '-m', 'finite checkout fixture']);
    run('git', ['config', 'core.sparseCheckout', 'true']);
    writeFileSync(join(directory, '.git/info/sparse-checkout'), checkouts[0].with['sparse-checkout']);
    run('git', ['read-tree', '-mu', 'HEAD']);
    for (const path of [evidence, ...currentHiveEvidence, ...codingEvidence]) {
      assert(existsSync(join(directory, path)), 'each real weight derivation input must be checked out: ' + path);
    }
    assert(existsSync(join(directory, reasoningEvidence)), 'the reasoning observation must survive the real checkout');
    assert(!existsSync(join(directory, unrelated)), 'historical archives stay excluded');
    run('python3', ['experiments/issue-1580/derive-native-hive-durations.py', '--check']);
    run('python3', ['experiments/issue-1580/derive-native-reasoning-durations.py', '--check']);
    run('python3', ['experiments/issue-1580/derive-native-coding-durations.py', '--check']);
  } finally { rmSync(directory, {recursive: true, force: true}); }
});


test('the native lane admits the shared output writer and every release caller', () => {
  for (const path of ['scripts/github-output.rs', 'scripts/check-release-needed.rs',
    'scripts/get-bump-type.rs', 'scripts/get-version.rs', 'scripts/publish-crate.rs',
    'scripts/wait-for-crate.rs', 'scripts/version-and-commit.rs']) {
    assert.equal(admission([path])['native-type-check'], true, path);
  }
});


test('the native lane admits package identity, updates and every parser consumer', () => {
  const inputs = ['scripts/cargo-package.rs', 'scripts/rust-paths.rs',
    'scripts/version-and-commit-tests.rs', 'scripts/create-github-release.rs',
    'rust/tests/unit/ci-cd/cargo_package_identity.rs',
    'rust/tests/unit/ci-cd/workspace_manifest_resolution.rs'];
  for (const name of readdirSync('scripts').filter(name => name.endsWith('.rs'))) {
    const path = 'scripts/' + name;
    if (readFileSync(path, 'utf8').includes('#[path = "rust-paths.rs"]')) inputs.push(path);
  }
  for (const path of new Set(inputs)) {
    assert.equal(admission([path])['native-type-check'], true, path);
    if (path.startsWith('scripts/') && path !== 'scripts/cargo-package.rs' &&
        !path.endsWith('-tests.rs')) {
      assert.match(readFileSync(path, 'utf8'), /\/\/! toml_edit = "0\.25\.15"/, path);
    }
  }
});


test('the native workflow target imports the actual changelog package reader', () => {
  const source = readFileSync('scripts/collect-changelog.rs', 'utf8');
  const reader = source.slice(source.indexOf('fn get_version_from_cargo('), source.indexOf('fn strip_frontmatter('));
  assert.match(reader, /rust_paths::read_package_info\(Path::new\(cargo_toml_path\)\)/u);
  assert.doesNotMatch(reader, /Regex|captures/u);
  assert.match(readFileSync('rust/tests/unit/workflow_contracts.rs', 'utf8'),
    /#\[path = "\.\.\/\.\.\/\.\.\/scripts\/collect-changelog\.rs"\]\nmod collect_changelog;/u);
});


test('native admission includes fallible changelog inventory and the real collector regression', () => {
  assert.equal(admission(['scripts/changelog-files.rs'])['native-type-check'], true);
  const source = readFileSync('scripts/collect-changelog.rs', 'utf8');
  assert.match(source, /#\[path = "changelog-files\.rs"\]\nmod changelog_files;/u);
  assert.match(source, /failed_fragment_or_existing_changelog_reads_preserve_every_input/u);
  assert.doesNotMatch(source, /filter_map\(\|e\| e\.ok\(\)\)|unwrap_or_default\(\)/u);
  assert.match(source, /changelog_files::remove_fragments\(&files\)\?/u);
});


test('the focused workflow target tests actual fallible bump selection', () => {
  const source = readFileSync('scripts/get-bump-type.rs', 'utf8');
  assert.match(source, /changelog_files::read_fragments\(Path::new\(changelog_dir\)\)\?/u);
  assert.doesNotMatch(source, /filter_map\(\|e\| e\.ok\(\)\)|if let Ok\(content\)/u);
  assert.match(readFileSync('rust/tests/unit/workflow_contracts.rs', 'utf8'),
    /#\[path = "\.\.\/\.\.\/\.\.\/scripts\/get-bump-type\.rs"\]\nmod get_bump_type;/u);
});


test('the real version release collector is shared with focused native CI', () => {
  assert.equal(admission(['scripts/version-changelog.rs'])['native-type-check'], true);
  const source = readFileSync('scripts/version-and-commit.rs', 'utf8');
  assert.match(source, /use version_changelog::collect_changelog_with_date;/u);
  assert.match(source, /let collected = match collect_changelog_with_date\(/u);
  const collector = readFileSync('scripts/version-changelog.rs', 'utf8');
  assert.match(collector, /changelog_files::read_fragments\(Path::new\(changelog_dir\)\)\?/u);
  assert.match(collector, /changelog_files::remove_fragments\(&files\)\?/u);
  assert.doesNotMatch(collector, /filter_map\(|unwrap_or_default\(\)/u);
  assert.match(readFileSync('rust/tests/unit/workflow_contracts.rs', 'utf8'),
    /#\[path = "\.\.\/\.\.\/\.\.\/scripts\/version-changelog\.rs"\]\nmod version_changelog;/u);
});


test('registry uncertainty is exercised by real release CLIs in hosted native CI', () => {
  const files = ['scripts/registry.rs', ...['Cargo.toml', 'Cargo.lock', 'cli_regressions.py']
    .map(name => 'experiments/issue-1580/release-registry/' + name)];
  for (const path of files) assert.equal(admission([path])['native-type-check'], true, path);
  const workflow = YAML.parse(readFileSync('.github/workflows/native-type-check.yml', 'utf8'));
  const job = workflow.jobs['release-registry'];
  const replay = job.steps.find(step => step.name === 'Verify real release registry requests and CLI decisions');
  assert(replay);
  assert.match(replay.run, /cargo test .*release-registry\/Cargo.toml --locked --lib/u);
  assert.match(replay.run, /python3 .*release-registry\/cli_regressions.py/u);
  assert.equal(job.if, '${{ inputs.compile-inputs-changed }}');
  assert.equal(job['timeout-minutes'], 10);
  for (const checkout of job.steps.filter(step => step.uses === 'actions/checkout@v7')) {
    assert(checkout.with['sparse-checkout'].includes('/experiments/issue-1580/release-registry/'));
  }
  for (const path of ['scripts/check-release-needed.rs', 'scripts/version-and-commit.rs']) {
    const source = readFileSync(path, 'utf8');
    assert.match(source, /registry::or_exit\(\s*"crates.io published versions",\s*registry::max_published/u);
    assert.match(source, /registry::version_exists\(crate_name, version\)/u);
    assert.doesNotMatch(source, /Err\(_\) => (?:false|None)/u);
  }
});


test('the lightweight script job executes the original split Rust suites and admits their complete source closure', () => {
  const manifest = readFileSync('experiments/issue-1580/release-registry/Cargo.toml', 'utf8');
  const targets = [...manifest.matchAll(/^\[\[test\]\]\s*\nname = "([^"]+)"\s*\npath = "([^"]+)"/gmu)]
    .map(([,name,path]) => ({name,path}));
  assert.deepEqual(targets, [
    {name: 'assemble_requirements', path: '../../../scripts/assemble-requirements.rs'},
    {name: 'normalize_ordered_lists', path: '../../../scripts/normalize-ordered-lists.rs'},
    {name: 'self_hosting_metric', path: '../../../scripts/self-hosting-metric.rs'},
  ]);
  const job = YAML.parse(readFileSync('.github/workflows/native-type-check.yml', 'utf8')).jobs['release-registry'];
  const commands = job.steps.map(step => step.run ?? '').join('\n');
  assert.match(commands, /cargo test .* --locked --lib --test assemble_requirements --test normalize_ordered_lists --test self_hosting_metric -- --test-threads=1 --nocapture/u);
  assert.match(commands, /python3 experiments\/issue-1580\/release-registry\/cli_regressions[.]py/u);
  const closure = [
    'scripts/assemble-requirements.rs', 'scripts/assemble-requirements-tests.rs',
    'scripts/normalize-ordered-lists.rs', 'scripts/normalize-ordered-lists-tests.rs',
    'scripts/lib/registered-module-paths.rs', 'scripts/self-hosting-metric.rs',
    'scripts/self-hosting-metric-cli.rs', 'scripts/self-hosting-ledger.rs',
    'scripts/self-hosting-ledger-rows.rs', 'scripts/self-hosting-retraction.rs',
    'scripts/self-hosting-attribution.rs', 'scripts/self-hosting-replay.rs',
    'scripts/self-development-loop.rs',
  ];
  for (const file of closure) {
    assert(existsSync(file), file);
    assert.equal(admission([file])['native-type-check'], true, file);
  }
  for (const [file,count] of [['scripts/assemble-requirements-tests.rs',22], ['scripts/normalize-ordered-lists-tests.rs',17]]) {
    assert.equal([...readFileSync(file, 'utf8').matchAll(/^#\[test\]/gmu)].length,count,file);
  }
});

const codingNames = [...readFileSync('scripts/ci-native-coding-durations.lino', 'utf8')
  .matchAll(/^  test "([^"]+)"$/gmu)].map(match => match[1]);
function executedCodingShard(observed) {
  assert.equal(observed.result.status, 0, observed.result.stderr);
  assert.equal(observed.invocations.length, 2);
  const [list, run] = observed.invocations;
  assert.deepEqual(list.slice(-5), ['--test', 'coding_contracts', '--', '--list', '--format', 'terse'].slice(-5));
  assert.equal(list[list.indexOf('--test') + 1], 'coding_contracts');
  assert.deepEqual(run.slice(0, 9), ['test', '--manifest-path', 'rust/Cargo.toml', '--locked',
    '--no-fail-fast', '--test', 'coding_contracts', '--', '--exact']);
  assert.deepEqual(run.slice(-2), ['--test-threads=1', '--nocapture']);
  const names = run.slice(9, -2); assert(names.length > 0);
  return names;
}
test('coding shards cover all 62 observed names and an unseen compiled case exactly once', {timeout: 15000}, () => {
  const names = [...codingNames, 'future_coding_module::future_case'];
  const actual = [1, 2].flatMap(index => executedCodingShard(observe(['coding-' + index], 0, {names})));
  assert.deepEqual(actual.toSorted(), names.toSorted());
  assert.equal(new Set(actual).size, actual.length);
  const weights = parseTestDurations(readFileSync('scripts/ci-native-coding-durations.lino', 'utf8'));
  const {shards, load} = planShards(names, 2, durationLookup(weights, 60));
  assert.deepEqual(partitionProblems(names, shards), {missing: [], repeated: [], extra: []});
  assert(Math.max(...load) < 450, 'retained coding costs leave compile headroom in the existing job');
});
test('coding shard selection remains deterministic under reordered compiled names', {timeout: 15000}, () => {
  for (const index of [1, 2]) assert.deepEqual(
    executedCodingShard(observe(['coding-' + index], 0, {names: codingNames})),
    executedCodingShard(observe(['coding-' + index], 0, {names: [...codingNames].reverse()})));
});
for (const index of [1, 2]) test('coding shard ' + index + ' propagates listing and execution failures', {timeout: 15000}, () => {
  const listed = observe(['coding-' + index], 0, {names: codingNames, listStatus: 19});
  assert.equal(listed.result.status, 19); assert.equal(listed.invocations.length, 1);
  const executed = observe(['coding-' + index], 17, {names: codingNames});
  assert.equal(executed.result.status, 17); assert.equal(executed.invocations.length, 2);
});
for (const names of [[], [codingNames[0]], [codingNames[0], codingNames[0]],
  ['unqualified_foreign_case', codingNames[0]], ['invalid name', codingNames[0]]]) {
  test('coding rejects incomplete or foreign listings: ' + JSON.stringify(names), {timeout: 15000}, () => {
    const observed = observe(['coding-2'], 0, {names});
    assert.notEqual(observed.result.status, 0);
    assert.equal(observed.invocations.length, 1);
  });
}
test('coding weights reproduce complete observations while retaining unfinished-case uncertainty', {timeout: 15000}, () => {
  const result = spawnSync('python3', ['experiments/issue-1580/derive-native-coding-durations.py', '--check'],
    {encoding: 'utf8', timeout: 10000, maxBuffer: 65536});
  assert.equal(result.status, 0, result.stderr);
  assert.equal(admission(['scripts/ci-native-coding-durations.lino'])['native-type-check'], true);
});
