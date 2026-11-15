// Issue #1487 reproduction: the minimal probe from the issue plus the coding
// case (`twice`), run against any checkout.
//   node experiments/issue-1487/issue-probe.mjs [checkout-root]
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const root = path.resolve(process.argv[2] ?? '.');
const load = (relative) => import(pathToFileURL(path.join(root, relative)).href);
const { WorkerHost } = await load('js/server/worker-host.mjs');
const { installNodeHost } = await load('js/agentic/node-host.mjs');
const { formalizeText, statementIdentity } = await load('js/agentic/crate/text_formalization.mjs');
const { restates } = await load('js/agentic/crate/dependency_summarization.mjs');
const worker = new WorkerHost();
await installNodeHost(worker);
worker.context.fetch = () => { throw new Error('offline probe'); };
const one = (text) => formalizeText(text, 'en')[0];
console.log('restates("2 is less than 1." , "1 is less than 2."):', restates(one('2 is less than 1.'), one('1 is less than 2.')));
console.log('same identity (true or false / true and false):', statementIdentity(one('The program returns true or false.')) === statementIdentity(one('The program returns true and false.')));
const negated = await worker.solve('Prove there are not infinitely many primes.');
console.log('negated theorem:', negated.intent, negated.evidence.filter((entry) => /proof_outcome|proof_binding/u.test(entry)));
const signature = 'Implement Python function count_vowels(text: str) -> int.';
for (const requirement of ['Return the number of vowels in the text.', 'Return twice the number of vowels in the text.']) {
  const answer = await worker.solve(`${signature} ${requirement}`);
  const source = answer.synthesizedProgram?.source?.trim().split('\n').pop() ?? '(no program)';
  console.log(`${requirement} ->`, source, answer.evidence.filter((entry) => entry.includes('unresolved_structure')));
}
process.exit(0);
