// Issue #703 R703-3, R703-4, R703-5 and the dispatch modes built on them
// (R924-7, R924-8, R933-13, R991-9, issue #1069) in the JavaScript root:
// js/agentic/crate/orchestration_dispatch.mjs, orchestration_incremental.mjs and
// orchestration_attribution.mjs mirror rust/src/orchestration/{dispatch,incremental,
// attribution}.rs. A node program stands in for the external agent.
//
// Rust twins, by name: in rust/tests/integration/issue_703_orchestration_followup.rs
// parallel_comparison_records_a_ledger_and_composes_the_winner,
// winner_selection_is_deterministic_for_a_recorded_ledger,
// comparison_rejects_duplicate_cli_identities_before_creating_candidates,
// comparison_refuses_to_overwrite_workspace_drift_after_candidates_fork,
// custom_output_directory_inside_workspace_is_excluded_from_candidate_copies,
// dispatch_output_cannot_escape_the_granted_workspace,
// dispatch_joins_every_worker_before_returning_an_error,
// universal_decomposition_dispatches_independent_leaves_in_parallel,
// decomposition_never_composes_changes_from_a_failed_agent; in
// issue_703_controller_boundaries.rs
// leaves_writing_different_bytes_to_one_path_are_refused_by_name; in
// issue_703_orchestration.rs public_cli_compares_multiple_explicitly_granted_bash_agents
// and comparison_cli_succeeds_when_one_verified_winner_passes; in
// issue_991_incremental_dispatch.rs the five incremental tests; in
// issue_1069_attributed_dispatch.rs the six attribution tests.

import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { after, before, test } from 'node:test';

import { DispatchError } from '../../../js/agentic/crate/orchestration_dispatch_error.mjs';
import { dispatchAgents, dispatchConfig, selectWinner } from '../../../js/agentic/crate/orchestration_dispatch.mjs';
import { grantFor } from '../../../js/agentic/crate/orchestration_permission.mjs';
import { canonicalSessionText, replaySession } from '../../../js/agentic/crate/orchestration_replay.mjs';
import { agentCommand, sessionPassed, verificationCommand } from '../../../js/agentic/crate/orchestration_runner.mjs';
import { readSession } from '../../../js/agentic/crate/orchestration_session_file.mjs';
import {
  bootHost, cleanupTemps, executableScript, fixtureCommand, makeTemp, runCli, pathWith, writeFixture,
  FIXTURE_RELEASE_ENV, FIXTURE_STARTED_ENV,
} from './support/issue-0703-orchestration.mjs';

let fixture;

before(async () => {
  await bootHost();
  fixture = writeFixture(makeTemp('fixture'));
});

after(cleanupTemps);

const COMPOUND = 'Add a paths-ignore filter for experiments to release.yml and make docs-changed respect excluded_folders.';
const ATOMIC = 'Add dev/log/ to the excluded_folders array.';

/** A dispatch of `task` over `clis` in `workspace`, granted and with the fixture commands for `modes`. */
function fixtureDispatch(task, workspace, clis, modes, mode = 'decompose') {
  const config = dispatchConfig(task, workspace, clis);
  config.mode = mode;
  config.permission = grantFor(workspace);
  for (const cli of clis) config.command_overrides.set(cli, fixtureCommand(fixture, modes[cli] ?? modes.default ?? 'success'));
  config.allowlisted_agent_commands.add(process.execPath);
  return config;
}

/** The `Display` of the error a dispatch rejects with. */
const dispatchRefusal = async (config) => {
  try {
    await dispatchAgents(config);
  } catch (error) {
    assert.ok(error instanceof DispatchError, String(error));
    return error;
  }
  assert.fail('the dispatch was not refused');
};

test('R703-4: parallel comparison records a ledger and composes the winner', async () => {
  const workspace = makeTemp('compare');
  writeFileSync(path.join(workspace, 'README.md'), 'before\n');
  const config = fixtureDispatch('add a README badge', workspace, ['codex', 'claude'], {}, 'compare');

  const report = await dispatchAgents(config);

  assert.equal(report.schema, 'formal-ai-dispatch-report-v1');
  assert.equal(report.sessions.length, 2);
  assert.deepEqual(report.ledger.entries.map((entry) => [entry.cli, entry.session_file, entry.passed, entry.diff_size]), [
    ['codex', 'sessions/000-codex.json', true, 'before\n'.length + 'fixture change\n'.length],
    ['claude', 'sessions/001-claude.json', true, 'before\n'.length + 'fixture change\n'.length],
  ]);
  assert.equal(report.ledger.winner, selectWinner(report.ledger.entries));
  assert.equal(readFileSync(path.join(workspace, 'README.md'), 'utf8'), 'fixture change\n');
  assert.equal(readFileSync(path.join(config.output_dir, 'comparison-ledger.json'), 'utf8'), canonicalSessionText(report.ledger));
  for (const entry of report.ledger.entries) assert.deepEqual(readSession(path.join(config.output_dir, entry.session_file)).task, 'add a README badge');
});

test('R703-4: winner selection is deterministic for a recorded ledger', () => {
  const entry = (cli, passed, diff, time) => ({ cli, task: 'task', passed, diff_size: diff, wall_time_ms: time, session_file: `${cli}.json` });
  const entries = [entry('codex', true, 20, 10), entry('claude', true, 10, 50), entry('agent', false, 1, 1)];
  assert.equal(selectWinner(entries), 'claude');
  assert.equal(selectWinner(entries.slice(2)), null);
  assert.equal(selectWinner([entry('b', true, 5, 5), entry('a', true, 5, 5)]), 'a', 'ties fall to the cli id');
});

test('R703-4: duplicate CLI identities are refused before any candidate exists', async () => {
  const workspace = makeTemp('duplicate-cli');
  const config = fixtureDispatch('add a README badge', workspace, ['codex', 'codex'], {}, 'compare');
  const error = await dispatchRefusal(config);
  assert.equal(error.message, 'duplicate_cli:codex');
  assert.equal(error.debug, 'DuplicateCli("codex")');
  assert.equal(existsSync(config.output_dir), false);
});

test('R703-1: dispatch without a grant or without a CLI is refused by name', async () => {
  const workspace = makeTemp('dispatch-denied');
  const config = dispatchConfig('add a README badge', workspace, ['codex']);
  assert.equal((await dispatchRefusal(config)).message, 'permission_denied');
  config.permission = grantFor(workspace);
  config.clis = [];
  assert.equal((await dispatchRefusal(config)).message, 'missing_cli');
});

test('R703-3: comparison refuses to overwrite workspace drift after the candidates fork', async () => {
  const workspace = makeTemp('workspace-drift');
  writeFileSync(path.join(workspace, 'README.md'), 'before\n');
  const config = fixtureDispatch('add a README badge', workspace, ['codex'], {}, 'compare');
  const started = path.join(workspace, 'candidate-started');
  const release = path.join(workspace, 'release-candidate');
  config.command_overrides.set('codex', fixtureCommand(fixture, 'coordinated_success', {
    [FIXTURE_STARTED_ENV]: started,
    [FIXTURE_RELEASE_ENV]: release,
  }));

  const dispatch = dispatchAgents(config);
  const deadline = Date.now() + 10000;
  while (!existsSync(started)) {
    assert.ok(Date.now() < deadline, 'candidate fixture did not signal startup');
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
  writeFileSync(path.join(workspace, 'README.md'), 'external update\n');
  writeFileSync(release, 'release\n');

  await assert.rejects(dispatch, (error) => error instanceof DispatchError && error.message.includes('io:workspace_drift:README.md'));
  assert.equal(readFileSync(path.join(workspace, 'README.md'), 'utf8'), 'external update\n');
});

test('R703-3: a custom output directory inside the workspace is excluded from candidate copies', async () => {
  const workspace = makeTemp('nested-output');
  writeFileSync(path.join(workspace, 'README.md'), 'before\n');
  const config = fixtureDispatch('add a README badge', workspace, ['codex'], {}, 'compare');
  config.output_dir = path.join(workspace, 'agent-artifacts');

  const report = await dispatchAgents(config);

  assert.equal(report.sessions.length, 1);
  assert.ok(existsSync(path.join(config.output_dir, 'comparison-ledger.json')));
  assert.equal(existsSync(path.join(config.output_dir, 'candidates/000-codex/agent-artifacts')), false);
});

test('R703-1: dispatch output cannot escape the granted workspace', async () => {
  const workspace = makeTemp('output-boundary');
  const outside = makeTemp('outside-output');
  const config = fixtureDispatch('add a README badge', workspace, ['codex'], {});
  config.output_dir = path.join(outside, 'agent-artifacts');
  assert.equal((await dispatchRefusal(config)).message, 'output_outside_workspace');
  assert.equal(existsSync(config.output_dir), false);

  config.output_dir = path.join(workspace, '..', 'escape');
  assert.equal((await dispatchRefusal(config)).message, 'output_outside_workspace');
  config.output_dir = workspace;
  assert.equal((await dispatchRefusal(config)).message, 'output_outside_workspace');
});

test('R703-4: every worker is joined before an error is returned', async () => {
  const workspace = makeTemp('join-on-error');
  const config = fixtureDispatch('add a README badge', workspace, ['codex', 'claude'], { claude: 'delayed_success' }, 'compare');
  const missing = path.join(workspace, 'missing-program');
  config.command_overrides.set('codex', agentCommand(missing));
  config.allowlisted_agent_commands.add(missing);
  const started = Date.now();

  const error = await dispatchRefusal(config);

  assert.equal(error.message, 'run:process:cannot find binary path');
  assert.equal(error.debug, 'Run(Process(Custom { kind: NotFound, error: "cannot find binary path" }))');
  assert.ok(Date.now() - started >= 100, 'the delayed worker finished before the error was returned');
  assert.equal(readFileSync(path.join(config.output_dir, 'candidates/001-claude/README.md'), 'utf8'), 'fixture change\n');
});

test('R703-3: universal decomposition dispatches independent leaves and composes their effects', async () => {
  const workspace = makeTemp('decompose');
  const config = fixtureDispatch('Create a README badge and add a release note.', workspace, ['codex', 'opencode'], {});

  const report = await dispatchAgents(config);

  // "Create a README badge" has no operation contract, so the approved
  // strategy plans it as four stages; the second sentence is one direct leaf.
  assert.deepEqual(report.tasks, [
    'Record independently checkable requirements for Create a README badge',
    'Add a regression test that reproduces Create a README badge',
    'Implement the smallest general change that satisfies Create a README badge',
    'Run the acceptance checks for Create a README badge',
    'add a release note.',
  ]);
  assert.equal(report.sessions.length, report.tasks.length);
  assert.deepEqual(report.sessions.map((session) => session.cli), ['codex', 'opencode', 'codex', 'opencode', 'codex']);
  assert.equal(report.composed_changes.length, 1);
  assert.equal(report.composed_changes[0].path, 'README.md');
  assert.equal(report.ledger.winner, null);
  assert.equal(readFileSync(path.join(workspace, 'README.md'), 'utf8'), 'fixture change\n');
});

test('R703-3: decomposition never composes changes from a failed agent', async () => {
  const workspace = makeTemp('failed-decomposition');
  writeFileSync(path.join(workspace, 'README.md'), 'before\n');
  const config = fixtureDispatch('Create a README badge and add a release note.', workspace, ['codex', 'opencode'], { codex: 'success', opencode: 'failed_change' });

  const report = await dispatchAgents(config);

  assert.ok(report.sessions.some((session) => !sessionPassed(session)));
  assert.equal(readFileSync(path.join(workspace, 'README.md'), 'utf8'), 'fixture change\n');
});

test('R703-3: leaves writing different bytes to one path are refused by name and leave the workspace untouched', async () => {
  const bin = makeTemp('conflict-bin');
  const workspace = makeTemp('conflict');
  const script = executableScript(bin, 'leaf-writer', '#!/bin/sh\nprintf \'%s\\n\' "$1" > README.md\n');
  const config = dispatchConfig('Create a README badge and add a release note.', workspace, ['codex']);
  config.permission = grantFor(workspace);
  config.allowlisted_agent_commands.add(script);
  config.command_overrides.set('codex', agentCommand(script, ['{task}']));

  const error = await dispatchRefusal(config);

  assert.equal(error.message, 'composition_conflict:README.md');
  assert.equal(existsSync(path.join(workspace, 'README.md')), false, 'a refused composition applies neither leaf');
});

test('R703-5: the public CLI compares several explicitly granted Bash agents', () => {
  const workspace = makeTemp('custom-bash-dispatch');
  writeFileSync(path.join(workspace, 'README.md'), 'before\n');
  const argv = JSON.stringify(['sh', '-c', 'printf \'# %s\\n\' "$1" > README.md', 'formal-ai-custom-agent', '{task}']);
  const result = runCli(['dispatch', '--cli', 'neural-a,neural-b', '--compare', '--target', 'vendor', '--task', 'custom neural result',
    '--workspace', workspace, '--command', `neural-a=${argv}`, '--command', `neural-b=${argv}`, '--allow-agent-command', 'sh']);
  assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
  const report = JSON.parse(result.stdout);
  assert.equal(report.sessions.length, 2);
  assert.ok(report.sessions.every((session) => session.events.some((event) => event.kind === 'custom_adapter_granted' && event.detail === 'sh')));
  assert.equal(readFileSync(path.join(workspace, 'README.md'), 'utf8'), '# custom neural result\n');
  assert.equal(result.stdout, `${JSON.stringify(report, null, 2)}\n`);
});

test('R703-4: the comparison CLI succeeds when one verified winner passes', () => {
  const bin = makeTemp('comparison-cli-bin');
  executableScript(bin, 'codex', '#!/bin/sh\nprintf \'passing winner\\n\' > README.md\n');
  executableScript(bin, 'claude', '#!/bin/sh\nprintf \'failed candidate\\n\' > README.md\nexit 7\n');
  const workspace = makeTemp('comparison-cli-winner');
  writeFileSync(path.join(workspace, 'README.md'), 'before\n');
  const result = runCli(['dispatch', '--cli', 'codex,claude', '--compare', '--target', 'vendor', '--task', 'replace README', '--workspace', workspace],
    { PATH: pathWith(bin) });
  assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
  assert.equal(readFileSync(path.join(workspace, 'README.md'), 'utf8'), 'passing winner\n');
  assert.equal(JSON.parse(result.stdout).ledger.winner, 'codex');
});

test('R703-4: a compare run with no passing candidate exits non-zero with agent_dispatch_failed', () => {
  const bin = makeTemp('comparison-cli-nowinner-bin');
  executableScript(bin, 'codex', '#!/bin/sh\nexit 7\n');
  const workspace = makeTemp('comparison-cli-nowinner');
  const result = runCli(['dispatch', '--cli', 'codex', '--compare', '--target', 'vendor', '--task', 'replace README', '--workspace', workspace],
    { PATH: pathWith(bin) });
  assert.equal(result.status, 1);
  assert.equal(result.stderr, 'Error: "agent_dispatch_failed"\n');
  assert.equal(JSON.parse(result.stdout).ledger.winner, null);
});

// ---- incremental dispatch (issue #991, R924-7, R924-8, R933-13, R991-9) ----

/** A CLI that can only carry a task up to `limit` characters, unless `done.txt` already has content. */
const sizeLimitedCli = (directory, name, limit) => executableScript(directory, name, [
  '#!/bin/sh', 'task="$1"',
  `if [ \${#task} -gt ${limit} ] && [ ! -s done.txt ]; then`,
  `  echo "task of \${#task} characters exceeds ${limit}" >&2`,
  '  exit 7', 'fi',
  'printf \'%s\\n\' "$task" >> done.txt', '',
].join('\n'));

const refusingCli = (directory, name) => executableScript(directory, name, '#!/bin/sh\necho \'this cli cannot do that\' >&2\nexit 7\n');
const capableCli = (directory, name) => executableScript(directory, name, '#!/bin/sh\nprintf \'%s\\n\' "$1" >> done.txt\n');

/** A CLI whose parent retry destroys effects that its children composed. */
const regressingCli = (directory, name) => executableScript(directory, name, [
  '#!/bin/sh', 'task="$1"', 'printf \'%s\\n\' "$task" > .current-task', 'case "$task" in',
  '  *\' and \'*)', '    if [ -f left.done ] && [ -f right.done ]; then', '      rm -f left.done',
  '      printf \'parent retry regressed composed effects\\n\' > regressed.txt', '    fi', '    ;;',
  '  *paths-ignore*) printf \'left\\n\' > left.done ;;', '  *docs-changed*) printf \'right\\n\' > right.done ;;',
  '  *) exit 9 ;;', 'esac', '',
].join('\n'));

function incrementalConfig(task, workspace, clis) {
  const config = dispatchConfig(task, workspace, clis.map(([cli]) => cli));
  config.mode = 'incremental';
  config.permission = grantFor(workspace);
  for (const [cli, program] of clis) {
    config.allowlisted_agent_commands.add(program);
    config.command_overrides.set(cli, agentCommand(program, ['{task}']));
  }
  return config;
}

test('R991-9: a task too big for the CLI is split from its failure and composed back up', async () => {
  const bin = makeTemp('incremental-bin');
  const workspace = makeTemp('incremental-split');
  const config = incrementalConfig(COMPOUND, workspace, [['codex', sizeLimitedCli(bin, 'codex', 70)]]);

  const report = await dispatchAgents(config);

  assert.equal(report.mode, 'incremental');
  const trace = report.incremental;
  assert.equal(trace.schema, 'formal-ai-incremental-trace-v1');
  assert.equal(trace.solved, true);
  assert.equal(trace.split_depth_reached, 1);
  assert.deepEqual(trace.blocked_tasks, []);
  assert.equal(trace.splits.length, 1);
  assert.equal(trace.splits[0].task, COMPOUND);
  assert.deepEqual(trace.splits[0].children, [
    'Add a paths-ignore filter for experiments to release.yml',
    'make docs-changed respect excluded_folders.',
  ]);
  assert.equal(trace.splits[0].failure_evidence, 'cli:codex status:Failed exit:7 verification:');
  assert.deepEqual(trace.steps.map((step) => [step.task, step.passed]), [
    [COMPOUND, false],
    ['Add a paths-ignore filter for experiments to release.yml', true],
    ['make docs-changed respect excluded_folders.', true],
    [COMPOUND, true],
  ]);
  assert.equal(readFileSync(path.join(workspace, 'done.txt'), 'utf8'), [
    'Add a paths-ignore filter for experiments to release.yml',
    'make docs-changed respect excluded_folders.',
    COMPOUND,
    '',
  ].join('\n'));
  assert.ok(report.composed_changes.some((change) => change.path === 'done.txt'));
  for (const step of trace.steps) {
    replaySession(readFileSync(path.join(config.output_dir, step.session_file), 'utf8'));
  }
  const learning = readFileSync(path.join(config.output_dir, 'learning.lino'), 'utf8');
  assert.ok(learning.includes('human_gated "true"'), learning);
  assert.ok(learning.includes('observation_count "4"'), learning);
  assert.ok(learning.includes('decision "awaiting_human_review"'), learning);
});

test('R924-8: verified child composition is not regressed by a redundant parent retry', async () => {
  const bin = makeTemp('incremental-composition-bin');
  const workspace = makeTemp('incremental-composition');
  writeFileSync(path.join(workspace, 'verify.sh'), [
    '#!/bin/sh', 'task=${FORMAL_AI_VERIFICATION_TASK:-$(cat .current-task)}', 'case "$task" in',
    '  *\' and \'*) test -f left.done && test -f right.done ;;', '  *paths-ignore*) test -f left.done ;;',
    '  *docs-changed*) test -f right.done ;;', '  *) exit 9 ;;', 'esac', '',
  ].join('\n'));
  const config = incrementalConfig(COMPOUND, workspace, [['codex', regressingCli(bin, 'codex')]]);
  config.allowlisted_commands.add('sh');
  config.verification.push(verificationCommand('sh', ['verify.sh']));

  const report = await dispatchAgents(config);

  const trace = report.incremental;
  assert.equal(trace.solved, true);
  assert.equal(trace.steps.length, 4);
  const last = trace.steps.at(-1);
  assert.deepEqual([last.task, last.cli, last.passed], [COMPOUND, 'composed-verifier', true]);
  assert.ok(existsSync(path.join(workspace, 'left.done')));
  assert.ok(existsSync(path.join(workspace, 'right.done')));
  assert.equal(existsSync(path.join(workspace, 'regressed.txt')), false);
  const replay = JSON.parse(readFileSync(path.join(config.output_dir, last.session_file), 'utf8'));
  assert.equal(replay.native_session, undefined);
  assert.equal(replay.program, 'verification-only');
  assert.ok(readFileSync(path.join(config.output_dir, 'learning.lino'), 'utf8').includes('observation_count "3"'));
});

test('R991-9: an irreducible failure escalates to the next CLI instead of stopping', async () => {
  const bin = makeTemp('incremental-escalation-bin');
  const workspace = makeTemp('incremental-escalation');
  const config = incrementalConfig(ATOMIC, workspace, [['codex', refusingCli(bin, 'codex')], ['claude', capableCli(bin, 'claude')]]);

  const report = await dispatchAgents(config);

  const trace = report.incremental;
  assert.equal(trace.solved, true);
  assert.equal(trace.split_depth_reached, 0, 'an atomic task cannot split');
  assert.equal(trace.splits.length, 1);
  assert.deepEqual(trace.splits[0].children, [], 'the recorded split says the task is irreducible');
  assert.deepEqual(trace.steps.map((step) => [step.cli, step.passed]), [['codex', false], ['claude', true]]);
  assert.equal(readFileSync(path.join(workspace, 'done.txt'), 'utf8'), `${ATOMIC}\n`, 'only the passing attempt may touch the workspace');
});

test('R991-9: a task no CLI can solve is reported blocked with all of its evidence', async () => {
  const bin = makeTemp('incremental-blocked-bin');
  const workspace = makeTemp('incremental-blocked');
  const config = incrementalConfig(ATOMIC, workspace, [['codex', refusingCli(bin, 'codex')], ['claude', refusingCli(bin, 'claude')]]);

  const report = await dispatchAgents(config);

  const trace = report.incremental;
  assert.equal(trace.solved, false);
  assert.deepEqual(trace.blocked_tasks, [ATOMIC]);
  assert.equal(trace.steps.length, 2);
  assert.deepEqual(report.composed_changes, []);
  assert.equal(existsSync(path.join(workspace, 'done.txt')), false);
  assert.equal(trace.proposals.length, 1);
  const [proposal] = trace.proposals;
  assert.equal(proposal.task, ATOMIC);
  assert.deepEqual(proposal.tried_clis, ['codex', 'claude']);
  assert.equal(proposal.status, 'human_review_required');
  assert.deepEqual(proposal.failure_evidence, [
    'cli:codex status:Failed exit:7 verification:',
    'cli:claude status:Failed exit:7 verification:',
  ]);
  const document = readFileSync(path.join(config.output_dir, 'proposals.lino'), 'utf8');
  assert.ok(document.includes(proposal.id), document);
  assert.ok(document.includes('status "human_review_required"'), document);
});

test('R991-9: a solved run writes an empty proposal document rather than none at all', async () => {
  const bin = makeTemp('incremental-solved-bin');
  const workspace = makeTemp('incremental-solved');
  const config = incrementalConfig(ATOMIC, workspace, [['codex', capableCli(bin, 'codex')]]);

  const report = await dispatchAgents(config);

  assert.equal(report.incremental.solved, true);
  assert.deepEqual(report.incremental.proposals, []);
  assert.equal(readFileSync(path.join(config.output_dir, 'proposals.lino'), 'utf8').trim(), 'incremental_proposals');
});

// ---- attributed dispatch (issue #1069) ----

const SESSION_ID = 'ses_issue_1069_attributed';
const PULL_REQUEST = 'https://github.com/link-assistant/formal-ai/pull/1070';

function git(repo, args) {
  const result = spawnSync('git', ['-C', repo, ...args], { encoding: 'utf8' });
  assert.equal(result.status, 0, `git ${args.join(' ')} failed: ${result.stderr}`);
  return result.stdout.trim();
}

function initializedRepo(label) {
  const workspace = makeTemp(label);
  git(workspace, ['init', '--quiet']);
  git(workspace, ['config', 'user.name', 'Formal AI Fixture']);
  git(workspace, ['config', 'user.email', 'formal-ai@example.invalid']);
  git(workspace, ['config', 'commit.gpgsign', 'false']);
  writeFileSync(path.join(workspace, 'README.md'), 'fixture\n');
  git(workspace, ['add', 'README.md']);
  git(workspace, ['commit', '--quiet', '-m', 'fixture']);
  return workspace;
}

const SESSION_LINE = `printf '%s\\n' 'formal-ai: orchestration-session-json:{"id":"${SESSION_ID}","resume_command":"agent --resume ${SESSION_ID} --no-fork"}' >&2\n`;

const attributedAgent = (bin) => executableScript(bin, 'agent', `#!/bin/sh\nprintf 'verified Formal AI effect\\n' > attributed.txt\n${SESSION_LINE}`);
const sessionlessAgent = (bin) => executableScript(bin, 'sessionless-agent', '#!/bin/sh\nprintf \'unattributable effect\\n\' > attributed.txt\n');
const multiEffectAgent = (bin) => executableScript(bin, 'multi-effect-agent', [
  '#!/bin/sh', 'task="$1"', 'case "$task" in', '  *\' and \'*)', '    test -f left.done && test -f right.done || exit 7',
  '    printf \'root\\n\' > root.done', '    ;;', '  *paths-ignore*) printf \'left\\n\' > left.done ;;',
  '  *docs-changed*) printf \'right\\n\' > right.done ;;', '  *) exit 9 ;;', 'esac', SESSION_LINE,
].join('\n'));

function attributedConfig(workspace, program, pullRequest) {
  const config = dispatchConfig('Create attributed.txt as one verified effect.', workspace, ['agent']);
  config.mode = 'incremental';
  config.permission = grantFor(workspace);
  config.allowlisted_agent_commands.add(program);
  config.command_overrides.set('agent', agentCommand(program, ['{task}']));
  config.pull_request = pullRequest;
  return config;
}

test('issue 1069: dispatch keeps native agent state outside the candidate worktree', async () => {
  const bin = makeTemp('issue-1069-native-home-bin');
  const workspace = makeTemp('issue-1069-native-home');
  const controller = executableScript(bin, 'formal-ai-controller', [
    '#!/bin/sh', 'orchestration_home=', 'while test "$#" -gt 0; do', '  case "$1" in',
    '    --orchestration-home) orchestration_home="$2"; shift 2 ;;', '    *) shift ;;', '  esac', 'done',
    'test -n "$orchestration_home" || exit 20', 'case "$orchestration_home/" in "$PWD/"*) exit 21 ;; esac',
    'mkdir -p "$orchestration_home"', 'printf \'native state is isolated\\n\' > "$orchestration_home/state"',
    'printf \'verified effect\\n\' > isolated.txt', '',
  ].join('\n'));
  const config = dispatchConfig('Create isolated.txt as one verified effect.', workspace, ['agent']);
  config.mode = 'incremental';
  config.permission = grantFor(workspace);
  config.controller_program = controller;

  const report = await dispatchAgents(config);

  assert.equal(report.incremental.solved, true);
  assert.equal(readFileSync(path.join(workspace, 'isolated.txt'), 'utf8'), 'verified effect\n');
});

test('issue 1069: incremental dispatch commits each verified effect with its session evidence', async () => {
  const bin = makeTemp('issue-1069-attribution-bin');
  const workspace = initializedRepo('issue-1069-attribution');
  const config = attributedConfig(workspace, attributedAgent(bin), PULL_REQUEST);

  const report = await dispatchAgents(config);

  assert.equal(report.incremental.solved, true);
  assert.equal(git(workspace, ['rev-list', '--count', 'HEAD']), '2');
  const message = git(workspace, ['show', '-s', '--format=%B', 'HEAD']);
  assert.equal(message, [
    'formal-ai: apply verified agent effect',
    '',
    `Formal-AI-Session: ${SESSION_ID}`,
    'Formal-AI-Evidence: .formal-ai-orchestration/sessions/000-agent.json',
    `Formal-AI-Pull-Request: ${PULL_REQUEST}`,
  ].join('\n'));
  assert.deepEqual(git(workspace, ['show', '--pretty=format:', '--name-only', 'HEAD']).split('\n'), [
    '.formal-ai-orchestration/sessions/000-agent.json',
    'attributed.txt',
  ]);
  const evidence = git(workspace, ['show', 'HEAD:.formal-ai-orchestration/sessions/000-agent.json']);
  assert.ok(evidence.includes(SESSION_ID));
  assert.equal(git(workspace, ['status', '--porcelain', '--untracked-files=no']), '');
});

test('issue 1069: attributed dispatch rejects a dirty worktree before starting an agent', async () => {
  const bin = makeTemp('issue-1069-dirty-bin');
  const workspace = initializedRepo('issue-1069-dirty');
  writeFileSync(path.join(workspace, 'human-work.txt'), 'do not commit\n');
  const config = attributedConfig(workspace, attributedAgent(bin), PULL_REQUEST);

  const error = await dispatchRefusal(config);

  assert.equal(error.message, 'attribution:workspace_not_clean');
  assert.equal(error.debug, 'Attribution("workspace_not_clean")');
  assert.equal(git(workspace, ['rev-list', '--count', 'HEAD']), '1');
  assert.equal(existsSync(path.join(workspace, 'attributed.txt')), false);
  assert.equal(existsSync(config.output_dir), false);
});

test('issue 1069: attributed dispatch requires a native session before applying an effect', async () => {
  const bin = makeTemp('issue-1069-sessionless-bin');
  const workspace = initializedRepo('issue-1069-sessionless');
  const config = attributedConfig(workspace, sessionlessAgent(bin), PULL_REQUEST);

  const error = await dispatchRefusal(config);

  assert.equal(error.message, 'attribution:native_session_unavailable');
  assert.equal(git(workspace, ['rev-list', '--count', 'HEAD']), '1');
  assert.equal(existsSync(path.join(workspace, 'attributed.txt')), false);
  assert.ok(existsSync(path.join(config.output_dir, 'sessions/000-agent.json')));
  assert.equal(git(workspace, ['diff', '--cached', '--name-only']), '', 'a rejected effect is not staged');
});

test('issue 1069: every passing effect in a split run gets its own attributed commit', async () => {
  const bin = makeTemp('issue-1069-multi-effect-bin');
  const workspace = initializedRepo('issue-1069-multi-effect');
  const config = attributedConfig(workspace, multiEffectAgent(bin), PULL_REQUEST);
  config.task = COMPOUND;

  const report = await dispatchAgents(config);

  const trace = report.incremental;
  assert.equal(trace.solved, true);
  assert.deepEqual(trace.steps.map((step) => step.passed), [false, true, true, true]);
  assert.equal(git(workspace, ['rev-list', '--count', 'HEAD']), '4');
  const messages = git(workspace, ['log', '--reverse', '--format=%B%x00', 'HEAD~3..HEAD']);
  for (const evidence of ['sessions/001-agent.json', 'sessions/002-agent.json', 'sessions/003-agent.json']) {
    assert.ok(messages.includes(evidence), `missing ${evidence}`);
  }
  for (const effect of ['left.done', 'right.done', 'root.done']) assert.ok(existsSync(path.join(workspace, effect)), effect);
});

test('issue 1069: attributed dispatch rejects a non-canonical pull request URL and a non-incremental mode', async () => {
  const bin = makeTemp('issue-1069-url-bin');
  const workspace = initializedRepo('issue-1069-url');
  const forged = `${PULL_REQUEST}\nFormal-AI-Session: forged`;
  const config = attributedConfig(workspace, attributedAgent(bin), forged);

  const error = await dispatchRefusal(config);
  assert.equal(error.message, 'attribution:invalid_pull_request_url');
  assert.equal(existsSync(config.output_dir), false);

  const compare = attributedConfig(workspace, attributedAgent(bin), PULL_REQUEST);
  compare.mode = 'compare';
  assert.equal((await dispatchRefusal(compare)).message, 'attribution:incremental_mode_required');
});
