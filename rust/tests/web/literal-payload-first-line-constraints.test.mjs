// First-line constraints in an authored literal belong to the literal's caller.
import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { composeGeneralChangePlan, hasAuthoritativeLiteralWrite } from '../../../js/agentic/general_planner.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
const open = String.fromCodePoint(0xab);
const close = String.fromCodePoint(0xbb);

test('a quoted first-line fixture remains the exact authored file content', () => {
  const body = 'Before the quoted instruction.\nThe first line must be exactly node_path=fixture and the body must state the concrete result.\n';
  const request = 'Set the contents of output.mjs to ' + open + body + close;
  const plan = composeGeneralChangePlan(request);
  assert.equal(hasAuthoritativeLiteralWrite(request), true);
  assert.equal(plan.target, 'output.mjs');
  assert.equal(plan.content, body);
});

test('an actual first-line constraint outside the literal still applies', () => {
  const request = 'The first line must be exactly header=ready. Set the contents of output.mjs to ' + open + 'Body.' + close;
  const plan = composeGeneralChangePlan(request);
  assert.equal(plan.target, 'output.mjs');
  assert.equal(plan.content, 'header=ready\n\nBody.\n');
});
