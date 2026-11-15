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
import { workItemRecordOf, parsesAsJsonObject } from '../../../js/agentic/work_item_report.mjs';
function recorded(plan) {
  assert.equal(parsesAsJsonObject(plan.answer), false, 'a work-item answer is a report, not a JSON object');
  const record = plan.record ?? workItemRecordOf(plan);
  assert.ok(record, 'a work-item final keeps its record');
  return record;
}
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
    state = recorded(result);
  assert.equal(result.stop, 'final');
  assert.equal(state.reason, 'UsefulCandidateReadbackVerified');
  assert.equal(state.schema, 'repository-work-item-controller/v1');
  // The edit is authored and independently accepted against the raw sources;
  // integration semantics stay open, so the result is partial (issue #1483).
  assert.equal(state.complete, false);
  assert.equal(state.verified, true);
  assert.equal(state.authored, true);
  assert.match(result.answer, /^Partly done:/u);
  const status = kind => state.checks.find(check => check.kind === kind)?.status;
  assert.equal(status('task-claims'), 'passed');
  assert.equal(status('source-anchored-facts'), 'passed');
  assert.equal(status('preimage-preserved'), 'passed');
  assert.deepEqual(state.claims.map(claim => [claim.name, claim.status]), [['command-stream', 'confirmed'], ['web-capture', 'confirmed'], ['web-search', 'confirmed'], ['@link-assistant/web-search', 'confirmed']]);
  const verified = state.requirements.filter(row => row.status === 'verified').map(row => row.text);
  assert.ok(verified.some(text => text.includes('`command-stream = "1.5"`')), 'the manifest requirement is verified');
  assert.ok(verified.some(text => text.includes('`getDefaultProviderIds`')), 'the source-usage requirement is verified');
  assert.ok(state.requirements.some(row => row.status === 'unverified' && row.text.includes('relationship to knowledge acquisition')), 'semantic requirements stay open');
  assert.deepEqual(state.authorshipBoundary, { mechanism: 'supplied-documentation-derivation', targetOutputObserved: true, autonomousCredit: 0, laterReviewerWork: 'not-observed' });
  assert.ok(state.candidateEdits[0].diff.lines.includes('+| `command-stream` | `1.5` | `dependencies` | `unqualified` | `{}` |'));
  assert.ok(state.remainingWork.includes('consumer-discovery'));
  assert.ok(state.remainingWork.includes('independent-integration-semantics'));
  const original = fixture.files.find(file => file.path.endsWith('.md')),
    guide = fs.readFileSync(path.join(root, original.path), 'utf8');
  assert.ok(guide.startsWith(original.source));
  assert.notEqual(hash(guide), original.sha256);
  const declarations = state.facts.filter(row => row.kind === 'manifest-declaration');
  for (const [name, version] of [['command-stream', '1.5'], ['web-capture', '0.3.37'], ['web-search', '0.6.0'], ['@link-assistant/web-search', '0.11.1']]) {
    assert.ok(declarations.some(row => row.name === name && row.version === version), name);
    assert.ok(guide.includes('| `' + name + '` | `' + version + '` | `dependencies` |'), 'physical declaration row: ' + name);
  }
  for (const [name, version, feature] of [['web-capture', '0.3.37', 'search'], ['web-search', '0.6.0', 'merge']]) {
    const options = JSON.stringify({'default-features': false, features: [feature]});
    assert.ok(guide.includes('| `' + name + '` | `' + version + '` | `dependencies` | `cfg(not(target_arch = "wasm32"))` | `' + options + '` |'), 'physical native target and literal options: ' + name);
  }
  for (const call of ['getDefaultProviderIds()', 'mergeResults(grouped, { strategy: "rrf", rrfK: k, removeDuplicates: true })']) assert.ok(guide.includes(call), 'physical observed source excerpt: ' + call);
  const usage = state.facts.find(row => row.kind === 'observed-source-usage');
  assert.ok(usage);
  for (const name of ['getDefaultProviderIds', 'mergeResults']) assert.ok(usage.calls.some(call => call.imported === name), name);
  for (const file of fixture.files.filter(file => file.path !== original.path)) assert.equal(hash(fs.readFileSync(path.join(root, file.path))), file.sha256);
  assert.equal(result.transcript.filter(row => row.tool === 'edit').length, 1);
  const again = await run(root, task),
    repeated = recorded(again);
  assert.equal(repeated.reason, 'VerifiedNoOp');
  assert.equal(repeated.verified, true);
  assert.equal(repeated.authored, false);
  assert.match(again.answer, /^Partly done already:/u);
  assert.ok(again.transcript.every(row => row.tool === 'read'));
  assert.equal(fs.readFileSync(path.join(root, original.path), 'utf8'), guide);
});
test('a version claim the manifest contradicts is rejected before any edit', async context => {
  const contradicted = task.replace('`command-stream = "1.5"`', '`command-stream = "1.9"`');
  assert.notEqual(contradicted, task);
  const root = workspace(context),
    result = await run(root, contradicted),
    state = recorded(result);
  assert.equal(state.reason, 'ContradictedIntegrationClaim');
  assert.equal(state.authored, false);
  assert.equal(state.verified, false);
  assert.ok(result.transcript.every(row => row.tool === 'read'));
  for (const file of fixture.files) assert.equal(hash(fs.readFileSync(path.join(root, file.path))), file.sha256);
  assert.deepEqual(state.unresolved, [{ reference: 'command-stream = "1.9"', reason: 'ContradictedIntegrationClaim', observed: ['1.5'] }]);
  assert.equal(state.checks.find(check => check.kind === 'task-claims')?.status, 'failed');
  assert.ok(state.requirements.some(row => row.status === 'contradicted' && row.text.includes('`command-stream = "1.9"`')));
  assert.match(result.answer, /`command-stream = "1\.9"`: `ContradictedIntegrationClaim`/u);
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
    state = recorded(result);
  assert.equal(state.reason, 'UsefulCandidateReadbackVerified');
  const guide = fs.readFileSync(path.join(root, 'docs/runtime.md'), 'utf8');
  assert.ok(guide.startsWith(files[2].source));
  assert.ok(guide.includes('`ribbon-core`'));
  assert.ok(guide.includes('`2.7`'));
  assert.ok(guide.includes('`decode`'));
  assert.ok(!guide.includes('command-stream'));
  assert.ok(state.facts.some(row => row.calls?.some(call => call.binding === 'readPacket')));
  // Held-out paths and component: the same independent checks accept the edit,
  // and its one requirement is verified because every term it names is established.
  assert.equal(state.verified, true);
  assert.equal(state.checks.find(check => check.kind === 'source-anchored-facts')?.status, 'passed');
  assert.equal(state.checks.find(check => check.kind === 'preimage-preserved')?.status, 'passed');
  assert.deepEqual(state.requirements.map(row => row.status), ['verified']);
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
    assert.equal(recorded(result).verified, false);
    assert.ok(result.transcript.every(row => row.tool === 'read'));
    assert.equal(fs.readFileSync(path.join(root, 'guide.md'), 'utf8'), files[2].source);
  }
});
test('a consumer-search coverage record is skipped by the anchored-facts check, not read as unavailable', async () => {
  const { sourceAnchoredFacts } = await import('../../../js/agentic/documentation_acceptance.mjs');
  const sources = [{ path: 'package.json', source: '{"dependencies":{"river":"1.2.3"}}' }];
  const declaration = { kind: 'manifest-declaration', manifest: 'package.json', name: 'river', version: '1.2.3', section: 'dependencies' };
  const coverage = { kind: 'observed-consumer-discovery', state: 'incomplete', gaps: ['search-truncated'] };
  const artifact = '| river | 1.2.3 | dependencies |\n';
  const anchored = sourceAnchoredFacts([declaration, coverage], sources, artifact);
  assert.deepEqual([anchored.status, anchored.checked, anchored.missing], ['passed', 1, []]);
  assert.equal(sourceAnchoredFacts([declaration, { kind: 'unknown-fact' }], sources, artifact).status, 'unavailable');
  assert.equal(sourceAnchoredFacts([coverage], sources, artifact).status, 'unavailable');
});
