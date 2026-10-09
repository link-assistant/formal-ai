import assert from 'node:assert/strict';
import test from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());
const { Progress } = await import('../../../js/agentic/progress.mjs');
const { recordReadbackCommand, recordObservationOutcome, planRecordReadbackStep } = await import('../../../js/agentic/evidence_record/record_observation.mjs');
const { finalResult, canDeliverFinal } = await import('../../../js/agentic/final_result.mjs');
const target = 'reports/result.md';
const content = 'node_path=fixture\nExact café αβ result observed.\n';
const initial = () => [{ role: 'user', content: 'Read input.txt and record the actual result in reports/result.md.' }];
function receipt(messages, name, argumentsValue, raw) {
  const id = 'record-' + messages.length;
  messages.push({ role: 'assistant', tool_calls: [{ id, type: 'function', function: { name, arguments: JSON.stringify(argumentsValue) } }] });
  messages.push({ role: 'tool', tool_call_id: id, name, content: raw });
}
function written() {
  const messages = initial();
  receipt(messages, 'write', { path: target, content }, '');
  return messages;
}
const shell = (stdout, exit_code = 0) => JSON.stringify({ stdout, exit_code });
const judge = (messages) => recordObservationOutcome(Progress.scan(messages), target, content).kind;
const plan = (messages, tools = ['read', 'write', 'bash']) => planRecordReadbackStep(initial()[0].content, target, content, Progress.scan(messages), tools);

test('a Write acknowledgement, including echoed content, remains pending observation', () => {
  for (const raw of ['', 'created', content]) {
    const messages = initial();
    receipt(messages, 'write', { path: target, content }, raw);
    assert.equal(judge(messages), 'pending');
    const next = plan(messages);
    assert.equal(next.kind, 'tool_calls');
    assert.equal(JSON.parse(next.calls[0].arguments).command, 'cat ' + target);
  }
});
test('a prior successful receipt cannot certify a later Write', () => {
  const messages = initial();
  receipt(messages, 'bash', { command: 'cat ' + target }, shell(content));
  receipt(messages, 'write', { path: target, content }, '');
  assert.equal(judge(messages), 'pending');
});
test('explicit successful shell receipts certify exact Unicode and newline bytes', () => {
  for (const raw of [shell(content), 'Output: ' + content + '\nExit Code: 0']) {
    const messages = written();
    receipt(messages, 'bash', { command: 'cat ' + target }, raw);
    assert.equal(judge(messages), 'verified');
    assert.equal(plan(messages), null);
  }
});
test('bare Read and framed Agent Read preserve exact content compatibility', () => {
  const framed = '<file>\n00001| node_path=fixture\n00002| Exact café αβ result observed.\n00003| \n\n(End of file - total 3 lines)\n</file>';
  for (const raw of [content, framed]) {
    const messages = written();
    receipt(messages, 'read', { filePath: target }, raw);
    assert.equal(judge(messages), 'verified');
  }
});
test('different paths and auxiliary commands never certify this record', () => {
  for (const [name, argumentsValue] of [['read', { path: 'other-reports/result.md' }], ['bash', { command: 'cat other-' + target }], ['bash', { command: 'printf ' + target }]]) {
    const messages = written();
    receipt(messages, name, argumentsValue, shell(content));
    assert.equal(judge(messages), 'pending');
  }
});
test('wrong bytes, bare shell stdout, nonzero and structured stdout remain failures', () => {
  for (const raw of [shell(content + 'wrong'), content, shell(content, 1), JSON.stringify({ stdout: { result: content }, exit_code: 0 }), JSON.stringify({ stdout: content, exit_code: 0, is_error: true })]) {
    const messages = written();
    receipt(messages, 'bash', { command: 'cat ' + target }, raw);
    assert.ok(['failed', 'mismatch'].includes(judge(messages)));
    const final = plan(messages);
    assert.equal(finalResult(final).disposition, 'failure');
    assert.equal(canDeliverFinal(final), false);
  }
});
test('a later failed receipt or failed Write cannot reuse earlier certification', () => {
  for (const name of ['bash', 'write']) {
    const messages = written();
    receipt(messages, 'bash', { command: 'cat ' + target }, shell(content));
    receipt(messages, name, name === 'bash' ? { command: 'cat ' + target } : { path: target, content }, JSON.stringify({ error: 'permission denied', is_error: true }));
    assert.equal(judge(messages), 'failed');
    assert.equal(finalResult(plan(messages)).disposition, 'failure');
  }
});
test('missing observation capability retains the written record and a typed Gap', () => {
  const messages = written();
  const final = plan(messages, ['write']);
  assert.equal(finalResult(final).disposition, 'gap');
  assert.equal(canDeliverFinal(final), false);
  assert.ok(final.answer.includes('without read-back verification'));
});
test('Read-only observation support uses the exact bound target', () => {
  const next = plan(written(), ['read', 'write']);
  assert.equal(next.calls[0].tool, 'read');
  assert.equal(JSON.parse(next.calls[0].arguments).path, target);
});
test('unknown written bytes remain a Gap without repeating an observation forever', () => {
  const final = planRecordReadbackStep(initial()[0].content, target, null, Progress.scan(written()), ['read', 'write', 'bash']);
  assert.equal(finalResult(final).disposition, 'gap');
});
test('unsafe filename bytes are quoted as one real shell operand', async () => {
  const fs = await import('node:fs');
  const { tmpdir } = await import('node:os');
  const { join, dirname } = await import('node:path');
  const { execFileSync } = await import('node:child_process');
  const directory = fs.mkdtempSync(join(tmpdir(), 'formal-ai-record-path-'));
  const path = "reports/a b';touch injected.md;#.md";
  try {
    fs.mkdirSync(dirname(join(directory, path)), { recursive: true });
    fs.writeFileSync(join(directory, path), content);
    assert.equal(execFileSync('bash', ['-c', recordReadbackCommand(path)], { cwd: directory, encoding: 'utf8' }), content);
    assert.equal(fs.existsSync(join(directory, 'injected.md')), false);
  } finally { fs.rmSync(directory, { recursive: true, force: true }); }
});
