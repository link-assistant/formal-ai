// Issue #1184 R1 and R8: every calling surface carries the answer's
// content-addressed derivation id (`SymbolicAnswer::derivation_id`,
// rust/src/engine_answer.rs) — the browser worker's `finalize` result
// (`derivationId`, read by js/app/app-conversation-hooks.jsx) and the
// JavaScript server's `/v1/responses` object (`derivation_id` beside
// `evidence_links`, as rust/src/protocol/output.rs `ResponseObject`). The
// constant is the one rust/tests/unit/issue_1184_derivation_records.rs pins.

import assert from 'node:assert/strict';
import { before, test } from 'node:test';

import { answerDerivationId } from '../../../js/agentic/crate/derivation.mjs';
import { responseFromSymbolic } from '../../../js/server/responses.mjs';
import { solveSymbolic } from '../../../js/server/solve.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

let host;
before(async () => {
  host = new WorkerHost();
  await host.boot();
});

test('the derivation id is the same constant in every root', () => {
  assert.equal(answerDerivationId('Kotlin programs are compiled with kotlinc.'), 'answer_91f06b150735a87e');
});

test('the browser worker answer carries the derivation id of its content', async () => {
  const result = await host.solve('1 + 1', []);
  assert.equal(result.derivationId, answerDerivationId(result.content));
});

test('the responses object carries derivation_id beside evidence_links', async () => {
  const symbolic = await solveSymbolic({ worker: host }, '1 + 1', []);
  const chatRequest = { messages: [{ role: 'user', content: '1 + 1' }] };
  const response = responseFromSymbolic({}, chatRequest, '1 + 1', symbolic);
  const keys = Object.keys(response);
  assert.equal(keys.indexOf('derivation_id'), keys.indexOf('evidence_links') + 1);
  assert.equal(response.derivation_id, answerDerivationId(symbolic.answer));
});
