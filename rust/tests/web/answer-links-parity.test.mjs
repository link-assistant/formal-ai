// Native rust/src/engine.rs::answer_links_notation contracts over real logs.
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { after, before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { symbolicFromWorker, solveSymbolic } from '../../../js/server/solve.mjs';
import { answerLinksNotation, answerLinksFromLog, solverLogFromWorker } from '../../../js/server/answer-links.mjs';
import { EventLog, finalizeSimple } from '../../../js/server/memory-answer.mjs';
import { parseLino } from '../../../js/server/lino.mjs';
import { stableId } from '../../../js/agentic/crate/engine_stable_identifier.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { host as agenticHost } from '../../../js/agentic/host.mjs';
import { thinkingStepsFromEvents } from '../../../js/server/solver-trace.mjs';
import { answerDerivationId, fromLino } from '../../../js/agentic/crate/derivation.mjs';
const fields = notation => {
  const parsed = parseLino(notation);
  const answer = parsed.name.startsWith('answer_') ? parsed : parsed.children.find(node => node.name.startsWith('answer_'));
  assert.ok(answer, 'the actual answer record remains independently addressable');
  return new Map(answer.children.slice(0, 6).map(node => [node.name, node.value]));
};
let host, root;
before(async () => { host = new WorkerHost(); await host.boot(); host.context.fetch = () => { throw new Error('verified cache required'); }; root = mkdtempSync(path.join(tmpdir(), 'answer-links-parity-')); });
after(() => rmSync(root, { recursive: true, force: true }));

test('the six native fields have exact bytes and response-link thinking', () => {
  assert.equal(answerLinksNotation('Z', 'odd', 'Answer', [{kind:'response',payload:'response:odd'}], 'trace_known'), [
    'answer_prompt_af64174c860250cd', '  prompt "Z"', '  intent "odd"', '  answer "Answer"', '  trace "trace_known"',
    '  steps "step_0 response response:odd"', '  thinking_steps "step_0 deformalize high response response:odd"',
  ].join('\n'));
});

test('the unchanged native greeting keeps its answer root and independent worker derivation', async () => {
  const result = await host.solve('Hi');
  const derivation = result.derivation;
  const symbolic = symbolicFromWorker(result);
  assert.equal(symbolic.answer, 'Hi, how may I help you?');
  assert.equal(symbolic.links_notation.split('\n')[0], 'answer_prompt_09275f07b5bb95ba');
  const record = fields(symbolic.links_notation);
  assert.deepEqual([...record.keys()], ['prompt', 'intent', 'answer', 'trace', 'steps', 'thinking_steps']);
  assert.equal(record.get('prompt'), 'Hi');
  assert.equal(record.get('intent'), 'greeting');
  assert.equal(record.get('answer'), symbolic.answer);
  assert.equal(record.get('trace'), solverLogFromWorker(result).lastOf('trace').id);
  assert.ok(record.get('thinking_steps').endsWith('deformalize high response response:greeting'));
  assert.equal(symbolic.thinking_steps.at(-1).detail, symbolic.answer);
  assert.equal(result.derivation, derivation);
  assert.equal(symbolic.worker_steps.find(step => step.step === 'meta_reason').derivation, derivation);
});

test('actual arithmetic events retain order and the native computation cluster', async () => {
  const result = await host.solve('What is 2 + 2?');
  const symbolic = symbolicFromWorker(result);
  const record = fields(symbolic.links_notation);
  assert.equal(symbolic.answer, '2 + 2 = 4');
  assert.ok(record.get('steps').includes('calculation 2 + 2 = 4;'));
  assert.ok(record.get('steps').includes('calculation:engine link-calculator;'));
  assert.ok(record.get('thinking_steps').includes('compute_engine detailed calculation:engine link-calculator;'));
  assert.deepEqual(symbolic.thinking_steps.map(step => step.order), Array.from({length:10}, (_, index) => index));
});

test('actual earlier answers and user turns precede the impulse and affect trace identity', async () => {
  const previous = symbolicFromWorker(await host.solve('Hi')).answer;
  const history = [{role:'user',content:'Hi'}, {role:'assistant',content:previous}];
  const result = await host.solve('What is 2 + 2?', history);
  const symbolic = symbolicFromWorker(result, history);
  assert.equal(symbolic.answer, '2 + 2 = 4');
  const record = fields(symbolic.links_notation);
  assert.ok(record.get('steps').startsWith(`step_0 prior_turn:user Hi; step_1 prior_turn:assistant ${previous}; step_2 impulse What is 2 + 2?;`));
  const events = symbolic.solver_events;
  const trace = events.findLast(event => event.kind === 'trace');
  assert.equal(record.get('trace'), stableId('trace', `trace:${events.indexOf(trace)}:${trace.payload}`));
  assert.notEqual(record.get('trace'), solverLogFromWorker(result).lastOf('trace').id);
});

test('genuine captured subjects and full provenance survive the API event projection', async () => {
  for (const prompt of ['Cite a definition of associative memory from Wikipedia', 'Cite a definition of entropy from Wikipedia']) {
    const result = await host.solve(prompt);
    const symbolic = symbolicFromWorker(result);
    const events = symbolic.solver_events;
    const capture = events.find(event => event.kind === 'source:http');
    assert.ok(capture, 'a real verified retrieval is logged at its observation boundary');
    assert.match(capture.payload, /^https:\/\/en\.wikipedia\.org\/api\/rest_v1\/page\/summary\/\S+ fetched_at=[1-9][0-9]* sha256=[0-9a-f]{64} cached=true$/u);
    const url = capture.payload.split(' ')[0];
    assert.ok(symbolic.evidence_links.includes(`source:http:${capture.payload.replaceAll(' ', ':')}`));
    assert.ok(symbolic.evidence_links.includes(`cache_hit:${url}`));
    assert.ok(fields(symbolic.links_notation).get('steps').includes(`source:http ${capture.payload};`));
    assert.ok(fields(symbolic.links_notation).get('steps').includes('response response:source-qualified-definition;'));
    assert.ok(symbolic.answer.includes(url));
    const relevant = events.filter(event => event.kind.startsWith('source-qualified-definition:') || ['source:http','cache_hit','source'].includes(event.kind));
    assert.deepEqual(relevant.map(event => event.kind), ['source-qualified-definition:term','source:http','cache_hit','source-qualified-definition:source','source','source-qualified-definition:kind','source-qualified-definition:status']);
  }
});

test('an unresolved provider preserves observed status without inventing retrieval', async () => {
  const symbolic = symbolicFromWorker(await host.solve('Cite a definition of entropy from Unknown provider'));
  assert.equal(symbolic.answer, 'No verified definition of entropy from Unknown provider: missing-source.');
  assert.deepEqual(symbolic.solver_events.filter(event => event.kind.startsWith('source-qualified-definition:')).map(event => [event.kind,event.payload]), [
    ['source-qualified-definition:term','entropy'], ['source-qualified-definition:status','missing-source'],
  ]);
  assert.ok(!symbolic.solver_events.some(event => event.kind === 'source:http' || event.kind === 'cache_hit'));
});

test('memory finalization records the actual appended trace and final-answer thinking separately', () => {
  const log = new EventLog([{kind:'impulse',payload:'remember zebra'}, {kind:'memory:retrieved',payload:'a genuine fixture turn'}]);
  const symbolic = finalizeSimple('remember zebra', log, 'conversation_recall', 'response:recall', 'Remembered zebra.', 1);
  assert.equal(symbolic.answer, 'Remembered zebra.');
  const record = fields(symbolic.links_notation);
  assert.equal(record.get('trace'), log.lastOf('trace').id);
  assert.ok(record.get('steps').includes('step_1 memory:retrieved a genuine fixture turn;'));
  assert.ok(record.get('thinking_steps').endsWith('deformalize high response response:recall'));
  assert.equal(symbolic.thinking_steps.at(-1).detail, 'Remembered zebra.');
});

test('served derivations append independently and preserve real source fetches and prior turns', async () => {
  const history = [{role:'user',content:'Hi'}, {role:'assistant',content:'Hi, how may I help you?'}];
  const symbolic = await solveSymbolic({worker:host,derivationRoot:root}, 'Cite a definition of entropy from Wikipedia', history);
  assert.ok(symbolic.links_notation.startsWith('answer_prompt_'));
  const file = path.join(root, 'data/cache/derivations', `${answerDerivationId(symbolic.answer)}.lino`);
  const bytes = readFileSync(file,'utf8');
  assert.ok(symbolic.links_notation.endsWith(`\n${bytes}`));
  const record = fromLino(bytes);
  assert.equal(record.fetches.length, 1);
  assert.equal(record.fetches[0].url, 'https://en.wikipedia.org/api/rest_v1/page/summary/entropy');
  assert.equal(record.fetches[0].sha256, symbolic.solver_events.find(event => event.kind === 'source:http').payload.match(/sha256=([0-9a-f]{64})/u)[1]);
  assert.ok(fields(symbolic.links_notation).get('steps').startsWith('step_0 prior_turn:user Hi; step_1 prior_turn:assistant Hi, how may I help you?;'));
});

test('absent actual log data remains absent and arbitrary Unicode uses native escaping', () => {
  assert.equal(symbolicFromWorker({intent:'unknown',content:'x',derivation:'unrelated'}).links_notation, '');
  assert.equal(answerLinksFromLog(new EventLog([{kind:'impulse',payload:'x'}]),'unknown','x'), '');
  const value = answerLinksNotation('Δ\n"quoted"\t\\', 'odd', 'A\rB', [], 'trace');
  assert.ok(value.includes('  prompt \'Δ\\n"quoted"\\t\\\\\''));
  assert.ok(value.includes('  answer "A\\rB"'));
  assert.equal(thinkingStepsFromEvents([]).length, 0);
});

test('the installed production agentic host preserves supplied history in its symbolic solve', async () => {
  await installNodeHost(host);
  const history = [{role:'user',content:'Hi'}, {role:'assistant',content:'Hi, how may I help you?'}];
  const symbolic = await agenticHost().solve('What is 2 + 2?', history);
  assert.equal(symbolic.answer, '2 + 2 = 4');
  assert.ok(fields(symbolic.links_notation).get('steps').startsWith('step_0 prior_turn:user Hi; step_1 prior_turn:assistant Hi, how may I help you?; step_2 impulse What is 2 + 2?;'));
});
