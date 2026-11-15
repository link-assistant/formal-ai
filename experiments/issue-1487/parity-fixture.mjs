// Issue #1487 — writes (with --write) or checks the shared JS/native fixture
// rust/tests/fixtures/proposition-structure-parity.json from the JS reader.
// The JS test rust/tests/web/issue-1487-proposition-identity.test.mjs and the
// Rust test rust/tests/unit/proposition_structure_parity.rs both read it.
import fs from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { formalizeText, statementStructure } from '../../js/agentic/crate/text_formalization.mjs';
import { structureKey } from '../../js/agentic/crate/proposition_structure.mjs';
import { checkedRelation, certifiesRestatement } from '../../js/agentic/crate/statement_relation.mjs';
import { signatureKey, signatureOf } from '../../js/agentic/crate/summarization_dedup.mjs';
await installNodeHost(new WorkerHost());
const PATH = new URL('../../rust/tests/fixtures/proposition-structure-parity.json', import.meta.url);
const structures = [
  ['1 is less than 2.', 'en'], ['2 is less than 1.', 'en'], ['2 is greater than 1.', 'en'], ['1 < 2', 'en'], ['1 <= 2', 'en'],
  ['1 is less than or equal to 2.', 'en'], ['a-b', 'en'], ['b-a', 'en'], ['Compute a - b.', 'en'], ['Compute b - a.', 'en'],
  ['The program returns true or false.', 'en'], ['The program returns true and false.', 'en'], ['Print hello twice.', 'en'],
  ['Print hello.', 'en'], ['Not all primes are odd.', 'en'], ['All primes are not odd.', 'en'], ['x + y * z', 'en'],
  ['Send an e-mail and/or a letter.', 'en'], ['程序返回真或假。', 'zh'], ['Программа возвращает истину или ложь.', 'ru'],
  ['प्रोग्राम सही या गलत लौटाता है।', 'hi'], ['Выведи привет дважды.', 'ru'], ['1969 and 1972', 'en'],
];
const relations = [
  ['2 is less than 1.', '1 is less than 2.'], ['1 is less than 2.', '1 is less than 2.'], ['2 is greater than 1.', '1 is less than 2.'],
  ['1 < 2', '1 is less than 2.'], ['1 <= 2', '1 < 2'], ['The program returns true or false.', 'The program returns true and false.'],
  ['The program returns true and false.', 'The program returns true or false.'], ['b-a', 'a-b'], ['Compute b - a.', 'Compute a - b.'],
  ['Print hello.', 'Print hello twice.'], ['Print hello twice.', 'Print hello.'], ['Print hello.', 'Print hello.'],
  ['Not all primes are odd.', 'All primes are not odd.'], ['1972', '1969 and 1972'], ['x + y * z', 'x + y * z'],
  ['x + y * z', 'x * y + z'],
];
// Statement-merge signatures (summarization dedup): a raw sentence, no language.
const signatures = [
  'Compute a - b.', 'Compute b - a.', '1 < 2', '2 < 1', 'x <= y', '1 is less than 2.', '2 is less than 1.',
  '2 is greater than 1.', '1 is at most 2.', 'The program returns true or false.', 'The program returns true and false.',
  'The program does not return true or false.', 'Not all tests pass.', 'All tests do not pass.', 'Print hello twice.',
  'Print hello.', 'The parser is fast.', 'The parser is not fast.', 'Не все тесты проходят.', 'Все тесты не проходят.',
  'Программа возвращает истину или ложь.', '程序返回真或假。', 'प्रोग्राम सही या गलत लौटाता है।', 'Выведи привет дважды.',
];
const one = (text, language) => {
  const statements = formalizeText(text, language);
  if (statements.length !== 1) throw new Error(`expected one statement for ${text}, got ${statements.length}`);
  return statements[0];
};
const fixture = {
  schema: 'proposition-structure-parity/v1',
  issue: 'https://github.com/link-assistant/formal-ai/issues/1487',
  structures: structures.map(([text, language]) => ({ text, language, key: structureKey(statementStructure(one(text, language))) })),
  relations: relations.map(([later, earlier]) => {
    const relation = checkedRelation(one(later, 'en'), one(earlier, 'en'));
    return { later, earlier, language: 'en', verdict: relation.verdict, reason: relation.reason, restates: certifiesRestatement(relation) };
  }),
  signatures: signatures.map((text) => ({ text, key: signatureKey(signatureOf(text)) })),
};
const text = `${JSON.stringify(fixture, null, 2)}\n`;
if (process.argv.includes('--write')) fs.writeFileSync(PATH, text);
else if (fs.readFileSync(PATH, 'utf8') !== text) { console.error('proposition-structure-parity.json is stale; run with --write'); process.exit(1); }
console.log(text);
process.exit(0);
