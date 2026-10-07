// Issue #1184 R1184-9, browser leg: the web UI's worker records a derivation
// for every answer and answers `explain <answer id>` the way the JavaScript
// server's `explain` request and `formal-ai explain` do.
//
// The worker (js/worker/formal_ai_worker_derivation.js) persists through an
// injected store; in the browser that store is the app's memory event log
// (js/memory.js): the record leaves the worker as the answer's
// `derivationRecord`, the app appends it as a `derivation` event, and the next
// request's `memoryEvents` snapshot reads it back. Every expected value here is
// computed by the shared JavaScript root, js/agentic/crate/derivation.mjs, so
// the worker and the server cannot write different records or wording.

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { createWorkerContext, evaluate, plain } from './support/browser-runtime.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import {
  RENDER_EMIT_KIND,
  answerDerivationId,
  explainText,
  fromLino,
  missMessage,
  recordFor,
  storePath,
  toLino,
} from '../../../js/agentic/crate/derivation.mjs';

let worker;
const posted = [];

before(async () => {
  await installNodeHost(new WorkerHost());
  worker = createWorkerContext({
    location: { href: 'http://localhost/worker/formal_ai_worker.js', search: '?jsfallback=1' },
    postMessage: (message) => posted.push(plain(message)),
  });
  await evaluate(worker, 'init()');
});

/** One web UI request through the worker's real `onmessage`. */
async function ask(prompt, memoryEvents = []) {
  const requestId = `derivation-${posted.length}`;
  await worker.onmessage({ data: { prompt, requestId, history: [], memoryEvents } });
  const reply = posted.find((message) => message.kind === 'message' && message.requestId === requestId);
  assert.ok(reply, `no reply for ${prompt}`);
  return reply;
}

/** The `derivation` event the app appends for a worker `derivationRecord`. */
const storedEvent = (record) => ({ kind: 'derivation', role: 'system', inputs: record.path, content: record.text });

test('every web UI answer links, records and hands its derivation to the memory store', async () => {
  const reply = await ask('2 + 2');
  const answerId = answerDerivationId(reply.content);
  assert.ok(reply.evidence.includes(`derivation:${answerId}`), JSON.stringify(reply.evidence));
  assert.ok(reply.evidence.includes(storePath('', answerId)));
  assert.equal(reply.derivationRecord.action, 'derivation');
  assert.equal(reply.derivationRecord.path, `data/cache/derivations/${answerId}.lino`);
  const record = fromLino(reply.derivationRecord.text);
  assert.equal(record.answer_id, answerId);
  assert.equal(record.rendering, `answer_id=${answerId};format=text`);
  assert.equal(reply.derivationRecord.text, toLino(record));
});

test('the worker projects the same record as the crate module for the same log', () => {
  const events = [
    { kind: 'web_search:request', payload: 'how to compile a Kotlin program' },
    { kind: 'source:http', payload: 'https://kotlinlang.org/docs/command-line.html fetched_at=2026-09-29T00:00:00Z sha256=9f86 cached=false' },
    { kind: 'source:http', payload: 'url=https://example.com/kotlin;fetched_at=2026-09-29T00:01:00Z;sha256=2c26;catalog_match=none' },
    { kind: 'decompose:part', payload: 'part=main' },
    { kind: 'recompose:bind', payload: 'main=println' },
    { kind: 'verify:evidence', payload: 'evidence_id=evidence_1;command=kotlinc main.kt;exit=0' },
    { kind: 'verify:evidence', payload: 'evidence_id=evidence_2;command=kotlin MainKt;exit=none' },
    { kind: 'grammar_correction', payload: 'rule=subject_verb_agreement;from=he go;to=he goes' },
    { kind: 'register_rewrite', payload: 'gonna=going to' },
  ];
  worker.__derivationEvents = events;
  const answer = plain(evaluate(worker, "finalizeWorkerDerivation({ content: 'fun main() = println()', evidence: [], solverEvents: __derivationEvents })"));
  const answerId = answerDerivationId('fun main() = println()');
  const expected = recordFor([...events, { kind: RENDER_EMIT_KIND, payload: `answer_id=${answerId};format=text` }], answerId);
  assert.equal(expected.applied_rules.length, 2);
  assert.equal(answer.derivationRecord.text, toLino(expected));
});

test('explain <answer id> prints the stored record as text and as links', async () => {
  const first = await ask('2 + 2');
  const answerId = answerDerivationId(first.content);
  const memoryEvents = [storedEvent(first.derivationRecord)];
  const explained = await ask(`explain ${answerId}`, memoryEvents);
  assert.equal(explained.intent, 'derivation_explain');
  assert.equal(explained.content, explainText(fromLino(first.derivationRecord.text)));
  assert.equal(explained.derivationRecord, null);
  const links = await ask(`explain ${answerId} --format links`, memoryEvents);
  assert.equal(links.content, first.derivationRecord.text);
});

test('explain of an unrecorded answer id is the seed miss text', async () => {
  const answerId = 'answer_0000000000000000';
  const missed = await ask(`explain ${answerId}`);
  assert.equal(missed.intent, 'derivation_record_missing');
  assert.equal(missed.content, missMessage('', answerId));
  const misfiled = await ask(`explain ${answerId}`, [{ kind: 'derivation', inputs: storePath('', answerId), content: 'derivation\n  answer_id "answer_1111111111111111"\n' }]);
  assert.equal(misfiled.content, missMessage('', answerId));
});
