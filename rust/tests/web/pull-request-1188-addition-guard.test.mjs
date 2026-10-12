// PR #1188 dogfooding (T80): `Add an assertion that add(2, 2) equals 4 to
// m.test.mjs.` replaced the whole test file with that sentence: the general
// change plan read "an assertion that …" as the file's new bytes. When the
// write verb is the seeded add action and the content comes before the file,
// the request names an addition to that file, never its whole new content;
// content a seeded content lead introduces (`containing`) is still bytes. The
// Rust twin is rust/tests/unit/agentic-coding/pull_request_1188_addition_guard.rs.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let composeGeneralChangePlan;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ composeGeneralChangePlan } = await import('../../../js/agentic/general_planner.mjs'));
});

describe('an addition to a file never becomes its whole content', () => {
  test('"Add <description> to <file>" is not a whole-file write', () => {
    assert.equal(composeGeneralChangePlan('Add an assertion that add(2, 2) equals 4 to m.test.mjs.'), null);
    assert.equal(composeGeneralChangePlan('Добавь проверку, что add(2, 2) равно 4, в m.test.mjs.'), null);
  });

  test('content a content lead introduces is still the file', () => {
    const plan = composeGeneralChangePlan("Add a file notes.txt containing 'hello'.");
    assert.equal(plan?.target, 'notes.txt');
    assert.equal(plan?.content, 'hello');
  });

  test('a write verb that is not an addition still writes the file', () => {
    const plan = composeGeneralChangePlan("Write 'hello' to notes.txt.");
    assert.equal(plan?.target, 'notes.txt');
    assert.equal(plan?.content, 'hello');
  });
});
