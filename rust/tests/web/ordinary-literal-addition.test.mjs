import { test, before } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, writeFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { ownedAdditiveLiteral, ownedDeclaredCreateFrame } from '../../../js/agentic/planner/owned_goals.mjs';
import { planLiteralAdditionStep } from '../../../js/agentic/literal_addition.mjs';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';

before(async () => { await installNodeHost(new WorkerHost()); });

async function replay(task, source, expected) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-append-'));
  try {
    if (source !== null) writeFileSync(join(directory, 'note.txt'), source);
    const result = await drive(planChatStep, directory, task, {
      tools: ['read', 'write'], steps: 6,
    });
    assert.equal(readFileSync(join(directory, 'note.txt'), 'utf8'), expected);
    assert.deepEqual(result.transcript.map(call => call.tool), ['read', 'write', 'read']);
    const write = JSON.parse(result.transcript[1].arguments);
    assert.equal(write.content, expected);
    assert.equal(Object.keys(write).some(key => /append/i.test(key)), false);
    assert.equal(result.stop, 'final');
    const history = [{ role: 'user', content: task }];
    for (const [index, step] of result.transcript.entries()) {
      const id = `physical-addition-${index}`;
      history.push({ role: 'assistant', content: '', tool_calls: [
        { id, type: 'function', function: { name: step.tool, arguments: step.arguments } },
      ] });
      history.push({ role: 'tool', name: step.tool, tool_call_id: id, content: step.result,
        is_error: step.is_error, source_read: step.source_read });
    }
    const observed = planLiteralAdditionStep(task, history, ['read', 'write']);
    assert.equal(observed.result.disposition, 'finding');
    assert.equal(observed.result.origin, 'literal-addition-observed');
    assert.equal(result.answer, observed.answer);
  } finally { rmSync(directory, { recursive: true, force: true }); }
}

test('unchanged append original preserves the existing source', async () => {
  await replay('append file note.txt containing hello', 'prior bytes', 'prior bytes\nhello\n');
});
for (const source of ['', 'prior\n', 'Error: this is authored text', '{"is_error":true}', 'αβ\n🙂']) {
  test('append treats owned source bytes as data: ' + JSON.stringify(source), async () => {
    const expected = source + (source === '' || source.endsWith('\n') ? '' : '\n') + 'hello\n';
    await replay('append file note.txt containing hello', source, expected);
  });
}
test('provider-observed missing destination creates the additive literal', async () => {
  await replay('append file note.txt containing hello', null, 'hello\n');
});
for (const task of [
  'Do not write. append file note.txt containing hello',
  'Someone said append file note.txt containing hello',
  'Read policy.txt first. append file note.txt containing hello',
  'Unknown obligation. append file note.txt containing hello',
  'append and prepend file note.txt containing hello',
  'append file note.txt containing "hello',
]) {
  test('unowned complete request refuses additive effects: ' + task, async () => {
    assert.equal(ownedAdditiveLiteral(task), null);
    const directory = mkdtempSync(join(tmpdir(), 'formal-append-refusal-'));
    try {
      writeFileSync(join(directory, 'note.txt'), 'unchanged');
      const result = await drive(planChatStep, directory, task, { tools: ['read', 'write'], steps: 6 });
      assert.equal(result.transcript.some(call => call.tool === 'write'), false);
      assert.equal(readFileSync(join(directory, 'note.txt'), 'utf8'), 'unchanged');
    } finally { rmSync(directory, { recursive: true, force: true }); }
  });
}
function readMessages(result) {
  return [
    { role: 'user', content: 'append file note.txt containing hello' },
    { role: 'assistant', tool_calls: [{ id: 'r', type: 'function', function: {
      name: 'read', arguments: JSON.stringify({ file_path: 'note.txt' }),
    } }] },
    { role: 'tool', tool_call_id: 'r', ...result },
  ];
}
for (const result of [
  { content: 'prior bytes' },
  { content: 'prior bytes', source_read: { path: 'note.txt', success: true, complete: false, format: 'raw' } },
  { content: 'prior bytes', source_read: { path: 'other.txt', success: true, complete: true, format: 'raw' } },
]) {
  test('unknown, truncated or foreign source receipt cannot authorize replacement: ' + JSON.stringify(result), () => {
    const plan = planLiteralAdditionStep('append file note.txt containing hello', readMessages(result), ['read', 'write']);
    assert.equal(plan?.kind, 'final');
  });
}

for (const [target, payload] of [['other.md', 'β🙂'], ['nested/ledger.log', 'first\nsecond']]) {
  test('declared destination and payload vary independently: ' + target, async () => {
    const directory = mkdtempSync(join(tmpdir(), 'formal-append-variable-'));
    try {
      const task = 'append file ' + target + ' containing «' + payload + '»';
      const result = await drive(planChatStep, directory, task, { tools: ['read', 'write'], steps: 6 });
      assert.equal(readFileSync(join(directory, target), 'utf8'), payload + '\n');
      assert.deepEqual(result.transcript.map(call => call.tool), ['read', 'write', 'read']);
    } finally { rmSync(directory, { recursive: true, force: true }); }
  });
}

// Locale expectations are authored from request semantics, independently of recognition.
const localeAppends = [
  '追加 文件 note.txt 内容为 hello',
  'add at the end of file note.txt containing hello',
  'добавь в конец файл note.txt с текстом hello',
  'जोड़ो अंत में फ़ाइल note.txt सामग्री के साथ hello',
  '添加 末尾 文件 note.txt 内容为 hello',
  'añade al final de archivo note.txt con el texto hello',
];
for (const task of localeAppends) {
  test('locale append explicitly preserves prior bytes through Read/Write/Read: ' + task, async () => {
    await replay(task, 'prior bytes α\n', 'prior bytes α\nhello\n');
  });
}
const unspecifiedAdditions = [
  'add file note.txt containing hello',
  'добавь файл note.txt с текстом hello',
  'जोड़ो फ़ाइल note.txt सामग्री के साथ hello',
  '添加 文件 note.txt 内容为 hello',
  'añade archivo note.txt con el texto hello',
  'agrega archivo note.txt con el texto hello',
];
// These unchanged noun-before-path requests declare creation by file_declared_noun.
// Existing entries still refuse exclusive creation; destination-before-noun cases below
// retain the unspecified insertion Gap without granting replacement permission.
for (const task of unspecifiedAdditions) {
  test('declared locale creation writes exact bytes only into an absent destination: ' + task, async () => {
    const frame = ownedDeclaredCreateFrame(task);
    assert.equal(frame?.target, 'note.txt');
    assert.equal(frame.content, 'hello');
    assert.equal(planLiteralAdditionStep(task, [{ role: 'user', content: task }], ['read', 'write']), null);
    const directory = mkdtempSync(join(tmpdir(), 'formal-declared-create-'));
    try {
      const result = await drive(planChatStep, directory, task, { steps: 8 });
      assert.equal(readFileSync(join(directory, 'note.txt'), 'utf8'), 'hello');
      assert.equal(result.stop, 'final');
      const creation = result.transcript.find(call => call.tool === 'write');
      assert.equal(creation?.source_creation?.success, true);
      assert.equal(creation.source_creation.exclusive, true);
      assert.equal(result.transcript.every((call, index) => call.tool !== 'read' || index > result.transcript.indexOf(creation)), true);
      assert.equal(result.transcript.some(call => call.tool === 'bash' && JSON.parse(call.arguments).command === 'cat note.txt'), true);
      const prior = 'prior bytes α\n';
      writeFileSync(join(directory, 'note.txt'), prior);
      const existing = await drive(planChatStep, directory, task, { steps: 8 });
      assert.equal(readFileSync(join(directory, 'note.txt'), 'utf8'), prior);
      const rejected = existing.transcript.find(call => call.tool === 'write');
      assert.equal(rejected?.is_error, true);
      assert.equal(rejected?.source_creation?.error_code, 'EEXIST');
      assert.equal(existing.transcript.some(call => call.tool === 'read'), false);
      assert.doesNotMatch(existing.answer ?? '', /Completed the general change/);
    } finally { rmSync(directory, { recursive: true, force: true }); }
  });
}
for (const task of [
  'add to file note.txt containing hello',
  'add to the file note.txt containing hello',
]) {
  test('destination-before-noun insertion requires a position and preserves existing bytes: ' + task, async () => {
    assert.equal(ownedDeclaredCreateFrame(task), null);
    const directory = mkdtempSync(join(tmpdir(), 'formal-add-position-'));
    try {
      const prior = 'prior bytes α\n';
      writeFileSync(join(directory, 'note.txt'), prior);
      const result = await drive(planChatStep, directory, task, { tools: ['read', 'write', 'bash'], steps: 6 });
      assert.equal(result.stop, 'final');
      assert.deepEqual(result.transcript.map(call => call.tool), ['read']);
      assert.equal(readFileSync(join(directory, 'note.txt'), 'utf8'), prior);
      const history = [{ role: 'user', content: task }];
      const observed = result.transcript[0];
      history.push({ role: 'assistant', tool_calls: [{ id: 'position-read', type: 'function', function: { name: observed.tool, arguments: observed.arguments } }] });
      history.push({ role: 'tool', tool_call_id: 'position-read', name: observed.tool, content: observed.result, source_read: observed.source_read, is_error: observed.is_error });
      const refused = planLiteralAdditionStep(task, history, ['read', 'write', 'bash']);
      assert.equal(refused.result.disposition, 'gap');
      assert.equal(refused.result.origin, 'literal-addition-position-unknown');
      assert.equal(result.answer, refused.answer);
    } finally { rmSync(directory, { recursive: true, force: true }); }
  });
}
