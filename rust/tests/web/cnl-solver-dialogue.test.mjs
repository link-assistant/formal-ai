// The five language cases run across three maintained files; base controls stay here.
import test from 'node:test';
import assert from 'node:assert/strict';
import {registerCnlDialogueCases, solve} from './helpers/cnl-solver-dialogue.mjs';

registerCnlDialogueCases(0);

test('prompts the solver already understood keep their answers', {timeout: 60_000}, async () => {
  // Issue #962 word operators in every language (rust/tests/unit/issue_962_word_operator_parity.rs).
  for (const [prompt, expected] of [['What is 2 plus 2?', '2 plus 2 = 4'], ['Сколько будет 2 плюс 2?', '2 плюс 2 = 4'],
    ['2 जोड़ 2 कितना होता है?', '2 जोड़ 2 = 4'], ['2 जमा 2 कितना होता है?', '2 जमा 2 = 4'],
    ['2 加 2 等于多少?', '2 加 2 = 4'], ['¿Cuánto es 2 más 2?', '2 más 2 = 4']]) {
    const sum = await solve(prompt);
    assert.equal(sum.content, expected, prompt);
    assert.ok(!(sum.evidence || []).some(item => String(item).startsWith('cnl:task')), `no CNL fallback for ${prompt}`);
  }
  // The task paraphrase reaches the canonical calculation.
  const paraphrase = await solve('add 10 and 3');
  assert.equal(paraphrase.intent, 'calculation');
  assert.equal(paraphrase.content, '10 + 3 = 13');
  assert.ok(paraphrase.evidence.includes('cnl:task:en'), JSON.stringify(paraphrase.evidence));
});

const MAX_PRECISION_CAT = 'Maximum precision uses specific words. Look up unfamiliar words in a dictionary to check '
  + 'their exact meaning.\n\nYes. Cats are animals.';

test('an in-dialog density switch re-renders the same controlled answer deterministically', {timeout: 60_000}, async () => {
  const question = 'Is a cat an animal?';
  const first = await solve(question);
  assert.equal(first.intent, 'cnl_answer');
  assert.equal(first.content, 'Yes. Every cat is an animal.');
  const history = [{role: 'user', content: question}, {role: 'assistant', content: first.content}];
  // A declared directive (a bare `max-precision` is not one) re-renders the previous answer.
  const switched = await solve('Use max-precision.', history);
  const again = await solve('Use max-precision.', history);
  assert.equal(switched.intent, 'cnl_paraphrase');
  assert.equal(switched.content, MAX_PRECISION_CAT);
  assert.equal(again.content, switched.content);
  const named = await solve('Rewrite in max-precision mode', history);
  assert.equal(named.content, MAX_PRECISION_CAT);
});
