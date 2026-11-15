import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { formalizeText, statementIdentity } from '../../js/agentic/crate/text_formalization.mjs';
import { restates } from '../../js/agentic/crate/dependency_summarization.mjs';
const worker = new WorkerHost();
await installNodeHost(worker);
const show = (s) => JSON.stringify({p: s.polarity, subj: s.subject, terms: s.terms, id: statementIdentity(s)});
for (const t of ['1 is less than 2.', '2 is less than 1.', 'The program returns true or false.', 'The program returns true and false.', 'a-b', 'b-a', '1 is less than or equal to 2.', 'Print hello twice.', 'Print hello.']) {
  for (const s of formalizeText(t, 'en')) console.log(t, '=>', show(s));
}
const [a] = formalizeText('1 is less than 2.', 'en');
const [b] = formalizeText('2 is less than 1.', 'en');
console.log('restates', restates(b, a));
const [c] = formalizeText('The program returns true or false.', 'en');
const [d] = formalizeText('The program returns true and false.', 'en');
console.log('same id', statementIdentity(c) === statementIdentity(d));
const answer = await worker.solve('Prove there are not infinitely many primes.');
console.log(answer.intent, answer.evidence, String(answer.answer ?? answer.text ?? '').slice(0, 600));
const answer2 = await worker.solve('Prove there are infinitely many primes.');
console.log(answer2.intent, answer2.evidence);
process.exit(0);
