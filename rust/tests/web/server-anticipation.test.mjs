// Anticipatory dreaming in the JavaScript server (js/server/anticipation.mjs,
// js/server/anticipation-expansion.mjs), case for case against
// rust/tests/unit/issue_705_anticipation.rs over the same scripted history.
//
// The Rust cases classify every probe with the offline solver. The
// JavaScript worker solves a text-transformation probe in about 30 s, so the
// scripted-history cases take their classifier as a fixture: the intents the
// history recorded, the real solver for prompts that carry a digit (the
// arithmetic class, which it answers at once), and `unknown` otherwise. The
// last cases run the real solver end to end over a history whose probes are
// all arithmetic.

import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';

import { normalizePrompt } from '../../../js/agentic/crate/engine.mjs';
import {
  ANTICIPATION_FRONTIER, anticipationConfig, anticipationLedgerLinksNotation, anticipationLedgerPath,
  anticipationPlanLinksNotation, applyAnticipation, planAnticipation, predictionHitEvent, prelearnPredictions,
  whyPrediction,
} from '../../../js/server/anticipation.mjs';
import { solveIntent } from '../../../js/server/dreaming-replay.mjs';
import { learningCycleLinksNotation } from '../../../js/server/learning-cycle.mjs';
import { learningCycleRecordPath, runCoreDreamingOnce } from '../../../js/server/dreaming-runtime.mjs';
import { SyncStore, exportLinksNotation, memoryEvent, parseLinksNotation } from '../../../js/server/memory-store.mjs';
import { answerFromPrelearnedCache } from '../../../js/server/standing-requirements.mjs';

const FIXED_TIME = 2000000000;

/** rust/tests/unit/issue_705_anticipation.rs `request`. */
function request(id, prompt, intent) {
  return memoryEvent({
    id, kind: 'message', role: 'user', intent, content: prompt, sent_at: `2026-08-01T00:00:${id}Z`, write_count: 1,
  });
}

/** `scripted_requests`: the last class is greeting; it was followed by three distinct classes. */
const scriptedRequests = () => [
  request('01', 'hello', 'greeting'),
  request('02', '2 + 2', 'calculation'),
  request('03', 'hello again', 'greeting'),
  request('04', 'reverse the words alpha beta', 'text_transformation'),
  request('05', 'hello once more', 'greeting'),
  request('06', 'describe frobulator705 resonance', 'unknown'),
  request('07', 'hello finally', 'greeting'),
];

const recorded = new Map(scriptedRequests().map((event) => [event.content, event.intent]));
const solved = new Map();
function fixtureClassify(prompt) {
  if (recorded.has(prompt)) return recorded.get(prompt);
  if (!/\d/u.test(prompt.replace(/frobulator705/gu, ''))) return 'unknown';
  if (!solved.has(prompt)) solved.set(prompt, solveIntent(prompt));
  return solved.get(prompt);
}

const plan = (events = scriptedRequests()) => planAnticipation(events, anticipationConfig(), fixtureClassify);

/** The `FixtureTransport` answers, as `execute_source_research` returns them. */
function fixtureResearch(calls) {
  return () => {
    calls.count += 1;
    const url = 'https://result.invalid/frobulator';
    const text = 'A frobulator705 is a deterministic anticipation fixture.';
    const capture = {
      source_url: url, fetched_at: String(FIXED_TIME), cached: false,
      sha256: createHash('sha256').update(`${text}\n`).digest('hex'),
    };
    return { search: { fused: [{ url, title: text, excerpt: text }], captures: [capture] }, pages: [{ ranking: { url }, capture }] };
  };
}

test('append-only intent transitions predict three next request classes', () => {
  const anticipated = plan();
  assert.equal(anticipated.current_class.id, 'intent:greeting');
  assert.deepEqual(anticipated.predictions.map((prediction) => prediction.class.id),
    ['intent:calculation', 'intent:text_transformation', 'intent:unknown']);
  for (const transition of anticipated.transitions) {
    assert.equal(transition.evidence.model, 'markov_transition');
    assert.equal(transition.evidence.transition_from, transition.from.id);
    assert.ok(transition.evidence_links.every((link) => link.startsWith('memory:')));
    assert.ok(!transition.to.id.includes(' '), 'a state is not raw prose');
  }
  const first = anticipated.predictions[0];
  assert.equal(whyPrediction(anticipated, first.id),
    `prediction=${first.id} class=intent:calculation transition_evidence=${first.transition_evidence_id} count=1 probability=0.333333`);
});

test('class expansion uses meaning, operation and parameter evidence', () => {
  const anticipated = plan();
  const variants = anticipated.predictions.flatMap((prediction) => prediction.variants);
  const sources = variants.map((variant) => variant.source);
  assert.ok(sources.some((source) => source.startsWith('meaning:')), sources.join(', '));
  assert.ok(sources.some((source) => source.startsWith('operation:')), sources.join(', '));
  assert.ok(sources.some((source) => source.startsWith('parameter:')), sources.join(', '));
  assert.ok(variants.some((variant) => variant.source === 'parameter:02' && variant.prompt === '2 + 2'));
  assert.ok(anticipated.probes.some((probe) => probe.prompt === '2 + 2' && probe.status === 'passed'));
});

test('every unknown or failed offline probe reaches the adoption frontier', () => {
  const anticipated = plan();
  const failures = anticipated.probes.filter((probe) => probe.status !== 'passed');
  assert.ok(failures.length > 0);
  assert.equal(anticipated.frontier.length, failures.length);
  assert.equal(anticipated.learning_cycle.frontier, ANTICIPATION_FRONTIER);
  assert.equal(anticipated.learning_cycle.frontier_items, failures.length);
  for (const failure of failures) assert.ok(anticipated.frontier.some((item) => item.prompt === failure.prompt));
  const cycle = learningCycleLinksNotation(anticipated.learning_cycle);
  assert.ok(cycle.includes('mode "proposal_only"'));
  assert.ok(cycle.includes('human_gated "true"'));
  assert.ok(anticipated.learning_cycle.proposals.every((proposal) => proposal.source.startsWith('learning_frontier:anticipation:')));
});

test('source prelearning is consent gated and keeps cache provenance and TTL', () => {
  const anticipated = plan();
  const calls = { count: 0 };
  const denied = prelearnPredictions(anticipated, 'denied', anticipationConfig(), fixtureResearch(calls));
  assert.equal(calls.count, 0, 'denial must perform no fetch');
  assert.ok(denied.attempts.length > 0);
  assert.ok(denied.attempts.every((attempt) => attempt.status === 'consent_required' && attempt.diagnostic === 'fetch_consent_required'));

  const granted = prelearnPredictions(anticipated, 'granted', anticipationConfig(), fixtureResearch(calls));
  assert.ok(granted.sources.length > 0);
  assert.ok(calls.count > 0);
  for (const source of granted.sources) {
    assert.equal(source.fetched_at, String(FIXED_TIME));
    assert.equal(source.expires_at, FIXED_TIME + 3600);
    assert.equal(source.sha256.length, 64);
    assert.ok(source.result_url.startsWith('https://result.invalid/'));
  }
});

test('prediction hits link later actual requests and a zero percent hit rate is honest', () => {
  const anticipated = plan();
  const denied = { attempts: [], sources: [] };
  let { events } = applyAnticipation([], anticipated, denied);
  const before = anticipationLedgerLinksNotation(anticipated, denied, events);
  assert.ok(before.includes('prediction_hits "0"'));
  assert.ok(before.includes('hit_rate_basis_points "0"'));

  const hit = predictionHitEvent(events, '2 + 2', 'actual-request', fixtureClassify);
  assert.equal(hit.kind, 'prediction_hit');
  assert.ok(hit.evidence.includes('actual-request'));
  assert.ok(hit.evidence.some((link) => link.startsWith('anticipation_prediction:')));
  events = [...events, hit];
  const after = anticipationLedgerLinksNotation(anticipated, denied, events);
  assert.ok(after.includes('prediction_hits "1"'));
  assert.ok(after.includes('hit_rate_basis_points "3333"'));
});

test('the same history produces byte-identical predictions and ledger', () => {
  const left = plan();
  const right = plan();
  assert.equal(anticipationPlanLinksNotation(left), anticipationPlanLinksNotation(right));
  const empty = { attempts: [], sources: [] };
  assert.equal(anticipationLedgerLinksNotation(left, empty, []), anticipationLedgerLinksNotation(right, empty, []));
});

test('changed transition evidence appends a new prediction revision', () => {
  const history = scriptedRequests();
  const first = plan(history);
  const firstCalculation = first.predictions.find((prediction) => prediction.class.id === 'intent:calculation');
  assert.equal(firstCalculation.count, 1);
  let { events } = applyAnticipation(history, first, { attempts: [], sources: [] });
  events = [...events, request('08', '10 + 1', 'calculation'), request('09', 'hello after arithmetic', 'greeting')];
  const updated = plan(events);
  const updatedCalculation = updated.predictions.find((prediction) => prediction.class.id === 'intent:calculation');
  assert.equal(updatedCalculation.count, 2);
  assert.notEqual(updatedCalculation.id, firstCalculation.id, 'append-only predictions version changed evidence');
  const applied = applyAnticipation(events, updated, { attempts: [], sources: [] });
  assert.ok(applied.outcome.prediction_records > 0);
  assert.ok(applied.events.some((event) => event.id === firstCalculation.id));
  assert.ok(applied.events.some((event) => event.id === updatedCalculation.id));
});

test('a held-out predicted prompt becomes answerable offline after prelearning', () => {
  const anticipated = plan();
  const heldOut = anticipated.predictions.flatMap((prediction) => prediction.variants)
    .filter((variant) => variant.source.startsWith('meaning:') && variant.prompt.includes('frobulator705'))
    .find((variant) => fixtureClassify(variant.prompt) === 'unknown').prompt;
  const calls = { count: 0 };
  const prelearning = prelearnPredictions(anticipated, 'granted', anticipationConfig(), fixtureResearch(calls));
  const { events, outcome } = applyAnticipation([], anticipated, prelearning);
  assert.ok(outcome.prelearned_aliases > 0);
  const recalled = answerFromPrelearnedCache(normalizePrompt, heldOut, events, FIXED_TIME + 1);
  assert.equal(recalled.intent, 'anticipation_cache');
  assert.equal(recalled.answer, 'A frobulator705 is a deterministic anticipation fixture.');
  assert.ok(recalled.evidence_links.some((link) => link.startsWith('source:http:')));
  assert.equal(answerFromPrelearnedCache(normalizePrompt, heldOut, events, FIXED_TIME + 3601), null, 'expired prelearning is not recalled');
});

test('the real solver: repeated actual requests do not inflate the class hit rate', () => {
  const history = [request('01', 'hello', 'greeting'), request('02', '2 + 2', 'calculation'), request('03', 'hello again', 'greeting')];
  const anticipated = planAnticipation(history);
  assert.equal(anticipated.predictions.length, 1);
  let { events } = applyAnticipation(history, anticipated, { attempts: [], sources: [] });
  for (const actual of ['actual-one', 'actual-two']) events = [...events, predictionHitEvent(events, '3 + 3', actual)];
  const ledger = anticipationLedgerLinksNotation(anticipated, { attempts: [], sources: [] }, events);
  assert.ok(ledger.includes('prediction_hits "2"'));
  assert.ok(ledger.includes('predicted_classes_hit "1"'));
  assert.ok(ledger.includes('hit_rate_basis_points "10000"'));
});

test('idle dreaming persists the ledger and the learning-cycle record; a later live request records a hit', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-anticipation-idle-'));
  const file = path.join(dir, 'memory.lino');
  fs.writeFileSync(file, exportLinksNotation(scriptedRequests(), 2));
  runCoreDreamingOnce(file, { env: { ...process.env, FORMAL_AI_LIVE_API: '0' }, classify: fixtureClassify });

  const ledger = fs.readFileSync(anticipationLedgerPath(file), 'utf8');
  assert.equal(anticipationLedgerPath(file), path.join(dir, 'memory.anticipation.lino'));
  assert.ok(ledger.includes('predictions "3"'), ledger);
  assert.ok(ledger.includes('prediction_hits "0"'), ledger);
  assert.ok(ledger.includes('mode "proposal_only"'), ledger);
  assert.ok(ledger.includes('status "consent_required"'), ledger);
  const record = fs.readFileSync(learningCycleRecordPath(file), 'utf8');
  assert.ok(record.includes('mode "proposal_only"') && !record.includes('proposals "0"'));
  assert.equal(parseLinksNotation(fs.readFileSync(file, 'utf8')).filter((event) => event.kind === 'anticipation_prediction').length, 3);

  const store = SyncStore.open({ ...process.env, FORMAL_AI_MEMORY_PATH: file, FORMAL_AI_RECORD_CHAT: '1' }, file);
  store.classify = fixtureClassify;
  store.recordChatExchangeWithTools('2 + 2', '4');
  const hit = store.events.find((event) => event.kind === 'prediction_hit');
  assert.ok(hit.evidence.some((link) => link.startsWith('anticipation_prediction:')));
  assert.ok(hit.evidence.some((link) => link.startsWith('chat_user_')));
  fs.rmSync(dir, { recursive: true, force: true });
});
