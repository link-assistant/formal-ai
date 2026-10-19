import fs from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import test from 'node:test';
const source = fs.readFileSync(new URL('../../../js/worker/formal_ai_worker_behavior_rules_and_self_facts.js', import.meta.url), 'utf8');
const start = source.indexOf('function extractUnquotedTranslationSurface(');
const end = source.indexOf('// Head-final Hindi:', start);
assert.ok(start >= 0 && end > start);
const context = vm.createContext({
  translationMarkers: () => ({circumfixFrames: [['translate ', ' to '], ['переведи ', ' на ']], suffixFrames: []}),
  meaningsWithRole: () => [{words: ['to english', 'на английский', 'to spanish']}],
  ROLE_TRANSLATION_TARGET_MARKER: 'translation_target_marker',
  extractHindiUnquotedTranslationSurface: () => null,
  extractChineseUnquotedTranslationSurface: () => null,
});
const composition = fs.readFileSync(new URL('../../../js/worker/formal_ai_worker_translation_composition.js', import.meta.url), 'utf8');
vm.runInContext(composition, context);
vm.runInContext(source.slice(start, end), context);
const cases = [
  ['Translate to English: Программист', 'Программист'],
  ['Translate to English: невиданный термин', 'невиданный термин'],
  ['Translate to Spanish: quantum orchard', 'quantum orchard'],
  ['Переведи на английский: новая профессия', 'новая профессия'],
  ['Translate apple to English', 'apple'],
  ['Translate to unknown: яблоко', null],
  ['Explain to English: яблоко', null],
  ['Translate to English: ', null],
  ['Translate apple to English: formal tone', 'apple'],
];
for (const [input, expected] of cases) {
  test(input, () => assert.equal(context.extractUnquotedTranslationSurface(input), expected));
}
