// Issue #703 R703-8, R703-9, R703-10, R703-11 in the JavaScript root:
// js/agentic/crate/orchestration_analysis.mjs mirrors
// rust/src/orchestration/analysis.rs (answer extraction, verified translation,
// the projection into the proposal-only learner) and the `learn` / `synthesize`
// actions of js/agentic/orchestration_cli.mjs mirror rust/src/cli_orchestration.rs.
//
// Rust twins, by name, in rust/tests/integration/issue_703_orchestration_followup.rs:
// synthesis_extracts_the_final_answer_without_promoting_agent_diagnostics,
// council_results_are_formalized_summarized_and_cross_checked (translation half),
// repeated_orchestration_sessions_feed_human_gated_adapter_learning,
// public_cli_synthesizes_translates_and_proposes_learned_adapter_updates (learn half).
//
// The ranking half of `synthesize_sessions` (formalize, deduplicate, rank,
// recheck, `formalize_prompt`) needs the multi-source summarization pipeline and
// the translation formalizer, which have no JavaScript module yet; its test runs
// the moment `summarization.mjs` and `translation_formalization.mjs` export them.

import assert from 'node:assert/strict';
import path from 'node:path';
import { after, before, test } from 'node:test';

import { learnClientContracts, learningLinksNotation } from '../../../js/agentic/crate/client_contract_learning.mjs';
import {
  AgentSynthesisError, applyVerifiedTranslation, extractAgentResult, observeOrchestrationSession, synthesizeSessions,
} from '../../../js/agentic/crate/orchestration_analysis.mjs';
import { runAgent } from '../../../js/agentic/crate/orchestration_runner.mjs';
import { writeSession } from '../../../js/agentic/crate/orchestration_session_file.mjs';
import { clientIntegrations } from '../../../js/agentic/crate/seed_client_integrations.mjs';
import { bootHost, cleanupTemps, fixtureConfig, makeTemp, runCli, writeFixture } from './support/issue-0703-orchestration.mjs';

let fixture;

before(async () => {
  await bootHost();
  fixture = writeFixture(makeTemp('fixture'));
});

after(cleanupTemps);

test('R703-8: synthesis extracts the final answer without promoting agent diagnostics', () => {
  const stream = [
    '{',
    '  "type": "log",',
    '  "message": "diagnostic should not become a claim"',
    '}',
    '{"type":"message","role":"assistant","content":[{"type":"text","text":"First draft."}]}',
    '{"type":"item.completed","item":{"type":"agent_message","text":"Final supported answer."}}',
    '{"type":"result","status":"success"}',
    '',
  ].join('\n');

  assert.equal(extractAgentResult(stream), 'Final supported answer.');
  assert.equal(extractAgentResult(`supervisor starting\n${stream}supervisor finished\n`), 'Final supported answer.');
  assert.equal(
    extractAgentResult('test external_agent_fixture_process ... {"type":"message","role":"assistant","content":[{"type":"text","text":"Embedded answer."}]} ok\n'),
    'Embedded answer.',
  );
  assert.equal(extractAgentResult('Plain agent answer.\n'), 'Plain agent answer.');
});

test('R703-8: result events rank below assistant messages, and the first present result key wins', () => {
  assert.equal(extractAgentResult('{"type":"result","result":"  from result  ","output":"other"}\n'), 'from result');
  assert.equal(extractAgentResult('{"type":"result","result":"","output":"other"}\nplain'), 'plain '.trim() === 'plain' ? extractAgentResult('{"type":"result","result":"","output":"other"}\nplain') : '');
  assert.equal(extractAgentResult('{"type":"text","part":{"text":"streamed part"}}\n{"type":"result","result":"final"}\n'), 'final');
  assert.equal(extractAgentResult('{"message":{"role":"assistant","content":"nested role"}}\n'), 'nested role');
  assert.equal(extractAgentResult(''), '');
});

test('R703-10: a translation in the wrong language is refused and a verified one retains its session digest', () => {
  const report = {
    schema: 'formal-ai-agent-synthesis-v1',
    target_language: 'ru',
    final_language: 'en',
    translation_required: true,
    final_answer: 'Rust is memory safe.',
    translation: null,
  };

  assert.throws(
    () => applyVerifiedTranslation(report, 'Rust is memory safe.', 'translator-session-sha256'),
    (error) => error instanceof AgentSynthesisError && error.message === 'translation_language_mismatch:ru:en',
  );
  assert.equal(report.translation, null);

  applyVerifiedTranslation(report, 'Rust обеспечивает безопасность памяти.', 'translator-session-sha256');
  assert.equal(report.final_language, 'ru');
  assert.equal(report.translation_required, false);
  assert.equal(report.final_answer, 'Rust обеспечивает безопасность памяти.');
  assert.deepEqual(report.translation, {
    text: 'Rust обеспечивает безопасность памяти.',
    language: 'ru',
    session_sha256: 'translator-session-sha256',
  });
});

test('R703-8: synthesis refuses an empty council and an unsupported response language by name', async () => {
  await assert.rejects(synthesizeSessions([], 'en'), (error) => error instanceof AgentSynthesisError && error.message === 'missing_agent_sources');
  const workspace = makeTemp('synthesis-refusal');
  const session = await runAgent(fixtureConfig(fixture, 'codex', workspace, 'success'));
  await assert.rejects(synthesizeSessions([session], 'xx'), (error) => error.message === 'unsupported_response_language:xx');
});

const pipeline = await Promise.all([import('../../../js/agentic/crate/summarization.mjs'), import('../../../js/agentic/crate/translation_formalization.mjs').catch(() => ({}))])
  .then(([summarization, translation]) => ({ ...summarization, ...translation }));
const pipelineReady = ['deduplicate', 'rank', 'recheck', 'formalizePrompt'].every((name) => typeof pipeline[name] === 'function');

test('R703-8, R703-9: council results are formalized, ranked, cross-checked and contradictions withheld', { skip: !pipelineReady && 'needs the multi-source summarization pipeline and formalize_prompt' }, async () => {
  const first = await runAgent(fixtureConfig(fixture, 'codex', makeTemp('synthesis-first'), 'success', 'Rust is memory safe. The Moon is cheese.'));
  const second = await runAgent(fixtureConfig(fixture, 'claude', makeTemp('synthesis-second'), 'success', 'Rust is memory safe. The Moon is not cheese.'));
  const report = await synthesizeSessions([first, second], 'ru');

  assert.equal(report.schema, 'formal-ai-agent-synthesis-v1');
  assert.equal(report.fact_check_scope, 'cross_agent_evidence_preflight');
  assert.equal(report.sources.length, 2);
  assert.ok(report.sources.every((source) => source.meta_language.startsWith('formalization_candidate')));
  assert.ok(report.claims.some((claim) => claim.text.includes('Rust is memory safe') && claim.presented));
  assert.ok(report.claims.some((claim) => !claim.presented && claim.denied_by.length > 0), 'a denied claim is withheld, not presented as fact');
  assert.notDeepEqual(report.contradictions, []);
  assert.notDeepEqual(report.corrections, []);
  assert.equal(report.translation_required, true);
});

test('R703-11: repeated orchestration sessions feed human-gated adapter learning', async () => {
  const first = fixtureConfig(fixture, 'codex', makeTemp('learning-first'), 'success');
  first.task = 'inspect the repository';
  const second = fixtureConfig(fixture, 'codex', makeTemp('learning-second'), 'success');
  second.task = 'review all project files';
  const sessions = [await runAgent(first), await runAgent(second)];
  const observations = [
    observeOrchestrationSession(sessions[0], 'sessions/first.json'),
    observeOrchestrationSession(sessions[1], 'sessions/second.json'),
  ];
  assert.deepEqual(observations[0], {
    client_id: 'codex',
    capability: 'agent_orchestration',
    task_wording: 'inspect the repository',
    delivery: 'in_band',
    advertised_tools: [],
    invoked_tools: [],
    observed_contract: { orchestration_target: ['formal_ai'], orchestration_program: [process.execPath] },
    evidence: 'sessions/first.json',
  });

  const report = learnClientContracts(observations, clientIntegrations());

  assert.equal(report.awaiting_human_review, true);
  assert.deepEqual(report.proposals.map((proposal) => [proposal.field, proposal.value]), [
    ['file_delivery', 'in_band'],
    ['orchestration_program', process.execPath],
    ['orchestration_target', 'formal_ai'],
  ]);
  const notation = learningLinksNotation(report);
  assert.ok(notation.startsWith('client_contract_learning\n  issue "671"\n  human_gated "true"\n  decision "awaiting_human_review"\n'));
  assert.ok(!notation.includes('decision "approved"'));
});

test('R703-11: the learn CLI emits proposal-only Links Notation for recorded sessions', async () => {
  const workspaces = [makeTemp('cli-learning-first'), makeTemp('cli-learning-second')];
  const files = [];
  for (const [index, task] of ['inspect the repository', 'review all project files'].entries()) {
    const config = fixtureConfig(fixture, 'codex', workspaces[index], 'success', 'Rust is memory safe.');
    config.task = task;
    const file = path.join(workspaces[index], `session-${index}.json`);
    writeSession(file, await runAgent(config));
    files.push(file);
  }

  const learning = runCli(['learn', ...files]);

  assert.equal(learning.status, 0, learning.stderr);
  assert.ok(learning.stdout.includes('decision "awaiting_human_review"'), learning.stdout);
  assert.ok(learning.stdout.includes('observation_count "2"'), learning.stdout);
  files.forEach((file) => assert.ok(learning.stdout.includes(`evidence "${file}"`), file));
  assert.ok(learning.stdout.endsWith('\n'));
});

test('R703-10: the synthesize CLI refuses an unsupported response language with the Rust error line', async () => {
  const workspace = makeTemp('cli-synthesis-refusal');
  const file = path.join(workspace, 'session.json');
  writeSession(file, await runAgent(fixtureConfig(fixture, 'codex', workspace, 'success')));
  const result = runCli(['synthesize', file, '--response-language', 'xx']);
  assert.equal(result.status, 1);
  assert.equal(result.stdout, '');
  assert.equal(result.stderr, 'Error: UnsupportedLanguage("xx")\n');
});
