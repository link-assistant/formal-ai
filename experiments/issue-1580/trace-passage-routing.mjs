// Supplied investigation harness; zero independent Formal AI authorship credit.
// Real worker routing with bounded local 404 transport; no provider is contacted.
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { gzipSync } from 'node:zlib';
import { createWorkerContext, evaluate, plain } from '../../rust/tests/web/support/browser-runtime.mjs';

const label = process.argv[2] || 'before';
if (!/^[a-z][a-z0-9-]{0,30}$/u.test(label)) throw new Error('invalid label');
const mode = process.argv[3] || 'guarded';
if (!['guarded', 'allow-candidate-research', 'standalone-research'].includes(mode)) throw new Error('invalid mode');
const worker = createWorkerContext();
await evaluate(worker, 'loadSeed()');
const calls = [];
worker.fetch = async (url) => {
  calls.push({ url: String(url), stack: new Error('local transport observation').stack });
  if (calls.length > 100) throw new Error('bounded probe call limit exceeded');
  return { ok: false, status: 404, headers: { get: () => '' }, text: async () => '' };
};
if (mode === 'allow-candidate-research') {
  // Negative control: undo only the existing speculative-research option.
  await evaluate(worker, `const guardedSolveImpl = solveImpl; solveImpl = (...args) => {
    args[5] = { ...args[5], metaSubTask: false }; return guardedSolveImpl(...args);
  };`);
}
const passage = 'Read this and answer: "The meeting moved from Tuesday to Thursday at 3 pm in room 204." When and where is the meeting?';
const prompt = mode === 'standalone-research' ? 'where is the meeting?' : passage;
const answer = plain(await worker.solve(prompt, [], {}, {}, [], {}));
const hashes = Object.fromEntries([
  'js/agentic/crate/meta_executor.mjs',
  'js/worker/formal_ai_worker_solve.js',
  'js/worker/formal_ai_worker_factual_qa.js',
].map((path) => [path, createHash('sha256').update(readFileSync(path)).digest('hex')]));
const result = {
  schema: 'issue-1580-passage-routing-probe/v1',
  label, mode, head: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(),
  prompt, hashes, calls, answer, actualNetworkRequests: 0,
  scope: 'Worker routing with local mocked 404 transport; no native execution or independent authorship credit.',
};
const raw = Buffer.from(JSON.stringify(result, null, 2) + '\n');
const base = 'experiments/issue-1580/evidence/checks/passage-routing-probe';
mkdirSync(base, { recursive: true });
writeFileSync(`${base}/${label}.json.gz`, gzipSync(raw), { flag: 'wx' });
console.log(JSON.stringify({ label, intent: answer.intent, shape: answer.metaExecution?.shape, observedTransportCalls: calls.length, rawSha256: createHash('sha256').update(raw).digest('hex') }, null, 2));
if (calls[0]) console.log(calls[0].stack);
console.log(JSON.stringify(answer.metaExecution, null, 2));
