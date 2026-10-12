// Issue #703 R703-4, R703-6, R703-11, R703-12 and the incremental case studies
// (R924-7, R933-13, R991-9) recomputed in JavaScript from the bytes committed
// under docs/case-studies/: every ledger, learning artefact, proposal document,
// split and composed-change set a Rust run wrote must come out of the JavaScript
// twins (js/agentic/crate/orchestration_*.mjs, recursive_execution.mjs,
// task_decomposition_tree.mjs, client_contract_learning.mjs) byte for byte.
//
// Rust twins, by name, in rust/tests/integration/issue_703_orchestration_followup.rs:
// formal_ai_authored_invariant_is_byte_pinned_to_its_session_evidence,
// cross_agent_correction_evidence_is_seeded_in_every_supported_language,
// real_formal_ai_controller_agent_run_replays_from_committed_bytes,
// real_formal_ai_correction_chain_resumes_one_native_session_and_pins_the_artifact,
// committed_parallel_comparison_ledger_records_the_selected_winner; and in
// rust/tests/unit/issue_933_self_authoring.rs the two incremental-dispatch tests.

import assert from 'node:assert/strict';
import { before, test } from 'node:test';

import { learnClientContracts, learningLinksNotation } from '../../../js/agentic/crate/client_contract_learning.mjs';
import { stableId } from '../../../js/agentic/crate/engine_stable_identifier.mjs';
import { pushLinoNode } from '../../../js/agentic/crate/links_format.mjs';
import { selectWinner } from '../../../js/agentic/crate/orchestration_dispatch.mjs';
import { observeOrchestrationSession } from '../../../js/agentic/crate/orchestration_analysis.mjs';
import { proposalLinksNotation } from '../../../js/agentic/crate/orchestration_incremental.mjs';
import { canonicalSessionText, replayContinuation, replaySession } from '../../../js/agentic/crate/orchestration_replay.mjs';
import { parseNativeSession, sessionDiffSize, sessionPassed, sessionSha256 } from '../../../js/agentic/crate/orchestration_runner.mjs';
import {
  attemptFailed, attemptPassed, isPassed, recursiveLeaf, solveRecursively, splitDepthReached,
} from '../../../js/agentic/crate/recursive_execution.mjs';
import { responseFor } from '../../../js/agentic/crate/seed.mjs';
import { clientIntegrations } from '../../../js/agentic/crate/seed_client_integrations.mjs';
import { decomposeTask, decompositionLinksNotation, leaves, splittingExecutor, subTaskLinksNotation } from '../../../js/agentic/crate/task_decomposition_tree.mjs';
import { bootHost, readCommitted } from './support/issue-0703-orchestration.mjs';

before(bootHost);

const AUTHORSHIP_SESSION = 'ses_050646852ffetdnQ73vR1yZ8la';
const CONTINUATION_SESSION = 'ses_04e25ba4cffeibfMekv188DNLX';
const CASE = 'docs/case-studies/issue-703/';

test('R703-12: the Formal AI authored invariant is byte pinned to its session evidence', () => {
  const repository = readCommitted('data/meta/orchestration-safety-invariant.lino');
  const captured = readCommitted(`${CASE}self-hosting-authorship/orchestration-safety-invariant.lino`);
  const evidence = readCommitted(`${CASE}self-hosting-authorship/agent-cli.log`);
  assert.equal(repository, captured);
  assert.ok(evidence.includes(AUTHORSHIP_SESSION));
  assert.ok(evidence.includes('"providerID": "formal-ai"'));
  assert.ok(evidence.includes('"tool": "write"'));
});

test('R703-6: the cross-agent correction evidence is seeded in every supported language', () => {
  assert.equal(responseFor('orchestration_cross_agent_denial', 'en'), 'Cross-agent denial: sources={sources}; probability={probability}');
  for (const language of ['en', 'ru', 'hi', 'zh']) {
    const template = responseFor('orchestration_cross_agent_denial', language);
    assert.ok(template.includes('{sources}') && template.includes('{probability}'), `${language}: ${template}`);
  }
});

test('R703-12: the real Formal AI controller agent run replays from its committed bytes', () => {
  const text = readCommitted(`${CASE}controller-agent-run/controller-session.json`);
  const session = replaySession(text);
  assert.equal(session.cli, 'agent');
  assert.ok(sessionPassed(session));
  assert.ok(session.stdout.includes('ses_05046b1c9ffe59CvG0N3QrrsV4'));
  assert.ok(session.stdout.includes('--no-retry-on-rate-limits'));
  assert.ok(session.stdout.includes('"name":"write"'));
  const change = session.changes.find((entry) => entry.path === 'controller-proof.lino');
  assert.equal(change.after_sha256, 'b70e7374ba7654d7d57a908ae0d1b7d40e7b1c1651d6bcc2dd80671d35777637');
  assert.equal(readCommitted(`${CASE}controller-agent-run/controller-proof.lino`), 'controller_proof and the phrase replayable workspace edit.');
});

test('R703-6: the real correction chain resumes one native session and pins the artifact', () => {
  const texts = ['controller-session', 'corrected-session', 'final-session'].map((name) => readCommitted(`${CASE}followup-authorship/${name}.json`));
  const [parent, corrected, finalSession] = texts.map(replaySession);

  assert.deepEqual(parent.changes, [], 'the first claim was disproved');
  for (const session of [parent, corrected, finalSession]) assert.equal(session.native_session.id, CONTINUATION_SESSION);
  assert.equal(corrected.continuation.parent_session_sha256, sessionSha256(parent));
  assert.equal(finalSession.continuation.parent_session_sha256, sessionSha256(corrected));
  replayContinuation(texts[0], texts[1]);
  replayContinuation(texts[1], texts[2]);

  const resume = finalSession.args.findIndex((arg, index) => arg === '--orchestration-resume' && finalSession.args[index + 1] === CONTINUATION_SESSION);
  assert.ok(resume >= 0, 'the Formal AI wrapper receives the recorded Agent session');
  assert.ok(resume < finalSession.args.indexOf('agent'), 'the resume flag precedes the controlled client');
  assert.ok(finalSession.stdout.includes(`--resume ${CONTINUATION_SESSION} --no-fork -p`));
  assert.equal(finalSession.native_session.resume_command, 'agent --resume ses_04e25ba4cffeibfMekv188DNLX --no-fork');

  assert.equal(readCommitted('data/meta/orchestration-continuation-invariant.lino'), 'orchestration_continuation resume_exact_native_id.');
  const change = finalSession.changes.find((entry) => entry.path === 'data/meta/orchestration-continuation-invariant.lino');
  assert.equal(change.after_sha256, '1a94da829c1803f18ea2bf0b16f93ec30c5edc1fa1c02936afb2383b467ec439');
});

test('R703-4: the committed parallel comparison ledger records the selected winner and is recomputed from its sessions', () => {
  const text = readCommitted(`${CASE}comparison/comparison-ledger.json`);
  const ledger = JSON.parse(text);
  assert.equal(ledger.schema, 'formal-ai-comparison-ledger-v1');
  assert.equal(ledger.selection_rule, 'pass,diff_size,wall_time,cli,session_file');
  assert.equal(ledger.entries.length, 2);
  assert.ok(ledger.entries.every((entry) => entry.passed));
  assert.equal(ledger.winner, 'codex');
  assert.equal(selectWinner(ledger.entries), ledger.winner);
  assert.equal(canonicalSessionText(ledger), text, 'the ledger is the canonical pretty rendering');

  const recomputed = ledger.entries.map((entry) => {
    const session = replaySession(readCommitted(`${CASE}comparison/${entry.session_file}`));
    assert.ok(sessionPassed(session));
    assert.equal(session.verification.length, 1);
    assert.equal(session.verification[0].program, 'test');
    return {
      cli: entry.cli, task: session.task, passed: sessionPassed(session), diff_size: sessionDiffSize(session),
      wall_time_ms: session.wall_time_ms, session_file: entry.session_file,
    };
  });
  assert.deepEqual(recomputed, ledger.entries);
});

/** The incremental case studies a Rust run captured: report, sessions, learning and proposals. */
const INCREMENTAL = [
  ['issue-924', 'docs/case-studies/issue-924/incremental-self-authorship/'],
  ['issue-933', 'docs/case-studies/issue-933/self-hosting-authorship/'],
];

for (const [label, directory] of INCREMENTAL) {
  const report = JSON.parse(readCommitted(`${directory}dispatch-report.json`));
  const trace = report.incremental;

  test(`R991-9: the ${label} dispatch report is the canonical shape its sessions and ledger recompute to`, () => {
    assert.equal(report.schema, 'formal-ai-dispatch-report-v1');
    assert.equal(report.mode, 'incremental');
    assert.equal(trace.schema, 'formal-ai-incremental-trace-v1');
    assert.deepEqual(report.tasks, trace.steps.map((step) => step.task));
    assert.equal(report.sessions.length, trace.steps.length);
    trace.steps.forEach((step, index) => {
      const session = report.sessions[index];
      assert.equal(canonicalSessionText(session), readCommitted(`${directory}${step.session_file}`), step.session_file);
      replaySession(canonicalSessionText(session));
      assert.deepEqual(report.ledger.entries[index], {
        cli: step.cli, task: session.task, passed: sessionPassed(session), diff_size: sessionDiffSize(session),
        wall_time_ms: session.wall_time_ms, session_file: step.session_file,
      });
      assert.equal(step.passed, sessionPassed(session));
    });
    assert.equal(report.ledger.winner, null);

    // The executor applies a passing attempt's effects in step order, the later
    // writer of a path replacing the earlier one.
    let composed = [];
    for (const session of report.sessions.filter(sessionPassed)) {
      for (const change of session.changes) composed = [...composed.filter((prior) => prior.path !== change.path), change];
    }
    assert.deepEqual(report.composed_changes, composed);
  });

  test(`R933-13: the ${label} failure-driven run is reproduced by the JavaScript controller and splitter`, async () => {
    const task = trace.steps[0].task;
    let cursor = 0;
    const replayed = {
      async attempt(node) {
        const step = trace.steps[cursor];
        cursor += 1;
        assert.equal(step.task_id, node.id, `step ${cursor - 1}: task identity`);
        assert.equal(step.task, node.goal, `step ${cursor - 1}: task text`);
        const evidence = `cli:${step.cli}`;
        return step.passed ? attemptPassed(evidence) : attemptFailed(evidence);
      },
      async extend_for() {
        return false;
      },
      async retry_after_children(node) {
        return this.attempt(node);
      },
    };
    const executor = splittingExecutor(replayed);
    const run = await solveRecursively(recursiveLeaf(stableId('dispatch_task', task), task), executor);

    assert.equal(cursor, trace.steps.length);
    assert.equal(isPassed(run), trace.solved);
    assert.equal(splitDepthReached(run), trace.split_depth_reached);
    assert.deepEqual(executor.splits.map((split) => [split.goal, split.split_depth, split.children]),
      trace.splits.map((split) => [split.task, split.split_depth, split.children]));

    // The same children come from the splitter on its own, with the committed identities.
    const decomposition = decomposeTask(task, 1);
    assert.deepEqual(decomposition.root.children.map((child) => child.text), trace.splits[0].children);
    assert.deepEqual(decomposition.root.children.map((child) => child.id), trace.steps.slice(1, 1 + trace.splits[0].children.length).map((step) => step.task_id));
    assert.deepEqual(leaves(decomposition).map((leaf) => leaf.text), trace.splits[0].children);
    const artifact = decompositionLinksNotation(decomposition);
    assert.ok(artifact.includes(`tree_digest "${stableId('task_decomposition_tree', subTaskLinksNotation(decomposition.root))}"`));
    for (const child of decomposition.root.children) assert.ok(artifact.includes(`\n${child.id}\n`), `${child.id} is a record of the reviewed artifact`);
  });

  test(`R703-11: the ${label} learning artefact and proposal document are recomputed byte for byte`, () => {
    const observations = trace.steps
      .map((step, index) => [step, report.sessions[index]])
      .filter(([step]) => step.cli !== 'composed-verifier')
      .map(([step, session]) => observeOrchestrationSession(session, step.session_file));
    assert.equal(learningLinksNotation(learnClientContracts(observations, clientIntegrations())), readCommitted(`${directory}learning.lino`));

    let document = pushLinoNode('', 0, 'incremental_proposals', null);
    for (const proposal of trace.proposals) document += proposalLinksNotation(proposal);
    assert.equal(document, readCommitted(`${directory}proposals.lino`));
  });
}

test('R703-2: the native session of every committed Agent CLI session is parsed from its own streams', () => {
  const integrations = clientIntegrations();
  const paths = [
    ...[0, 1, 2, 3].map((index) => `docs/case-studies/issue-924/incremental-self-authorship/sessions/00${index}-agent.json`),
    ...[0, 1, 2, 3, 4].map((index) => `docs/case-studies/issue-933/self-hosting-authorship/sessions/00${index}-agent.json`),
    `${CASE}followup-authorship/controller-session.json`,
    `${CASE}followup-authorship/corrected-session.json`,
    `${CASE}followup-authorship/final-session.json`,
  ];
  for (const file of paths) {
    const session = JSON.parse(readCommitted(file));
    const integration = integrations.find((entry) => entry.id === session.cli);
    assert.ok(session.native_session, file);
    assert.deepEqual(parseNativeSession(session.stderr, session.stdout, integration), session.native_session, file);
  }
});

test('R703-11: a failed committed session is not a passing one, and an empty stream carries no native session', () => {
  const failure = replaySession(readCommitted('docs/case-studies/issue-921/formal-ai-to-hive-mind/failure-session.json'));
  assert.equal(failure.exit_code, 23);
  assert.equal(sessionPassed(failure), false);
  assert.equal(parseNativeSession('', '', null), null);
  assert.equal(parseNativeSession('formal-ai: orchestration-session-json:not json\n', '{"a":', null), null);
  assert.deepEqual(parseNativeSession(' formal-ai: orchestration-session-json:{"id":"s1","resume_command":"c"}\n', '', null), { id: 's1', resume_command: 'c' });
  const codex = clientIntegrations().find((entry) => entry.id === 'codex');
  assert.deepEqual(parseNativeSession('', '{"type":"thread.started","thread_id":"t-7"}\n', codex), { id: 't-7', resume_command: 'codex exec resume t-7' });
  assert.deepEqual(parseNativeSession('', '{"a":{"b":[{"sessionId":"deep"}]}}', codex), { id: 'deep', resume_command: 'codex exec resume deep' });
});
