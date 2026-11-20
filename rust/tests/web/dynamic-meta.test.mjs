import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

import { createRecipeDiscovery } from '../../../js/server/dynamic-meta/discovery.mjs';
import { sha256 } from '../../../js/server/dynamic-meta/packages.mjs';
import { compileRecipe, readRecipe, renderRecipe, runRecipe } from '../../../js/server/dynamic-meta/recipe.mjs';
import { createMetaRuntime } from '../../../js/server/dynamic-meta/runtime.mjs';
import { startCoreDreaming } from '../../../js/server/dreaming-runtime.mjs';
import { startServer } from '../../../js/server/main.mjs';
import { startCli } from './support/structured-log-fixture.mjs';

const admin = { admin: true };
const sampleManifest = readFileSync(new URL('../../../experiments/dynamic-meta/package.json', import.meta.url), 'utf8');
const sampleInstructions = readFileSync(new URL('../../../experiments/dynamic-meta/instructions.md', import.meta.url), 'utf8');
const manifest = (fields = {}) => JSON.stringify({ ...JSON.parse(sampleManifest), ...fields });

function useRuntime(t, options) {
  const runtime = createMetaRuntime(options);
  t.after(() => runtime.close());
  return runtime;
}

const install = (runtime, payload = sampleManifest, actor = admin) =>
  runtime.act({ action: 'install', manifest: payload, confirm: true }, actor);
const substitute = (runtime, query, options) => runtime.act({ action: 'substitute', query, confirm: true, ...options });
const inspect = (runtime) => runtime.act({ action: 'inspect' });
const command = (runtime, text, confirm = true, actor = admin) =>
  runtime.act({ action: 'command', command: text, confirm }, actor);

test('CRUD schedules distinct cycles, excludes reads, and normalizes two top-level groups', async (t) => {
  const runtime = useRuntime(t);
  await install(runtime, manifest({
    rules: [
      ...JSON.parse(sampleManifest).rules,
      { id: 'add-child', when: { event: 'update', source: 1, target: 1 }, substitute: '() ((child: @id 0))' },
      { id: 'observe-delete', when: { event: 'delete', source: 1, target: 0 }, substitute: '() ((audit: @old-target @old-source))' },
    ],
  }));
  const created = await runtime.act({ action: 'dataset', dataset: 'example/sample', confirm: true });
  assert.deepEqual(created.trace.map(({ cycle, events }) => [cycle, events.map(({ kind }) => kind)]), [
    [0, ['create']], [1, ['update']], [2, ['create']],
  ]);
  const deleted = await substitute(runtime, '((child: 1 0)) ()');
  assert.deepEqual(deleted.trace.map(({ events }) => events.map(({ kind }) => kind)), [['delete'], ['create']]);
  assert.deepEqual(deleted.trace[1].events[0].after, { id: 2, from_id: 0, to_id: 1 });
  const noChange = await substitute(runtime, '((input: input input) (audit: 0 input))\n((input: input input) (audit: 0 input))');
  assert.deepEqual(noChange.trace[0].events, []);
  const replay = await substitute(runtime, noChange.trace[0].query);
  assert.deepEqual(replay.trace, noChange.trace);
  const changed = await substitute(runtime, '((input: input input) (audit: 0 input))\n((input: 0 0) (audit: input 0))');
  assert.deepEqual(changed.trace[0].events.map(({ kind }) => kind), ['update', 'update']);
  const persisted = await runtime.act({ action: 'receipt', revision: created.revision });
  assert.deepEqual(persisted.trace, created.trace);
});

test('bounded cascades roll back the initiating mutation and receipts', async (t) => {
  const runtime = useRuntime(t);
  await substitute(runtime, '() ((state: 0 0))');
  await install(runtime, manifest({
    rules: [
      { id: 'a', when: { event: 'update', source: 1, target: 0 }, substitute: '((@id: 1 0)) ((@id: 1 1))' },
      { id: 'b', when: { event: 'update', source: 1, target: 1 }, substitute: '((@id: 1 1)) ((@id: 1 0))' },
    ],
  }));
  const before = await inspect(runtime);
  await assert.rejects(substitute(runtime, '((state: 0 0)) ((state: 1 0))'), /Trigger cycle limit exceeded/u);
  assert.deepEqual(await inspect(runtime), before);
  await assert.rejects(runtime.act({ action: 'receipt', revision: before.revision + 1 }), { status: 404 });
  await assert.rejects(substitute(runtime, '() ((invalid: 99 99))'), /99/u);
  assert.deepEqual(await inspect(runtime), before);
});

test('trigger queries use the same cycle graph and reject incompatible writes atomically', async (t) => {
  const runtime = useRuntime(t);
  await substitute(runtime, '() ((left: 0 0) (right: left 0))');
  const when = { event: 'create', source: 0, target: 1 };
  const updateLeft = { id: 'a', when, substitute: '((left: 0 0)) ((left: left left))' };
  await install(runtime, manifest({ rules: [
    updateLeft,
    { id: 'b', when, substitute: '((right: (left: 0 0) 0)) ((right: left right))' },
    { id: 'c', when, substitute: '() ((one: left left))' },
    { id: 'd', when, substitute: '() ((two: right right))' },
  ] }));
  const result = await substitute(runtime, '() ((trigger: 0 1))');
  assert.deepEqual(result.trace.slice(1).map(({ cycle, events }) => [cycle, events[0]?.after]), [
    [1, { id: 1, from_id: 1, to_id: 1 }], [1, { id: 2, from_id: 1, to_id: 2 }],
    [1, { id: 4, from_id: 1, to_id: 1 }], [1, { id: 5, from_id: 2, to_id: 2 }],
  ]);
  assert.deepEqual((await inspect(runtime)).graph.names.slice(-2), [[4, 'one'], [5, 'two']]);

  const conflict = useRuntime(t);
  await substitute(conflict, '() ((left: 0 0))');
  await install(conflict, manifest({ rules: [updateLeft,
    { id: 'b', when, substitute: '((left: 0 0)) ((left: 0 left))' },
  ] }));
  const before = await inspect(conflict);
  await assert.rejects(substitute(conflict, '() ((trigger: 0 1))'), /Conflicting link substitutions in the same cycle/u);
  assert.deepEqual(await inspect(conflict), before);
});

test('live rule changes default off and are gated by operator confirmation and authority', async (t) => {
  const runtime = useRuntime(t);
  const payload = manifest({
    rules: [
      { id: 'turn-off', when: { event: 'create', source: 0, target: 0 }, setRule: { target: 'worker', enabled: false } },
      { id: 'worker', when: { event: 'create', source: 0, target: 0 }, substitute: '((@id: 0 0)) ((@id: @id @id))' },
    ],
  });
  await assert.rejects(install(runtime, payload, {}), { status: 403 });
  await install(runtime, payload);
  await assert.rejects(command(runtime, 'Enable dynamic self-modification.', true, {}), { status: 403 });
  await assert.rejects(command(runtime, 'Enable dynamic self-modification.', false), { status: 409 });
  await assert.rejects(substitute(runtime, '() ((first: 0 0))', { confirm: false }), { status: 409 });
  assert.equal((await substitute(runtime, '() ((first: 0 0))')).trace[1].decision, 'self-modification-disabled');
  await command(runtime, 'Enable dynamic self-modification.');
  const read = await substitute(runtime, '() ((second: 0 0))');
  assert.equal(read.trace[1].decision, 'disabled');
  assert.equal(read.trace[2].rule, 'example/worker');
  assert.equal((await inspect(runtime)).rules['example/worker'], false);
  assert.equal((await substitute(runtime, '() ((third: 0 0))')).trace.length, 2);
});

test('packages use typed doublets for npm, crate and GitHub exports, and invoke only configured adapters', async (t) => {
  const runtime = useRuntime(t, { adapters: new Map([['echo', (input) => ({ observed: input })]]) });
  await assert.rejects(install(runtime, manifest({ origin: { kind: 'npm', locator: '@example/plugin', revision: 'latest' } })), /version or Git commit/u);
  for (const [kind, locator, revision] of [
    ['npm', '@example/plugin', '1.0.0'], ['crate', 'example_plugin', '2.0.0'], ['github', 'example/plugin', 'a3efcfb'],
  ]) {
    await install(runtime, manifest({
      id: kind, origin: { kind, locator, revision },
      integrations: [{ id: 'call', adapter: 'echo' }, { id: 'unavailable', adapter: 'missing' }],
    }));
  }
  const state = await inspect(runtime);
  for (const kind of ['npm', 'crate', 'github']) {
    assert.match(state.types, new RegExp(`\\(Package/${kind}: Package Package/${kind}\\)`, 'u'));
    assert.match(state.types, new RegExp(`\\(package/${kind}/call: integrations sha256/`, 'u'));
  }
  assert.match(state.types, /\(Package: Type Package\)/u);
  assert.equal(state.packages[0].origin.locator, '@example/plugin');
  await assert.rejects(runtime.act({ action: 'integration', integration: 'npm/call', confirm: true }), { status: 403 });
  await assert.rejects(runtime.act({ action: 'integration', integration: 'npm/unavailable', confirm: true }, admin), { status: 404 });
  assert.deepEqual(await runtime.act({ action: 'integration', integration: 'npm/call', input: 'data', confirm: true }, admin), {
    result: { observed: 'data' },
  });
});

test('analysis previews rule priority and certifies shrinking recipes without applying them', async (t) => {
  const runtime = useRuntime(t);
  const source = 'Replace "aa" with "a".\nReplace "baa" with "baaaa".\nReplace "x" with "yx" and stop.';
  const analysis = await runtime.act({ action: 'analyze', from: 'english', source });
  assert.equal(analysis.termination, 'strictly_decreasing');
  assert.deepEqual(analysis.rules.map(({ index, shadowedBy, lengthChange, effect }) => [index, shadowedBy, lengthChange, effect]), [
    [0, null, -1, 'update'], [1, 0, 2, 'update'], [2, null, 1, 'update'],
  ]);
  assert.equal((await runtime.act({ action: 'analyze', from: 'english', source: 'Replace "a" with "aa".' })).termination, 'unproven');
  assert.equal((await inspect(runtime)).revision, 0);
  await install(runtime, manifest({ recipes: [{ id: 'shrink', language: 'english', source }] }));
  assert.deepEqual(await runtime.act({ action: 'analyze', recipe: 'example/shrink' }), analysis);
  assert.equal((await inspect(runtime)).revision, 1);
});

test('all supported language directions preserve rules, generated programs run, and compiled execution agrees', () => {
  const instructions = 'Replace "é😀b" with "β".\nReplace "ab" with "X".\nReplace "b" with "Y" and stop.';
  const program = readRecipe('english', instructions);
  const languages = ['english', 'links', 'javascript', 'python'];
  for (const from of languages) {
    for (const to of languages) {
      const intermediate = readRecipe(from, renderRecipe(from, program));
      assert.deepEqual(readRecipe(to, renderRecipe(to, intermediate)), program);
    }
  }
  const compiled = compileRecipe(program);
  const input = 'bb ab é😀b ab';
  const interpreted = runRecipe(program, input);
  assert.deepEqual(interpreted, {
    output: 'Yb X β X',
    trace: [{ rule_index: 0, byte_offset: 6 }, { rule_index: 1, byte_offset: 3 },
      { rule_index: 1, byte_offset: 8 }, { rule_index: 2, byte_offset: 0 }],
    halt: { kind: 'terminal_rule', index: 2 },
  });
  assert.deepEqual(runRecipe(program, input, { compiled }), interpreted);
  const insert = readRecipe('english', 'Replace "x" with "z".\nReplace "" with "+" and stop.');
  for (const sample of ['', 'x', 'xxx', 'aax']) {
    assert.deepEqual(runRecipe(insert, sample, { compiled: compileRecipe(insert) }), runRecipe(insert, sample));
  }
  const priority = readRecipe('english', [
    'Replace "first" with "1" and stop.',
    'Replace "second" with "2" and stop.',
    'Replace "third" with "3" and stop.',
    'Replace "fourth" with "4" and stop.',
    'Replace "x[y]" with "5" and stop.',
    'Replace "β?" with "6" and stop.',
  ].join('\n'));
  for (const candidate of [priority, { rules: [...priority.rules, { pattern: '', replacement: '0', terminal: true }] }]) {
    const dispatchTable = compileRecipe(candidate);
    for (const sample of ['x[y] first', 'β? α😀x[y] x[y]', 'x[y] β?', 'xy β', '']) {
      assert.deepEqual(runRecipe(candidate, sample, { compiled: dispatchTable }), runRecipe(candidate, sample));
    }
  }
  const python = spawnSync('python3', ['-c', `${renderRecipe('python', program)}\nprint(rewrite(${JSON.stringify(input)}))`], { encoding: 'utf8' });
  assert.equal(python.status, 0, python.stderr);
  assert.equal(python.stdout.trimEnd(), interpreted.output);
  const javascript = spawnSync(process.execPath, ['--input-type=module', '-e',
    `${renderRecipe('javascript', program)}\nconsole.log(rewrite(${JSON.stringify(input)}))`], { encoding: 'utf8' });
  assert.equal(javascript.status, 0, javascript.stderr);
  assert.equal(javascript.stdout.trimEnd(), interpreted.output);
  const invalidLimit = spawnSync(process.execPath, ['--input-type=module', '-e',
    `${renderRecipe('javascript', program)}\nrewrite('sample', 0)`], { encoding: 'utf8' });
  assert.equal(invalidLimit.status, 1);
  assert.match(invalidLimit.stderr, /Error: Recipe input or step limit is out of bounds/u);
  assert.throws(() => readRecipe('javascript', `${renderRecipe('javascript', program)}\nthrow new Error('extra code')`), /differs from/u);
});

test('usage yields reviewable promotion and demotion, and an approved compile survives restart', async (t) => {
  const dir = mkdtempSync(join(tmpdir(), 'formal-ai-meta-'));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const path = join(dir, 'meta.sqlite');
  const first = createMetaRuntime({ path });
  try {
    await install(first);
    const call = { action: 'run', recipe: 'example/normalize', input: 'teh teh' };
    const dynamic = await first.act(call);
    assert.equal(dynamic.output, 'the the');
    for (let i = 1; i < 16; i += 1) await first.act(call);
    const reviewed = await new Promise((resolve, reject) => {
      const deadline = setTimeout(() => reject(new Error('Idle review did not run')), 1000);
      const dreaming = startCoreDreaming({
        env: {}, idleSeconds: 0, run: () => {}, afterRun: () => {
          clearTimeout(deadline);
          dreaming.stop();
          resolve(first.review());
        },
      });
      t.after(() => dreaming.stop());
    });
    assert.deepEqual(reviewed.proposals, [{ command: 'Compile recipe example/normalize', uses: 16, window: 16 }]);
    assert.deepEqual((await inspect(first)).compiled, []);
    await command(first, 'Compile recipe example/normalize');
    const observed = await first.act({ action: 'compare', left: call.recipe, right: 'example/normalize-once', input: call.input });
    assert.equal(observed.left.implementation, 'compiled');
    assert.deepEqual(observed.left.trace, dynamic.trace);
    assert.equal(observed.right.output, 'the teh');
    assert.equal(observed.complete, true);
    assert.equal(observed.observedAgreement, false);
  } finally {
    first.close();
  }

  const second = useRuntime(t, { path });
  const persisted = await inspect(second);
  assert.deepEqual(persisted.compiled, ['example/normalize']);
  await assert.rejects(substitute(second, '() ((a: 0 0))', { expectedRevision: persisted.revision - 1 }), { status: 409 });
  for (let i = 0; i < 64; i += 1) await second.act({ action: 'run', recipe: 'example/normalize-once', input: 'teh' });
  const suggestions = second.review().proposals;
  assert.ok(suggestions.some(({ command }) => command === 'Interpret recipe example/normalize'));
  await command(second, 'Interpret recipe example/normalize');
  const result = await second.act({ action: 'run', recipe: 'example/normalize', input: 'teh teh' });
  assert.equal(result.implementation, 'dynamic');
  assert.equal(result.output, 'the the');
});

test('discovers bounded online instructions, retaining source identity and rejecting redirects and digest changes', async () => {
  const requests = [];
  const instructions = '```javascript\nconst arbitrary = true;\n```\n' + sampleInstructions +
    '\n```\n(matching pattern)\n(substitution pattern)\n```\n' +
    '\n```\n() ((a: 0 0) (b: 0 0))\n```\n';
  const discover = createRecipeDiscovery({
    env: {}, origins: ['https://docs.example.test/'],
    fetchImpl: (url, options) => {
      requests.push([url, options.redirect]);
      if (url.endsWith('redirect')) return new Response('', { status: 302, headers: { location: 'https://other.example.test/' } });
      return new Response(url.endsWith('.json') ? sampleManifest : instructions, { status: 200 });
    },
  });
  const url = 'https://docs.example.test/recipes.md';
  const result = await discover([
    { url: 'https://other.example.test/', language: 'markdown' },
    { url: 'https://docs.example.test/redirect', language: 'markdown' },
    { url, language: 'markdown', sha256: 'stale' },
    { url, language: 'markdown', sha256: sha256(instructions) },
    { url: 'https://docs.example.test/manifest.json', language: 'plugin' },
  ]);
  assert.deepEqual(result.slice(0, 3).map(({ error }) => error), [
    'Recipe source is outside the server allowlist', 'Source returned HTTP 302', 'Source digest mismatch',
  ]);
  assert.equal(result[3].candidates.length, 3);
  assert.deepEqual(result[3].rejected, [{ language: 'javascript', line: 1, reason: 'Unsupported recipe language or program' }]);
  const recipe = result[3].candidates[0];
  const dataset = result[3].candidates[2];
  assert.deepEqual(recipe.provenance, { url, sha256: sha256(instructions), line: 8 });
  assert.equal(runRecipe(readRecipe('links', recipe.links), 'teh teh').output, 'the the');
  assert.deepEqual(requests, [
    ['https://docs.example.test/redirect', 'manual'], [url, 'manual'], [url, 'manual'],
    ['https://docs.example.test/manifest.json', 'manual'],
  ]);
  const runtime = createMetaRuntime();
  try {
    const { manifest: payload, sha256: digest } = result[4];
    await assert.rejects(runtime.act({ action: 'install', manifest: payload + ' ', sha256: digest, confirm: true }, admin), { status: 409 });
    assert.equal((await runtime.act({ action: 'install', manifest: payload, sha256: digest, confirm: true }, admin)).package, 'example');
    const imported = manifest({
      id: 'imported', rules: [],
      recipes: [{ id: 'normalize', language: 'links', source: recipe.links, provenance: recipe.provenance }],
      datasets: [{ id: 'doublets', substitute: dataset.substitute, provenance: dataset.provenance }],
    });
    await install(runtime, imported);
    const applied = await runtime.act({ action: 'dataset', dataset: 'imported/doublets', confirm: true });
    assert.equal(applied.trace[0].events.length, 2);
    const { packages } = await inspect(runtime);
    assert.deepEqual(packages[1].recipes[0].provenance, recipe.provenance);
    assert.deepEqual(packages[1].datasets[0].provenance, dataset.provenance);
  } finally {
    runtime.close();
  }
});

test('HTTP credentials and chat route run the installed recipe against the same persistent runtime', async (t) => {
  const dir = mkdtempSync(join(tmpdir(), 'formal-ai-meta-http-'));
  const { server, url } = await startServer({
    env: {
      FORMAL_AI_MEMORY_PATH: join(dir, 'memory.lino'),
      FORMAL_AI_DYNAMIC_META_STORE: join(dir, 'meta.sqlite'),
      FORMAL_AI_DYNAMIC_META_ADMIN_TOKEN: 'fixture-admin-token',
      FORMAL_AI_API_BEARER_TOKEN: 'fixture-app-token',
      FORMAL_AI_LOG_LEVEL: 'silent',
    },
  });
  t.after(async () => {
    await new Promise((resolve) => server.close(resolve));
    rmSync(dir, { recursive: true, force: true });
  });
  const bearer = { authorization: 'Bearer fixture-app-token' };
  const operator = { 'x-formal-ai-meta-admin': 'fixture-admin-token' };
  let endpoint = url;
  if (process.env.FORMAL_AI_DYNAMIC_META_NATIVE) {
    const native = await startCli(t, {
      binary: process.env.FORMAL_AI_DYNAMIC_META_NATIVE,
      env: {
        FORMAL_AI_API_BEARER_TOKEN: 'fixture-app-token',
        FORMAL_AI_DYNAMIC_META_LOOPBACK: new URL(url).host,
      },
    });
    await native.ready();
    endpoint = native.url;
  }
  const post = (path, payload, headers) => fetch(endpoint + path, {
    method: 'POST', headers: { 'content-type': 'application/json', ...headers },
    body: JSON.stringify(payload),
  });
  const payload = { action: 'install', manifest: sampleManifest, confirm: true, admin: true };
  assert.equal((await fetch(endpoint + '/v1/dynamic-meta', { method: 'OPTIONS' })).status, 204);
  assert.equal((await post('/v1/dynamic-meta', payload)).status, 401);
  assert.equal((await post('/v1/dynamic-meta', payload, bearer)).status, 403);
  assert.equal((await post('/v1/dynamic-meta', payload, operator)).status, 200);
  const analysis = await post('/v1/dynamic-meta', { action: 'analyze', recipe: 'example/normalize' }, bearer);
  assert.equal(analysis.status, 200);
  assert.equal((await analysis.json()).termination, 'unproven');
  const response = await post('/v1/chat/completions', {
    model: 'formal-ai', messages: [{ role: 'user', content: 'Use recipe example/normalize to rewrite "teh teh".' }],
  }, bearer);
  assert.equal(response.status, 200);
  const completion = await response.json();
  const result = JSON.parse(completion.choices[0].message.content);
  assert.deepEqual(result, {
    revision: 2, implementation: 'dynamic', output: 'the the',
    trace: [{ rule_index: 0, byte_offset: 0 }, { rule_index: 0, byte_offset: 4 }],
    halt: { kind: 'no_applicable_rule' },
  });
  const receiptResponse = await post('/v1/dynamic-meta', { action: 'receipt', revision: 2 }, bearer);
  assert.deepEqual(await receiptResponse.json(), { action: 'run', ...result });
});
