import assert from 'node:assert/strict';
import fs from 'node:fs';
import {before, test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {Progress, qualifiedToolAttempt} from '../../../js/agentic/progress.mjs';
import {Capability} from '../../../js/agentic/capability.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {evidenceWindowStart} from '../../../js/agentic/planner/continuation.mjs';
import {continuationEvidenceWindowStart} from '../../../js/agentic/portable/continuation_state.mjs';
let worker;
before(async () => {worker = new WorkerHost(); await installNodeHost(worker);});
const task = 'Update a held-out source and verify its exact bytes.';
const argumentsValue = {path: 'src/held-out.txt', content: 'observed bytes\n'};
function writtenSession(request = task) {
  return [{role: 'user', content: request},
    {role: 'assistant', content: '', tool_calls: [{id: 'bound-write', type: 'function', function: {name: 'Write', arguments: JSON.stringify(argumentsValue)}}]},
    {role: 'tool', name: 'Write', tool_call_id: 'bound-write', content: 'written', is_error: false}];
}
function qualified(messages) {return qualifiedToolAttempt(Progress.scan(messages), Capability.Write, argumentsValue);}

test('the shared checked kernel agrees with the independent chronological window oracle', () => {
  const states = [[false, false, 'other'], [true, true, 'cue'], [true, false, 'text:one'], [true, false, 'text:two']];
  let checked = 0;
  function visit(rows, remaining) {
    const users = rows.map(row => row[0]), cues = rows.map(row => row[1]), identities = rows.map(row => row[2]);
    let expected = 0, active = null;
    for (let index = 0; index < rows.length; index += 1) {
      if (users[index] && !cues[index]) {
        if (active !== identities[index]) expected = index + 1;
        active = identities[index];
      }
    }
    const before = JSON.stringify([users, cues, identities]);
    assert.equal(continuationEvidenceWindowStart(users, cues, identities), expected);
    assert.equal(JSON.stringify([users, cues, identities]), before);
    checked += 1;
    if (remaining) for (const state of states) visit([...rows, state], remaining - 1);
  }
  visit([], 6);
  assert.equal(checked, 5461);
  for (const [users, cues, identities] of [[[true], [], ['text:one']], [[true], [false], []]]) {
    assert.equal(continuationEvidenceWindowStart(users, cues, identities), users.length);
  }
});

test('identical contiguous requests retain only their correctly bound successful observation', () => {
  const messages = writtenSession();
  const before = qualified(messages);
  assert.ok(before);
  for (const continuation of [[{role: 'user', content: task}], [{role: 'user', content: task}, {role: 'user', content: task}],
    [{role: 'user', content: 'continue'}, {role: 'user', content: task}]]) {
    const continued = [...messages, ...continuation];
    assert.equal(evidenceWindowStart(continued), 1);
    assert.ok(qualified(continued));
    assert.equal(Progress.scan(continued).done(Capability.Write), true);
  }
});

test('changed exact bytes and intervening different requests open fresh windows', () => {
  for (const changed of [task + ' ', task + '\n', task.replace('held-out', 'another'), task.toUpperCase()]) {
    const messages = [...writtenSession(), {role: 'user', content: changed}];
    assert.equal(evidenceWindowStart(messages), messages.length);
    assert.equal(qualified(messages), null);
  }
  const intervening = [...writtenSession(), {role: 'user', content: 'An independent request.'}, {role: 'user', content: task}];
  assert.equal(evidenceWindowStart(intervening), intervening.length);
  assert.equal(qualified(intervening), null);
});

test('multipart and unknown content cannot acquire identity from equal rendered prose', () => {
  for (const content of [[{type: 'text', text: task}], [{type: 'text', text: task}, {type: 'image_url', image_url: {url: 'fixture:changed-image'}}],
    [{type: 'text', text: 'continue'}, {type: 'image_url', image_url: {url: 'fixture:another-image'}}], null, '']) {
    const messages = [...writtenSession(), {role: 'user', content}];
    assert.equal(evidenceWindowStart(messages), messages.length);
    assert.equal(qualified(messages), null);
  }
  const sameParts = [{type: 'text', text: task}, {type: 'image_url', image_url: {url: 'fixture:same-image'}}];
  const messages = writtenSession(sameParts);
  messages.push({role: 'user', content: structuredClone(sameParts)});
  assert.equal(evidenceWindowStart(messages), messages.length);
  assert.equal(qualified(messages), null);
});

test('coalescing cannot qualify unbound, forged or duplicate call identifiers', () => {
  const unbound = writtenSession(); unbound[2].tool_call_id = 'foreign-result';
  assert.notEqual(qualified([...unbound, {role: 'user', content: task}])?.succeeded, true);
  const forged = writtenSession(); forged[2].name = 'Read';
  assert.notEqual(qualified([...forged, {role: 'user', content: task}])?.succeeded, true);
  const duplicate = writtenSession(); duplicate.splice(2, 0, structuredClone(duplicate[1]));
  assert.notEqual(qualified([...duplicate, {role: 'user', content: task}])?.succeeded, true);
});

function append(messages, call, receipt, failed = false) {
  const id = 'bound-' + messages.length;
  messages.push({role: 'assistant', content: '', tool_calls: [{id, type: 'function', function: {name: call.tool, arguments: call.arguments}}]},
    {role: 'tool', name: call.tool, tool_call_id: id, content: receipt, is_error: failed});
}

test('a whole current recipe retains source writes, successful verification and commit across an identical objective', async () => {
  const objective = 'Resolve the GitHub issue at https://github.com/example/continuation-fixture/issues/9 in this repository. Keep the solution on branch issue-9-continuation.';
  const page = '## Task\nWrite a Python program that prints exactly "Orbit 91" to stdout. Create a GitHub Actions workflow that runs the program.';
  const messages = [{role: 'user', content: objective}];
  const tools = ['Write', 'Bash', 'WebFetch'];
  const observed = [];
  let final;
  for (let index = 0; index < 16; index += 1) {
    const plan = await planChatStep(messages, tools);
    if (plan?.kind === 'final') {final = plan; break;}
    assert.equal(plan?.kind, 'tool_calls', JSON.stringify(plan));
    assert.equal(plan.calls.length, 1);
    const call = plan.calls[0], arguments_ = JSON.parse(call.arguments);
    assert.notEqual(arguments_.append_mode, 'atomic_record_append', 'no forged append receipt');
    let receipt = 'written';
    if (call.tool === 'WebFetch' || arguments_.command?.includes('gh issue view')) receipt = page;
    else if (call.tool === 'Bash' && arguments_.command?.includes('git commit')) receipt = 'a'.repeat(40) + '\n';
    else if (call.tool === 'Bash') receipt = JSON.stringify({exit_code: 0, stdout: 'Orbit 91\n', stderr: ''});
    append(messages, call, receipt);
    observed.push({call, arguments_, receipt});
  }
  assert.ok(final, 'the original bounded recipe must complete before continuity is checked');
  assert.ok(observed.some(row => row.call.tool === 'Write' && row.arguments_.content));
  assert.ok(observed.some(row => row.arguments_.command === 'sh tests/verify-output.sh'));
  assert.ok(observed.some(row => row.arguments_.command?.includes('git commit')));
  assert.match(JSON.stringify(final), /Committed and pushed/);
  const repeated = [...messages, {role: 'assistant', content: final.answer}, {role: 'user', content: objective}];
  const retained = Progress.scan(repeated);
  for (const row of observed.filter(row => row.call.tool === 'Write')) assert.ok(qualifiedToolAttempt(retained, Capability.Write, row.arguments_));
  for (const row of observed.filter(row => row.call.tool === 'Bash')) assert.equal(retained.latestRunOutputFor(row.arguments_.command), row.receipt);
  assert.deepEqual(await planChatStep(repeated, tools), final, 'no repeated source/workflow writes or compiler invocation');
  const changed = [...messages, {role: 'user', content: objective + ' Also change the output.'}];
  assert.equal(evidenceWindowStart(changed), changed.length);
  assert.equal(Progress.scan(changed).done(Capability.Write), false);
});

test('an identical objective never supplies missing installation authority', async () => {
  await installNodeHost(worker, {prerequisiteGrant: null});
  const request = 'Compile the requested program.';
  const messages = [{role: 'user', content: request}];
  append(messages, {tool: 'Bash', arguments: JSON.stringify({command: 'kotlinc Main.kt -include-runtime -d Main.jar'})},
    'Exit code 127\n/bin/bash: line 1: kotlinc: command not found', true);
  messages.push({role: 'user', content: request});
  const plan = await planChatStep(messages, ['Bash', 'Write']);
  assert.equal(plan.kind, 'final');
  assert.match(JSON.stringify(plan), /FORMAL_AI_INSTALL_GRANT=workspace/);
  assert.doesNotMatch(JSON.stringify(plan), /prerequisite-installer\.py/);
});


test('non-user receipts are not read or copied while observing request identity', () => {
  const messages = [{role: 'user', content: task}, {role: 'tool', get content() {throw new Error('receipt content must remain unobserved');}}, {role: 'assistant', get content() {throw new Error('assistant content must remain unobserved');}}, {role: 'user', content: task}];
  assert.equal(evidenceWindowStart(messages), 1);
});
