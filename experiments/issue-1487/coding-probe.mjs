// Issue #1487: do coding requests that differ only in operand order,
// comparison strength, connective, negation or count reach different
// obligation graphs, candidate programs and checks?
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { formalizeRequest, gapReport } from '../../js/agentic/crate/intent_formalization_obligations.mjs';
const host = new WorkerHost();
await installNodeHost(host);
host.context.fetch = () => { throw new Error('offline'); };
const prompts = process.argv.slice(2).length ? process.argv.slice(2) : [
  'Write a JavaScript function that returns a - b.',
  'Write a JavaScript function that returns b - a.',
  'Write a JavaScript function that returns whether a < b.',
  'Write a JavaScript function that returns whether a <= b.',
  'Write a Python program that prints "hello" twice.',
  'Write a Python program that prints "hello".',
  'Write a JavaScript function that returns a and b.',
  'Write a JavaScript function that returns a or b.',
  'Write a JavaScript function that returns not a.',
  'Frobnicate the quux so that it glorps.',
];
for (const prompt of prompts) {
  const graph = formalizeRequest(prompt);
  const r = await host.solve(prompt);
  console.log('=== ', prompt);
  console.log('graph', JSON.stringify(graph.nodes.map((n) => [n.clause, n.expectation, n.children.length])), JSON.stringify(graph.classifications), 'gaps', gapReport(graph));
  console.log('intent', r.intent, 'evidence', JSON.stringify(r.evidence));
  console.log(String(r.answer ?? r.text ?? JSON.stringify(r)).slice(0, 900));
}
