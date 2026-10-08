// R1138-3-5: JavaScript replay of protocol_callers.rs and repository_workspace_protocol.rs.
// T1000-T1003: document-driven repository runners with an injected file/process port.
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { after, before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let protocol;
let runner;
let authoring;
let stageFunctions;
const directories = [];

before(async () => {
  await installNodeHost(new WorkerHost());
  protocol = await import('../../../js/agentic/crate/repository_workspace.mjs');
  runner = await import('../../../js/agentic/crate/repository_workspace_runner.mjs');
  authoring = await import('../../../js/agentic/crate/repository_workspace_authoring.mjs');
  stageFunctions = await import('../../../js/agentic/crate/repository_workspace_stages.mjs');
});
after(() => directories.forEach((directory) => rmSync(directory, { recursive: true, force: true })));

function fixture(overrides = {}) {
  const root = mkdtempSync(join(tmpdir(), 'formal-ai-protocol-'));
  directories.push(root);
  const git = (...argumentsList) => execFileSync('git', argumentsList, { cwd: root, encoding: 'utf8' });
  git('init', '--quiet');
  writeFileSync(join(root, 'lists.rs'), 'pub const TEST_NAMES: &[&str] = &["alpha"];\n');
  git('add', '--all');
  git('-c', 'user.name=Protocol Test', '-c', 'user.email=protocol@example.invalid', 'commit', '--quiet', '-m', 'base');
  const calls = [];
  const io = {
    read: async (directory, path) => readFileSync(join(directory, path), 'utf8'),
    write: async (directory, path, contents) => writeFileSync(join(directory, path), contents),
    sourceFiles: async () => [['lists.rs', readFileSync(join(root, 'lists.rs'), 'utf8')]],
    run: async (directory, program, argumentsList, policy) => {
      calls.push({ program, argumentsList, policy });
      if (program === 'npm') return { exit_code: 0, stdout: 'passed', ...overrides };
      try {
        return { exit_code: 0, stdout: execFileSync(program, argumentsList, { cwd: directory, encoding: 'utf8' }) };
      } catch (error) { return { exit_code: error.status, stdout: error.stdout ?? '', stderr: String(error.stderr ?? '') }; }
    },
  };
  const task = { requirement: 'Edit the TEST_NAMES list to add "beta"', clone: { base_commit: git('rev-parse', 'HEAD').trim() },
    tests: { line: 'npm test', names: ['list membership'] } };
  return { root, io, task, calls };
}

function authoringStages(steps, calls, commitStatus = 'requested') {
  return Object.fromEntries(steps.map((step) => [step.id, async () => {
    calls.push(step.id);
    return step.id === 'commit' ? commitStatus : 'observed';
  }]));
}

test('three callers consume identical declared stage names and structural runner never serves', async () => {
  const steps = protocol.loadProtocol();
  const ids = steps.map((step) => step.id);
  for (const caller of ['swe_bench', 'solve']) {
    const workspace = fixture();
    const outcome = await runner.executeWorkspaceProtocol(workspace, workspace.task, { caller });
    assert.equal(outcome.stopped_at, null);
    assert.deepEqual(protocol.traceStageIds(outcome.trace_document), ids);
    assert.ok(outcome.diff.includes('+pub const TEST_NAMES: &[&str] = &["alpha", "beta"];'));
    assert.deepEqual(outcome.edited, ['lists.rs']);
    assert.ok(outcome.need_ledger.rows.filter((row) => row.route !== 'commit').every((row) => row.status === 'satisfied'));
    assert.ok(outcome.observations.every((record) => record.for_need && record.produced_by));
    assert.equal(readFileSync(join(workspace.root, 'repository-protocol.lino'), 'utf8'), outcome.trace_document);
    assert.deepEqual(protocol.traceStageStatuses(outcome.trace_document).filter(([stage]) => ['serve', 'session'].includes(stage)),
      [['serve', 'not_applicable'], ['session', 'not_applicable']]);
    assert.ok(!workspace.calls.some((call) => call.argumentsList[0] === 'serve'));
  }
  const calls = [];
  const outcome = await authoring.runAuthoringWith(authoringStages(steps, calls), { steps });
  assert.deepEqual(protocol.traceStageIds(outcome.trace_document), ids);
  assert.equal(outcome.committed, false);
  assert.ok(!calls.includes('commit'));
});

test('removing the serve stage from the document invokes no server port', async () => {
  const steps = protocol.loadProtocol().filter((step) => step.id !== 'serve');
  const calls = [];
  const outcome = await authoring.runAuthoringWith(authoringStages(protocol.loadProtocol(), calls), { steps });
  assert.ok(!calls.includes('serve'));
  assert.ok(!protocol.traceStageIds(outcome.trace_document).includes('serve'));
});

test('opened authoring commit gate is written before the caller commits', async () => {
  const steps = protocol.loadProtocol();
  const calls = [];
  const traces = [];
  const stages = authoringStages(steps, calls);
  stages.commit = async () => {
    assert.equal(protocol.traceStageStatuses(traces.at(-1)).find(([stage]) => stage === 'commit')[1], 'requested');
    return 'requested';
  };
  const outcome = await authoring.runAuthoringWith(stages, { steps, commit: true, writeTrace: async (text) => traces.push(text) });
  assert.equal(outcome.committed, true);
  assert.equal(protocol.traceStageStatuses(outcome.trace_document).at(-1)[1], 'requested');
});

test('base mismatch stops before editing and states the open requirement', async () => {
  const workspace = fixture();
  workspace.task.clone.base_commit = 'f'.repeat(40);
  const outcome = await runner.executeWorkspaceProtocol(workspace, workspace.task);
  assert.equal(outcome.stopped_at.id, 'clone');
  assert.match(outcome.open[0], /base commit f{40} was not observed/u);
  assert.deepEqual(outcome.edited, []);
  assert.equal(outcome.trace.stages.find((stage) => stage.id === 'locate').status, 'not_reached');
  assert.ok(outcome.need_ledger.rows.every((row) => row.status === 'planned'));
});

test('failed tests, missing prerequisites and deadlines never satisfy verification', async () => {
  for (const failure of [{ exit_code: 1, stdout: 'failed' }, { missing: true, stderr: 'absent' },
    { timed_out: true, elapsed_seconds: 1801 }]) {
    const workspace = fixture(failure);
    const outcome = await runner.executeWorkspaceProtocol(workspace, workspace.task);
    assert.equal(outcome.stopped_at.id, 'verify');
    assert.equal(outcome.need_ledger.rows.find((row) => row.route === 'verify').status, 'planned');
    assert.equal(outcome.diff, '');
    assert.equal(outcome.trace.stages.find((stage) => stage.id === 'commit').status, 'not_reached');
  }
});

test('ambiguous declarations remain unresolved and commands use seed argument shapes', () => {
  const source = 'const TEST_NAMES = ["alpha"];';
  assert.deepEqual(stageFunctions.locateRepositoryTargets([['one.rs', source], ['two.rs', source]], 'Edit the TEST_NAMES list to add "beta"'), []);
  assert.equal(stageFunctions.allowsRepositoryCommand('git', ['checkout', '--detach', 'object']), true);
  assert.equal(stageFunctions.allowsRepositoryCommand('git', ['checkout', 'branch']), false);
  assert.equal(stageFunctions.allowsRepositoryCommand('sh', ['-c', 'text']), false);
  assert.deepEqual(stageFunctions.repositoryCommandArguments('python3 -c ""'), ['python3', '-c', '']);
});

test('unobserved authoring prerequisites keep the commit gate closed; failed stages stop', async () => {
  const steps = protocol.loadProtocol();
  const calls = [];
  const stages = authoringStages(steps, calls);
  delete stages.verify;
  const outcome = await authoring.runAuthoringWith(stages, { steps, commit: true });
  assert.equal(outcome.committed, false);
  assert.ok(!calls.includes('commit'));
  stages.read = async () => { throw new Error('read failed'); };
  const failure = await authoring.runAuthoringWith(stages, { steps });
  assert.equal(failure.stopped_at.id, 'read');
  assert.deepEqual(failure.open, ['read failed']);
  assert.equal(failure.trace.stages.find((stage) => stage.id === 'serve').status, 'not_reached');
});
