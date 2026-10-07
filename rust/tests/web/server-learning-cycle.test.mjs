// The proposal-only learning cycle the JavaScript server's idle dreaming
// records (js/server/learning-cycle.mjs), against
// rust/tests/unit/issue_701_learning_adoption.rs. Both runtimes render the
// fixture frontier to the same committed record,
// rust/tests/fixtures/learning-cycle/record.lino.

import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  GOOGLE_TRENDS_FRONTIER, GOOGLE_TRENDS_FRONTIER_RECORD_FILE, LEARNED_REQUEST_OPENERS_SEED_FILE, googleTrendsLearningCycle,
  learningCycleLinksNotation, parseFrontierRecord, renderPromotionProposals, runLearningCycle, validated,
} from '../../../js/server/learning-cycle.mjs';
import { readRepoFile } from '../../../js/server/lino.mjs';

const FIXTURES = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'fixtures', 'learning-cycle');

test('a fixture frontier renders the exact learning-cycle record both runtimes commit to', () => {
  const items = parseFrontierRecord(fs.readFileSync(path.join(FIXTURES, 'frontier.lino'), 'utf8'));
  assert.equal(items.length, 12);
  assert.deepEqual(items[8], {
    rank: 9, query: 'Марс', language: 'ru', variation: 'tell_me_about', prompt: 'Расскажи про Марс', engine_intent: 'unknown',
  });
  const run = runLearningCycle('fixture', items);
  assert.equal(`${learningCycleLinksNotation(run)}\n`, fs.readFileSync(path.join(FIXTURES, 'record.lino'), 'utf8'));
});

test('the Google Trends cycle emits promotion proposals in the issue-656 shape', () => {
  const run = googleTrendsLearningCycle();
  assert.equal(run.frontier, GOOGLE_TRENDS_FRONTIER);
  assert.ok(run.proposals.length > 0, 'the cycle must propose something');
  assert.ok(run.candidates.some((candidate) => candidate.held_out.length > 0));
  for (const candidate of run.candidates.filter(validated)) assert.ok(candidate.held_out.every((entry) => entry.passed));
  for (const proposal of run.proposals) {
    assert.ok(proposal.source.startsWith('learning_frontier:google-trends:'));
    assert.equal(proposal.seed_file, LEARNED_REQUEST_OPENERS_SEED_FILE);
    assert.notEqual(proposal.seed_lino, '');
  }
  const rendered = renderPromotionProposals(run.proposals);
  assert.equal(rendered.split('\n').filter((line) => line === '  proposal').length, run.proposals.length);
});

test('the cycle is deterministic and reproducible offline', () => {
  const first = learningCycleLinksNotation(googleTrendsLearningCycle());
  assert.equal(first, learningCycleLinksNotation(googleTrendsLearningCycle()));
  assert.ok(first.includes('mode "proposal_only"'));
  assert.ok(first.includes('human_gated "true"'));
});

test('the frozen frontier record spans every supported language, all unrouted', () => {
  const items = parseFrontierRecord(readRepoFile(GOOGLE_TRENDS_FRONTIER_RECORD_FILE));
  for (const language of ['en', 'ru', 'hi', 'zh']) assert.ok(items.some((item) => item.language === language), language);
  assert.ok(items.every((item) => item.engine_intent === 'unknown'));
});

test('every failure to adopt is preserved as a durable record', () => {
  const run = googleTrendsLearningCycle();
  const record = learningCycleLinksNotation(run);
  assert.equal(run.blocked.filter((entry) => entry.reason === 'held_out_validation_failed').length,
    run.candidates.length - run.candidates.filter(validated).length);
  for (const blocked of run.blocked) {
    assert.notEqual(blocked.reason, '');
    assert.ok(record.includes(blocked.reason));
  }
});
