import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';

import { exportLinksNotation, memoryEvent } from '../../../js/server/memory-store.mjs';
import { startCli } from './support/structured-log-fixture.mjs';

const requestMarker = 'parcelretentionmarker1592';
const responseMarker = 'violet parcel tracking code 1592';
const dialogId = 'retention-case';
const chatRoute = '/v1/chat/completions';
const seedMemory = exportLinksNotation([memoryEvent({
  id: 'parcel',
  kind: 'message',
  role: 'user',
  content: `The warehouse supervisor supplied the ${responseMarker}.`,
})], 1);

async function post(run, route, body) {
  return fetch(run.url + route, {
    method: 'POST',
    headers: { 'content-type': 'application/json', 'x-formal-ai-dialog-id': dialogId },
    body: JSON.stringify(body),
    signal: AbortSignal.timeout(6500),
  });
}

async function chat(run, content) {
  const response = await post(run, chatRoute, { model: 'formal-ai', messages: [{ role: 'user', content }] });
  assert.equal(response.status, 200);
  return (await response.json()).choices[0].message.content;
}

async function health(run) {
  const response = await fetch(run.url + '/health', { signal: AbortSignal.timeout(2000) });
  assert.equal(response.status, 200);
  return response.json();
}

test('default diagnostics record request metadata and leave full dialogs opt-in', { timeout: 15000 }, async t => {
  const run = await startCli(t, { env: { FORMAL_AI_RECORD_CHAT: '1' } });
  await run.ready();
  assert.ok(await chat(run, `Did I mention ${requestMarker}?`));
  const status = await health(run);
  assert.equal(status.memory.mode, 'read_write');
  assert.equal(status.logs.status, 'writable');
  assert.equal(status.logs.dialog_capture, false);
  await run.stop();

  assert.equal(fs.existsSync(path.join(run.home, 'dialog-logs')), false);
  const diagnostics = fs.readFileSync(path.join(run.home, '.formal-ai', 'logs', `serve-${run.pid}.lino`), 'utf8');
  assert.equal(diagnostics, run.stderr);
  assert.ok(!diagnostics.includes(requestMarker));
  assert.ok(!run.stdout.includes(requestMarker));
  const records = run.records();
  const received = records.find(record => record.kind === 'request.received' && record.metadata.path === chatRoute);
  assert.ok(received.metadata.request_bytes > 0);
  assert.equal(received.payload?.content, undefined);
  const terminal = records.find(record => record.kind === 'request.terminal' && record.metadata.request_id === received.metadata.request_id);
  assert.equal(terminal.metadata.status, 200);
  assert.ok(terminal.metadata.response_bytes > 0);
  assert.ok(Number.isFinite(terminal.metadata.elapsed_ms));
});

test('serve help describes the retention defaults and opt-ins', () => {
  const entry = fileURLToPath(new URL('../../../js/server/main.mjs', import.meta.url));
  const help = execFileSync(process.execPath, [entry, 'serve', '--help'], { encoding: 'utf8' });
  assert.match(help, /metadata only/u);
  assert.match(help, /FORMAL_AI_DIALOG_LOG_DIR.*off by default/u);
  assert.match(help, /--ephemeral.*FORMAL_AI_EPHEMERAL=1/u);
});

test('an explicit dialog directory records an exchange and its conversation', { timeout: 15000 }, async t => {
  const run = await startCli(t, { env: { FORMAL_AI_DIALOG_LOG_DIR: 'explicit-dialogs' } });
  await run.ready();
  assert.ok(await chat(run, `Did I mention ${requestMarker}?`));
  assert.equal((await health(run)).logs.dialog_capture, true);
  await run.stop();

  const directory = path.join(run.home, 'explicit-dialogs');
  for (const suffix of ['.lino', '.conversation.lino']) {
    assert.ok(fs.readFileSync(path.join(directory, dialogId + suffix), 'utf8').includes(requestMarker));
  }
});

for (const mode of ['--ephemeral', 'FORMAL_AI_EPHEMERAL=1']) {
  test(`${mode} answers from memory without writing diagnostics, memory, or derivations`, { timeout: 15000 }, async t => {
    const run = await startCli(t, {
      args: ['--trace', ...(mode === '--ephemeral' ? ['--ephemeral'] : [])],
      env: {
        ...(mode === 'FORMAL_AI_EPHEMERAL=1' ? { FORMAL_AI_EPHEMERAL: '1' } : {}),
        FORMAL_AI_DREAMING: '1',
        FORMAL_AI_RECORD_CHAT: '1',
        FORMAL_AI_DIALOG_LOG_DIR: 'explicit-dialogs',
        FORMAL_AI_LOG_DIR: 'explicit-logs',
        FORMAL_AI_NATIVE_RESPONSE_CAPTURE_DIR: 'capture',
        FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR: 'false',
      },
    });
    await run.ready();
    const status = await health(run);
    assert.equal(status.memory.mode, 'read_only');
    assert.equal(status.logs.status, 'disabled');
    assert.equal(status.logs.dialog_capture, false);
    assert.equal(status.logs.derivation_capture, false);
    assert.ok(!fs.existsSync(run.env.FORMAL_AI_MEMORY_PATH));

    fs.writeFileSync(run.env.FORMAL_AI_MEMORY_PATH, seedMemory);
    assert.match(await chat(run, 'Did I mention violet parcel tracking code?'), new RegExp(responseMarker));
    assert.match(await chat(run, `Did I mention ${requestMarker}?`), new RegExp(requestMarker));
    const answer = await post(run, '/v1/responses', { model: 'formal-ai', input: '2 + 2' });
    assert.equal(answer.status, 200);
    const result = await answer.json();
    assert.match(result.output.find(item => item.type === 'message').content[0].text, /4/u);
    assert.ok(result.evidence_links.some(link => link.startsWith('derivation:')));
    assert.ok(!result.evidence_links.some(link => link.startsWith('derivation_record:') || link.endsWith('.lino')));
    const imported = await post(run, '/v1/memory/import', { content: requestMarker });
    assert.equal(imported.status, 409);
    await imported.text();
    await run.stop();

    assert.equal(run.stdout, '');
    assert.equal(run.stderr, '');
    assert.equal(fs.readFileSync(run.env.FORMAL_AI_MEMORY_PATH, 'utf8'), seedMemory);
    assert.deepEqual(fs.readdirSync(run.home), ['memory.lino']);
    for (const key of ['XDG_DATA_HOME', 'XDG_CACHE_HOME', 'XDG_STATE_HOME', 'XDG_CONFIG_HOME', 'TMPDIR']) {
      assert.deepEqual(fs.readdirSync(run.env[key]), [], key);
    }
  });
}
