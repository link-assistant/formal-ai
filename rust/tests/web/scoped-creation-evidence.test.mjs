import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {composeGeneralChangePlan} from '../../../js/agentic/general_planner.mjs';
import {planGeneralChangeStep} from '../../../js/agentic/general_execution.mjs';
import {applyPatchInput} from '../../../js/agentic/crate/protocol_responses_apply_patch.mjs';
import {executeResult} from '../../../experiments/js_dogfood/drive.mjs';

before(async () => installNodeHost(new WorkerHost(), {sourceUsageRustGrammar: false}));
const original = 'Create candidate.json containing exactly this content: «{"schema":"read-report-trial/v1","case":{"source":{"enabled":false},"report":{"key":"enabled","expected":true,"status":"contradicted"},"then":{"operation":"create","unconditional":true}}}\n».';
const payload = 'α🙂\nError: authored data\n';
const tools = ['read', 'apply_patch'];
function box(t, target = 'candidate.txt', content = payload) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'scoped-create-proof-'));
  t.after(() => fs.rmSync(directory, {recursive: true, force: true}));
  const task = 'Create ' + target + ' with the following exact content: «' + content + '».';
  const plan = composeGeneralChangePlan(task);
  assert.equal(plan?.target, target);
  assert.equal(plan.content, content);
  return {directory, target, content, task, plan, messages: [{role: 'user', content: task}]};
}
function step(fixture, names = tools) {
  return planGeneralChangeStep(fixture.messages, names, fixture.plan);
}
function one(plan, name) {
  assert.equal(plan?.kind, 'tool_calls');
  assert.equal(plan.calls.length, 1);
  assert.equal(plan.calls[0].tool, name);
  return plan.calls[0];
}
function append(fixture, call, result) {
  const id = 'actual-scoped-create-' + fixture.messages.length;
  fixture.messages.push({role: 'assistant', tool_calls: [{
    id, type: 'function', function: {name: call.tool, arguments: call.arguments},
  }]}, {role: 'tool', tool_call_id: id, name: call.tool, ...result});
}
function truthfulGap(plan) {
  assert.equal(plan?.kind, 'final');
  assert.equal(plan.result?.disposition, 'gap');
  assert.doesNotMatch(plan.answer, /created .*through the workspace|completed the file change/i);
}
function absent(fixture) {
  assert.equal(fs.existsSync(path.join(fixture.directory, fixture.target)), false);
}
function preflight(fixture) {
  const read = one(step(fixture), 'read');
  append(fixture, read, executeResult(fixture.directory, read));
}
function patch(fixture, result = {content: 'Success: added owned target'}) {
  const call = one(step(fixture), 'apply_patch');
  const args = JSON.parse(call.arguments);
  assert.equal(args.path ?? args.file_path, fixture.target);
  assert.equal(args.content, fixture.content);
  const raw = applyPatchInput(call.arguments);
  assert.equal(typeof raw, 'string');
  // Explicit mock provider executes the emitted single AddFile bytes; this is
  // physical fixture IO, not evidence that every client AddFile is exclusive.
  fs.writeFileSync(path.join(fixture.directory, fixture.target), args.content, {flag: 'wx'});
  append(fixture, {...call, arguments: raw}, result);
}

for (const separator of ['\u2028', '\u2029']) {
  test('canonical AddFile decodes LF lines without stripping Unicode-authored plus ' + separator.codePointAt(0), async () => {
    const {canonicalCreation, canonicalCreationPath} = await import('../../../js/agentic/qualified_tool_observation.mjs');
    const content = 'alpha' + separator + '+literal\n';
    const raw = applyPatchInput(JSON.stringify({path: 'heldout.txt', content}));
    assert.deepEqual(canonicalCreation(raw), {path: 'heldout.txt', content});
    assert.equal(canonicalCreationPath(raw), 'heldout.txt');
    assert.equal(canonicalCreation(raw.replace('*** End Patch\n', '*** Add File: other.txt\n+x\n*** End Patch\n')), null);
  });
}
test('pending canonical AddFile does not imply execution or permit repetition', t => {
  const fixture = box(t);
  preflight(fixture);
  const call = one(step(fixture), 'apply_patch');
  fixture.messages.push({role: 'assistant', tool_calls: [{
    id: 'pending-add', type: 'function', function: {name: call.tool, arguments: applyPatchInput(call.arguments)},
  }]});
  truthfulGap(step(fixture));
  absent(fixture);
});
test('a wrong-ID patch receipt cannot mint creation evidence', t => {
  const fixture = box(t);
  preflight(fixture);
  const call = one(step(fixture), 'apply_patch');
  fixture.messages.push({role: 'assistant', tool_calls: [{
    id: 'declared-add', type: 'function', function: {name: call.tool, arguments: applyPatchInput(call.arguments)},
  }]}, {role: 'tool', tool_call_id: 'foreign-id', name: call.tool, content: 'Success'});
  truthfulGap(step(fixture));
  absent(fixture);
});
test('duplicate patch receipts cannot certify a physically created target', t => {
  const fixture = box(t);
  preflight(fixture);
  patch(fixture);
  const receipt = fixture.messages.at(-1);
  fixture.messages.push({...receipt});
  truthfulGap(step(fixture));
  assert.equal(fs.readFileSync(path.join(fixture.directory, fixture.target), 'utf8'), payload);
});
test('foreign mutation before the final target Read invalidates creation proof', t => {
  const fixture = box(t);
  preflight(fixture);
  patch(fixture);
  append(fixture, {tool: 'edit', arguments: JSON.stringify({path: 'other.txt', old: 'a', new: 'b'})},
    {content: 'Success'});
  append(fixture, {tool: 'read', arguments: JSON.stringify({path: fixture.target})},
    executeResult(fixture.directory, {tool: 'read', arguments: JSON.stringify({path: fixture.target})}));
  truthfulGap(step(fixture));
  assert.equal(fs.readFileSync(path.join(fixture.directory, fixture.target), 'utf8'), payload);
});
test('pending mutation before the final target Read invalidates creation proof', t => {
  const fixture = box(t);
  preflight(fixture);
  patch(fixture);
  fixture.messages.push({role: 'assistant', tool_calls: [{
    id: 'pending-foreign', type: 'function', function: {
      name: 'edit', arguments: JSON.stringify({path: 'other.txt', old: 'a', new: 'b'}),
    },
  }]});
  const call = {tool: 'read', arguments: JSON.stringify({path: fixture.target})};
  append(fixture, call, executeResult(fixture.directory, call));
  truthfulGap(step(fixture));
  assert.equal(fs.readFileSync(path.join(fixture.directory, fixture.target), 'utf8'), payload);
});
test('wrong Read bytes do not certify the actual created content', t => {
  const fixture = box(t);
  preflight(fixture);
  patch(fixture);
  const call = one(step(fixture), 'read');
  const result = executeResult(fixture.directory, call);
  append(fixture, call, {...result, content: 'wrong observed bytes'});
  assert.notEqual(step(fixture).result?.disposition, 'finding');
  assert.equal(fs.readFileSync(path.join(fixture.directory, fixture.target), 'utf8'), payload);
});
test('an actual canonical framed Run can verify a scoped AddFile target', async t => {
  const fixture = box(t);
  preflight(fixture);
  patch(fixture);
  const {ownedCreationReadbackCommand} = await import('../../../js/agentic/general_execution.mjs');
  const call = {tool: 'run_shell_command', arguments: JSON.stringify({
    command: ownedCreationReadbackCommand(fixture.target),
  })};
  append(fixture, call, executeResult(fixture.directory, {...call, tool: 'bash'}));
  const final = step(fixture, [...tools, 'run_shell_command']);
  assert.equal(final.result?.disposition, 'finding');
  assert.equal(fs.readFileSync(path.join(fixture.directory, fixture.target), 'utf8'), payload);
});

test('qualified foreign Run gets one target observation then a stable noncompletion gap', t => {
  const fixture = box(t);
  preflight(fixture);
  patch(fixture);
  fs.writeFileSync(path.join(fixture.directory, 'other.txt'), 'unrelated');
  const foreign = {tool: 'run_shell_command', arguments: JSON.stringify({command: 'cat other.txt'})};
  append(fixture, foreign, executeResult(fixture.directory, {...foreign, tool: 'bash'}));
  const names = [...tools, 'run_shell_command'];
  const recovery = one(step(fixture, names), 'read');
  append(fixture, recovery, executeResult(fixture.directory, recovery));
  const count = fixture.messages.length;
  truthfulGap(step(fixture, names));
  truthfulGap(step(fixture, names));
  assert.equal(fixture.messages.length, count);
  assert.equal(fs.readFileSync(path.join(fixture.directory, fixture.target), 'utf8'), payload);
});
test('unbound foreign Run cannot trigger a recovery observation', t => {
  const fixture = box(t);
  preflight(fixture);
  patch(fixture);
  fixture.messages.push({role: 'assistant', tool_calls: [{
    id: 'foreign-run-declared', type: 'function', function: {
      name: 'run_shell_command', arguments: JSON.stringify({command: 'cat other.txt'}),
    },
  }]}, {role: 'tool', name: 'run_shell_command', tool_call_id: 'wrong-foreign-run', content: 'Output: unrelated\nExit Code: 0'});
  truthfulGap(step(fixture, [...tools, 'run_shell_command']));
  assert.equal(fs.readFileSync(path.join(fixture.directory, fixture.target), 'utf8'), payload);
});

test('original Codex zero-effect creation is a truthful missing-preflight gap', t => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'codex-zero-effect-'));
  t.after(() => fs.rmSync(directory, {recursive: true, force: true}));
  const plan = composeGeneralChangePlan(original);
  assert.equal(plan?.target, 'candidate.json');
  truthfulGap(planGeneralChangeStep([{role: 'user', content: original}], ['apply_patch', 'exec_command'], plan));
  assert.equal(fs.existsSync(path.join(directory, 'candidate.json')), false);
});
test('scoped AddFile uses actual absence preflight and fresh physical target Read', t => {
  const fixture = box(t);
  absent(fixture);
  preflight(fixture);
  absent(fixture);
  patch(fixture);
  assert.equal(fs.readFileSync(path.join(fixture.directory, fixture.target), 'utf8'), payload);
  const read = one(step(fixture), 'read');
  append(fixture, read, executeResult(fixture.directory, read));
  const final = step(fixture);
  assert.equal(final.kind, 'final');
  assert.equal(final.result?.disposition, 'finding');
  assert.equal(fs.readFileSync(path.join(fixture.directory, fixture.target), 'utf8'), payload);
});
test('scoped AddFile cannot overwrite an existing target', t => {
  const fixture = box(t);
  fs.writeFileSync(path.join(fixture.directory, fixture.target), 'protected prior bytes');
  preflight(fixture);
  truthfulGap(step(fixture));
  assert.equal(fs.readFileSync(path.join(fixture.directory, fixture.target), 'utf8'), 'protected prior bytes');
});
test('direct Write retains its exclusive provider route without a fabricated preread', t => {
  const fixture = box(t);
  absent(fixture);
  const call = one(step(fixture, ['write', 'read']), 'write');
  assert.equal(JSON.parse(call.arguments).content, payload);
  absent(fixture);
});
test('remote patch name cannot acquire a workspace creation route', t => {
  const fixture = box(t);
  truthfulGap(step(fixture, ['mcp__github__apply_patch', 'exec_command']));
  absent(fixture);
});
test('non-roundtripping no-newline content is an honest AddFile representation gap', t => {
  const fixture = box(t, 'heldout.md', 'no final newline');
  truthfulGap(step(fixture));
  absent(fixture);
});
test('failed patch receipt cannot be converted to successful creation or repeated', t => {
  const fixture = box(t);
  preflight(fixture);
  const call = one(step(fixture), 'apply_patch');
  append(fixture, {...call, arguments: applyPatchInput(call.arguments)}, {content: 'provider rejected AddFile', is_error: true});
  const result = step(fixture);
  assert.equal(result.kind, 'final');
  assert.notEqual(result.result?.disposition, 'finding');
  assert.doesNotMatch(result.answer, /created .*through the workspace/i);
  absent(fixture);
});
