// Every authoritative artifact needs its own exact target and successful readback.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { mkdtempSync, readFileSync, writeFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep, planChatStepResolved } from '../../../js/agentic/planner.mjs';
import { drive, executeResult } from '../../../experiments/js_dogfood/drive.mjs';
import { Progress, qualifiedCommandAttempt, sourceReadPath } from '../../../js/agentic/progress.mjs';
import { readOnlyAttempt } from '../../../js/agentic/progress_command_readback.mjs';
import { composeGeneralChangePlan } from '../../../js/agentic/general_planner.mjs';
import { ownedCreationObservation, ownedCreationReadbackCommand } from '../../../js/agentic/general_execution.mjs';
import { ownedDeclaredCreateFrame } from '../../../js/agentic/planner/owned_goals.mjs';
import { observedPayload } from '../../../js/agentic/tool_result.mjs';
import { finalResult, FinalDisposition, canDeliverFinal } from '../../../js/agentic/final_result.mjs';
import { obligations, successfullyDischarged } from '../../../js/agentic/task_obligations.mjs';
import { clausesWithSpans } from '../../../js/agentic/crate/obligation_ledger.mjs';
import { wordsForRole } from '../../../js/agentic/write_lexicon.mjs';
import { planObligationsStep } from '../../../js/agentic/planner/obligations.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
const original = 'First, create file a.txt with exactly this content «alpha». Second, create file b.txt with exactly this content «beta».';
async function fixture(task, expected, options = {}) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-literal-transaction-'));
  try {
    for (const [name, text] of Object.entries(options.initial ?? {})) writeFileSync(join(directory, name), text);
    const out = await drive(planChatStep, directory, task, { steps: 8, ...(options.tools ? { tools: options.tools } : {}) });
    for (const [name, text] of Object.entries(expected)) assert.equal(existsSync(join(directory, name)) ? readFileSync(join(directory, name), 'utf8') : null, text);
    return {...out, task};
  } finally { rmSync(directory, { recursive: true, force: true }); }
}
function transcriptMessages(out) {
  const messages = [{role: 'user', content: out.task}];
  for (const [index, entry] of out.transcript.entries()) {
    const id = 'verified-artifact-' + index;
    messages.push({role: 'assistant', tool_calls: [{id, type: 'function',
      function: {name: entry.tool, arguments: entry.arguments}}]},
    {role: 'tool', name: entry.tool, tool_call_id: id, content: entry.result,
      is_error: entry.is_error, source_read: entry.source_read});
  }
  return messages;
}
function exactReceipts(out, expected) {
  const progress = Progress.scan(transcriptMessages(out));
  const clauses = obligations(out.task)?.map(node => node.clause) ?? [out.task];
  for (const [path, bytes] of Object.entries(expected)) {
    const plan = clauses.map(composeGeneralChangePlan).find(plan => plan?.target === path);
    assert.ok(plan, path + ' has its own composed obligation');
    if (ownedDeclaredCreateFrame(plan.goal) !== null) {
      const proof = ownedCreationObservation(plan, progress);
      assert.ok(proof, path + ' has a fresh independently bound exact observation');
      assert.equal(proof.target, path);
      assert.equal(proof.source, bytes);
      if (proof.method === 'read-tool') {
        assert.equal(proof.command, null);
        assert.equal(proof.exitCode, null);
        assert.equal(proof.sourceRead.complete, true);
        assert.equal(proof.sourceRead.source, bytes);
      } else {
        assert.equal(proof.command, ownedCreationReadbackCommand(path));
        assert.equal(proof.exitCode, 0);
        assert.equal(proof.sourceRead, null);
      }
    } else {
      // The existing replacement owner retains its actual unframed verification.
      const receipts = out.transcript.filter(entry => entry.tool === 'bash'
        && JSON.parse(entry.arguments).command === plan.verification_command);
      assert.equal(receipts.length, 1, path + ' has one independent legacy receipt');
      assert.equal(observedPayload(receipts[0].result), bytes);
    }
  }
  assert.equal(out.stop, 'final');
}
test('unchanged T2379 writes and independently verifies both artifacts', async () => {
  const expected = { 'a.txt': 'alpha', 'b.txt': 'beta' };
  exactReceipts(await fixture(original, expected), expected);
});
test('empty and authored failure bytes discharge typed Read observations without process status', async () => {
  const task = 'First, create file empty.txt with exactly this content «». Second, create file status.txt with exactly this content «Error: failed».';
  const expected = {'empty.txt': '', 'status.txt': 'Error: failed'};
  const out = await fixture(task, expected, {tools: ['read', 'write']});
  exactReceipts(out, expected);
  assert.equal(successfullyDischarged(task, transcriptMessages(out)), true);
});

test('quoted operation and teaching keywords remain literal in every obligation', async () => {
  const quote = String.fromCharCode(96);
  const first = 'When ' + quote + 'input' + quote + ' then ' + quote + 'output' + quote;
  const second = 'rename a.txt to gone.txt and replace alpha with wrong';
  const task = 'First, create file a.txt with exactly this content «' + first + '». Second, create file b.txt with exactly this content «' + second + '».';
  const out = await fixture(task, { 'a.txt': first, 'b.txt': second, 'gone.txt': null });
  exactReceipts(out, { 'a.txt': first, 'b.txt': second });
  assert.equal(out.transcript.filter((entry) => entry.tool === 'write').length, 2);
});

function appendPhysical(messages, directory, call, override = null) {
  const result = executeResult(directory, call);
  const id = 'physical-obligation-' + messages.length;
  messages.push({role: 'assistant', tool_calls: [{id, type: 'function',
    function: {name: call.tool, arguments: call.arguments}}]},
  {role: 'tool', name: call.tool, tool_call_id: id, ...(override ?? result)});
}
function physicalReadHistory(directory) {
  const messages = [{role: 'user', content: original}];
  for (const [path, content] of [['a.txt', 'alpha'], ['b.txt', 'beta']]) {
    appendPhysical(messages, directory, {tool: 'write',
      arguments: JSON.stringify({path, content})});
  }
  for (const path of ['a.txt', 'b.txt']) {
    appendPhysical(messages, directory, {tool: 'read', arguments: JSON.stringify({path})});
  }
  return messages;
}
test('a first-file receipt cannot certify a second file or its failed readback', async t => {
  const directory = mkdtempSync(join(tmpdir(), 'literal-obligation-failure-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  const messages = physicalReadHistory(directory);
  const failed = messages.at(-1);
  failed.content = 'Read failed: b.txt';
  failed.is_error = true;
  failed.source_read = {path: 'b.txt', success: false, complete: false, format: 'raw'};
  const plan = await planObligationsStep(original, messages, ['read', 'write'], obligations(original));
  assert.equal(plan.kind, 'final');
  assert.equal(successfullyDischarged(original, messages), false);
  assert.equal(canDeliverFinal(plan), false);
  assert.match(plan.answer, /b\.txt/);
  assert.doesNotMatch(plan.answer, /Completed the general change request/);
  assert.equal(readFileSync(join(directory, 'a.txt'), 'utf8'), 'alpha');
  assert.equal(readFileSync(join(directory, 'b.txt'), 'utf8'), 'beta');
});
test('unbound bare shell receipts without writes cannot discharge either artifact', async () => {
  const messages = [{role: 'user', content: original}];
  for (const [id, command, content] of [
    ['first', 'cat a.txt', 'Output: alpha\nExit Code: 0'],
    ['second', 'cat b.txt', 'Output: beta\nExit Code: 1']
  ]) {
    messages.push({role: 'assistant', tool_calls: [{id, type: 'function',
      function: {name: 'bash', arguments: JSON.stringify({command})}}]},
    {role: 'tool', name: 'bash', tool_call_id: id, content});
  }
  const plan = await planObligationsStep(original, messages, ['bash', 'write'], obligations(original));
  assert.equal(successfullyDischarged(original, messages), false);
  assert.equal(canDeliverFinal(plan), false);
  assert.equal(plan.kind, 'tool_calls');
  assert.equal(plan.calls[0].tool, 'write');
  assert.equal(JSON.parse(plan.calls[0].arguments).path, 'a.txt');
});
test('refused, partial, foreign, wrong and stale Read proof never completes the transaction', async t => {
  const directory = mkdtempSync(join(tmpdir(), 'literal-obligation-refusal-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  const good = physicalReadHistory(directory);
  assert.equal(successfullyDischarged(original, good), true);
  const mutations = [
    messages => { messages.at(-1).is_error = true; },
    messages => { messages.at(-1).source_read.complete = false; },
    messages => { messages.at(-1).source_read.path = 'a.txt'; },
    messages => { delete messages.at(-1).source_read; },
    messages => { messages.at(-1).content = 'wrong'; },
    messages => { messages.at(-1).content += '\n'; },
    messages => { messages.at(-1).tool_call_id = 'unmatched'; },
    messages => {
      appendPhysical(messages, directory, {tool: 'write',
        arguments: JSON.stringify({path: 'b.txt', content: 'beta'})});
    }
  ];
  for (const mutate of mutations) {
    const messages = structuredClone(good);
    mutate(messages);
    assert.equal(successfullyDischarged(original, messages), false);
    const plan = await planObligationsStep(original, messages, ['read', 'write'], obligations(original));
    assert.equal(canDeliverFinal(plan), false);
    assert.equal(readFileSync(join(directory, 'a.txt'), 'utf8'), 'alpha');
    assert.equal(readFileSync(join(directory, 'b.txt'), 'utf8'), 'beta');
  }
});
test('shell-only artifacts discharge through actual framed post-write observations', async () => {
  const expected = {'a.txt': 'alpha', 'b.txt': 'beta'};
  exactReceipts(await fixture(original, expected, {tools: ['write', 'bash']}), expected);
});
test('write-only delivery retains first target and reports verification unavailable', async () => {
  const out = await fixture(original, { 'a.txt': 'alpha', 'b.txt': null }, { tools: ['write'] });
  assert.equal(out.transcript.length, 1);
  assert.equal(out.transcript[0].tool, 'write');
  assert.equal(out.stop, 'final');
  assert.doesNotMatch(out.answer, /Completed the general change request/);
  const messages = [{ role: 'user', content: original }, { role: 'assistant', tool_calls: [{ id: 'write', type: 'function', function: { name: 'write', arguments: out.transcript[0].arguments } }] }, { role: 'tool', name: 'write', tool_call_id: 'write', content: out.transcript[0].result }];
  const final = await planChatStepResolved(messages, ['write']);
  assert.equal(finalResult(final).disposition, FinalDisposition.Gap);
  const projected = await planChatStep(messages, ['write']);
  assert.equal(finalResult(projected).disposition, FinalDisposition.Unknown);
});
test('read-only clients cannot mutate either literal target', async () => {
  const out = await fixture(original, { 'a.txt': 'existing', 'b.txt': null }, { tools: ['read'], initial: { 'a.txt': 'existing' } });
  assert.ok(out.transcript.every((entry) => entry.tool === 'read'));
  assert.doesNotMatch(out.answer ?? '', /Completed the general change request/);
});
test('a successful artifact never hides an underivable mandatory clause', async () => {
  const task = 'First, create file a.txt with exactly this content «alpha». Second, derive the missing compiler contract.';
  const out = await fixture(task, { 'a.txt': 'alpha' });
  assert.equal(out.stop, 'final');
  assert.match(out.answer, /no_artifact_in_clause/);
  assert.doesNotMatch(out.answer, /Completed the general change request/);
});

test('all seeded enumeration surfaces inside literal operands remain payload', () => {
  const quote = String.fromCharCode(96);
  const wrappers = [['«', '»'], [quote, quote], [quote.repeat(4) + 'text' + String.fromCharCode(10), String.fromCharCode(10) + quote.repeat(4)], [quote.repeat(12) + 'text' + String.fromCharCode(10), String.fromCharCode(10) + quote.repeat(12)]];
  const surfaces = wordsForRole('enumeration_cue');
  assert.equal(surfaces.length, 34);
  for (const surface of surfaces) {
    for (const [opening, closing] of wrappers) {
      const literal = opening + 'λ🙂 Alpha. ' + surface + ' beta.' + closing;
      const request = 'Create a.txt with exactly this content ' + literal;
      const clauses = clausesWithSpans(request);
      assert.equal(clauses.length, 1, surface + ' stays in literal');
      assert.equal(clauses[0][0], request);
      assert.deepEqual(clauses[0][1], [0, Buffer.byteLength(request)]);
    }
  }
});
test('every seeded outer cue preserves two complete literals and exact UTF8 spans', () => {
  for (const surface of wordsForRole('enumeration_cue')) {
    const request = 'First, create a.txt with exactly this content «λ🙂 Alpha. Next beta.». ' + surface + ', create b.txt with exactly this content «Gamma. Then delta.».';
    const clauses = clausesWithSpans(request);
    assert.equal(clauses.length, 2, surface + ' starts a genuine outer obligation');
    for (const [clause, [start, end]] of clauses) assert.equal(Buffer.from(request).subarray(start, end).toString('utf8'), clause);
    assert.ok(clauses[0][0].includes('«λ🙂 Alpha. Next beta.»'));
    assert.ok(clauses[1][0].includes('«Gamma. Then delta.»'));
  }
});
test('closed twelve-character fence with internal instructions has one exact readback', async () => {
  const quote = String.fromCharCode(96);
  const body = ['λ🙂 Alpha.', 'Next beta.', 'When ' + quote + 'input' + quote + ' then ' + quote + 'output' + quote + '.', 'Also list behavior rules.', ''].join(String.fromCharCode(10));
  const request = 'Set the contents of a.txt to exactly this content:' + String.fromCharCode(10) + quote.repeat(12) + 'text' + String.fromCharCode(10) + body + quote.repeat(12);
  const result = await fixture(request, { 'a.txt': body });
  exactReceipts(result, { 'a.txt': body });
  assert.equal(result.transcript.filter((entry) => entry.tool === 'write').length, 1);
  assert.doesNotMatch(result.answer, /Behavior rule compiled|Understood\. I'll avoid/);
});
test('independent outer artifacts retain nested enumeration bytes and receipts', async () => {
  const first = 'λ🙂 Alpha. Next beta.';
  const second = 'Gamma. Then delta.';
  const request = 'First, create a.txt with exactly this content «' + first + '». Second, create b.txt with exactly this content «' + second + '».';
  exactReceipts(await fixture(request, { 'a.txt': first, 'b.txt': second }), { 'a.txt': first, 'b.txt': second });
});

function physicalShellHistory(directory) {
  const messages = [{role: 'user', content: original}];
  for (const [path, content] of [['a.txt', 'alpha'], ['b.txt', 'beta']]) {
    appendPhysical(messages, directory, {tool: 'write',
      arguments: JSON.stringify({path, content})});
  }
  for (const path of ['a.txt', 'b.txt']) {
    appendPhysical(messages, directory, {tool: 'bash',
      arguments: JSON.stringify({command: ownedCreationReadbackCommand(path)})});
  }
  return messages;
}
test('only exact known qualified one-command readbacks may cross another artifact proof', async t => {
  const directory = mkdtempSync(join(tmpdir(), 'literal-readonly-command-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  const good = physicalShellHistory(directory);
  assert.equal(successfullyDischarged(original, good), true);
  const mutations = [
    messages => {
      messages.at(-2).tool_calls[0].function.arguments = JSON.stringify({
        command: ownedCreationReadbackCommand('b.txt'), extra: 'not declared'
      });
    },
    messages => {
      messages.at(-2).tool_calls[0].function.arguments = JSON.stringify({
        command: 'cat b.txt'
      });
    },
    messages => {
      messages.at(-2).tool_calls[0].function.arguments = JSON.stringify({
        command: ownedCreationReadbackCommand('other.txt')
      });
    },
    messages => { messages.at(-1).tool_call_id = 'unmatched'; },
    messages => { messages.at(-1).name = 'run_command'; },
    messages => {
      const args = JSON.parse(messages[3].tool_calls[0].function.arguments);
      args.filePath = 'other.txt';
      messages[3].tool_calls[0].function.arguments = JSON.stringify(args);
    }
  ];
  for (const mutate of mutations) {
    const messages = structuredClone(good);
    mutate(messages);
    const progress = Progress.scan(messages);
    const attempt = progress.attempts.at(-1);
    assert.equal(readOnlyAttempt(progress, attempt, qualifiedCommandAttempt, sourceReadPath), false);
    assert.equal(successfullyDischarged(original, messages), false);
    const plan = await planObligationsStep(original, messages, ['write', 'bash'], obligations(original));
    assert.equal(canDeliverFinal(plan), false);
    assert.equal(readFileSync(join(directory, 'a.txt'), 'utf8'), 'alpha');
    assert.equal(readFileSync(join(directory, 'b.txt'), 'utf8'), 'beta');
  }
});
test('a failed known readback is readonly but never a successful artifact proof', async t => {
  const directory = mkdtempSync(join(tmpdir(), 'literal-readonly-failed-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  const messages = physicalShellHistory(directory);
  messages.at(-1).content = 'Output: (beta)\nExit Code: 1';
  messages.at(-1).is_error = true;
  const progress = Progress.scan(messages);
  assert.equal(readOnlyAttempt(progress, progress.attempts.at(-1),
    qualifiedCommandAttempt, sourceReadPath), true);
  const clauses = obligations(original).map(node => composeGeneralChangePlan(node.clause));
  assert.ok(ownedCreationObservation(clauses[0], progress));
  assert.equal(ownedCreationObservation(clauses[1], progress), null);
  assert.equal(successfullyDischarged(original, messages), false);
  const plan = await planObligationsStep(original, messages, ['write', 'bash'], obligations(original));
  assert.equal(canDeliverFinal(plan), false);
  assert.equal(readFileSync(join(directory, 'a.txt'), 'utf8'), 'alpha');
  assert.equal(readFileSync(join(directory, 'b.txt'), 'utf8'), 'beta');
});
