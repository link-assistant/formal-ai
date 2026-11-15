// Prints the proposition structure and identity of issue #1487 sentences.
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { formalizeText, statementIdentity, statementStructure } from '../../js/agentic/crate/text_formalization.mjs';
await installNodeHost(new WorkerHost());
const cases = process.argv.length > 2 ? [[process.argv[2], process.argv[3] ?? 'en']] : [
  ['1 is less than 2.', 'en'], ['2 is less than 1.', 'en'], ['2 is greater than 1.', 'en'], ['1 < 2', 'en'], ['1 <= 2', 'en'],
  ['1 is less than or equal to 2.', 'en'], ['a-b', 'en'], ['b-a', 'en'], ['Compute a - b.', 'en'], ['Compute b - a.', 'en'],
  ['The program returns true or false.', 'en'], ['The program returns true and false.', 'en'], ['Print hello twice.', 'en'],
  ['Print hello.', 'en'], ['Not all primes are odd.', 'en'], ['All primes are not odd.', 'en'], ['x + y * z', 'en'],
  ['Send an e-mail and/or a letter.', 'en'], ['程序返回真或假。', 'zh'], ['Программа возвращает истину или ложь.', 'ru'],
  ['El programa devuelve verdadero o falso.', 'es'], ['प्रोग्राम सही या गलत लौटाता है।', 'hi'], ['Выведи привет дважды.', 'ru'],
  ['There are not infinitely many primes.', 'en'], ['1969 and 1972', 'en']];
for (const [t, l] of cases) for (const s of formalizeText(t, l)) console.log(t, '=>', JSON.stringify(statementStructure(s)), '\n   ', statementIdentity(s));
process.exit(0);
