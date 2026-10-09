// R1188-U19 (extends R526-1): the best translation is the one that survives
// the round trip source -> meta -> target -> meta -> source, for sentences
// and texts, across every ordered pair of en, ru, hi, zh and es. Pins
// js/agentic/crate/round_trip_translation.mjs on a few exact cases, checks the
// sentence corpus, and holds the measured per-pair survival to its ratchet
// (data/meta/round-trip-translation-ratchet.lino). The Rust twin is
// rust/tests/unit/issue_1188_round_trip_translation.rs.

import assert from 'node:assert/strict';
import { before, describe, test } from 'node:test';

import {
  LANGUAGES, compareRatchet, installTextHost, readRatchet,
} from '../../../scripts/lib/text-capability-measures.mjs';

let text;
let roundTrip;

before(async () => {
  installTextHost();
  text = await import('../../../js/agentic/crate/text_formalization.mjs');
  roundTrip = await import('../../../js/agentic/crate/round_trip_translation.mjs');
});

describe('choosing the surface that survives the round trip', () => {
  test('the first listed surface loses to the one whose meaning comes back', () => {
    const [statement] = text.formalizeText('fix', 'en');
    const candidates = text.surfacesIn(statement.subject.id, 'ru');
    assert.equal(candidates[0], 'добавь');
    assert.notEqual(text.resolveSurface(candidates[0], 'ru'), statement.subject.id);
    const best = roundTrip.bestSurface(statement.subject, 'en', 'ru');
    assert.notEqual(best, candidates[0]);
    assert.ok(candidates.includes(best), best);
    assert.equal(text.resolveSurface(best, 'ru'), statement.subject.id);
  });

  test('the round trip chooses among offered surfaces', () => {
    assert.equal(roundTrip.roundTripChoice('fix', 'en', 'ru', ['добавь', 'исправить']), 1);
    assert.equal(roundTrip.roundTripChoice('fix', 'en', 'ru', ['добавь']), null);
    assert.equal(roundTrip.roundTripChoice('blarg', 'en', 'ru', ['бларг']), null);
    assert.equal(roundTrip.roundTripChoice('hello', 'en', 'ru', ['привет', 'здравствуйте']), 0);
  });

  test('a word crosses to every language and comes back as itself', () => {
    const expected = { ru: 'яблоко', hi: 'सेब', zh: '苹果', es: 'manzana' };
    for (const [target, surface] of Object.entries(expected)) {
      const trip = roundTrip.roundTrip('apple', 'en', target);
      assert.deepEqual([trip.forward, trip.backward, trip.survives], [surface, 'apple', true], target);
    }
  });

  test('a sentence crosses statement by statement and keeps its meanings', () => {
    const trip = roundTrip.roundTrip('La manzana es una fruta.', 'es', 'en');
    assert.equal(trip.forward, 'apple instance of fruit');
    assert.equal(trip.backward, 'manzana es una fruta');
    assert.deepEqual([trip.survives, trip.survivingTerms, trip.knownTerms], [true, 3, 3]);
    assert.equal(roundTrip.translateText('The apple is a fruit.', 'en', 'zh'), '苹果是水果');
  });

  test('an unknown word stays in its source form and the sentence does not count as surviving', () => {
    const trip = roundTrip.roundTrip('The apple is red.', 'en', 'ru');
    assert.equal(trip.forward, 'яблоко red');
    assert.equal(trip.survives, false);
    assert.deepEqual([trip.survivingTerms, trip.knownTerms], [1, 1]);
  });
});

describe('the sentence corpus and its ratchet', () => {
  test('a few hundred sentences, the same number in every language', async () => {
    const { readCorpus } = await import('../../../scripts/measure-round-trip-translation.mjs');
    const corpus = readCorpus();
    assert.ok(corpus.length >= 300, `${corpus.length} sentences`);
    for (const language of LANGUAGES) {
      assert.equal(corpus.filter((sentence) => sentence.language === language).length, corpus.length / LANGUAGES.length, language);
    }
    assert.ok(corpus.some((sentence) => sentence.origin.startsWith('seed-response ')));
    assert.ok(corpus.some((sentence) => sentence.origin.startsWith('page https://')));
  });

  test('all twenty ordered pairs are measured and none falls below its ratchet', async () => {
    const { RATCHET_FILE, measureRoundTripTranslation } = await import('../../../scripts/measure-round-trip-translation.mjs');
    const { measures } = await measureRoundTripTranslation();
    const pairs = measures.filter(([name]) => name.endsWith('-sentence-survival') && name !== 'sentence-survival');
    assert.equal(pairs.length, 20);
    const recorded = readRatchet(RATCHET_FILE);
    assert.ok(recorded.size > 0, RATCHET_FILE);
    assert.deepEqual(compareRatchet(measures, recorded).falls, []);
  });
});

// General ambiguity: the report action and plural report artifact keep distinct
// meaning identities when a new noun owns their shared Chinese surface.
test('reporting actions and plural artifacts each retain a reversible Chinese surface', () => {
  assert.equal(text.resolveSurface('上报', 'zh'), 'report_issue_action');
  assert.equal(text.resolveSurface('报告', 'zh'), 'reports');
  for (const [surface, language, forward, backward] of [
    ['report', 'en', '上报', 'report'],
    ['сообщить', 'ru', '上报', 'сообщи'],
    ['रिपोर्ट', 'hi', '上报', 'रिपोर्ट'],
    ['reports', 'en', '报告', 'reports'],
    ['submit', 'en', '提交', 'submit'],
  ]) {
    const trip = roundTrip.roundTrip(surface, language, 'zh');
    assert.deepEqual([trip.forward, trip.backward, trip.survives, trip.survivingTerms, trip.knownTerms],
      [forward, backward, true, 1, 1], surface);
  }
});
