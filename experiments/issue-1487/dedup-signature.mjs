// Issue #1487: does the summarization dedup signature merge statements that
// differ only in operand order, comparison, connective, negation scope or an
// explicit count?
import { installTextHost } from '../../scripts/lib/text-capability-measures.mjs';
installTextHost();
const { signatureOf, signatureKey } = await import('../../js/agentic/crate/summarization_dedup.mjs');
const pairs = [
  ['Compute a - b.', 'Compute b - a.'],
  ['1 < 2', '2 < 1'],
  ['x < y', 'x <= y'],
  ['The parser is fast and correct.', 'The parser is fast or correct.'],
  ['Not all tests pass.', 'All tests do not pass.'],
  ['Print hello twice.', 'Print hello.'],
  ['1 is less than 2.', '2 is less than 1.'],
  ['1 is less than 2.', '2 is greater than 1.'],
  ['x is less than y.', 'x is at most y.'],
  ['The program returns true or false.', 'The program returns true and false.'],
  ['Every test passes.', 'Some test passes.'],
  ['Не все тесты проходят.', 'Все тесты не проходят.'],
  ['The parser is fast.', 'The parser is fast.'],
  ['The parser is fast.', 'The parser is not fast.'],
  ['Alice pays Bob.', 'Bob pays Alice.'],
];
for (const [left, right] of pairs) {
  const a = signatureKey(signatureOf(left));
  const b = signatureKey(signatureOf(right));
  console.log(`${a === b ? 'MERGE ' : 'apart '} ${JSON.stringify(left)} [${a}] vs ${JSON.stringify(right)} [${b}]`);
}
