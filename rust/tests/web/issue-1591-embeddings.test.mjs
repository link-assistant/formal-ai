import assert from 'node:assert/strict';
import { test } from 'node:test';
import { startCli } from './support/structured-log-fixture.mjs';

const model = 'formal-ai-embeddings';
const dimensions = 4096;
const inputs = ['warehouse supervisor', 'forklift operator'];
const token = 'embeddings-test-token';
const headers = { authorization: `Bearer ${token}`, 'content-type': 'application/json' };
const launch = {
  binary: process.env.FORMAL_AI_TEST_NATIVE_CLI,
  args: ['--ephemeral', '--silent'],
  env: { FORMAL_AI_API_BEARER_TOKEN: token },
};

async function embed(run, body, { path = '/v1/embeddings', auth = headers, status = 200 } = {}) {
  const response = await fetch(run.url + path, {
    method: 'POST', headers: auth, body: JSON.stringify(body), signal: AbortSignal.timeout(3000),
  });
  assert.equal(response.status, status);
  assert.equal(response.headers.get('content-type'), 'application/json');
  return response.json();
}

function cosine(left, right) {
  const dot = left.reduce((sum, weight, index) => sum + weight * right[index], 0);
  return dot / (Math.hypot(...left) * Math.hypot(...right));
}

test('the OpenAI embeddings API advertises and returns fixed, reproducible concept vectors', { timeout: 20_000 }, async t => {
  const first = await startCli(t, launch);
  await first.ready(headers);
  const body = { model: 'formal-ai', input: inputs };
  const result = await embed(first, body);
  assert.deepEqual(Object.keys(result).sort(), ['data', 'model', 'object', 'usage']);
  assert.equal(result.model, model);
  assert.equal(result.object, 'list');
  assert.deepEqual(result.usage, { prompt_tokens: 37, total_tokens: 37 });
  assert.deepEqual(result.data.map(({ object, index }) => [object, index]), [['embedding', 0], ['embedding', 1]]);
  const [warehouse, forklift] = result.data.map(({ embedding }) => embedding);
  for (const vector of [warehouse, forklift]) {
    assert.equal(vector.length, dimensions);
    assert.ok(vector.every(value => value === 0 || value === 1));
    assert.equal(vector.filter(Boolean).length, 1);
  }
  assert.equal(cosine(warehouse, forklift), 0);

  for (const path of ['/v1/models', '/api/openai/v1/models']) {
    const response = await fetch(first.url + path, { headers, signal: AbortSignal.timeout(3000) });
    assert.equal(response.status, 200);
    const listing = await response.json();
    assert.equal(listing.data[0].id, 'formal-ai');
    assert.deepEqual(listing.data.find(item => item.id === model), {
      id: model, slug: model, capabilities: ['embeddings'], dimensions, object: 'model', owned_by: 'link-assistant',
    });
  }

  const base64 = await embed(first, {
    model, input: inputs[0], dimensions, encoding_format: 'base64',
  }, { path: '/api/openai/v1/embeddings' });
  const bytes = Buffer.from(base64.data[0].embedding, 'base64');
  assert.equal(bytes.length, dimensions * 4);
  assert.deepEqual(warehouse.map((_, index) => bytes.readFloatLE(index * 4)), warehouse);

  assert.deepEqual(await embed(first, body), result);
  await first.stop();
  const restarted = await startCli(t, launch);
  await restarted.ready(headers);
  assert.deepEqual(await embed(restarted, body), result);
});

test('English synonyms and Russian translations share concept coordinates', { timeout: 12_000 }, async t => {
  const run = await startCli(t, launch);
  await run.ready(headers);
  const request = {
    model,
    input: ['storekeeper', 'stockroom attendant', 'КЛАДОВЩИК', 'भंडारपाल', '仓库保管员', 'almacenero', 'forklift operator',
      'tomato', 'томат', 'помидор', 'картофель', 'foldqlxvunknown'],
  };
  const result = await embed(run, request);
  const [storekeeper, synonym, russian, hindi, chinese, spanish, forklift, tomato, tomat, pomidor, potato, unknown] =
    result.data.map(({ embedding }) => embedding);
  assert.equal(cosine(storekeeper, synonym), 1);
  assert.equal(cosine(storekeeper, russian), 1);
  for (const translation of [hindi, chinese, spanish]) {
    assert.deepEqual(translation, storekeeper);
  }
  assert.ok(cosine(storekeeper, russian) > cosine(storekeeper, forklift));
  assert.equal(cosine(tomat, pomidor), 1);
  assert.equal(cosine(tomato, pomidor), 1);
  assert.ok(cosine(pomidor, tomat) > cosine(pomidor, potato));
  assert.ok(unknown.every(value => value === 0));

  if (launch.binary) {
    await run.stop();
    const javascript = await startCli(t, { ...launch, binary: undefined });
    await javascript.ready(headers);
    assert.deepEqual(await embed(javascript, request), result);
  }
});

test('invalid embeddings requests return an error and both aliases require authentication', { timeout: 12_000 }, async t => {
  const run = await startCli(t, launch);
  await run.ready(headers);
  for (const path of ['/v1/embeddings', '/api/openai/v1/embeddings']) {
    await embed(run, { model, input: inputs }, { path, auth: {}, status: 401 });
  }
  for (const [request, field] of [
    [null, /JSON object/],
    [{ input: inputs }, /model/],
    [{ model: 'unknown-embedding-model', input: inputs }, /unsupported model/],
    [{ model, input: [314, 159] }, /input/],
    [{ model, input: [] }, /input/],
    [{ model, input: ['tomato', '  '] }, /input/],
    [{ model, input: 'a'.repeat(8193) }, /input/],
    [{ model, input: inputs, dimensions: 128 }, /dimensions/],
    [{ model, input: inputs, encoding_format: 'hex' }, /encoding_format/],
  ]) {
    const { error } = await embed(run, request, { status: 400 });
    assert.equal(error.type, 'formal_ai_error');
    assert.match(error.message, field);
  }
});
