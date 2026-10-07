// Issue #127 (R174): data/seed/fact-captures.lino must declare structured
// Wikidata entity records for the nine pre-warmed countries (Russia, Japan,
// France, Germany, China, India, United States, United Kingdom, Brazil),
// each with labels in all four supported languages (en, ru, hi, zh).

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import test from 'node:test';

import { REPO_ROOT } from './support/browser-runtime.mjs';

const CAPTURES = readFileSync(join(REPO_ROOT, 'data/seed/fact-captures.lino'), 'utf8');

// The nine pre-warmed country Wikidata QIDs as declared in the seed.
const COUNTRY_QIDS = ['Q17', 'Q159', 'Q142', 'Q183', 'Q148', 'Q668', 'Q30', 'Q145', 'Q155'];

// The four required label languages.
const LANGUAGES = ['en', 'ru', 'hi', 'zh'];

test('R174: fact-captures.lino holds entity records for all nine pre-warmed countries', () => {
  for (const qid of COUNTRY_QIDS) {
    assert.ok(
      CAPTURES.includes(`entity ${qid}`),
      `fact-captures.lino is missing entity record for ${qid}`,
    );
  }
});

test('R174: every pre-warmed country entity has labels in en, ru, hi, and zh', () => {
  for (const qid of COUNTRY_QIDS) {
    const entityStart = CAPTURES.indexOf(`entity ${qid}\n`);
    assert.notEqual(entityStart, -1, `entity ${qid} not found`);
    // Find the next entity boundary to scope the label search.
    const entityEnd = CAPTURES.indexOf('\n  entity ', entityStart + 1);
    const block = entityEnd === -1 ? CAPTURES.slice(entityStart) : CAPTURES.slice(entityStart, entityEnd);
    for (const lang of LANGUAGES) {
      assert.ok(
        block.includes(`label ("${lang}" `),
        `entity ${qid} is missing a "${lang}" label`,
      );
    }
  }
});
