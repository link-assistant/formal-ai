import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { test } from 'node:test';
import { promisify } from 'node:util';

import { startServer } from '../../js/server/main.mjs';
import { startCli } from '../../rust/tests/web/support/structured-log-fixture.mjs';

const run = promisify(execFile);

test('the real Agent CLI receives the trace of a compiled plugin recipe', async (t) => {
  const directory = await mkdtemp(join(tmpdir(), 'formal-ai-meta-agent-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const appToken = 'dynamic-meta-agent-test';
  const adminToken = 'dynamic-meta-operator-test';
  const bearer = { authorization: `Bearer ${appToken}` };
  const { server, url } = await startServer({
    agentMode: true,
    env: {
      ...process.env,
      FORMAL_AI_API_BEARER_TOKEN: appToken,
      FORMAL_AI_DYNAMIC_META_ADMIN_TOKEN: adminToken,
      FORMAL_AI_DYNAMIC_META_STORE: join(directory, 'meta.sqlite'),
      FORMAL_AI_LOG_LEVEL: 'silent',
      FORMAL_AI_MEMORY_PATH: join(directory, 'memory.lino'),
      FORMAL_AI_SOURCE_USAGE_RUST_GRAMMAR: 'false',
    },
  });
  try {
    let endpoint = url;
    if (process.env.FORMAL_AI_DYNAMIC_META_NATIVE) {
      const native = await startCli(t, {
        binary: resolve(process.env.FORMAL_AI_DYNAMIC_META_NATIVE),
        lifetime: 90_000,
        env: {
          FORMAL_AI_API_BEARER_TOKEN: appToken,
          FORMAL_AI_DYNAMIC_META_LOOPBACK: new URL(url).host,
        },
      });
      await native.ready(bearer);
      endpoint = native.url;
    }
    const send = async (body, credentials) => {
      const response = await fetch(`${endpoint}/v1/dynamic-meta`, {
        method: 'POST',
        headers: { 'content-type': 'application/json', ...credentials },
        body: JSON.stringify(body),
      });
      const result = await response.json();
      assert.equal(response.status, 200, JSON.stringify(result));
      return result;
    };
    const admin = { 'x-formal-ai-meta-admin': adminToken };
    const manifest = await readFile(new URL('./package.json', import.meta.url), 'utf8');
    await send({ action: 'install', manifest, confirm: true }, admin);
    const dataset = await send({ action: 'dataset', dataset: 'example/sample', confirm: true }, bearer);
    assert.deepEqual(dataset.trace.map(({ events }) => events.map(({ kind }) => kind)), [['create'], ['update']]);
    const command = await send({ action: 'command', command: 'Compile recipe example/normalize', confirm: true }, admin);

    const config = {
      provider: {
        formalai: {
          name: 'Formal AI',
          npm: '@ai-sdk/openai-compatible',
          options: { baseURL: `${endpoint}/v1`, apiKey: appToken },
          models: { 'formal-ai': { name: 'Formal AI' } },
        },
      },
      model: 'formalai/formal-ai',
    };
    const { stdout } = await run(process.env.AGENT || 'agent', [
      '--model', config.model, '--permission-mode', 'auto', '--output-format', 'stream-json', '--compact-json',
      '--disable-stdin', '--no-summarize-session', '--prompt', 'Run recipe example/normalize on "teh teh".',
    ], {
      cwd: directory,
      env: { ...process.env, LINK_ASSISTANT_AGENT_CONFIG_CONTENT: JSON.stringify(config) },
      timeout: 60_000,
      maxBuffer: 1024 * 1024,
    });
    const rows = stdout.split('\n').filter((line) => line.startsWith('{')).map((line) => JSON.parse(line));
    const assistant = rows.find((row) => row.type === 'message' && row.role === 'assistant');
    assert.ok(assistant?.session_id, 'Agent CLI did not record a response');
    const result = JSON.parse(assistant.content[0].text);
    assert.deepEqual(result, {
      revision: command.revision + 1,
      implementation: 'compiled',
      output: 'the the',
      trace: [{ rule_index: 0, byte_offset: 0 }, { rule_index: 0, byte_offset: 4 }],
      halt: { kind: 'no_applicable_rule' },
    });
    const receipt = await send({ action: 'receipt', revision: result.revision }, bearer);
    assert.deepEqual(receipt, { action: 'run', ...result });
  } finally {
    await new Promise((resolve) => server.close(resolve));
  }
});
