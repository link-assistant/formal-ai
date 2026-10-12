import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { ownedDeclaredCreateFrame } from '../../../js/agentic/planner/owned_goals.mjs';
import { planLiteralAdditionStep } from '../../../js/agentic/literal_addition.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
const tools = ['read_file', 'write_file'];
test('source-declared renamed destinations precede insertion dispatch', () => {
  for (const request of ['add file renamed-note.txt containing αβ', 'añade archivo altered.txt con el texto γ']) {
    assert.notEqual(ownedDeclaredCreateFrame(request), null);
    assert.equal(planLiteralAdditionStep(request, [{ role: 'user', content: request }], tools), null);
  }
});
test('seeded nominal whole-file phrase supports punctuation and owned literal bytes', () => {
  assert.deepEqual(ownedDeclaredCreateFrame('new file: renamed-β.txt, contents: hello'), { target: 'renamed-β.txt', content: 'hello' });
  assert.deepEqual(ownedDeclaredCreateFrame('new file: result.txt, contents: "append overwrite prepend"'), { target: 'result.txt', content: 'append overwrite prepend' });
});
test('unknown, conditional, attributed and malformed nominal requests refuse', () => {
  for (const request of ['Deploy something. new file: note.txt, contents: hello', 'If ready, new file: note.txt, contents: hello', 'The manual says "new file: note.txt, contents: hello"', 'new file: note.txt, contents: "hello', 'new file: note.txt, contents: "hello". Deploy it.']) {
    assert.equal(ownedDeclaredCreateFrame(request), null, request);
  }
});
test('policy and insertion intent cannot become nominal creation', () => {
  for (const request of ['Do not write. new file: note.txt, contents: hello', 'Do not read. new file: note.txt, contents: hello', 'Append "hello" to note.txt.', 'Overwrite file note.txt containing hello']) {
    assert.equal(ownedDeclaredCreateFrame(request), null, request);
  }
});
test('destination-before-noun permits only its ordinary guard Read', () => {
  const request = 'add to the file renamed-note.txt containing hello';
  assert.equal(ownedDeclaredCreateFrame(request), null);
  const messages = [{ role: 'user', content: request }];
  const read = planLiteralAdditionStep(request, messages, tools);
  assert.equal(read.kind, 'tool_calls');
  assert.equal(read.calls[0].tool, 'read_file');
  assert.equal(JSON.parse(read.calls[0].arguments).path, 'renamed-note.txt');
  messages.push({ role: 'assistant', tool_calls: [{ id: 'guard', function: { name: 'read_file', arguments: read.calls[0].arguments } }] }, { role: 'tool', tool_call_id: 'guard', name: 'read_file', content: 'original observed bytes' });
  const next = planLiteralAdditionStep(request, messages, tools);
  assert.equal(next.kind, 'final');
  assert.doesNotMatch(next.answer, /Completed the general change/);
});
