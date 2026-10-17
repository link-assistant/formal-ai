import fs from 'node:fs';
import assert from 'node:assert/strict';
import test from 'node:test';
import {createWorkerContext, evaluate, plain} from './support/browser-runtime.mjs';
// Lexical API pages are observations, not complete-request answer fixtures.
const pages = {
  'Веб': '{{t|en|Web}}',
  'программист': '{{t|en|programmer}}',
  'Программист': '{{t|en|Programmer}}',
  'разработчик': '{{t|en|developer}}',
  'Ведущий': '{{t|en|Host}}{{t|en|Leading}}',
  'алый': '{{t|en|scarlet}}',
  'сад': '{{t|en|orchard}}',
  'космический': '{{t|en|cosmic}}',
  'Веб-программист': '{{t|en|-- Английский-->}}',
};
const worker = createWorkerContext();
const ready = Promise.resolve(evaluate(worker, 'loadSeed()')).then(() => {
  worker.fetch = async (url) => {
    const query = new URL(String(url)).searchParams;
    if (query.get('action') === 'opensearch') {
      const search = query.get('search');
      const lemma = {алые: 'алый', сады: 'сад'}[search];
      return {ok: true, json: async () => [search, lemma ? [lemma] : []]};
    }
    const page = query.get('page');
    const text = pages[page];
    return {ok: text !== undefined, status: text === undefined ? 404 : 200,
      json: async () => ({parse: {wikitext: {'*': text}}})};
  };
});
async function solve(prompt) {
  await ready;
  return plain(await worker.solve(prompt, [], {}, {}, [], {}));
}
for (const [surface, expected] of [
  ['Веб-программист', 'Web-programmer'],
  ['Программист', 'Programmer'],
  ['алый сад', 'scarlet orchard'],
  ['космический сад', 'cosmic orchard'],
  ['алые сады', 'scarlet orchard'],
]) {
  test('complete lexical evidence: ' + surface, async () => {
    const prompt = surface === 'Программист' ? 'Translate to English: ' + surface : 'Translate "' + surface + '" to English';
    const answer = await solve(prompt);
    assert.equal(answer.intent, 'translate_ru_to_en');
    assert.equal(answer.content, '"' + expected + '"');
    assert.ok(answer.evidence.includes('translation_obligation:complete_target'));
    assert.ok(answer.evidence.some(value => value.startsWith('translation_source:')));
  });
}
for (const surface of ['Ведущий разработчик', 'алый неизвестность', 'Team Lead']) {
  test('unknown or ambiguous components remain named gaps: ' + surface, async () => {
    const answer = await solve('Translate "' + surface + '" to English');
    if (surface === 'Team Lead') {
      // Identity has no cross-language knowledge claim; original issue requested Russian.
      const original = await solve('Translate "Team Lead" to Russian');
      assert.ok(original.evidence.includes('translation_gap:Team Lead'));
      return;
    }
    assert.ok(answer.evidence.includes('translation_gap:' + surface));
    assert.ok(answer.evidence.includes('translation_obligation:knowledge_gap'));
    assert.ok(!answer.content.includes('response:translate'));
  });
}
test('diagnostic and partial target strings cannot satisfy the obligation', async () => {
  await ready;
  for (const value of ['-- Английский-->', 'response:translate', 'programmer неизвестно', '']) {
    assert.equal(evaluate(worker, 'targetTranslationSurfaceComplete(' + JSON.stringify(value) + ', "en")'), false);
  }
});

test('inflected component evidence names the actual lemma page', async () => {
  const answer = await solve('Translate "алые сады" to English');
  assert.ok(answer.evidence.some(value => value.includes('wiktionary:ru:алый#translations->en')));
  assert.ok(answer.evidence.some(value => value.includes('wiktionary:ru:сад#translations->en')));
});
