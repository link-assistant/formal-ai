// Finite replay against retained publisher JSON, not a live-fetch claim.
// Supplied experiment: no independent authorship credit.
import { createWorkerContext, evaluate, plain } from '../../rust/tests/web/support/browser-runtime.mjs';
import { routingFactReplay } from '../../rust/tests/web/support/routing-fact-replay.mjs';

const requests = [];
const worker = createWorkerContext();
worker.fetch = routingFactReplay(worker.fetch, requests);
await evaluate(worker, 'loadSeed()');
for (const prompt of ['Make sense of this: what is the capital of France?', 'What is the capital of the USA?', 'столица россии', '谁创建了 Rust?']) {
  const before = requests.length;
  const answer = plain(await worker.solve(prompt));
  console.log(JSON.stringify({prompt, intent: answer.intent, content: answer.content, evidence: answer.evidence, requests: requests.slice(before)}));
}
