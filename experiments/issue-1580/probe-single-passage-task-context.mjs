// Supplied bounded context experiment; zero independent authorship credit.
import assert from 'node:assert/strict';
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
  if (calls.length > 60) throw Error('finite local transport limit');
  return { ok: false, status: 404, headers: { get: () => '' }, text: async () => '' };
};
// This changes only this disposable VM, never production files or the planner.
await evaluate(worker, `
  globalThis.contextProbeBindings = [];
  globalThis.contextProbeWithheld = [];
  globalThis.contextProbeEnabled = false;
  const originalCrateModule = crateModule;
  const originalExecutor = crateModule('crate/meta_executor.mjs');
  crateModule = (name) => name === 'crate/meta_executor.mjs'
    ? { ...originalExecutor, solveAnswer: contextProbeSolveAnswer } : originalCrateModule(name);
  const contextProbeSolveAnswer = async (request, answer, subTask, options = {}) => {
    const prompt = request.text;
    const runOptions = { ...options, withheldTests: contextProbeWithheld };
    if (!contextProbeEnabled) return originalExecutor.solveAnswer(request, answer, subTask, runOptions);
    const executor = originalExecutor;
    const passage = promptTextPassage(prompt);
    const occurrences = [];
    if (passage) {
      // The experiment accepts only one quoted passage. Multiple possible
      // bindings are refused, rather than assigning a longest/nearest guess.
      for (const [open, close] of FACTUAL_QA_QUOTE_PAIRS) {
        let at = prompt.indexOf(open);
        while (at >= 0) {
          const end = prompt.indexOf(close, at + open.length);
          if (end < 0) break;
          if (open !== "'" || (!/\\p{L}/u.test(prompt[at - 1] || '') && !/\\p{L}/u.test(prompt[end + 1] || ''))) {
            occurrences.push([at, end + close.length]);
          }
          at = prompt.indexOf(open, end + close.length);
        }
      }
    }
    return (await executor.solveAnswer({ text: prompt, language: detectLanguage(prompt) }, answer, async (text) => {
      const direct = await subTask(text);
      const test = executor.deriveTaskTest({ id: 'probe-leaf', relation: null, children: [] });
      if (executor.runTaskTest(test, direct, []).passed || !passage || occurrences.length !== 1 || !prompt.includes(text)) return direct;
      const local = promptTextPassage(text);
      const question = (local ? text.slice(local.end) : text).replace(/^[\\s.,;:!?。，；：！？)\\]—-]+/u, '').trim();
      if (!factualQaSurfaces(FACTUAL_QA_ROLE_INTERROGATIVE).some((surface) => factualQaLocate(normalizePrompt(question), surface) >= 0)) return direct;
      const bound = prompt.slice(passage.start, passage.end) + ' ' + question;
      const candidate = tryPromptTextQuestion(bound, normalizePrompt(bound));
      contextProbeBindings.push({ text, question, bound, directIntent: direct?.intent, candidate });
      return candidate || direct;
    }, runOptions));
  };
`);

const passage = '"The meeting moved from Tuesday to Thursday at 3 pm in room 204."';
const rows = [];
const cases = [
  { name: 'original', prompt: `Read this and answer: ${passage} When and where is the meeting?` },
  { name: 'held-out-facts', prompt: 'Read this and answer: "The concert moved from Monday to Friday at 8 pm in hall 517." When and where is the concert?' },
  { name: 'withheld-child', prompt: `Read this and answer: ${passage} When and where is the meeting?`, withheld: ['t0.3'] },
  { name: 'quoted-connective', prompt: 'Given the text: "The library and cafe moved to Friday at 7 pm in room 519." When and where is the library?' },
  { name: 'independent-arithmetic', prompt: `Read this and answer: ${passage} When is the meeting and calculate 2 + 2` },
  { name: 'two-independent-passages', prompt: 'Given the text: "The meeting moved to Thursday at 3 pm in room 204." When is the meeting and given the text: "The concert moved to Friday at 8 pm in hall 517." Where is the concert?' },
  { name: 'uncovered-subject', prompt: `Read this and answer: ${passage} When and where is the glacier?` },
  { name: 'standalone-arithmetic', prompt: 'Calculate 2 + 2' },
];
for (const item of cases) {
  const outcomes = [];
  for (const enabled of [false, true]) {
    worker.contextProbeEnabled = enabled;
    worker.contextProbeWithheld = item.withheld || [];
    worker.contextProbeBindings.length = 0;
    const before = calls.length;
    const answer = plain(await worker.solve(item.prompt, [], {}, {}, [], { metaSubTask: true }));
    const outcome = { enabled, answer, bindings: plain(worker.contextProbeBindings), localTransportCalls: calls.length - before };
    outcomes.push(outcome);
    console.log(JSON.stringify({ name: item.name, enabled, intent: answer.intent, content: answer.content, status: answer.metaExecution?.status, shape: answer.metaExecution?.shape, bindings: outcome.bindings.length, calls: outcome.localTransportCalls }));
  }
  rows.push({ ...item, outcomes });
}
const original = rows.find((row) => row.name === 'original');
assert.equal(original.outcomes[0].answer.metaExecution.status, 'unresolved');
assert.equal(original.outcomes[1].answer.metaExecution.status, 'resolved');
assert.equal(original.outcomes[1].bindings.length, 2);
assert.equal(rows.find((row) => row.name === 'withheld-child').outcomes[1].answer.metaExecution.status, 'unresolved');
assert.ok(rows.every((row) => row.outcomes.every((outcome) => outcome.localTransportCalls === 0)));
const sourceHashes = Object.fromEntries(['js/agentic/crate/meta_executor.mjs', 'js/worker/formal_ai_worker_solve.js', 'js/worker/formal_ai_worker_factual_qa.js'].map((path) => [path, createHash('sha256').update(readFileSync(path)).digest('hex')]));
const raw = Buffer.from(JSON.stringify({ schema: 'issue-1580-single-passage-context-experiment/v1', head: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(), sourceHashes, rows, calls, actualNetworkRequests: 0, productionModified: false, nativeExecuted: false, independentAuthorshipCredit: 0 }, null, 2) + '\n');
writeFileSync('experiments/issue-1580/evidence/checks/passage-routing-probe/single-passage-task-context.json.gz', gzipSync(raw), { flag: 'wx' });
