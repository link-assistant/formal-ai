// Exercise the historical native prompts through actual JS planner and workspace I/O.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStepResolved } from '../../../js/agentic/planner.mjs';
import { composeGeneralChangePlan, PLAN_PATH } from '../../../js/agentic/general_planner.mjs';
import { ownedCreationReadbackCommand } from '../../../js/agentic/general_execution.mjs';
import { executeResult } from '../../../experiments/js_dogfood/drive.mjs';
before(async () => installNodeHost(new WorkerHost()));
const PROMPT = 'Create a file hello.txt containing exactly: Hello World';
const LEGACY_PROMPT = 'Set the contents of hello.txt to «Hello World»';
const TOOLS = ['read_file', 'write_file', 'run_command'];
const aliases = { read_file: 'read', write_file: 'write', run_command: 'bash', run_shell_command: 'bash', bash: 'bash' };
async function run(prompt, { tools = TOOLS, inject = null } = {}) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-observed-contract-'));
  const messages = [{ role: 'user', content: prompt }], calls = [];
  try {
    for (let turn = 0; turn < 12; turn += 1) {
      const step = await planChatStepResolved(messages, tools);
      assert.ok(step, 'a declared plan must progress');
      if (step.kind === 'final') return { calls, answer: step.answer, result: step.result, messages,
        read: path => readFileSync(join(directory, path), 'utf8') };
      assert.equal(step.kind, 'tool_calls');
      assert.equal(step.calls.length, 1);
      const call = step.calls[0], args = JSON.parse(call.arguments);
      const override = inject?.(call, args, calls);
      const receipt = override === undefined || override === null
        ? executeResult(directory, { ...call, tool: aliases[call.tool] })
        : typeof override === 'string' ? {content: override} : override;
      const output = receipt.content;
      calls.push({ ...call, args, result: output, injected: override !== undefined && override !== null });
      const id = 'contract-' + turn;
      messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function',
        function: { name: call.tool, arguments: call.arguments } }] },
        { role: 'tool', tool_call_id: id, name: call.tool, ...receipt });
    }
    assert.fail('bounded contract did not terminate');
  } finally {
    // Capture bytes before removing the selected workspace.
    const record = calls.findLast(call => call.tool === 'write_file' && call.args.path !== PLAN_PATH);
    if (record) record.actualBytes = (() => { try { return readFileSync(join(directory, record.args.path), 'utf8'); } catch { return null; } })();
    rmSync(directory, { recursive: true, force: true });
  }
}
function assertTarget(run, path, bytes) {
  const writes = run.calls.filter(call => call.tool === 'write_file' && call.args.path === path);
  assert.equal(writes.length, 1);
  assert.equal(writes[0].args.content, bytes);
  assert.equal(writes[0].actualBytes, bytes);

}
const literals = [
['Create a file a.txt containing exactly: alpha','a.txt','alpha'],
['Create file b.txt containing exactly beta','b.txt','beta'],
['Please create the file c.txt containing exactly: two words','c.txt','two words'],
['Create a file d.txt containing exactly, punctuation stays!','d.txt',', punctuation stays!'],
['Create a file e.txt containing exactly \`code bytes\`','e.txt','code bytes'],
['Create a file f.txt containing exactly: 12345','f.txt','12345'],
['Create a file g.txt containing exactly: Mixed CASE','g.txt','Mixed CASE'],
['Create a file h.txt containing exactly: a:b:c','h.txt','a:b:c'],
['Create a file i.txt containing exactly: braces {stay}','i.txt','braces {stay}'],
['Create a file j.txt containing exactly: symbols #!?','j.txt','symbols #!?'],
['Create a file k.txt containing exactly: tabs are words','k.txt','tabs are words'],
['Create a file l.txt containing exactly: slash/value','l.txt','slash/value'],
['Create a file m.txt containing exactly: dot.value','m.txt','dot.value'],
['Create a file n.txt containing exactly: under_score','n.txt','under_score'],
['Create a file o.txt containing exactly: dash-value','o.txt','dash-value'],
['Create a file p.txt containing exactly: quoted value','p.txt','quoted value'],
['Create a file q.txt containing exactly: unicode café','q.txt','unicode café'],
['Create a file r.txt containing exactly: JSON-ish [1,2]','r.txt','JSON-ish [1,2]'],
['Create a file s.txt containing exactly: equals=a','s.txt','equals=a'],
['Create a file t.txt containing exactly: final payload','t.txt','final payload'],
['Create a file named notes.txt with the content first draft','notes.txt','first draft'],
['Save a file called report.md with the text quarterly summary','report.md','quarterly summary'],
["Write hello.py saying print('hi')",'hello.py',"print('hi')"],
['Generate a file named data.json containing {"ok": true}','data.json','{"ok": true}'],
['new file: notes.txt, contents: hello','notes.txt','hello']
];
test('literal matrix binds declared targets and exact bytes without executing tools', () => {
  for (const [prompt, target, content] of literals) {
    const plan = composeGeneralChangePlan(prompt);
    assert.equal(plan.mode, 'literal_file', prompt);
    assert.equal(plan.target, target, prompt);
    assert.equal(plan.content, content, prompt);
    assert.notEqual(plan.target, PLAN_PATH);
  }
});

test('original literal matrices preserve target bytes through target-only fresh readback', {
  skip: process.env.FAST_PR_PROFILE === 'true'
    ? '25-delivery planner and physical workspace matrix runs on main; contract and refusal controls remain active'
    : false,
}, async () => {
  for (const [prompt, path, bytes] of literals) {
    const result = await run(prompt);
    assertTarget(result, path, bytes);
    assert.deepEqual(result.calls.map(call => call.tool), ['write_file', 'read_file']);
    assert.ok(result.calls.every(call => call.args.path === path));
    assert.ok(!result.calls.some(call => call.args.path === PLAN_PATH));
    assertVerified(result);
  }
});

function completeRawRead(path, content) {
  return {content, source_read: {path, success: true, complete: true, format: 'raw'}};
}
function assertVerified(result) {
  assert.equal(result.result?.disposition, 'finding');
  assert.match(result.answer, /Completed the (?:file change|general change request)/u);
}
function assertUnverified(result) {
  assert.notEqual(result.result?.disposition, 'finding');
  assert.doesNotMatch(result.answer, /Completed the (?:file change|general change request)/u);
}
for (const [name, output, success] of [
  ['explicit error', {content: JSON.stringify({is_error: true,
    content: 'cat: hello.txt: No such file or directory\nExit Code: 1'}),
    is_error: true, source_read: {path: 'hello.txt', success: false,
      complete: false, format: 'raw', error_code: 'EIO'}}, false],
  ['nonzero status', {content: JSON.stringify({exit_code: 1, status: 'failed',
    output: 'cat: hello.txt: No such file'}), is_error: true,
    source_read: {path: 'hello.txt', success: false, complete: false,
      format: 'raw', exit_code: 1}}, false],
  ['wrong bytes', completeRawRead('hello.txt', 'Goodbye World'), false],
  ['extra newline bytes', completeRawRead('hello.txt', 'Hello World\n'), false],
  ['truncated matching bytes', {content: 'Hello World', source_read: {
    path: 'hello.txt', success: true, complete: false, format: 'raw'}}, false],
  ['foreign matching bytes', completeRawRead('other.txt', 'Hello World'), false],
  ['matching bytes', completeRawRead('hello.txt', 'Hello World'), true]
]) test('original verification outcome: ' + name, async () => {
  const result = await run(PROMPT, {
    inject: (call, args) => call.tool === 'read_file' && args.path === 'hello.txt'
      ? output : null
  });
  assertTarget(result, 'hello.txt', 'Hello World');
  assert.equal(result.calls.filter(call => call.injected).length, 1);
  assert.deepEqual(result.calls.map(call => call.tool), ['write_file', 'read_file']);
  if (success) assertVerified(result);
  else assertUnverified(result);
});
test('target write is retried once after an actual missing-target read, then stops', async () => {
  const result = await run(LEGACY_PROMPT, { inject: (call, args) => call.tool === 'write_file' && args.path === 'hello.txt'
    ? JSON.stringify({ is_error: true, error: 'Read the file before writing' }) : null });
  const targets = result.calls.filter(call => call.args.path === 'hello.txt');
  assert.deepEqual(targets.map(call => call.tool), ['write_file', 'read_file', 'write_file']);
  assert.ok(targets.every(call => call.tool === 'read_file' || call.args.content === 'Hello World'));
  assert.equal(result.calls.filter(call => call.args.command === 'cat hello.txt').length, 1);
  assert.ok(!result.answer.includes('Completed the general change request'));
  assert.match(result.answer, /hello\.txt/u);
  assert.match(result.answer, /Exit Code: 1/u);
});
test('write-only target succeeds with an explicit missing auxiliary receipt', async () => {
  const result = await run(PROMPT, { tools: ['write_file'] });
  assertTarget(result, 'hello.txt', 'Hello World');
  assert.equal(result.calls.length, 1);
  assert.equal(result.result?.disposition, 'gap');
  assertUnverified(result);
});
test('write-only actual target error is terminal and never becomes completion', async () => {
  const result = await run(PROMPT, { tools: ['write_file'], inject: () => '<tool_use_error>Error writing file</tool_use_error>' });
  assert.equal(result.calls.length, 1);
  assert.equal(result.calls[0].args.path, 'hello.txt');
  assert.match(result.answer, /Error writing file/u);
  assert.ok(!result.answer.includes('Completed the general change request'));
});
test('shell-only event persistence uses actual command stdout before target verification', async () => {
  for (const tools of [['write_file', 'run_shell_command'], ['write_file', 'bash']]) {
    const result = await run(LEGACY_PROMPT, { tools });
    assertTarget(result, 'hello.txt', 'Hello World');
    assert.ok(result.calls[0].args.command.startsWith('mkdir -p -- .formal-ai && (lock='));
    assert.ok(result.calls[0].result.includes(composeGeneralChangePlan(PROMPT).target));
    assert.match(result.answer, /Completed the general change request/u);
  }
});

test('a generic command declaration does not grant an atomic shell journal', async () => {
  const result = await run(PROMPT, { tools: ['write_file', 'run_command'] });
  assertTarget(result, 'hello.txt', 'Hello World');
  assert.deepEqual(result.calls.map(call => call.tool), ['write_file', 'run_command']);
  assert.equal(result.calls[0].args.path, 'hello.txt');
  assert.equal(result.calls[1].args.command, ownedCreationReadbackCommand('hello.txt'));
  assert.ok(!result.calls.some(call => call.args.path === PLAN_PATH));
  assertVerified(result);
});

test('original Create failed precondition is terminal without retries or physical effects', async () => {
  const result = await run(PROMPT, {
    inject: call => call.tool === 'write_file'
      ? JSON.stringify({is_error: true, error: 'Read the file before writing'}) : null
  });
  assert.deepEqual(result.calls.map(call => call.tool), ['write_file']);
  assert.equal(result.calls[0].args.path, 'hello.txt');
  assert.equal(result.calls[0].args.content, 'Hello World');
  assert.equal(result.calls[0].actualBytes, null);
  assert.equal(result.result?.disposition, 'failure');
  assertUnverified(result);
});
