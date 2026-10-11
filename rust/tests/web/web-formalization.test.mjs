// R1188-U18: page formalization. Every sentence of a page becomes formal
// statements in the links network, nothing is dropped silently, and the
// statements deformalize back to the same known terms. Pins the shared text
// formalizer (js/agentic/crate/text_formalization.mjs) and the page report
// (js/agentic/crate/page_formalization.mjs) on a few exact cases, checks the
// cached Wikipedia corpus, and holds the measured page formalization to its
// ratchet (data/meta/web-formalization-ratchet.lino). The Rust twin is
// rust/tests/unit/issue_1188_web_formalization.rs.

import assert from 'node:assert/strict';
import { statSync } from 'node:fs';
import { join } from 'node:path';
import { before, describe, test } from 'node:test';

import { REPO_ROOT } from '../../../js/server/lino.mjs';
import {
  LANGUAGES, PAGE_CORPUS_DIRECTORY, compareRatchet, installTextHost, readPages, readRatchet,
} from '../../../scripts/lib/text-capability-measures.mjs';

let text;
let page;

before(async () => {
  installTextHost();
  text = await import('../../../js/agentic/crate/text_formalization.mjs');
  page = await import('../../../js/agentic/crate/page_formalization.mjs');
});

const identities = (source, language) => text.formalizeText(source, language).map(text.statementIdentity);

describe('the shared text formalizer', () => {
  test('tokens keep apostrophes and numbers whole and set Han runs apart', () => {
    assert.deepEqual(text.tokens("Earth's orbit is 378,000 km; 1.2 billion. 地球的卫星（月球）"), [
      { surface: "Earth's", han: false },
      { surface: 'orbit', han: false },
      { surface: 'is', han: false },
      { surface: '378,000', han: false },
      { surface: 'km', han: false },
      { surface: '1.2', han: false },
      { surface: 'billion', han: false },
      { surface: '地球的卫星', han: true },
      { surface: '月球', han: true },
    ]);
  });

  test('an anaphor and a clause continuation keep the subject; a negation cue denies', () => {
    assert.deepEqual(identities('The Moon orbits Earth. It does not emit light, and it has 1.2 billion craters.', 'en'), [
      'asserted|moon|name:earth unknown:orbit',
      'denied|moon|file_whole_write_action unknown:light',
      'asserted|moon|number:1.2 unknown:billion unknown:crater',
    ]);
    assert.deepEqual(identities('Луна вращается вокруг Земли. Она не светится.', 'ru'), [
      'asserted|moon|name:земли unknown:вокруг unknown:враща',
      'denied|moon|unknown:свет',
    ]);
  });

  test('every language reads through the same lexicon', () => {
    assert.deepEqual(identities('सेब एक फल है।', 'hi'), ['asserted|wikidata_item_apple|wikidata_item_fruit']);
    assert.deepEqual(identities('La manzana es una fruta.', 'es'), [
      'asserted|wikidata_item_apple|wikidata_item_fruit wikidata_property_instance_of',
    ]);
    assert.deepEqual(identities('我喜欢苹果。', 'zh'), ['asserted|software_object_lead_word|unknown:喜欢 wikidata_item_apple']);
  });

  test('a shared surface goes to the meaning in more languages, then with fewer surfaces, then declared first', async () => {
    // PR #1188 (LEXEMES): the surface count spans every language, so one
    // meaning holds a shared word in each language and the rank still tells
    // meanings apart once every meaning is written in all five languages.
    const { lexicon } = await import('../../../js/agentic/crate/seed_meanings.mjs');
    const english = text.meaningIndex('en');
    assert.equal(english.get('apple'), 'wikidata_item_apple');
    assert.equal(text.meaningIndex('hi').get('सेब'), 'wikidata_item_apple');
    const excluded = [...text.FUNCTION_WORD_ROLES, text.NEGATION_ROLE];
    // A rank orders lower first: more languages, fewer surfaces, earlier declared.
    const outranks = (candidate, held) => {
      for (let index = 0; index < 3; index += 1) {
        if (candidate[index] !== held[index]) return candidate[index] < held[index];
      }
      return false;
    };
    const ranks = new Map();
    lexicon().forEach((entry, order) => {
      if (entry.roles.some((role) => excluded.includes(role))) return;
      const languages = new Set(entry.lexemes.map((lexeme) => lexeme.language)).size;
      const size = entry.lexemes.reduce((sum, lexeme) => sum + lexeme.words.length, 0);
      const lexeme = entry.lexemes.find((candidate) => candidate.language === 'en');
      for (const word of lexeme?.words ?? []) {
        const key = text.phraseKey(word.text);
        if (key === null) continue;
        const rank = [-languages, size, order, entry.slug];
        if (!ranks.has(key) || outranks(rank, ranks.get(key))) ranks.set(key, rank);
      }
    });
    for (const [key, rank] of ranks) assert.equal(english.get(key), rank[3], key);
  });

  test('a sentence-opening word is a name only when the text writes it as one elsewhere', () => {
    assert.deepEqual(identities('Earth is orbited by the Moon. The Moon orbits Earth.', 'en'), [
      'asserted|name:earth|moon unknown:orbit',
      'asserted|moon|name:earth unknown:orbit',
    ]);
    assert.deepEqual(identities('Yesterday the Moon rose.', 'en'), ['asserted|unknown:yesterday|moon unknown:rose']);
  });

  test('a statement deformalizes subject first, in any language', () => {
    const [statement] = text.formalizeText('The Moon does not orbit Earth.', 'en');
    assert.equal(text.deformalizeStatement(statement, 'en'), 'Moon not orbit Earth');
    assert.equal(text.deformalizeStatement(statement, 'ru'), 'Луна не orbit Earth');
    assert.equal(text.deformalizeStatement(statement, 'zh'), '月球不 orbit Earth');
  });
});

describe('page formalization', () => {
  test('a page reports every sentence, its unknown words and its surviving facts', () => {
    const report = page.formalizePage('The Moon orbits Earth. It does not emit light, and it has 1.2 billion craters. La la la.', 'en');
    assert.deepEqual(
      { covered: report.covered, statements: report.statements, factual: report.factual, survived: report.survived, terms: report.terms, unknown: report.unknown },
      { covered: 0, statements: 4, factual: 3, survived: 3, terms: 13, unknown: 7 },
    );
    assert.deepEqual(report.sentences.map((sentence) => sentence.unknown), [
      ['orbits'],
      ['light', 'billion', 'craters'],
      ['La', 'la', 'la'],
    ]);
  });

  test('a sentence whose terms are all known is covered and its fact survives', () => {
    const report = page.formalizePage('सेब एक फल है।', 'hi');
    assert.equal(report.covered, 1);
    assert.equal(report.survived, 1);
    assert.equal(report.unknown, 0);
  });

  test('nothing on a cached page is dropped silently', async () => {
    const { sentences } = await import('../../../js/agentic/crate/formalization_segment.mjs');
    for (const language of LANGUAGES) {
      for (const cached of readPages(language)) {
        const report = page.formalizePage(cached.text, language);
        assert.equal(report.sentences.length, sentences(cached.text).length, cached.url);
        const listed = report.sentences.reduce((sum, sentence) => sum + sentence.unknown.length, 0);
        assert.equal(listed, report.unknown, cached.url);
      }
    }
  });
});

describe('the cached corpus and its ratchet', () => {
  test('every language holds the same topics, each with its source URL and revision id', () => {
    for (const language of LANGUAGES) {
      const pages = readPages(language);
      assert.equal(pages.length, 8, language);
      for (const cached of pages) {
        assert.match(cached.url, new RegExp(`^https://${language}\\.wikipedia\\.org/wiki/`), cached.title);
        assert.ok(Number(cached.revision) > 0, cached.title);
        assert.ok(cached.text.length > 0, cached.title);
      }
      const bytes = statSync(join(REPO_ROOT, PAGE_CORPUS_DIRECTORY, `${language}.lino`)).size;
      assert.ok(bytes < 64 * 1024, `${language} corpus is ${bytes} bytes; intro sections only`);
    }
  });

  test('page formalization does not fall below its ratchet', async () => {
    const { RATCHET_FILE, measureWebFormalization } = await import('../../../scripts/measure-web-formalization.mjs');
    const { measures } = await measureWebFormalization();
    const recorded = readRatchet(RATCHET_FILE);
    assert.ok(recorded.size > 0, RATCHET_FILE);
    assert.deepEqual(compareRatchet(measures, recorded).falls, []);
  });
});
