// Issue #703 R703-1, R703-2, R703-5, R703-6, R703-7, R703-10 in the JavaScript
// root: one permission-gated run of an external agent CLI. The runner twin
// (js/agentic/crate/orchestration_runner.mjs, orchestration_permission.mjs,
// orchestration_workspace.mjs) and the CLI twin (js/agentic/orchestration_cli.mjs)
// mirror rust/src/orchestration/{runner,permission,workspace}.rs and
// rust/src/cli_orchestration.rs. A node program stands in for the external
// agent the Rust tests re-execute their own test binary as.
//
// Rust twins, by name, in rust/tests/integration/issue_703_orchestration.rs:
// qwen_orchestration_uses_noninteractive_auto_edit_config,
// seed_registry_pins_each_clients_exact_native_resume_contract,
// external_agent_exports_the_canonical_workspace_as_pwd,
// external_agent_run_requires_an_explicit_permission_grant,
// arbitrary_agent_program_requires_a_separate_explicit_grant,
// a_registered_cli_label_does_not_bypass_the_custom_program_grant,
// external_agent_permission_is_scoped_to_one_workspace,
// all_required_seed_adapters_capture_process_and_workspace_events,
// direct_vendor_entrypoints_keep_editing_and_prompt_arguments,
// public_cli_runs_an_explicitly_granted_agent_through_bash,
// public_cli_resumes_the_recorded_vendor_session_with_correction_evidence;
// in issue_703_orchestration_followup.rs: timeout_is_recorded_without_an_implicit_retry,
// timeout_terminates_descendant_processes,
// verification_commands_must_be_explicitly_allowlisted,
// verification_timeout_is_recorded_and_fails_the_session,
// failed_external_process_is_visible_in_the_session,
// recorded_session_replays_byte_for_byte, replay_rejects_valid_but_noncanonical_json,
// a_proven_false_claim_resumes_the_same_native_session_with_parent_evidence,
// continuation_rejects_a_client_that_reports_a_different_native_session;
// in issue_703_orchestration_languages.rs:
// vendor_orchestration_preserves_tasks_in_every_supported_language;
// in issue_703_controller_boundaries.rs:
// a_custom_entrypoint_without_a_task_placeholder_is_refused_before_it_runs.
// Not ported: the wrapper path through `formal-ai with` (the Rust binary), see
// the controller-argv test below for what the runner itself contributes.

import assert from 'node:assert/strict';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { after, before, test } from 'node:test';

import { clientIntegrations } from '../../../js/agentic/crate/seed_client_integrations.mjs';
import { grantFor } from '../../../js/agentic/crate/orchestration_permission.mjs';
import { ReplayError, canonicalSessionText, replaySession } from '../../../js/agentic/crate/orchestration_replay.mjs';
import {
  AgentRunError, agentCommand, agentRunConfig, resumeAgent, runAgent, sessionPassed, sessionSha256,
  verificationCommand, withCommand, withPermission,
} from '../../../js/agentic/crate/orchestration_runner.mjs';
import { readSession, writeSession } from '../../../js/agentic/crate/orchestration_session_file.mjs';
import {
  REQUIRED_CLIS, bootHost, cleanupTemps, executableScript, fixtureCommand, fixtureConfig,
  makeTemp, processIsRunning, runCli, withPath, writeFixture,
} from './support/issue-0703-orchestration.mjs';

let fixture;

before(async () => {
  await bootHost();
  fixture = writeFixture(makeTemp('fixture'));
});

after(cleanupTemps);

/** The error a rejected run throws, so its `Display` can be asserted exactly. */
const refusal = async (run) => {
  try {
    await run();
  } catch (error) {
    assert.ok(error instanceof AgentRunError, String(error));
    return error;
  }
  assert.fail('the run was not refused');
};

test('R703-2: the registry pins Qwen to non-interactive auto-edit and every client to its exact resume argv', () => {
  const integrations = clientIntegrations();
  const qwen = integrations.find((client) => client.id === 'qwen');
  assert.equal(qwen.invocation.temp_home_config_path, '.qwen/settings.json');
  assert.deepEqual(qwen.invocation.temp_home_json_settings.find(([key]) => key === 'tools.approvalMode'), ['tools.approvalMode', 'auto-edit']);

  const expected = [
    ['agent', 'agent --resume session-703 --no-fork', ['--resume', '{session_id}', '--no-fork']],
    ['claude', 'claude --resume session-703', ['--resume', '{session_id}']],
    ['codex', 'codex exec resume session-703', ['resume', '{session_id}']],
    ['gemini', 'gemini --resume session-703', ['--resume', '{session_id}']],
    ['opencode', 'opencode --session session-703', ['--session', '{session_id}']],
    ['qwen', 'qwen --resume session-703', ['--resume', '{session_id}']],
  ];
  for (const [cli, display, argv] of expected) {
    const integration = integrations.find((entry) => entry.id === cli);
    assert.equal(integration.invocation.resume_command.replace('{session_id}', 'session-703'), display);
    assert.deepEqual(integration.invocation.resume_args, argv);
    assert.equal(integration.verification.surface, 'cli');
  }
});

test('R703-1: the external agent sees the canonical workspace as its PWD', async () => {
  const workspace = makeTemp('canonical-pwd');
  const session = await runAgent(fixtureConfig(fixture, 'opencode', workspace, 'assert_pwd'));
  assert.equal(session.status, 'succeeded', session.stderr);
});

test('R703-1: an ungranted run and a grant for another workspace are both refused with permission_denied', async () => {
  const workspace = makeTemp('denied');
  const denied = await refusal(() => runAgent(agentRunConfig('codex', 'add a README badge', workspace)));
  assert.equal(denied.message, 'permission_denied');
  assert.equal(denied.debug, 'PermissionDenied');

  const other = makeTemp('not-granted');
  const config = withPermission(agentRunConfig('codex', 'add a README badge', other), grantFor(workspace));
  assert.equal((await refusal(() => runAgent(config))).message, 'permission_denied');
});

test('R703-5: an arbitrary agent program needs its own exact grant on top of the workspace grant', async () => {
  const workspace = makeTemp('custom-agent-grant');
  const command = fixtureCommand(fixture, 'success');
  const config = withCommand(withPermission(agentRunConfig('private-neural-tui', 'answer the task', workspace), grantFor(workspace)), command);

  const denied = await refusal(() => runAgent(config));
  assert.equal(denied.message, `agent_command_not_allowlisted:${process.execPath}`);
  assert.equal(denied.debug, `AgentCommandNotAllowlisted(${JSON.stringify(process.execPath)})`);

  config.allowlisted_agent_commands.add(command.program);
  const session = await runAgent(config);
  assert.equal(session.cli, 'private-neural-tui');
  assert.ok(sessionPassed(session));
  assert.deepEqual(session.events.map((event) => event.kind), [
    'permission_granted',
    'custom_adapter_granted',
    'adapter_selected',
    'process_started',
    'process_succeeded',
    'workspace_effect',
  ]);
});

test('R703-5: a registered CLI label does not bypass the custom program grant and forges no native session', async () => {
  const workspace = makeTemp('registered-label-custom-grant');
  const command = fixtureCommand(fixture, 'success', { FORMAL_AI_ISSUE_703_OUTPUT: '{"session_id":"forged"}' });
  const config = withCommand(withPermission(agentRunConfig('codex', 'answer the task', workspace), grantFor(workspace)), command);

  assert.equal((await refusal(() => runAgent(config))).message, `agent_command_not_allowlisted:${command.program}`);

  config.allowlisted_agent_commands.add(command.program);
  const session = await runAgent(config);
  assert.equal(session.native_session, undefined);
  assert.ok(session.events.some((event) => event.kind === 'custom_adapter_granted' && event.detail === command.program));
  assert.ok(session.events.some((event) => event.kind === 'process_started' && event.detail === command.program));
});

test('R703-2: all six registered adapters capture process and workspace events', async () => {
  for (const cli of REQUIRED_CLIS) {
    const workspace = makeTemp(cli);
    writeFileSync(path.join(workspace, 'README.md'), 'before\n');
    const session = await runAgent(fixtureConfig(fixture, cli, workspace, 'success'));
    assert.equal(session.cli, cli);
    assert.equal(session.status, 'succeeded');
    assert.equal(session.stdout, 'fixture_stdout\n');
    assert.equal(session.stderr, 'fixture_stderr\n');
    assert.equal(session.changes.length, 1);
    assert.equal(session.changes[0].path, 'README.md');
    assert.equal(session.changes[0].kind, 'modified');
    assert.ok(session.events.some((event) => event.kind === 'workspace_effect'));
  }
});

test('R703-1: a timeout is recorded once, kills the process, and is never retried', async () => {
  const workspace = makeTemp('timeout');
  const config = fixtureConfig(fixture, 'codex', workspace, 'timeout');
  config.timeout_ms = 200;
  const session = await runAgent(config);
  assert.equal(session.status, 'timed_out');
  assert.equal(session.exit_code, 137, 'command-stream reports 128 + SIGKILL');
  assert.equal(session.events.filter((event) => event.kind === 'process_started').length, 1);
  assert.deepEqual(session.events.map((event) => event.kind).slice(-1), ['process_timed_out']);
});

test('R703-1: a timeout terminates the descendant processes of the agent', { skip: process.platform === 'win32' }, async () => {
  const workspace = makeTemp('descendant-timeout');
  const config = fixtureConfig(fixture, 'codex', workspace, 'descendant_timeout');
  // Not the behaviour under test: the margin that makes the descendant exist
  // when the timeout reaches it (a kill before the spawn would prove nothing).
  config.timeout_ms = 2000;
  const session = await runAgent(config);
  assert.equal(session.status, 'timed_out');
  const descendant = readFileSync(path.join(workspace, 'descendant-pid'), 'utf8').trim();
  const deadline = Date.now() + 5000;
  while (processIsRunning(descendant)) {
    assert.ok(Date.now() < deadline, `descendant ${descendant} was still running 5s after the agent timed out`);
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
  assert.equal(existsSync(path.join(workspace, 'descendant-survived')), false);
});

test('R703-3: a verification command must be explicitly allowlisted', async () => {
  const workspace = makeTemp('allowlist');
  const config = fixtureConfig(fixture, 'codex', workspace, 'success');
  config.verification.push(verificationCommand('unreviewed-command', []));
  const error = await refusal(() => runAgent(config));
  assert.equal(error.message, 'command_not_allowlisted:unreviewed-command');
  assert.equal(existsSync(path.join(workspace, 'README.md')), false, 'nothing ran');
});

test('R703-1: a verification timeout is recorded and fails the session', async () => {
  const workspace = makeTemp('verification-timeout');
  const config = fixtureConfig(fixture, 'codex', workspace, 'success');
  config.timeout_ms = 300;
  config.allowlisted_commands.add(process.execPath);
  config.verification.push(verificationCommand(process.execPath, ['-e', 'setTimeout(() => {}, 5000)']));
  const session = await runAgent(config);
  assert.equal(sessionPassed(session), false);
  assert.equal(session.verification.length, 1);
  assert.equal(session.verification[0].timed_out, true);
  assert.equal(session.verification[0].passed, false);
  assert.deepEqual(session.events.map((event) => event.kind), [
    'permission_granted', 'custom_adapter_granted', 'adapter_selected', 'process_started', 'process_succeeded',
    'verification_started', 'verification_timed_out', 'workspace_effect',
  ]);
});

test('R703-1: a non-zero exit is evidence in the session, not a controller error', async () => {
  const workspace = makeTemp('failed');
  const session = await runAgent(fixtureConfig(fixture, 'agent', workspace, 'failed'));
  assert.equal(session.status, 'failed');
  assert.equal(session.exit_code, 7);
  assert.equal(session.events.filter((event) => event.kind === 'process_started').length, 1);
  assert.equal(sessionPassed(session), false);
});

test('R703-7: a recorded session replays byte for byte, and valid but non-canonical JSON is rejected', async () => {
  const workspace = makeTemp('replay');
  const session = await runAgent(fixtureConfig(fixture, 'gemini', workspace, 'success'));
  const file = path.join(workspace, 'session.json');
  writeSession(file, session);
  const bytes = readFileSync(file, 'utf8');
  assert.equal(bytes, canonicalSessionText(session));
  assert.deepEqual(replaySession(bytes), session);
  assert.deepEqual(readSession(file), session);

  assert.throws(() => replaySession(JSON.stringify(session)), (error) => error instanceof ReplayError && error.message === 'non_canonical_session');
});

test('R703-6: a proven false claim resumes the same native session with the parent evidence', async () => {
  const workspace = makeTemp('native-resume');
  const initial = fixtureConfig(fixture, 'agent', workspace, 'native_session', 'The release date is 2027.');
  const parent = await runAgent(initial);
  assert.equal(parent.native_session.id, 'ses_issue_703');
  assert.equal(parent.native_session.resume_command, 'agent --resume ses_issue_703');

  const request = {
    cli: 'agent',
    claim: 'The release date is 2027.',
    evidence: 'The signed release record says 2026.',
    source_session_sha256: sessionSha256(parent),
  };
  const resumed = fixtureConfig(fixture, 'agent', workspace, 'success', 'The corrected release date is 2026.');
  resumed.task = 'The corrected release date is 2026.';
  const corrected = await resumeAgent(parent, request, resumed);

  assert.deepEqual(corrected.continuation, {
    parent_session_sha256: request.source_session_sha256,
    native_session_id: 'ses_issue_703',
    disproved_claim: request.claim,
    evidence: request.evidence,
  });
  assert.equal(
    corrected.task,
    'Continue the requested correction in this exact session.\n'
      + 'Disproved claim: The release date is 2027.\n'
      + 'Evidence: The signed release record says 2026.\n'
      + 'Re-evaluate the claim against the evidence, correct the result, and state what changed.\n'
      + 'Complete this correction goal: The corrected release date is 2026.',
    'the task stays last so literal task parsers cannot consume correction evidence',
  );
  assert.deepEqual(corrected.native_session, parent.native_session, 'the parent session is carried on a client that reports none');
  assert.ok(corrected.events.some((event) => event.kind === 'native_session_resumed'));
  assert.deepEqual(replaySession(canonicalSessionText(corrected)), corrected);
});

test('R703-6: continuation fails closed on a changed digest, a changed client, a missing session or a fresh native session', async () => {
  const workspace = makeTemp('native-resume-mismatch');
  const initial = fixtureConfig(fixture, 'agent', workspace, 'native_session');
  initial.task = 'state the release date';
  const parent = await runAgent(initial);
  const request = {
    cli: 'agent',
    claim: 'The release date is 2027.',
    evidence: 'The signed release record says 2026.',
    source_session_sha256: sessionSha256(parent),
  };

  const fresh = fixtureConfig(fixture, 'agent', workspace, 'mismatched_native_session');
  assert.equal((await refusal(() => resumeAgent(parent, request, fresh))).message, 'continuation_mismatch');
  assert.equal((await refusal(() => resumeAgent(parent, { ...request, source_session_sha256: '0'.repeat(64) }, fresh))).message, 'continuation_mismatch');
  assert.equal((await refusal(() => resumeAgent(parent, { ...request, cli: 'codex' }, fresh))).message, 'continuation_mismatch');

  const plain = await runAgent(fixtureConfig(fixture, 'agent', workspace, 'success'));
  const plainRequest = { ...request, source_session_sha256: sessionSha256(plain) };
  assert.equal((await refusal(() => resumeAgent(plain, plainRequest, fresh))).message, 'native_session_unavailable');
});

test('R703-2: vendor entrypoints keep the editing and prompt arguments each client needs', async () => {
  const bin = makeTemp('vendor-cli-bin');
  for (const cli of REQUIRED_CLIS) executableScript(bin, cli, '#!/bin/sh\nprintf \'%s\\n\' "$*"\n');
  const expected = {
    agent: ['--model', 'formal-ai', '--permission-mode', 'auto', '--no-retry-on-rate-limits', '--output-format', 'stream-json', '-p', 'vendor task'],
    claude: ['--model', 'formal-ai', '--permission-mode', 'acceptEdits', '--output-format', 'stream-json', '--verbose', '--print', 'vendor task'],
    codex: ['-m', 'formal-ai', 'exec', '--skip-git-repo-check', '--sandbox', 'workspace-write', '--json', 'vendor task'],
    gemini: ['-m', 'formal-ai', '--skip-trust', '--approval-mode', 'auto_edit', '--output-format', 'stream-json', '-p', 'vendor task'],
    opencode: ['run', '-m', 'formal-ai', '--auto', '--format', 'json', 'vendor task'],
    qwen: ['--model', 'formal-ai', '--output-format', 'stream-json', '-p', 'vendor task'],
  };
  await withPath(bin, async () => {
    for (const cli of REQUIRED_CLIS) {
      const workspace = makeTemp(`vendor-${cli}`);
      const config = withPermission(agentRunConfig(cli, 'vendor task', workspace), grantFor(workspace));
      config.target = 'vendor';
      const session = await runAgent(config);
      assert.ok(sessionPassed(session), `${cli}: ${session.stderr}`);
      assert.deepEqual(session.args, expected[cli], cli);
      assert.equal(session.stdout, `${expected[cli].join(' ')}\n`, `${cli}: the client received exactly these arguments`);
    }
  });
});

test('R703-2: a Formal AI target launches the controller with the loopback, model, home and task arguments', async () => {
  const bin = makeTemp('controller-bin');
  const controller = executableScript(bin, 'formal-ai-controller', '#!/bin/sh\nprintf \'%s\\n\' "$@"\n');
  const workspace = makeTemp('controller');
  const config = withPermission(agentRunConfig('codex', 'add a README badge', workspace), grantFor(workspace));
  config.controller_program = controller;
  const session = await runAgent(config);
  assert.equal(session.program, controller);
  assert.deepEqual(session.args, [
    'with', '--orchestration', '--base-url', 'http://127.0.0.1:8080', '--model', 'formal-ai', '--non-interactive',
    '--orchestration-home', path.join(workspace, '.formal-ai-orchestration', 'native-sessions', 'codex'),
    'codex', 'add a README badge',
  ]);
  assert.equal(session.stdout.split('\n').at(-2), 'add a README badge');
});

test('R703-10: vendor orchestration forwards the task byte for byte in every supported language', async () => {
  const bin = makeTemp('languages-bin');
  executableScript(bin, 'codex', '#!/bin/sh\nprintf \'%s\\n\' "$@"\n');
  const cases = [
    ['en', 'add a README badge'],
    ['ru', 'добавь значок в README'],
    ['hi', 'README में बैज जोड़ें'],
    ['zh', '在 README 中添加徽章'],
  ];
  await withPath(bin, async () => {
    for (const [language, task] of cases) {
      const workspace = makeTemp(language);
      const file = path.join(workspace, 'session.json');
      const config = withPermission(agentRunConfig('codex', task, workspace), grantFor(workspace));
      config.target = 'vendor';
      writeSession(file, await runAgent(config));
      const session = readSession(file);
      assert.equal(session.task, task, `${language}: recorded task`);
      assert.equal(session.args.at(-1), task, `${language}: task forwarded to the client`);
      assert.equal(session.stdout.split('\n').filter(Boolean).at(-1), task, `${language}: the client received the task byte for byte`);
    }
  });
});

test('R703-5: the public CLI runs an explicitly granted Bash agent and refuses an ungranted one', () => {
  const workspace = makeTemp('custom-bash-cli');
  const file = path.join(workspace, 'bash-session.json');
  const argv = JSON.stringify(['sh', '-c', 'printf \'bash-agent:%s\\n\' "$1"', 'formal-ai-custom-agent', '{task}']);
  const base = ['run', '--cli', 'private-neural-agent', '--target', 'vendor', '--task', 'answer through bash', '--workspace', workspace, '--command', argv];

  const denied = runCli(base);
  assert.equal(denied.status, 1);
  assert.equal(denied.stderr, 'Error: AgentCommandNotAllowlisted("sh")\n');

  const allowed = runCli([...base, '--allow-agent-command', 'sh', '--session', file]);
  assert.equal(allowed.status, 0, allowed.stderr);
  const session = readSession(file);
  assert.equal(session.cli, 'private-neural-agent');
  assert.equal(session.stdout.trim(), 'bash-agent:answer through bash');
  assert.ok(session.events.some((event) => event.kind === 'custom_adapter_granted'));
  assert.equal(allowed.stdout, canonicalSessionText(session), 'stdout is the canonical pretty session');
});

test('R703-5: a custom entrypoint without a {task} placeholder is refused before it runs', () => {
  const workspace = makeTemp('custom-no-task');
  const file = path.join(workspace, 'session.json');
  const argv = JSON.stringify(['sh', '-c', 'printf ran > ran.txt']);
  const refused = runCli(['run', '--cli', 'private-neural-agent', '--target', 'vendor', '--task', 'answer through bash',
    '--workspace', workspace, '--command', argv, '--allow-agent-command', 'sh', '--session', file]);
  assert.equal(refused.status, 1);
  assert.equal(refused.stderr, 'Error: "missing_task_placeholder"\n');
  assert.equal(existsSync(file), false, 'a refused entrypoint records no session');
  assert.equal(existsSync(path.join(workspace, 'ran.txt')), false, 'a refused entrypoint never runs');
});

test('R703-6: the public CLI resumes the recorded vendor session with the correction evidence', () => {
  const bin = makeTemp('native-resume-cli-bin');
  executableScript(bin, 'agent', [
    '#!/bin/sh',
    'printf \'%s\\n\' \'{"type":"result","session_id":"ses_vendor_703","result":"vendor answer"}\'',
    'printf \'argv:%s\\n\' "$*"',
    '',
  ].join('\n'));
  const env = { PATH: `${bin}${path.delimiter}${process.env.PATH}` };
  const workspace = makeTemp('native-resume-cli');
  const parentFile = path.join(workspace, 'parent.json');
  const correctedFile = path.join(workspace, 'corrected.json');

  const initial = runCli(['run', '--cli', 'agent', '--target', 'vendor', '--task', 'state the release date', '--workspace', workspace, '--session', parentFile], env);
  assert.equal(initial.status, 0, initial.stderr);
  assert.equal(readSession(parentFile).native_session.id, 'ses_vendor_703');

  const resumed = runCli(['resume', '--parent', parentFile, '--task', 'correct the release date', '--workspace', workspace,
    '--disproved-claim', 'The release date is 2027.', '--evidence', 'The signed release record says 2026.', '--session', correctedFile], env);
  assert.equal(resumed.status, 0, `${resumed.stdout}${resumed.stderr}`);
  const corrected = readSession(correctedFile);
  const resume = corrected.args.indexOf('--resume');
  assert.deepEqual(corrected.args.slice(resume, resume + 2), ['--resume', 'ses_vendor_703']);
  assert.ok(corrected.args.includes('--no-fork'));
  assert.ok(corrected.task.includes('The release date is 2027.'));
  assert.ok(corrected.task.includes('The signed release record says 2026.'));
  assert.equal(corrected.continuation.native_session_id, 'ses_vendor_703');
});

test('R703-7: the public CLI replays a recorded session and rejects a tampered one', () => {
  const workspace = makeTemp('cli-replay');
  const file = path.join(workspace, 'session.json');
  const argv = JSON.stringify(['sh', '-c', 'printf done', 'x', '{task}']);
  assert.equal(runCli(['run', '--cli', 'private', '--target', 'vendor', '--task', 't', '--workspace', workspace, '--command', argv, '--allow-agent-command', 'sh', '--session', file]).status, 0);
  const replay = runCli(['replay', file]);
  assert.equal(replay.status, 0, replay.stderr);
  assert.equal(replay.stdout, readFileSync(file, 'utf8'));

  writeFileSync(file, readFileSync(file, 'utf8').replace('"stdout": "done"', '"stdout": "edited"'));
  assert.equal(runCli(['replay', file]).status, 0, 'free-text fields are bound by the parent digest, not the event chain');
  writeFileSync(file, readFileSync(file, 'utf8').replace('"status": "succeeded"', '"status": "failed"'));
  const tampered = runCli(['replay', file]);
  assert.equal(tampered.status, 1);
  assert.equal(tampered.stderr, 'Error: event_binding:status\n');
});
