import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
const denied = () => {
  throw Error('LocalProductWasmForbidden');
};
await installNodeHost(new WorkerHost({
  WebAssembly: {
    ...WebAssembly,
    compile: denied,
    instantiate: denied,
    Module: denied,
    Instance: denied
  }
}));
const fixture = JSON.parse(fs.readFileSync(new URL('../fixtures/repository-documentation-original.json', import.meta.url), 'utf8'));
const hash = text => createHash('sha256').update(text).digest('hex');
const task = fixture.issue.title + '\n\n' + fixture.issue.body;
function workspace(context, files = fixture.files) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), 'formal-ai-repository-docs-')));
  context.after(() => fs.rmSync(root, {
    recursive: true,
    force: true
  }));
  for (const file of files) {
    const target = path.join(root, file.path);
    fs.mkdirSync(path.dirname(target), {
      recursive: true
    });
    fs.writeFileSync(target, file.source);
  }
  return root;
}
const run = (root, prompt) => drive(planChatStep, root, prompt, {
  steps: 24,
  tools: ['read', 'edit'],
  allowedCommands: []
});
test('original pre-fix work item produces useful partial documentation and a verified no-write repeat', async context => {
  for (const file of fixture.files) assert.equal(hash(file.source), file.sha256);
  const root = workspace(context),
    result = await run(root, task),
    state = JSON.parse(result.answer);
  assert.equal(result.stop, 'final');
  assert.equal(state.reason, 'UsefulCandidateReadbackVerified');
  assert.equal(state.schema, 'repository-work-item-controller/v1');
  assert.equal(state.complete, false);
  assert.equal(state.verified, false);
  assert.equal(state.authored, false);
  assert.ok(state.remainingWork.includes('consumer-discovery'));
  assert.ok(state.remainingWork.includes('independent-integration-semantics'));
  const original = fixture.files.find(file => file.path.endsWith('.md')),
    guide = fs.readFileSync(path.join(root, original.path), 'utf8');
  assert.ok(guide.startsWith(original.source));
  assert.notEqual(hash(guide), original.sha256);
  const declarations = state.facts.filter(row => row.kind === 'manifest-declaration');
  for (const [name, version] of [['command-stream', '1.5'], ['web-capture', '0.3.37'], ['web-search', '0.6.0'], ['@link-assistant/web-search', '0.11.1']]) {
    assert.ok(declarations.some(row => row.name === name && row.version === version), name);
    assert.ok(guide.includes('`' + name + '`'));
  }
  const usage = state.facts.find(row => row.kind === 'observed-source-usage');
  assert.ok(usage);
  for (const name of ['getDefaultProviderIds', 'mergeResults']) assert.ok(usage.calls.some(call => call.imported === name), name);
  for (const file of fixture.files.filter(file => file.path !== original.path)) assert.equal(hash(fs.readFileSync(path.join(root, file.path))), file.sha256);
  assert.equal(result.transcript.filter(row => row.tool === 'edit').length, 1);
  const again = await run(root, task);
  assert.equal(JSON.parse(again.answer).reason, 'VerifiedNoOp');
  assert.ok(again.transcript.every(row => row.tool === 'read'));
  assert.equal(fs.readFileSync(path.join(root, original.path), 'utf8'), guide);
});
test('renamed paths and an unrelated component use observed declarations and aliased calls', async context => {
  const files = [{
    path: 'native/Project.toml',
    source: '[dependencies]\nribbon-core = {version="2.7",default-features=false,features=["parse"]}\n'
  }, {
    path: 'web/client.mjs',
    source: 'import {decode as readPacket} from "ribbon-core";\nexport function load(value){return readPacket(value,{mode:"strict"});}\n'
  }, {
    path: 'docs/runtime.md',
    source: '# Runtime\n\nPreserve these local notes.\n'
  }];
  const root = workspace(context, files),
    prompt = 'Document the ribbon-core dependency in docs/runtime.md based on native/Project.toml and web/client.mjs. Preserve the existing guide and include observed declarations and imported calls.';
  const result = await run(root, prompt),
    state = JSON.parse(result.answer);
  assert.equal(state.reason, 'UsefulCandidateReadbackVerified');
  const guide = fs.readFileSync(path.join(root, 'docs/runtime.md'), 'utf8');
  assert.ok(guide.startsWith(files[2].source));
  assert.ok(guide.includes('`ribbon-core`'));
  assert.ok(guide.includes('`2.7`'));
  assert.ok(guide.includes('`decode`'));
  assert.ok(!guide.includes('command-stream'));
  assert.ok(state.facts.some(row => row.calls?.some(call => call.binding === 'readPacket')));
});
test('invalid manifest evidence and negated documentation instructions never write', async context => {
  const files = [{
    path: 'one.json',
    source: '{"dependencies":{"river":"1","river":"2"}}'
  }, {
    path: 'two.json',
    source: '{"dependencies":{"sand":"3"}}'
  }, {
    path: 'guide.md',
    source: '# Keep this guide\n'
  }];
  const root = workspace(context, files);
  for (const prompt of ['Document dependencies from one.json and two.json in guide.md. Preserve existing text.', 'Do not document dependencies from one.json or two.json in guide.md. Preserve existing text.']) {
    const result = await run(root, prompt);
    assert.equal(result.stop, 'final');
    assert.equal(JSON.parse(result.answer).verified, false);
    assert.ok(result.transcript.every(row => row.tool === 'read'));
    assert.equal(fs.readFileSync(path.join(root, 'guide.md'), 'utf8'), files[2].source);
  }
});
