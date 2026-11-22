// Issue #1550: the census of the answer paths (acceptance criterion c) and the
// turn-derived records (acceptance criterion d).
//
// (c) Both production entries end in the recursive procedure: the native
// `solve_with_history_probability_store_and_intent_cache` has no exit before or
// after its cascade closure, runs the closure through the controlled output
// hook (`controlled_solve`, issue #1584) and hands the hook's draft to
// `run_meta_procedure`; the worker's `solve` has exactly two exits (the
// diagnostic rerun, which copies the inner turn's `metaExecution`, and the
// procedure's run). The native `rust/tests/unit/meta_executor_census.rs`
// solves every specification prompt of the fixture and checks the trace; this
// file checks the fixture is current and runs a sample through the worker.
// `scripts/check-specification-in-javascript.mjs` runs the full corpus census.
//
// (d) `record_candidates`, `record_validation`, `record_selection` and their
// JavaScript twins hold no constant that is not derived from the turn.
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { before, test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import {
  JS_ENTRY,
  RUST_ENTRY,
  censusFixtureText,
  jsEntryCensus,
  rustEntryCensus,
  solveImplCallersOutsideSolve,
} from './helpers/issue-1550-procedure-census.mjs';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const FIXTURE = 'rust/tests/fixtures/procedure-census-prompts.json';
const read = (file) => readFileSync(new URL(file, new URL('../../../', import.meta.url)), 'utf8');
// The cascade's methods inside the closure; each returns the closure's draft,
// which the procedure records as a candidate. Change only with a method the
// procedure records (never with an exit of the entry).
const CASCADE_METHOD_RETURNS = 20;

let host;
before(async () => {
  host = new WorkerHost();
  await installNodeHost(host);
});

test('the native entry has no exit around the procedure', () => {
  const census = rustEntryCensus(read(RUST_ENTRY));
  assert.equal(census.exitsBefore, 0, `${RUST_ENTRY}: an exit before the cascade skips the procedure`);
  assert.equal(census.exitsAfter, 0, `${RUST_ENTRY}: an exit after the cascade skips the procedure`);
  assert.equal(census.procedureAfter, true, `${RUST_ENTRY}: the cascade draft must reach run_meta_procedure`);
  assert.equal(
    census.cascadeReturns,
    CASCADE_METHOD_RETURNS,
    `${RUST_ENTRY}: the cascade's returns changed; a new early return must be a method whose draft the procedure records`,
  );
});

test('the worker entry exits only through the procedure', () => {
  const census = jsEntryCensus(read(JS_ENTRY));
  assert.deepEqual(census.exits, [
    'projectControlledAnswer(diagnostic, turn.config, options)',
    'projectControlledAnswer(metaAttachLearned(await metaRunProcedure(prompt, await metaResolveImpasse(prompt, answer, prefs), subTask)), turn.config, options)',
  ], `${JS_ENTRY}: solve gained an exit that may skip metaRunProcedure`);
  // One drafting function: the turn's draft, the canonical reading of a task
  // paraphrase (issue #1584) and every sub-task all reach solveImpl through it.
  assert.equal(census.solveImplCalls, 1);
  const files = readdirSync(new URL('../../../js/worker/', import.meta.url))
    .filter((name) => name.endsWith('.js'))
    .map((name) => `js/worker/${name}`);
  assert.deepEqual(solveImplCallersOutsideSolve(ROOT, files), [], 'solveImpl is called outside solve');
});

test('controlled-output rewrites retain the recursive procedure trace', {timeout: 30000}, async () => {
  const answer = await host.solve('Rewrite in max-precision mode', [
    {role: 'assistant', content: 'I know everything.'},
  ]);
  assert.equal(answer.intent, 'cnl_paraphrase');
  assert.equal(answer.content, 'Maximum precision uses specific words. Look up unfamiliar words in a dictionary to check their exact meaning.\n\nI am omniscient.');
  assert.ok(answer.metaExecution, 'rewrites must pass through the recursive procedure');
  assert.deepEqual(answer.toolCalls, []);
});

test('controlled dialogue answers retain the recursive procedure trace', {timeout: 30000}, async () => {
  const answer = await host.solve('Is a cat an animal?', []);
  assert.equal(answer.intent, 'cnl_answer');
  assert.equal(answer.content, 'Yes. Every cat is an animal.');
  assert.ok(answer.metaExecution, 'controlled dialogue answers must pass through the recursive procedure');
});

test('the census fixture lists every prompt of the specification corpus', () => {
  assert.equal(
    read(FIXTURE),
    censusFixtureText(ROOT),
    `${FIXTURE} is stale: run node experiments/issue-1550/census-fixture.mjs --write`,
  );
});

test('a sample of the corpus prompts is answered through the procedure', async () => {
  const { prompts } = JSON.parse(read(FIXTURE));
  const sample = prompts.filter((_, index) => index % 8 === 0);
  assert.ok(sample.length > 0);
  const bypassed = [];
  for (const prompt of sample) {
    const answer = await host.solve(prompt);
    if (!answer?.metaExecution) bypassed.push(prompt);
  }
  assert.deepEqual(bypassed, [], 'answers without the recursive procedure');
});

const FORBIDDEN = [
  'LinkLight', 'Doublet', 'FormalLeaf', 'primary', 'secondary', 'accepted_without_extra_constraints',
  'smallest_sufficient', 'no_prime_in_range', 'prime_between_',
];

/** The text of `fn name(` up to the closing brace at the start of a line. */
function rustFn(source, name) {
  const start = source.search(new RegExp(`\\bfn ${name}\\b`, 'u'));
  assert.ok(start >= 0, `fn ${name} is gone`);
  const end = source.indexOf('\n}\n', start);
  return source.slice(start, end);
}

/** The text of a JavaScript function up to the closing brace at the start of a line. */
function jsFn(source, name) {
  const start = source.search(new RegExp(`function ${name}\\(`, 'u'));
  assert.ok(start >= 0, `function ${name} is gone`);
  return source.slice(start, source.indexOf('\n}\n', start));
}

test('the candidate, validation and selection records hold no constant that is not derived from the turn', () => {
  const helpers = read('rust/src/solver_helpers/mod.rs');
  const executor = read('rust/src/meta_executor.rs');
  const bodies = {
    record_candidates: rustFn(helpers, 'record_candidates'),
    record_validation: rustFn(helpers, 'record_validation'),
    record_answer_test: rustFn(helpers, 'record_answer_test'),
    record_selection: rustFn(helpers, 'record_selection'),
    answer_validation: rustFn(executor, 'answer_validation'),
    selection_record: rustFn(executor, 'selection_record'),
    solverPrimeIntervalValidation: jsFn(read('js/worker/formal_ai_worker_candidate_validation.js'), 'solverPrimeIntervalValidation'),
    solverEventLog: jsFn(read('js/worker/formal_ai_worker_solver_events.js'), 'solverEventLog'),
    answerValidation: jsFn(read('js/agentic/crate/meta_executor.mjs'), 'answerValidation'),
    selectionRecord: jsFn(read('js/agentic/crate/meta_executor.mjs'), 'selectionRecord'),
  };
  for (const [name, body] of Object.entries(bodies)) {
    for (const literal of FORBIDDEN) {
      assert.ok(!body.includes(literal), `${name} holds the constant ${literal}`);
    }
  }
  const synthesis = read('rust/src/solver_synthesis.rs');
  assert.doesNotMatch(synthesis, /\b(?:1\.15|1\.05|0\.9)\b/u, 'solver_synthesis.rs holds a fixed composition prior');
});

test('the records are computed from the turn: executed tests and their verdicts', async () => {
  const executor = await import('../../../js/agentic/crate/meta_executor.mjs');
  assert.equal(executor.answerValidation('greeting', 'Hello!'), 'answered(greeting)=true');
  assert.equal(executor.answerValidation('unknown', ''), 'answered(unknown)=false');
  assert.equal(
    executor.selectionRecord(['number_property_prime(15)=false', 'number_property_prime(17)=true'], 'x'),
    '17=1/2',
  );
  assert.equal(executor.selectionRecord(['answered(unknown)=false'], 'unknown'), 'unknown=0/1');
  const answer = await host.solve('Find a prime number between 14 and 20');
  assert.equal(answer.content, '17');
  const validations = answer.solverEvents
    .filter((event) => event.kind === 'validation')
    .map((event) => event.payload);
  assert.deepEqual(validations, [14, 15, 16, 17].map((number) => `number_property_prime(${number})=${number === 17}`));
});
