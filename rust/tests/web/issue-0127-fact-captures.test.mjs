// Issue #127 (R174): the nine pre-warmed countries (Russia, Japan, France,
// Germany, China, India, United States, United Kingdom, Brazil) carry
// structured `(relation, subject_qid, value_qid, subject_label, value_label)`
// triples with labels in every supported language. Since issue #1172 R9 the
// triples are not written into data/seed/facts.lino: each is derived from the
// committed Wikidata captures in data/seed/fact-captures.lino, where the
// country's claim for the property that grounds the `capital` relation
// (data/seed/meanings-facts.lino) names a value item captured beside it.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { REPO_ROOT } from './support/browser-runtime.mjs';

const read = (relative) => readFileSync(join(REPO_ROOT, relative), 'utf8');
const CAPTURES = read('data/seed/fact-captures.lino');
const COUNTRY_QIDS = ['Q17', 'Q159', 'Q142', 'Q183', 'Q148', 'Q668', 'Q30', 'Q145', 'Q155'];
const LANGUAGES = ['en', 'ru', 'hi', 'zh'];

/** The lines of one captured entity record, or null. */
function entity(qid) {
  const start = CAPTURES.indexOf(`\n  entity ${qid}\n`);
  if (start === -1) return null;
  const end = CAPTURES.indexOf('\n  entity ', start + 1);
  return CAPTURES.slice(start, end === -1 ? undefined : end);
}

/** The property the `capital` relation is grounded in. */
function capitalProperty() {
  const relation = /\n {2}capital\n {4}grounded-in (P\d+)\n/u.exec(read('data/seed/meanings-facts.lino'));
  assert.ok(relation, 'meanings-facts.lino grounds the capital relation in a Wikidata property');
  return relation[1];
}

test('R174: each pre-warmed country has a capital triple whose value item is captured', () => {
  const property = capitalProperty();
  for (const qid of COUNTRY_QIDS) {
    const subject = entity(qid);
    assert.ok(subject, `fact-captures.lino has no entity record for ${qid}`);
    const claim = new RegExp(`claim \\("${property}" "(Q\\d+)" "(?:preferred|normal)"\\)`, 'u').exec(subject);
    assert.ok(claim, `${qid} has no ${property} claim`);
    assert.ok(entity(claim[1]), `${qid}'s ${property} value ${claim[1]} is not captured beside it`);
  }
});

test('R174: both ends of every triple carry a label in en, ru, hi and zh', () => {
  const property = capitalProperty();
  for (const qid of COUNTRY_QIDS) {
    const value = new RegExp(`claim \\("${property}" "(Q\\d+)"`, 'u').exec(entity(qid))[1];
    for (const [role, item] of [['subject', qid], ['value', value]]) {
      for (const language of LANGUAGES) {
        assert.match(entity(item), new RegExp(`label \\("${language}" "[^"]+"\\)`, 'u'), `${role} ${item} lacks a ${language} label`);
      }
    }
  }
});
