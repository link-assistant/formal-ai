// Actual append-site provenance survives native ordering and trace curation.
import assert from 'node:assert/strict';
import test from 'node:test';
import { hasHost, installHost } from '../../../js/agentic/host.mjs';
import { parseLino, readRepoFile } from '../../../js/server/lino.mjs';
import { nativeSolverLog } from '../../../js/agentic/crate/solver_event_projection.mjs';
import { describeTurn } from '../../../js/server/debug-stage.mjs';
import { symbolicFromWorker } from '../../../js/server/solve.mjs';
import { localizeThinkingSteps } from '../../../js/server/thinking.mjs';
import { thinkingStepsFromEvents } from '../../../js/server/solver-trace.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

const host = new WorkerHost();
if (!hasHost()) installHost({ readText: readRepoFile, parseLino });
const options = { timeout: 15000 };
const answerSites = {
  rust: ['rust/src/solver_helpers/mod.rs', 'record_answer_test'],
  js: ['js/worker/formal_ai_worker_solver_events.js', 'solverEventLog'],
};
const candidateSites = {
  rust: ['rust/src/solver_helpers/mod.rs', 'record_validation'],
  js: ['js/worker/formal_ai_worker_candidate_validation.js', 'solverPrimeIntervalValidation'],
};

function verifySites(steps, expected) {
  const index = steps.findIndex((step) => step.source_event === 'validation');
  assert.ok(index >= 0);
  const locations = describeTurn(steps).stages[index];
  for (const [runtime, [path, symbol]] of Object.entries(expected)) {
    const site = locations[runtime];
    assert.ok(site, runtime);
    assert.equal(site.path, path);
    assert.equal(site.symbol, symbol);
    assert.ok(site.line > 0);
    assert.ok(site.excerpt.includes(symbol));
  }
}

test('origin survives ordering and localization without changing trace identity', options, () => {
  const event = { kind: 'validation', payload: 'answered(calculation)=true', source_origin: 'validation:answer' };
  const input = { rawSolverEvents: [event] };
  const projected = nativeSolverLog(input);
  assert.deepEqual(projected, [event]);
  const traced = thinkingStepsFromEvents(projected);
  const plain = thinkingStepsFromEvents([{ kind: event.kind, payload: event.payload }]);
  assert.equal(traced[0].source_origin, event.source_origin);
  assert.equal(localizeThinkingSteps(traced)[0].source_origin, event.source_origin);
  const { source_origin, ...unchanged } = traced[0];
  assert.deepEqual(unchanged, plain[0]);
  assert.equal('source_origin' in plain[0], false);
  assert.deepEqual(input.rawSolverEvents, [event]);
});

test('answer validation names the appender even when another validator was considered', options, async () => {
  for (const prompt of ['What is 2 + 2?', 'What is 9 * 7?']) {
    const result = await host.solve(prompt);
    const events = result.rawSolverEvents.filter((event) => event.kind === 'validation');
    assert.equal(events.length, 1);
    assert.equal(events[0].source_origin, 'validation:answer');
    assert.equal(events[0].payload, 'answered(calculation)=true');
    assert.ok(result.evidence.some((link) => link.startsWith('validation:')));
    const steps = symbolicFromWorker(result).thinking_steps;
    assert.equal(steps.find((step) => step.source_event === 'validation').source_origin, 'validation:answer');
    verifySites(steps, answerSites);
  }
});

test('candidate validation keeps its actual source and every executed verdict', options, async () => {
  for (const prompt of ['Pick a prime between 14 and 18', 'Choose a prime between 8 and 12']) {
    const result = await host.solve(prompt);
    const events = result.rawSolverEvents.filter((event) => event.kind === 'validation');
    assert.ok(events.length > 1);
    assert.ok(events.every((event) => event.source_origin === 'validation:candidate'));
    assert.ok(events.slice(0, -1).every((event) => event.payload.endsWith('=false')));
    assert.ok(events.at(-1).payload.endsWith('=true'));
    assert.ok(result.evidence.some((link) => link.startsWith('validation:')));
    verifySites(symbolicFromWorker(result).thinking_steps, candidateSites);
  }
});

test('an unknown recorded origin is unavailable rather than a guessed emitter', options, () => {
  const steps = thinkingStepsFromEvents([{ kind: 'validation', payload: 'executed=true', source_origin: 'validation:unregistered' }]);
  assert.equal(steps[0].source_origin, 'validation:unregistered');
  assert.deepEqual(describeTurn(steps).stages, [{ rust: null, js: null }]);
});

test('origin metadata belongs only to the event that emitted it', options, () => {
  const steps = thinkingStepsFromEvents([
    { kind: 'impulse', payload: 'question', source_origin: 'impulse:unregistered' },
    { kind: 'validation', payload: 'executed=true' },
  ]);
  assert.equal(steps[0].source_origin, 'impulse:unregistered');
  assert.equal('source_origin' in steps[1], false);
});
