// Prints how the shared formalizer tokenizes the issue #1487 sentences.
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { readWords } from '../../js/agentic/crate/text_formalization.mjs';
await installNodeHost(new WorkerHost());
const cases = process.argv.length > 2 ? [[process.argv[2], process.argv[3] ?? 'en']] : [
  ['The program returns true or false.', 'en'], ['程序返回真或假。', 'zh'], ['Все числа не больше двух.', 'ru'],
  ['Print hello twice.', 'en'], ['x < y and y <= z', 'en'], ['2 is less than 1.', 'en'], ['Not all primes are odd.', 'en'],
  ['El programa devuelve verdadero o falso.', 'es'], ['प्रोग्राम सही या गलत लौटाता है।', 'hi'], ['Print hello two times.', 'en']];
for (const [t, l] of cases) console.log(t, JSON.stringify(readWords(t, l, true).map((w) => [w.surface, w.kind, w.id])));
process.exit(0);
