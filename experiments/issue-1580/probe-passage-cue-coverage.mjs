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
transform('const mask = quotedMask(original);', `const mask = quotedMask(original);
  if (globalThis.parentCoverageProbePrefixEnabled) {
    const openings = ruleValues('quote-pair').map(pair => Array.from(pair)[0]);
    for (const cue of globalThis.parentCoverageProbeCueSurfaces) {
      const chars = Array.from(cue).map(character => character.toLowerCase());
      for (let at = 0; at < original.length; at += 1) {
        if (!markerMatches(lowered, original, mask, at, chars, containsCjk(cue))) continue;
        let next = at + chars.length;
        while (next < original.length && !openings.includes(original[next]) && normalizePrompt(original[next]) === '') next += 1;
        if (!openings.includes(original[next])) continue;
        mask.fill(true, at, at + chars.length);
        globalThis.parentCoverageProbeCueEvents.push({cue, start: at, end: at + chars.length});
      }
    }
  }`);
transform("canCompose: (task) => task.depth !== 0 || ruleValues('unresolved-intent').includes(answer.intent),",
  "canCompose: (task, outcomes) => task.depth !== 0 || ruleValues('unresolved-intent').includes(answer.intent) || (globalThis.parentCoverageProbeEnabled && ['calculation', 'prompt_text_gap', 'prompt_text_answer'].includes(answer.intent) && outcomes.some(outcome => outcome.candidate?.intent === 'prompt_text_answer') && outcomes.every(outcome => ['prompt_text_answer', 'calculation'].includes(outcome.candidate?.intent))),");
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
  globalThis.parentCoverageProbePrefixEnabled = false;
  globalThis.parentCoverageProbeCueSurfaces = factualQaSurfaces(FACTUAL_QA_ROLE_TEXT_CUE);
  globalThis.parentCoverageProbeCueEvents = [];
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
  { name: 'operation-before-passage-cue', prompt: `Calculate 2 + 2 and read this and answer: ${passage} When is the meeting?` },
  { name: 'concrete-output-contract', prompt: 'Write a Python function to add two integers and include a test.', mockContract: {intent: 'write_program', content: '', confidence: 0} },
];
for (const item of cases) {
  const outcomes = [];
  for (const [mode, enabled, coverage] of [['baseline', false, false], ['binding', true, false], ['coverage', true, true]]) {
    worker.contextProbeEnabled = enabled;
    worker.parentCoverageProbeEnabled = coverage;
    worker.parentCoverageProbePrefixEnabled = coverage;
    worker.parentCoverageProbeCueEvents.length = 0;
    worker.parentCoverageProbeEvents.length = 0;
    worker.contextProbeWithheld = item.withheld || [];
    if (coverage && item.withheld?.length) {
      worker.contextProbeWithheldPrompt = item.prompt;
      // Keep the same semantic last-child obligation after the cue repairs
      // the tree from three children to two; do not withhold a missing id.
      worker.contextProbeWithheld = plain(await evaluate(worker, `(() => {
        const executor = crateModule('crate/meta_executor.mjs');
        const root = executor.planTasks(contextProbeWithheldPrompt, detectLanguage(contextProbeWithheldPrompt), executor.decompositionBounds());
        const leaves = node => node.children.length ? node.children.flatMap(leaves) : [node.id];
        return leaves(root).slice(-1);
      })()`));
    }
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
    const outcome = { mode, enabled, coverage, cueEvents: plain(worker.parentCoverageProbeCueEvents), withheldTests: plain(worker.contextProbeWithheld), coverageEvents: plain(worker.parentCoverageProbeEvents), answer, bindings: plain(worker.contextProbeBindings), localTransportCalls: calls.length - before };
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
  ['withheld-child', !resolved('withheld-child') && candidate('withheld-child').metaExecution.events.some(event => event.includes('reason=no_test'))],
  ['quoted-connective', resolved('quoted-connective')],
  ['independent-arithmetic', resolved('independent-arithmetic') && /Thursday/.test(candidate('independent-arithmetic').content) && /2 \+ 2 = 4/.test(candidate('independent-arithmetic').content)],
  ['two-independent-passages', resolved('two-independent-passages') && /Thursday/.test(candidate('two-independent-passages').content) && /hall 517/.test(candidate('two-independent-passages').content)],
  ['uncovered-subject', !resolved('uncovered-subject')],
  ['standalone-arithmetic', candidate('standalone-arithmetic').intent === baseline('standalone-arithmetic').intent && candidate('standalone-arithmetic').content === baseline('standalone-arithmetic').content],
  ['operation-before-passage-cue', resolved('operation-before-passage-cue') && /Thursday/.test(candidate('operation-before-passage-cue').content) && /2 \+ 2 = 4/.test(candidate('operation-before-passage-cue').content)],
  ['concrete-output-contract', candidate('concrete-output-contract').intent === baseline('concrete-output-contract').intent && candidate('concrete-output-contract').content === baseline('concrete-output-contract').content && candidate('concrete-output-contract').metaExecution.method !== 'composition'],
];
assert.ok(rows.every(row => row.outcomes.every(outcome => outcome.localTransportCalls === 0)));
// Execute all 25 existing table-host fixtures in the patched VM and compare
// their complete observable results, including every event, unchanged.
worker.parentCoverageProbeEnabled = true;
worker.parentCoverageProbePrefixEnabled = true;
const fixture = JSON.parse(readFileSync('rust/tests/fixtures/meta-executor-parity.json', 'utf8'));
const fixtureCompatibility = [];
for (const entry of fixture.cases) {
  worker.contextProbeFixture = entry;
  const actual = plain(await evaluate(worker, `(async () => {
    const executor = crateModule('crate/meta_executor.mjs');
    const entry = contextProbeFixture;
    const own = (table, text) => Object.hasOwn(table, text) ? table[text] : null;
    const run = await executor.executeProcedure({text: entry.text, language: entry.language}, {
      draft: async text => own(entry.answers, text), cached: async text => own(entry.cached, text),
    }, {bounds: entry.bounds, withheldTests: entry.withheld});
    return {shape: executor.decompositionShape(run.root), leaves: executor.taskLeaves(run.root),
      status: run.outcome.status, content: run.outcome.candidate?.content ?? null,
      events: run.events.map(event => event.kind + ' ' + event.payload)};
  })()`));
  let passed = true;
  try { assert.deepEqual(actual, entry.expected); } catch { passed = false; }
  fixtureCompatibility.push({name: entry.name, passed, ...(passed ? {} : {actual, expected: entry.expected})});
}
const structuralControls = [];
for (const text of ['Read this and answer', 'Read this and answer: Count the letters and report the vowels']) {
  worker.contextProbeStructuralText = text;
  worker.parentCoverageProbePrefixEnabled = false;
  const baseline = plain(await evaluate(worker, "crateModule('crate/meta_executor.mjs').splitRequest(contextProbeStructuralText, 'en')"));
  worker.parentCoverageProbePrefixEnabled = true;
  const candidate = plain(await evaluate(worker, "crateModule('crate/meta_executor.mjs').splitRequest(contextProbeStructuralText, 'en')"));
  let passed = true;
  try { assert.deepEqual(candidate, baseline); } catch { passed = false; }
  structuralControls.push({text, baseline, candidate, passed});
}
console.log(JSON.stringify({fixtureCompatibility, structuralControls}));
console.log(JSON.stringify({qualification, passed: qualification.filter(([, passed]) => passed).length, total: qualification.length, productionAdopted: false, statementMetadataIsGeneralProof: false}));
const sourceHashes = Object.fromEntries(['js/agentic/crate/meta_executor.mjs', 'js/worker/formal_ai_worker_solve.js', 'js/worker/formal_ai_worker_factual_qa.js', 'js/worker/formal_ai_worker_crate_meta_executor.js', 'data/seed/meanings-facts.lino', 'data/seed/text-formalization.lino', 'rust/tests/fixtures/meta-executor-parity.json'].map((path) => [path, createHash('sha256').update(readFileSync(path)).digest('hex')]));
const raw = Buffer.from(JSON.stringify({ schema: 'issue-1580-passage-cue-coverage-experiment/v2', probeSourceSha256: createHash('sha256').update(readFileSync(new URL(import.meta.url))).digest('hex'), qualification, fixtureCompatibility, structuralControls, seededCueSurfaces: plain(worker.parentCoverageProbeCueSurfaces), statementMetadataIsGeneralProof: false, head: execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim(), sourceHashes, rows, calls, actualNetworkRequests: 0, productionModified: false, nativeExecuted: false, independentAuthorshipCredit: 0 }, null, 2) + '\n');
writeFileSync('experiments/issue-1580/evidence/checks/passage-routing-probe/passage-cue-coverage-normalized.json.gz', gzipSync(raw), { flag: 'wx' });
