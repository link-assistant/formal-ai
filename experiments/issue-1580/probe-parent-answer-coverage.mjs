// Supplied bounded context experiment; zero independent authorship credit.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { gzipSync } from 'node:zlib';
import { execFileSync } from 'node:child_process';
import { createWorkerContext, evaluate, plain } from '../../rust/tests/web/support/browser-runtime.mjs';

const worker = createWorkerContext();
await evaluate(worker, 'loadSeed()');
// Every change below applies only to one disposable worker VM. Production,
// generated artifacts, shared fixtures and their expected traces stay intact.
let factory = readFileSync('js/worker/formal_ai_worker_crate_meta_executor.js', 'utf8');
const transform = (before, after) => {
  assert.equal(factory.split(before).length, 2, before);
  factory = factory.replace(before, after);
};
transform('answer ? { intent: answer.intent, content: answer.content, confidence: answer.confidence } : null;',
  'answer ? { intent: answer.intent, content: answer.content, confidence: answer.confidence, evidence: answer.evidence } : null;');
transform('host.canCompose?.(node)', 'host.canCompose?.(node, childOutcomes)');
transform("canCompose: (task) => task.depth !== 0 || ruleValues('unresolved-intent').includes(answer.intent),",
  "canCompose: (task, outcomes) => task.depth !== 0 || ruleValues('unresolved-intent').includes(answer.intent) || (globalThis.parentCoverageProbeEnabled && answer.intent === 'calculation' && outcomes.some(outcome => outcome.candidate?.intent === 'prompt_text_answer') && outcomes.every(outcome => ['prompt_text_answer', 'calculation'].includes(outcome.candidate?.intent))),");
transform('const verdict = runTaskTest(test, candidate, childOutcomes);', `const verdict = runTaskTest(test, candidate, childOutcomes);
    if (globalThis.parentCoverageProbeEnabled) {
      const coverage = globalThis.parentCoverageProbeCheck(node, test, candidate, childOutcomes);
      if (coverage.applies) {
        verdict.executed.push('experiment:parent_coverage');
        if (!coverage.passed) {
          verdict.passed = false;
          verdict.failed ||= 'experiment:parent_coverage';
        }
        globalThis.parentCoverageProbeEvents.push({task: node.id, method: candidate.method, intent: candidate.intent, childIntents: childOutcomes.map(outcome => ({task: outcome.task, status: outcome.status, intent: outcome.candidate?.intent})), ...coverage});
      }
    }`);
await evaluate(worker, factory);
await evaluate(worker, `
  crateInstances.delete('crate/meta_executor.mjs');
  globalThis.parentCoverageProbeEnabled = false;
  globalThis.parentCoverageProbeEvents = [];
  globalThis.parentCoverageProbeCheck = (node, test, candidate, outcomes) => {
    const resolved = outcomes.filter(outcome => outcome.status === 'resolved');
    const selected = test.requires === 'any' ? resolved.slice(0, 1) : resolved;
    const passage = selected.filter(outcome => outcome.candidate?.intent === 'prompt_text_answer');
    if (!node.children.length || !passage.length) return {applies: false, passed: true};
    const required = selected.map(outcome => outcome.task);
    if (candidate.method === 'composition') {
      const passed = required.every(task => candidate.parts?.includes(task));
      return {applies: true, passed, required, reason: passed ? 'composition-retains-required-parts' : 'missing-composition-part'};
    }
    const facts = value => Array.isArray(value?.evidence) ? value.evidence : [];
    const parent = facts(candidate.source);
    const roles = values => values.filter(value => value.startsWith('prompt_text:asked:')).flatMap(value => value.slice('prompt_text:asked:'.length).split(','));
    const statements = values => values.filter(value => value.startsWith('prompt_text:statement:'));
    let passed = candidate.intent === 'prompt_text_answer';
    const obligations = [];
    for (const child of passage) {
      const evidence = facts(child.candidate.source);
      const asked = roles(evidence);
      const grounded = statements(evidence);
      const covers = asked.length > 0 && grounded.length > 0
        && asked.every(role => roles(parent).includes(role))
        && grounded.every(statement => statements(parent).includes(statement));
      obligations.push({task: child.task, asked, grounded, covers});
      passed &&= covers;
    }
    // A direct passage answer cannot discharge an independent calculation.
    passed &&= selected.every(outcome => outcome.candidate.intent === 'prompt_text_answer');
    return {applies: true, passed, required, obligations, reason: passed ? 'same-statement-and-question-roles' : 'parent-does-not-cover-child-evidence'};
  };
`);
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
      return parentCoverageProbeEnabled && candidate?.intent === 'prompt_text_gap' ? direct : (candidate || direct);
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
  { name: 'concrete-output-contract', prompt: 'Write a Python function to add two integers and include a test.', mockContract: {intent: 'write_program', content: '', confidence: 0} },
];
for (const item of cases) {
  const outcomes = [];
  for (const [mode, enabled, coverage] of [['baseline', false, false], ['binding', true, false], ['coverage', true, true]]) {
    worker.contextProbeEnabled = enabled;
    worker.parentCoverageProbeEnabled = coverage;
    worker.parentCoverageProbeEvents.length = 0;
    worker.contextProbeWithheld = item.withheld || [];
    worker.contextProbeBindings.length = 0;
    const before = calls.length;
    // This control isolates the executor contract with supplied candidates;
    // it does not claim end-to-end code generation or a concrete program.
    worker.contextProbeControlPrompt = item.prompt;
    worker.contextProbeControlAnswer = item.mockContract || null;
    const answer = item.mockContract
      ? plain((await evaluate(worker, `crateModule('crate/meta_executor.mjs').solveAnswer(
        {text: contextProbeControlPrompt, language: 'en'}, contextProbeControlAnswer,
        async text => ({intent: 'understood', content: text, confidence: 1}))`)).answer)
      : plain(await worker.solve(item.prompt, [], {}, {}, [], { metaSubTask: true }));
    const outcome = { mode, enabled, coverage, coverageEvents: plain(worker.parentCoverageProbeEvents), answer, bindings: plain(worker.contextProbeBindings), localTransportCalls: calls.length - before };
    outcomes.push(outcome);
    console.log(JSON.stringify({ name: item.name, mode, intent: answer.intent, content: answer.content, status: answer.metaExecution?.status, shape: answer.metaExecution?.shape, bindings: outcome.bindings.length, calls: outcome.localTransportCalls }));
  }
  rows.push({ ...item, outcomes });
}
const baseline = name => rows.find(row => row.name === name).outcomes[0].answer;
const candidate = name => rows.find(row => row.name === name).outcomes[2].answer;
const resolved = name => candidate(name).metaExecution?.status === 'resolved';
const qualification = [
  ['original', resolved('original')],
  ['held-out-facts', resolved('held-out-facts')],
  ['withheld-child', !resolved('withheld-child')],
  ['quoted-connective', resolved('quoted-connective')],
  ['independent-arithmetic', resolved('independent-arithmetic') && /Thursday/.test(candidate('independent-arithmetic').content) && /2 \+ 2 = 4/.test(candidate('independent-arithmetic').content)],
  ['two-independent-passages', resolved('two-independent-passages') && /Thursday/.test(candidate('two-independent-passages').content) && /hall 517/.test(candidate('two-independent-passages').content)],
  ['uncovered-subject', !resolved('uncovered-subject')],
  ['standalone-arithmetic', candidate('standalone-arithmetic').intent === baseline('standalone-arithmetic').intent && candidate('standalone-arithmetic').content === baseline('standalone-arithmetic').content],
  ['concrete-output-contract', candidate('concrete-output-contract').intent === baseline('concrete-output-contract').intent && candidate('concrete-output-contract').content === baseline('concrete-output-contract').content && candidate('concrete-output-contract').metaExecution.method !== 'composition'],
];
assert.ok(rows.every(row => row.outcomes.every(outcome => outcome.localTransportCalls === 0)));
console.log(JSON.stringify({qualification, passed: qualification.filter(([, passed]) => passed).length, total: qualification.length, productionAdopted: false, statementMetadataIsGeneralProof: false}));
const sourceHashes = Object.fromEntries(['js/agentic/crate/meta_executor.mjs', 'js/worker/formal_ai_worker_solve.js', 'js/worker/formal_ai_worker_factual_qa.js'].map((path) => [path, createHash('sha256').update(readFileSync(path)).digest('hex')]));
const raw = Buffer.from(JSON.stringify({ schema: 'issue-1580-parent-answer-coverage-experiment/v1', qualification, statementMetadataIsGeneralProof: false, head: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(), sourceHashes, rows, calls, actualNetworkRequests: 0, productionModified: false, nativeExecuted: false, independentAuthorshipCredit: 0 }, null, 2) + '\n');
writeFileSync('experiments/issue-1580/evidence/checks/passage-routing-probe/parent-answer-coverage.json.gz', gzipSync(raw), { flag: 'wx' });
