// R1188-U21 (extends R197): Formal AI summarizes algorithmically, with no
// language model. It keeps the statements the other statements depend on,
// drops a statement whose formal content an earlier one already holds
// (by formal identity, not by string), and renders the kept core concisely.
// Pins js/agentic/crate/dependency_summarization.mjs on exact cases and holds
// the measured summarization of the cached pages and issue bodies to its
// ratchet (data/meta/text-summarization-ratchet.lino). The Rust twin is
// rust/tests/unit/issue_1188_dependency_summarization.rs.

import assert from 'node:assert/strict';
import { before, describe, test } from 'node:test';

import { compareRatchet, installTextHost, readRatchet } from '../../../scripts/lib/text-capability-measures.mjs';

const TEXT = 'The Moon orbits Earth. Earth is orbited by the Moon. The Moon has craters. Craters cover the Moon. Paris is a city.';

let summarization;

before(async () => {
  installTextHost();
  summarization = await import('../../../js/agentic/crate/dependency_summarization.mjs');
});

describe('dependency summarization', () => {
  test('a restatement in other words is dropped by its formal identity', () => {
    const entries = summarization.sentenceStatements(TEXT, 'en');
    assert.equal(entries.length, 5);
    assert.equal(summarization.restates(entries[1].statement, entries[0].statement), true);
    assert.equal(summarization.restates(entries[2].statement, entries[0].statement), false);
    const { unique, duplicates } = summarization.withoutDuplicates(entries);
    assert.deepEqual([unique.length, duplicates], [4, 1]);
  });

  test('the graph links a statement to the earlier ones whose subject it mentions', () => {
    const summary = summarization.summarizeByDependency(TEXT, 'en');
    assert.deepEqual(summary.statements.map((node) => [node.dependsOn, node.dependents]), [
      [[], [1, 2]],
      [[0], [2]],
      [[0, 1], []],
      [[], []],
    ]);
  });

  test('the kept core is the depended-on root and its most depended-on statement', () => {
    const summary = summarization.summarizeByDependency(TEXT, 'en');
    assert.deepEqual(summary.kept, [0, 1]);
    assert.equal(summary.text, 'The Moon orbits Earth. The Moon has craters.');
    const facts = summarization.keyFacts(TEXT, 'en');
    assert.deepEqual(facts, ['moon', 'name:earth', 'unknown:orbit']);
    assert.equal(summarization.retainedFacts(summary, facts), 3);
  });

  test('a fragment that ends its sentence in the summary is closed with its punctuation', () => {
    const summary = summarization.summarizeByDependency(
      'The Moon orbits Earth, and it has craters. Craters are old. The Moon is bright.',
      'en',
    );
    assert.ok(summary.text.endsWith('.'), summary.text);
    assert.equal(summarization.closingOf('月球是卫星。'), '。');
    assert.equal(summarization.closingOf('no closing'), '');
  });

  test('the summary keeps one statement in three', () => {
    assert.deepEqual([0, 1, 2, 3, 4, 7].map(summarization.keepBudget), [0, 1, 1, 1, 2, 3]);
  });
});

describe('the corpus and its ratchet', () => {
  test('an issue body is read as prose', async () => {
    const { proseOf } = await import('../../../scripts/measure-dependency-summarization.mjs');
    const body = '## Goal\n\nUse [the cache](https://example.org/x) <b>now</b>.\n\n```js\ncode();\n```\n| a | b |\n- Keep `it` small.';
    assert.equal(proseOf(body), 'Goal\nUse the cache now .\nKeep it small.');
  });

  test('compression, kept key facts and removed duplicates do not fall below their ratchet', async () => {
    const { RATCHET_FILE, measureDependencySummarization } = await import('../../../scripts/measure-dependency-summarization.mjs');
    const { measures } = await measureDependencySummarization();
    const recorded = readRatchet(RATCHET_FILE);
    assert.ok(recorded.size > 0, RATCHET_FILE);
    assert.deepEqual(compareRatchet(measures, recorded).falls, []);
  });
});
