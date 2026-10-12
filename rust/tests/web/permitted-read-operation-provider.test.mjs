import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, readFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { installDefaultNodeSourceHost } from '../../../js/server/default-node-source-bootstrap.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
await installDefaultNodeSourceHost(new WorkerHost());
const request = 'Do not write. Read input.txt.';
async function fixture(fn) {
  const dir = mkdtempSync(join(tmpdir(), 'permitted-read-provider-'));
  try {
    writeFileSync(join(dir, 'input.txt'), 'owned α😀 input\n');
    return await fn(dir);
  } finally {
    rmSync(dir, {
      recursive: true,
      force: true
    });
  }
}
test('write prohibition preserves the owned physical Read only', () => fixture(async dir => {
  const r = await drive(planChatStep, dir, request);
  assert.equal(r.stop, 'final');
  assert.equal(r.transcript.length, 1);
  assert.equal(r.transcript[0].tool, 'read');
  assert.equal(r.transcript[0].source_read.complete, true);
  assert.equal(r.transcript[0].result, 'owned α😀 input\n');
  assert.equal(readFileSync(join(dir, 'input.txt'), 'utf8'), 'owned α😀 input\n');
}));
for (const tool of ['write', 'edit', 'bash']) test('caller ' + tool + ' cannot replace the live owned Read', () => fixture(async dir => {
  const args = tool === 'write' ? {
    path: 'input.txt',
    content: 'forged'
  } : tool === 'edit' ? {
    path: 'input.txt',
    old_string: 'owned',
    new_string: 'forged'
  } : {
    command: 'printf forged > input.txt'
  };
  const r = await drive(() => ({
    kind: 'tool_calls',
    calls: [{
      tool,
      arguments: JSON.stringify(args)
    }]
  }), dir, request);
  assert.equal(r.stop, 'unbound-read-operation');
  assert.equal(r.transcript.length, 0);
  assert.equal(readFileSync(join(dir, 'input.txt'), 'utf8'), 'owned α😀 input\n');
}));
test('caller foreign Read arguments cannot replace exact current source arguments', () => fixture(async dir => {
  const r = await drive(() => ({
    kind: 'tool_calls',
    calls: [{
      tool: 'read',
      arguments: JSON.stringify({
        path: 'input.txt'
      })
    }]
  }), dir, request);
  assert.equal(r.stop, 'unbound-read-operation');
  assert.equal(r.transcript.length, 0);
}));
test('caller cannot add a write after the actual Read', () => fixture(async dir => {
  let turn = 0;
  const r = await drive(async (messages, tools) => turn++ ? {
    kind: 'tool_calls',
    calls: [{
      tool: 'write',
      arguments: JSON.stringify({
        path: 'later.txt',
        content: 'forged'
      })
    }]
  } : await planChatStep(messages, tools), dir, request);
  assert.equal(r.stop, 'unbound-read-operation');
  assert.equal(r.transcript.length, 1);
  assert.equal(r.transcript[0].tool, 'read');
  assert(!existsSync(join(dir, 'later.txt')));
}));
for (const request of ['Do not write. Create file blocked.txt containing hello', 'Create file blocked.txt containing «hello» Publish it.']) test('unowned creation remains effect-free: ' + request, () => fixture(async dir => {
  const r = await drive(() => ({
    kind: 'tool_calls',
    calls: [{
      tool: 'write',
      arguments: JSON.stringify({
        path: 'blocked.txt',
        content: 'hello'
      })
    }]
  }), dir, request);
  assert.equal(r.transcript.length, 0);
  assert(!existsSync(join(dir, 'blocked.txt')));
}));
for (const request of ['Create.', 'Create something.']) test('semantic creation cannot gain caller invented bytes: ' + request, () => fixture(async dir => {
  const r = await drive(() => ({
    kind: 'tool_calls',
    calls: [{
      tool: 'write',
      arguments: JSON.stringify({
        path: 'unowned.txt',
        content: 'invented'
      })
    }]
  }), dir, request);
  assert.equal(r.transcript.length, 0);
  assert(!existsSync(join(dir, 'unowned.txt')));
}));
test('exact write adapter retains explicit effect without private creation receipt', () => fixture(async dir => {
  const r = await drive(() => ({
    kind: 'tool_calls',
    calls: [{
      tool: 'write',
      arguments: JSON.stringify({
        path: 'adapter.txt',
        content: 'adapter'
      })
    }]
  }), dir, 'write', {
    tools: ['write'],
    steps: 1
  });
  assert.equal(readFileSync(join(dir, 'adapter.txt'), 'utf8'), 'adapter');
  assert.equal(r.transcript.length, 1);
  assert.equal(r.transcript[0].source_creation, undefined);
}));
test('mixed adapter batch is refused before all effects', () => fixture(async dir => {
  const r = await drive(() => ({
    kind: 'tool_calls',
    calls: [{
      tool: 'write',
      arguments: JSON.stringify({
        path: 'adapter.txt',
        content: 'adapter'
      })
    }, {
      tool: 'read',
      arguments: JSON.stringify({
        path: 'input.txt'
      })
    }]
  }), dir, 'write', {
    tools: ['write', 'read'],
    steps: 1
  });
  assert.equal(r.stop, 'unbound-adapter-operation');
  assert.equal(r.transcript.length, 0);
  assert(!existsSync(join(dir, 'adapter.txt')));
}));
test('callback user mutation cannot replace the bound original Read', () => fixture(async dir => {
  writeFileSync(join(dir, 'secret.txt'), 'foreign');
  const r = await drive(async (messages, tools) => {
    messages.find(m => m.role === 'user').content = 'Read secret.txt.';
    return planChatStep(messages, tools);
  }, dir, request);
  assert.equal(r.stop, 'unbound-read-operation');
  assert.equal(r.transcript.length, 0);
}));
test('callback nested physical metadata never changes the actual Read receipt', () => fixture(async dir => {
  let turn = 0;
  const r = await drive(async (messages, tools) => {
    if (turn++) {
      const prior = messages.find(m => m.role === 'tool');
      prior.source_read.path = 'secret.txt';
    }
    return planChatStep(messages, tools);
  }, dir, request);
  assert.equal(r.stop, 'final');
  assert.equal(r.transcript.length, 1);
  assert.equal(r.transcript[0].source_read.path, 'input.txt');
  assert.equal(r.transcript[0].result, 'owned α😀 input\n');
}));
test('piped CLI drains a large owned Read and the final answer', () => fixture(async dir => {
  const {
    spawnSync
  } = await import('node:child_process');
  const {
    fileURLToPath
  } = await import('node:url');
  const text = 'owned α😀 line\n'.repeat(9000);
  writeFileSync(join(dir, 'input.txt'), text);
  const args = [...process.execArgv.filter(arg => arg !== '--test'), fileURLToPath(new URL('../../../experiments/js_dogfood/drive.mjs', import.meta.url)), '--dir', dir, '--steps', '4', 'Read input.txt.'];
  const r = spawnSync(process.execPath, args, {
    encoding: 'utf8',
    timeout: 50000,
    maxBuffer: 8 * 1024 * 1024
  });
  assert.equal(r.status, 0, r.stderr);
  assert(r.stdout.includes(text));
  assert(r.stdout.includes('== answer =='));
  assert.equal(readFileSync(join(dir, 'input.txt'), 'utf8'), text);
}));
for (const mode of ['user', 'receipt']) test('fallthrough callback cannot change private ' + mode, () => fixture(async dir => {
  writeFileSync(join(dir, 'secret.txt'), 'foreign');
  let turn = 0;
  const fallthrough = async (messages, tools) => {
    if (mode === 'user') messages.find(m => m.role === 'user').content = 'Read secret.txt.';else if (turn++) messages.find(m => m.role === 'tool').source_read.path = 'secret.txt';
    return planChatStep(messages, tools);
  };
  const r = await drive(async () => null, dir, request, {
    fallthrough,
    steps: 3
  });
  if (mode === 'user') {
    assert.equal(r.stop, 'unbound-read-operation');
    assert.equal(r.transcript.length, 0);
  } else {
    assert.equal(r.stop, 'final');
    assert.equal(r.transcript.length, 1);
    assert.equal(r.transcript[0].source_read.path, 'input.txt');
    assert.equal(r.transcript[0].result, 'owned α😀 input\n');
  }
  assert(!JSON.stringify(r).includes('foreign'));
}));
