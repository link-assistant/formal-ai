// Exercise the real runner with a finite Cargo stub; no Rust build is performed.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync, mkdirSync, existsSync, readdirSync, readFileSync, writeFileSync, rmSync} from 'node:fs';
import {join, resolve, dirname} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import YAML from 'yaml';
import {admission} from '../../scripts/ci-pull-request-admission.mjs';
const groups = {
  core: ['calendar_currency', 'openai_forced_functions', 'meta_executor_parity', 'synthesis_contracts',
    'cnl_output', 'cnl_parity', 'unknown_reasoning', 'data_files', 'scheduling_contracts',
    'operational_contracts', 'workflow_contracts'],
  coding: ['coding_contracts'], reasoning: ['reasoning_contracts'], hive: ['reasoning_contracts'],
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
for (const [group, targets] of Object.entries(groups).filter(([name]) => !name.startsWith('hive-'))) {
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
  assert.deepEqual(matrix.strategy.matrix.group, Object.keys(groups).filter(group => !['core', 'hive'].includes(group)));
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

function executedShard(observed) {
  assert.equal(observed.result.status, 0, observed.result.stderr);
  assert.equal(observed.invocations.length, 2, 'one exact listing and one test execution');
  const [list, run] = observed.invocations;
  assert.deepEqual(list.slice(-4), ['--', '--list', '--format', 'terse']);
  assert.equal(list[list.indexOf('--test') + 2], 'issue_1133_hive_mind_three_runs::');
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

test('native Hive duration weights reproduce completed hosted observations', {timeout: 15000}, () => {
  const observed = spawnSync('python3', ['experiments/issue-1580/derive-native-hive-durations.py', '--check'],
    {encoding: 'utf8', timeout: 10000, maxBuffer: 65536});
  assert.equal(observed.status, 0, observed.stderr);
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

test('the real JavaScript sparse checkout retains only the Hive weight evidence it consumes', {timeout: 15000}, () => {
  const workflow = YAML.parse(readFileSync('.github/workflows/layered-ci.yml', 'utf8'));
  const checkouts = Object.values(workflow.jobs).flatMap(job => job.steps ?? [])
    .filter(step => step.with?.['sparse-checkout']?.includes('!/experiments/*/evidence/**/*.gz'));
  assert.equal(checkouts.length, 1);
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-hive-checkout-'));
  const evidence = 'experiments/issue-1580/evidence/ci/native-families-7fce5c19/job-136011692528.stdout.gz';
  const unrelated = 'experiments/issue-1580/evidence/ci/unrelated-historical.log.gz';
  const run = (command, args) => {
    const result = spawnSync(command, args, {cwd: directory, encoding: 'utf8', timeout: 5000,
      maxBuffer: 65536, env: {...process.env, GIT_CONFIG_GLOBAL: '/dev/null', GIT_CONFIG_NOSYSTEM: '1'}});
    assert.equal(result.error, undefined, String(result.error));
    assert.equal(result.status, 0, result.stdout + result.stderr);
  };
  try {
    run('git', ['init', '--quiet']);
    for (const path of [evidence, 'scripts/ci-native-hive-durations.lino',
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
    assert(existsSync(join(directory, evidence)), 'the real weight derivation input must be checked out');
    assert(!existsSync(join(directory, unrelated)), 'historical archives stay excluded');
    run('python3', ['experiments/issue-1580/derive-native-hive-durations.py', '--check']);
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
