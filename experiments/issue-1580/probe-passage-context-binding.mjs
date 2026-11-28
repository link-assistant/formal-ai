// Supplied finite context-binding investigation; zero independent authorship credit.
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { gzipSync } from 'node:zlib';
import { execFileSync } from 'node:child_process';
import { createWorkerContext, evaluate, plain } from '../../rust/tests/web/support/browser-runtime.mjs';
const worker = createWorkerContext();
await evaluate(worker, 'loadSeed()');
const calls = [];
worker.fetch = async (url) => {
  calls.push(String(url));
  if (calls.length > 50) throw Error('finite local transport limit');
  return { ok: false, status: 404, headers: { get: () => '' }, text: async () => '' };
};
const passage = '"The meeting moved from Tuesday to Thursday at 3 pm in room 204."';
const requests = [
  ['whole', `Read this and answer: ${passage} When and where is the meeting?`],
  ['original-when-child', `answer: ${passage} When`],
  ['bare-when-with-context', `${passage} When`],
  ['bare-where-with-context', `${passage} where is the meeting?`],
  ['cued-when-with-context', `Read this: ${passage} When`],
  ['cued-where-with-context', `Read this: ${passage} where is the meeting?`],
  ['read-child-with-context', `${passage} Read this`],
  ['independent-arithmetic-with-context', `${passage} Calculate 2 + 2`],
];
const rows = [];
for (const [name, prompt] of requests) {
  const before = calls.length;
  const answer = plain(await worker.solve(prompt, [], {}, {}, [], { metaSubTask: true }));
  rows.push({ name, prompt, answer, localTransportCalls: calls.length - before });
  console.log(JSON.stringify({ name, intent: answer.intent, content: answer.content, status: answer.metaExecution?.status, calls: calls.length - before }));
}
const sourceHashes = Object.fromEntries(['js/agentic/crate/meta_executor.mjs', 'js/worker/formal_ai_worker_solve.js', 'js/worker/formal_ai_worker_factual_qa.js'].map((path) => [path, createHash('sha256').update(readFileSync(path)).digest('hex')]));
const raw = Buffer.from(JSON.stringify({ schema: 'issue-1580-context-binding-probe/v1', head: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(), sourceHashes, rows, calls, actualNetworkRequests: 0, nativeExecuted: false, independentAuthorshipCredit: 0 }, null, 2) + '\n');
writeFileSync('experiments/issue-1580/evidence/checks/passage-routing-probe/context-binding-fragments.json.gz', gzipSync(raw), { flag: 'wx' });
