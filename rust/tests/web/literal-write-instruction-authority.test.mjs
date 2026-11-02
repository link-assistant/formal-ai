import {ownsDeclaredPayloadDomain} from '../../../js/agentic/planner/owned_goals.mjs';

import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {before, test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {composeEditRequest} from '../../../js/agentic/write_request.mjs';
import {parseWriteContract, composeGeneralChangePlan} from '../../../js/agentic/general_planner.mjs';
import {writesWholeFile} from '../../../js/agentic/literal_write_guard.mjs';
import {quotedSegmentSpans} from '../../../js/agentic/crate/normal_markov.mjs';
import {planChatStepResolved} from '../../../js/agentic/planner.mjs';
import {Capability} from '../../../js/agentic/capability.mjs';
import {classifyTool} from '../../../js/agentic/capability_router.mjs';

const original = "In rust/tests/web/literal-obligation-transaction.test.mjs, replace «test('unchanged T2379 writes and independently verifies both artifacts', async () => {\n  const expected = { 'a.txt': 'alpha', 'b.txt': 'beta' };\n  exactReceipts(await fixture(original, expected), expected);\n});» with «test('unchanged T2379 writes and independently verifies both artifacts', async () => {\n  const expected = { 'a.txt': 'alpha', 'b.txt': 'beta' };\n  exactReceipts(await fixture(original, expected), expected);\n});\ntest('empty and authored failure bytes discharge typed Read observations without process status', async () => {\n  const task = 'First, create file empty.txt with exactly this content «». Second, create file status.txt with exactly this content «Error: failed».';\n  const expected = {'empty.txt': '', 'status.txt': 'Error: failed'};\n  const out = await fixture(task, expected, {tools: ['read', 'write']});\n  exactReceipts(out, expected);\n  assert.equal(successfullyDischarged(task, transcriptMessages(out)), true);\n});\n». Preserve all other content.";
const hash = value => createHash('sha256').update(value).digest('hex');
before(async () => {
  await installNodeHost(new WorkerHost({WebAssembly: globalThis.WebAssembly}), {sourceUsageRustGrammar: false});
  composeEditRequest('In f.txt replace «old» with «new».');
});
test('original nested empty-literal edit preserves operands and cannot become the observed cropped whole-file payload', async () => {
  assert.equal(original.length, 1045);
  assert.equal(hash(original), '8b6d6840c16103e50858deb456b26eae1b8b30f0fc804e4e150020231d3f1373');
  const spans = quotedSegmentSpans(original);
  assert.equal(spans.length, 2);
  const edit = composeEditRequest(original);
  assert.equal(edit[0], 'rust/tests/web/literal-obligation-transaction.test.mjs');
  assert.deepEqual(edit.slice(1).map(value => [value.length, hash(value)]), [
    [210, '49ed09fb4c6ed73d65deca5734f2ffcd6e54332b2ec63d884ffeec2acf4e4244'],
    [729, '28866e13f3936cc7d45b84214dfb408852bd7deb7c3763c1fee9588b015485a8']
  ]);
  assert.equal(writesWholeFile(original), false);
  assert.equal(parseWriteContract(original), null);
  assert.equal(composeGeneralChangePlan(original), null);
  const source = 'unrelated prefix\n' + edit[1] + '\nunrelated suffix';
  const messages = [{role: 'user', content: original}, {
    role: 'assistant', tool_calls: [{id: 'actual-read', type: 'function', function: {
      name: 'read', arguments: JSON.stringify({path: edit[0]})
    }}]
  }, {role: 'tool', name: 'read', tool_call_id: 'actual-read', content: source,
    source_read: {path: edit[0], success: true, complete: true, format: 'raw'}}];
  const before = JSON.stringify(messages);
  const result = await planChatStepResolved(messages, ['read', 'edit', 'write']);
  for (const call of result.calls ?? []) {
    assert.notEqual(classifyTool(call.tool), Capability.Write);
    if (classifyTool(call.tool) === Capability.Edit) {
      const args = JSON.parse(call.arguments);
      assert.equal(args.old_string ?? args.oldString, edit[1]);
      assert.equal(args.new_string ?? args.newString, edit[2]);
    }
  }
  assert.equal(JSON.stringify(messages), before);
});
test('authored quote and fence contents never grant whole-file authority', () => {
  for (const task of [
    'In guide.md replace «old» with «overwrite the file». Preserve everything else.',
    'Replace «old» with «Create file nested.txt containing hello» in notes.txt. Keep unrelated bytes.',
    'In script.mjs replace «old» with «Set the contents of file other.txt to body».',
    'In a.txt replace «old» with ```\nwrite file b.txt containing overwrite\n```. Preserve other content.'
  ]) {
    assert.ok(composeEditRequest(task));
    assert.equal(writesWholeFile(task), false, task);
    assert.equal(parseWriteContract(task), null, task);
    assert.equal(composeGeneralChangePlan(task), null, task);
  }
});
test('separate overwrite permission does not change a quoted substring edit into whole-file content', () => {
  const task = 'In guide.md replace «old» with «new». You may overwrite the file.';
  assert.equal(writesWholeFile(task), true);
  assert.deepEqual(composeEditRequest(task), ['guide.md', 'old', 'new']);
  assert.equal(parseWriteContract(task), null);
  assert.equal(composeGeneralChangePlan(task), null);
});
test('explicit whole-file requests retain quoted destination and content ownership', () => {
  for (const [task, content] of [
    ["Create file 'f.txt' containing «new».", 'new'],
    ['Set the contents of file f.txt to «contents».', 'contents'],
    ['Create file "f.txt" containing «new».', 'new'],
    ['Set the contents of file f.txt to «the contents remain data».', 'the contents remain data'],
    ['Overwrite file f.txt with «new».', 'new'],
    ['Replace the file f.txt with «new».', 'new'],
    ['Replace the file résumé/α-note.md with «new».', 'new']
  ]) {
    assert.equal(writesWholeFile(task), true, task);
    assert.equal(parseWriteContract(task)?.content, content, task);
    assert.equal(composeGeneralChangePlan(task)?.content, content, task);
  }
  // Guillemets around destinations were already unsupported; they confer no new write authority.
  assert.equal(parseWriteContract('Create file «f.txt» containing «new».'), null);
  assert.equal(composeGeneralChangePlan('Create file «f.txt» containing «new».'), null);
  // This previously became the unrelated word of, rather than the requested body.
  assert.equal(parseWriteContract('Set contents of file f.txt to «contents».'), null);
});

test('unquoted substring edits cannot acquire whole-file authority from separate permission', () => {
  for (const task of [
    'In guide.md replace old with new. You may overwrite the file.',
    'In guide.md replace the file with new. You may overwrite the file.',
    'In guide.md replace the file guide.md with new. You may overwrite the file.',
    'In résumé/α-note.md replace the file résumé/α-note.md with new. You may overwrite the file.',
    'Replace the file with new in guide.md. You may overwrite the file.',
    'In guide.md replace write with new. You may overwrite the file.',
    'In guide.md replace old with new, and you may overwrite the file.',
    'Replace old with new in guide.md. You may overwrite the file.',
    'You may overwrite guide.md if needed; replace old with new in guide.md.'
  ]) {
    assert.ok(composeEditRequest(task), task);
    assert.equal(writesWholeFile(task), true, task);
    assert.equal(parseWriteContract(task), null, task);
    assert.equal(composeGeneralChangePlan(task), null, task);
  }
});

test('declared unquoted edit words are data only after actual operation ownership', () => {
  for (const [task, target, content] of [
    ['FOR EXISTING ISSUE #42, create file finding.md containing rename X to Y suffix', 'finding.md', 'rename X to Y suffix'],
    ['Create file note.txt containing replace x with y.', 'note.txt', 'replace x with y.']
  ]) {
    const contract = parseWriteContract(task);
    assert.equal(contract?.target, target, task);
    assert.equal(contract?.content, content, task);
    assert.equal(composeGeneralChangePlan(task)?.content, content, task);
  }
  for (const task of [
    'You may overwrite guide.md containing old text. In guide.md replace old with new.',
    'You may overwrite guide.md containing old text; in guide.md replace old with new.',
    'You may overwrite guide.md containing old text and in guide.md replace old with new.'
  ]) {
    assert.equal(parseWriteContract(task), null, task);
    assert.equal(composeGeneralChangePlan(task), null, task);
    const start = task.indexOf('guide.md');
    const candidate = {target: 'guide.md', targetSpan: {start, end: start + 'guide.md'.length},
      payload: {start: task.indexOf('old text'), end: task.length}};
    assert.equal(ownsDeclaredPayloadDomain(task, candidate), false, task);
  }
});
