// PR #1188 dogfooding (T21): `Replace 'rust/src/x.rs' with 'data/seed/x.lino'
// in check.mjs.` planned nothing. The edit reader took the quoted new text as
// the file to edit: a literal that is exactly a path may name the file, and the
// cue after it ("in") read as the target cue. A path the request leaves
// unquoted now names the file first; a quoted path still does when nothing else
// can. The Rust twin is rust/tests/unit/pull_request_1188_quoted_path_payload.rs.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let composeEditRequest;
let firstPath;
let planChatStep;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ composeEditRequest } = await import('../../../js/agentic/write_request.mjs'));
  ({ firstPath } = await import('../../../js/agentic/crate/capability_routing.mjs'));
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
});

describe('a quoted path in an edit request is payload, not the target', () => {
  test('the new text may be a path', () => {
    assert.deepEqual(composeEditRequest("Replace 'abc' with 'data/seed/x.lino' in check.mjs."),
      ['check.mjs', 'abc', 'data/seed/x.lino']);
    assert.deepEqual(composeEditRequest("Replace 'feature_capability.rs' with 'features.lino' in check.mjs."),
      ['check.mjs', 'feature_capability.rs', 'features.lino']);
  });

  test('a quoted path still names the file when nothing else does', () => {
    assert.deepEqual(composeEditRequest("Replace 'a' with 'b' in 'notes.txt'."), ['notes.txt', 'a', 'b']);
  });

  test('a literal led by the words that say what it is stands for the literal (T25)', () => {
    assert.deepEqual(composeEditRequest("Replace the heading '# Title' with '# Project' in README.md."),
      ['README.md', '# Title', '# Project']);
    assert.deepEqual(composeEditRequest("Replace the word 'cat' with 'dog' in a.txt."), ['a.txt', 'cat', 'dog']);
  });

  test('the planner reads the unquoted file first', async () => {
    const plan = await planChatStep(
      [{ role: 'user', content: "Replace 'rust/src/x.rs' with 'data/seed/x.lino' in check.mjs." }],
      ['read', 'write', 'edit', 'bash'],
    );
    assert.equal(plan?.kind, 'tool_calls');
    assert.equal(plan.calls[0].tool, 'read');
    assert.equal(JSON.parse(plan.calls[0].arguments).filePath, 'check.mjs');
  });

  test('a sentence mark after one quoted literal is the sentence\'s, not the content\'s (T26)', async () => {
    const messages = [{ role: 'user', content: "Create a file new.txt containing 'hello'." }];
    let written = null;
    for (let step = 0; step < 4 && written === null; step += 1) {
      const plan = await planChatStep(messages, ['read', 'write', 'edit', 'bash']);
      assert.equal(plan?.kind, 'tool_calls');
      const [call] = plan.calls;
      const args = JSON.parse(call.arguments);
      if (call.tool === 'write' && args.filePath === 'new.txt') written = args.content;
      const id = `call_${step}`;
      messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
      messages.push({ role: 'tool', tool_call_id: id, content: '' });
    }
    assert.equal(written, 'hello');
  });

  test('a path inside a quoted payload never outranks the file the request names (T36)', () => {
    assert.equal(firstPath(`Insert the line "    body '{"token":"p"}'" after the line "    path '/v1/x/learn'" in r.lino.`),
      'r.lino');
    assert.equal(firstPath("Read 'notes.txt'."), 'notes.txt');
    assert.equal(firstPath('Open (docs/x.md).'), 'docs/x.md');
  });

  test('a target cue inside the quoted new text is payload (T37)', () => {
    assert.deepEqual(composeEditRequest("Replace 'works.' with 'works. It reads the named file.' in ledger.md."),
      ['ledger.md', 'works.', 'works. It reads the named file.']);
  });
});
