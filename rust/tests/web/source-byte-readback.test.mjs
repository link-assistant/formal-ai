// Exact source observations require successful, target-bound command receipts.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { mkdtempSync, readFileSync, writeFileSync, mkdirSync, rmSync, existsSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { observedBytesMatch, observedPayload } from '../../../js/agentic/tool_result.mjs';
import { resultForCommand } from '../../../js/agentic/code_artifact.mjs';
import { planWorkspaceChangeStep } from '../../../js/agentic/workspace_change.mjs';
import { planStructuredEditStep } from '../../../js/agentic/structured_edit.mjs';
import { planGeneratedSourceStep } from '../../../js/agentic/code_task.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
const shell = stdout => 'Output: ' + stdout + '\nExit Code: 0';
const json = stdout => JSON.stringify({ stdout, exit_code: 0 });
function record(messages, command, result, id = 'call-' + messages.length) {
  messages.push({ role: 'assistant', tool_calls: [{ id, function: { name: 'bash', arguments: JSON.stringify({ command }) } }] },
    { role: 'tool', tool_call_id: id, content: result });
}
test('explicit successful receipts preserve exact JSON, Unicode, whitespace and authored error narratives', () => {
  for (const bytes of ['', '  λ🙂\n', '{"a":1}\n', 'Error: this is authored data\nfailed is also data\n', 'x\r\n']) {
    for (const receipt of [shell, json]) {
      const raw = receipt(bytes);
      assert.equal(observedPayload(raw), bytes);
      assert.equal(observedBytesMatch(raw, bytes), true);
      assert.equal(observedBytesMatch(raw, bytes + '!'), false);
    }
  }
});
test('matching bytes cannot erase missing status, failure status or typed harness errors', () => {
  const bytes = 'exact\n';
  for (const raw of [bytes, JSON.stringify({ stdout: bytes }), shell(bytes).replace('Exit Code: 0', 'Exit Code: 1'),
    JSON.stringify({ stdout: bytes, exit_code: 1 }), JSON.stringify({ stdout: bytes, exit_code: 0, is_error: true }),
    JSON.stringify({ stdout: bytes, exit_code: 0, error: 'denied' })]) assert.equal(observedBytesMatch(raw, bytes), false);
});
test('lookup binds exact command, nearest call identity and latest matching result', () => {
  const messages = [{ role: 'user', content: 'verify source' }];
  record(messages, 'cat f.txt', shell('old'));
  record(messages, 'cat other.txt', shell('new'));
  assert.equal(resultForCommand(messages, 'cat f.txt'), shell('old'));
  assert.equal(resultForCommand(messages, 'cat missing.txt'), null);
  record(messages, 'cat f.txt', JSON.stringify({ stdout: 'old', exit_code: 1 }));
  assert.equal(observedBytesMatch(resultForCommand(messages, 'cat f.txt'), 'old'), false);
  record(messages, 'cat unrelated.txt', shell('new'), 'reused');
  record(messages, 'cat f.txt', shell('new'), 'reused');
  assert.equal(resultForCommand(messages, 'cat f.txt'), shell('new'));
});
const cases = [
  { label: 'whole rewrite without Edit', verification: 'digest', plan: planWorkspaceChangeStep, task: 'In f.txt replace every «old» with «new».', path: 'f.txt', source: 'λ🙂 old old\n', expected: 'λ🙂 new new\n' },
  { label: 'structured list insertion', verification: 'digest', plan: planStructuredEditStep, task: 'In f.rs add «c» to the list ITEMS alongside «a» and «b».', path: 'f.rs', source: 'const ITEMS: &[&str] = &["a", "b"];\n', expected: 'const ITEMS: &[&str] = &["a", "b", "c"];\n' },
  { label: 'generated source', plan: planGeneratedSourceStep, task: 'Create f.rs containing a public Rust function named held_out_value returning 29.', path: 'f.rs', source: null, expected: 'pub fn held_out_value() -> i64 {\n    29\n}\n' }
];
function verificationCommand(item, path = item.path) {
  return (item.verification === 'digest' ? 'sha256sum -- ' : 'cat ') + path;
}
function expectedObservation(item, path = item.path) {
  return item.verification === 'digest'
    ? createHash('sha256').update(item.expected, 'utf8').digest('hex') + '  ' + path + '\n' : item.expected;
}
function run(item, receipt) {
  const root = mkdtempSync(join(tmpdir(), 'formal-ai-byte-receipt-'));
  const target = join(root, item.path);
  if (item.source !== null) writeFileSync(target, item.source);
  const messages = [{ role: 'user', content: item.task }];
  let answer = null;
  const calls = [];
  try {
    for (let step = 0; step < 8; step += 1) {
      const plan = item.plan(item.task, messages, ['read', 'write', 'bash']);
      assert.ok(plan, item.label);
      if (plan.kind === 'final') { answer = plan.answer; break; }
      assert.equal(plan.calls.length, 1);
      const call = plan.calls[0], args = JSON.parse(call.arguments);
      calls.push(call);
      let result;
      if (call.tool === 'read') result = existsSync(join(root, args.path)) ? readFileSync(join(root, args.path), 'utf8') : 'Error: ENOENT: no such file or directory';
      else if (call.tool === 'write') { mkdirSync(dirname(join(root, args.path)), { recursive: true }); writeFileSync(join(root, args.path), args.content); result = ''; }
      else {
        assert.equal(args.command, verificationCommand(item));
        const stdout = execFileSync('/bin/sh', ['-c', args.command], { cwd: root, encoding: 'utf8' });
        assert.equal(stdout, expectedObservation(item));
        assert.equal(readFileSync(target, 'utf8'), item.expected);
        result = receipt(stdout);
      }
      const id = 'tool-' + step;
      messages.push({ role: 'assistant', tool_calls: [{ id, function: { name: call.tool, arguments: call.arguments } }] },
        { role: 'tool', tool_call_id: id, content: result });
    }
    assert.equal(readFileSync(target, 'utf8'), item.expected);
    assert.ok(answer);
    return { answer, messages, calls };
  } finally { rmSync(root, { recursive: true, force: true }); }
}
for (const item of cases) {
  test(item.label + ' accepts actual exact shell and typed JSON observations', () => {
    for (const receipt of [shell, json]) {
      const outcome = run(item, receipt);
      assert.doesNotMatch(outcome.answer, /Verification failed/i);
      assert.equal(outcome.calls.filter(call => call.tool === 'write').length, 1);
      assert.equal(outcome.calls.filter(call => call.tool === 'bash').length, 1);
    }
  });
  test(item.label + ' rejects failed matching observation even after correct physical write', () => {
    for (const receipt of [stdout => JSON.stringify({ stdout, exit_code: 1 }), stdout => JSON.stringify({ stdout, exit_code: 0, is_error: true }), stdout => shell(item.verification === 'digest' ? '0'.repeat(64) + stdout.slice(64) : stdout + '!')]) {
      assert.match(run(item, receipt).answer, /Verification failed/i);
    }
  });
  test(item.label + ' ignores unrelated commands and obeys newest failed target receipt', () => {
    const outcome = run(item, shell);
    const window = outcome.messages.slice(0, -2);
    record(window, verificationCommand(item, 'unrelated.txt'), shell(expectedObservation(item, 'unrelated.txt')));
    const pending = item.plan(item.task, window, ['read', 'write', 'bash']);
    assert.equal(pending.kind, 'tool_calls');
    assert.equal(JSON.parse(pending.calls[0].arguments).command, verificationCommand(item));
    record(window, verificationCommand(item), shell(expectedObservation(item)));
    record(window, verificationCommand(item), JSON.stringify({ stdout: expectedObservation(item), exit_code: 1 }));
    assert.match(item.plan(item.task, window, ['read', 'write', 'bash']).answer, /Verification failed/i);
  });
  if (item.verification === 'digest') {
    test(item.label + ' rejects a successful wrong hash while physical source bytes remain exact', () => {
      const wrongHash = '0'.repeat(64);
      assert.notEqual(wrongHash, createHash('sha256').update(item.expected, 'utf8').digest('hex'));
      assert.match(run(item, stdout => json(wrongHash + stdout.slice(64))).answer, /Verification failed/i);
    });
  }
}

test('presentation or ambiguous transport cannot certify unobserved byte strings', () => {
  assert.equal(observedBytesMatch(JSON.stringify({ stdout: { a: 1 }, exit_code: 0 }), '{\n  "a": 1\n}'), false);
  assert.equal(observedBytesMatch('Directory: .\nOutput: x\n\nExit Code: 0', 'x'), false);
  for (const raw of [JSON.stringify({ stdout: 'x', exit_code: 0, status: 'blocked', is_error: true }),
    JSON.stringify({ stdout: 'x', exit_code: 0, status: 'blocked', error: 'denied' })]) assert.equal(observedBytesMatch(raw, 'x'), false);
  assert.equal(observedBytesMatch('Command: cat f.txt\nOutput: x\n\nExit Code: 0', 'x\n'), true);
});

function composite(receipt) {
  const root = mkdtempSync(join(tmpdir(), 'formal-ai-composite-receipt-'));
  const task = 'Create f.rs containing a public Rust function named held_out_value returning 29 and register the module in lib.rs.';
  const expected = { 'f.rs': 'pub fn held_out_value() -> i64 {\n    29\n}\n', 'lib.rs': 'pub mod base;\npub mod f;\n' };
  writeFileSync(join(root, 'lib.rs'), 'pub mod base;\n');
  const messages = [{ role: 'user', content: task }];
  const receipts = [];
  let answer;
  try {
    for (let step = 0; step < 8; step += 1) {
      const plan = planWorkspaceChangeStep(task, messages, ['read', 'write', 'bash']);
      assert.ok(plan);
      if (plan.kind === 'final') { answer = plan.answer; break; }
      const call = plan.calls[0], args = JSON.parse(call.arguments);
      let result = '';
      if (call.tool === 'read') result = readFileSync(join(root, args.path), 'utf8');
      else if (call.tool === 'write') writeFileSync(join(root, args.path), args.content);
      else {
        const path = args.command.slice(4);
        assert.equal(args.command, 'cat ' + path);
        assert.ok(Object.hasOwn(expected, path));
        const stdout = execFileSync('/bin/sh', ['-c', args.command], { cwd: root, encoding: 'utf8' });
        assert.equal(stdout, expected[path]);
        receipts.push(args.command);
        result = receipt(stdout, path);
      }
      const id = 'composite-' + step;
      messages.push({ role: 'assistant', tool_calls: [{ id, function: { name: call.tool, arguments: call.arguments } }] },
        { role: 'tool', tool_call_id: id, content: result });
    }
    assert.ok(answer);
    assert.equal(readFileSync(join(root, 'f.rs'), 'utf8'), expected['f.rs']);
    return { answer, receipts, registry: readFileSync(join(root, 'lib.rs'), 'utf8'), expected };
  } finally { rmSync(root, { recursive: true, force: true }); }
}
test('composite source and whole registration require independent exact actual receipts', () => {
  for (const receipt of [shell, json]) {
    const outcome = composite(receipt);
    assert.deepEqual(outcome.receipts, ['cat f.rs', 'cat lib.rs']);
    assert.equal(outcome.registry, outcome.expected['lib.rs']);
    assert.doesNotMatch(outcome.answer, /Verification failed/i);
  }
});
test('either failed composite receipt blocks completion while preserving actual file assertions', () => {
  for (const target of ['f.rs', 'lib.rs']) {
    const outcome = composite((stdout, path) => path === target ? JSON.stringify({ stdout, exit_code: 1 }) : shell(stdout));
    assert.match(outcome.answer, /Verification failed/i);
    assert.equal(outcome.registry, target === 'f.rs' ? 'pub mod base;\n' : outcome.expected['lib.rs']);
    assert.deepEqual(outcome.receipts, target === 'f.rs' ? ['cat f.rs'] : ['cat f.rs', 'cat lib.rs']);
  }
});
