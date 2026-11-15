// Issue #1486: a captured Hive draft failure is replayed offline, in seconds,
// without Docker, a release or Rust compilation. Each fixture under
// data/benchmarks/fixtures/draft-replays/ is the verbatim request the Hive
// client sent on the attempt that failed (extracted by
// scripts/extract-draft-replay.mjs). The replay drives the JavaScript planner
// from that request to its final answer with deterministic repository tool
// results and asserts the captured failure does not recur.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { basename, join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStepResolved } from '../../../js/agentic/planner.mjs';
import { workItemRecordOf } from '../../../js/agentic/work_item_report.mjs';
import { completeOwnedSourceReadFrame } from '../../../js/agentic/tool_result.mjs';
import { encodeDialogTyped } from '../../../js/logging/typed-codec.mjs';
import { capturedReason, extractDraftReplay, requireDraftReplay, responseText } from '../../../scripts/lib/draft-replay.mjs';

const root = new URL('../../../', import.meta.url).pathname;
const directory = join(root, 'data/benchmarks/fixtures/draft-replays');
const map = JSON.parse(readFileSync(join(root, 'data/benchmarks/focused-test-map.json'), 'utf8'));
const fixtures = readdirSync(directory).filter(name => name.endsWith('.json')).sort()
  .map(name => [name, JSON.parse(readFileSync(join(directory, name), 'utf8'))]);
let tracked = null;

before(async () => installNodeHost(new WorkerHost({
  WebAssembly: {
    compile() { throw Error('LocalProductWasmForbidden'); },
    instantiate() { throw Error('LocalProductWasmForbidden'); }
  }
})));

function trackedFiles() {
  tracked ??= execFileSync('git', ['ls-files'], { cwd: root, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 }).split('\n').filter(Boolean);
  return tracked;
}

function repositoryFile(path) {
  const full = join(root, String(path ?? '').replace(/^\/+/u, ''));
  return typeof path === 'string' && path !== '' && existsSync(full) && statSync(full).isFile() ? readFileSync(full, 'utf8') : null;
}

/** Deterministic tool results: tracked files for a name search, file text for a read. */
function answer(tool, args) {
  if (tool === 'read') {
    const path = args.filePath ?? args.file_path ?? args.path;
    const file = repositoryFile(path);
    return file === null ? 'Error: File not found: ' + path : completeOwnedSourceReadFrame(file);
  }
  if (tool === 'glob') {
    const name = /^\*\*\/([^*/?[\]{}]+)$/u.exec(args.pattern ?? '')?.[1];
    return name ? trackedFiles().filter(path => basename(path) === name).join('\n') : '';
  }
  return '';
}

async function replay(fixture, steps = 24) {
  const messages = structuredClone(fixture.messages);
  const transcript = [];
  for (let step = 0; step < steps; step += 1) {
    const plan = await planChatStepResolved(messages, fixture.tools);
    if (plan?.kind !== 'tool_calls') return { plan, transcript };
    const call = plan.calls[0];
    const args = JSON.parse(call.arguments);
    transcript.push([call.tool, args]);
    const id = 'replay-' + step;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] },
      { role: 'tool', tool_call_id: id, name: call.tool, content: answer(call.tool, args) });
  }
  assert.fail('the replay did not end within ' + steps + ' steps: ' + JSON.stringify(transcript));
}

test('every captured replay is complete and its failure family has focused tests', () => {
  assert.ok(fixtures.length > 0, 'no captured replay fixture');
  for (const [name, fixture] of fixtures) {
    requireDraftReplay(fixture);
    const family = map.families.find(entry => entry.id === fixture.family);
    assert.ok(family, name + ': family ' + fixture.family + ' is not in the focused test map');
    assert.ok(family.tests.includes('rust/tests/web/draft-request-replay.test.mjs'), name + ': its family does not run the replay');
    if (fixture.captured.reason) assert.ok(family.reasons.includes(fixture.captured.reason), name + ': reason not mapped to its family');
  }
});

for (const [name, fixture] of fixtures) {
  test(name + ': the captured ' + fixture.captured.reason + ' does not recur on the current planner', async () => {
    const { plan, transcript } = await replay(fixture);
    assert.equal(plan?.kind, 'final', JSON.stringify(plan));
    const record = workItemRecordOf(plan);
    const reason = record?.reason ?? capturedReason(plan.answer);
    assert.notEqual(reason, fixture.captured.reason, plan.answer);
    assert.equal(String(plan.answer).includes(fixture.captured.reason), false, plan.answer);
    if (fixture.family === 'work-item-read-policy') {
      const read = transcript.filter(([tool, args]) => tool === 'read' && repositoryFile(args.filePath ?? args.file_path ?? args.path) !== null);
      assert.ok(read.length > 0, 'no repository file was read: ' + JSON.stringify(transcript));
      assert.ok(record?.observations?.some(observation => observation.complete), JSON.stringify(record?.observations));
    }
  });
}

test('the extractor keeps the final request verbatim and maps its captured reason to a family', () => {
  const request = { model: 'm', messages: [{ role: 'user', content: 'Fix issue 7' }], tools: [{ type: 'function', function: { name: 'read' } }] };
  const final = JSON.stringify({ reason: 'ReadPolicyRefused', observations: [] });
  const sse = ['data: ' + JSON.stringify({ choices: [{ delta: { content: final.slice(0, 9) } }] }),
    'data: ' + JSON.stringify({ choices: [{ delta: { content: final.slice(9) } }] }), 'data: [DONE]', ''].join('\n\n');
  assert.equal(responseText(sse), final);
  const log = [
    encodeDialogTyped({ dialog_id: 'dialog_1', request_body: JSON.stringify({ ...request, messages: [] }), response_body: '' }),
    encodeDialogTyped({ dialog_id: 'dialog_1', request_body: JSON.stringify(request), response_body: sse })
  ].join('\n');
  const fixture = extractDraftReplay(log, { issue: 7, run: 1, sourceCommit: 'a'.repeat(40), mapping: map });
  assert.equal(fixture.family, 'work-item-read-policy');
  assert.deepEqual(fixture.messages, request.messages);
  assert.deepEqual(fixture.tools, ['read']);
  assert.equal(fixture.captured.kind, 'final');
  requireDraftReplay(fixture);
  const unmapped = encodeDialogTyped({ dialog_id: 'dialog_2', request_body: JSON.stringify(request), response_body: JSON.stringify({ choices: [{ message: { content: '{"reason":"NewReason"}' } }] }) });
  assert.throws(() => extractDraftReplay(unmapped, { issue: 7, mapping: map }), /UnmappedFailureFamily/);
  assert.throws(() => extractDraftReplay('', { issue: 7, mapping: map }), /DialogHasNoRequest/);
  assert.throws(() => requireDraftReplay({ ...fixture, messages: [] }), /DraftReplayMessages/);
});
