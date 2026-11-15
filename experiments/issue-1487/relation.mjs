// Prints the checked relation (and restates) of issue #1487 statement pairs.
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { formalizeText } from '../../js/agentic/crate/text_formalization.mjs';
import { checkedRelation } from '../../js/agentic/crate/statement_relation.mjs';
import { restates } from '../../js/agentic/crate/dependency_summarization.mjs';
await installNodeHost(new WorkerHost());
const pairs = [['2 is less than 1.', '1 is less than 2.'], ['1 is less than 2.', '1 is less than 2.'], ['2 is greater than 1.', '1 is less than 2.'],
  ['1 < 2', '1 is less than 2.'], ['1 <= 2', '1 < 2'], ['The program returns true or false.', 'The program returns true and false.'],
  ['The program returns true and false.', 'The program returns true or false.'], ['b-a', 'a-b'], ['Print hello.', 'Print hello twice.'],
  ['Print hello.', 'Print hello.'], ['Not all primes are odd.', 'All primes are not odd.'], ['1972', '1969 and 1972'], ['x + y * z', 'x + y * z']];
for (const [a, b] of pairs) {
  const [later] = formalizeText(a, 'en'); const [earlier] = formalizeText(b, 'en');
  console.log(JSON.stringify(a), 'vs', JSON.stringify(b), '=>', JSON.stringify(checkedRelation(later, earlier)), 'restates', restates(later, earlier));
}
process.exit(0);
