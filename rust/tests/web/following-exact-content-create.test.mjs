import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync, readFileSync, rmSync, existsSync, realpathSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {planGeneralChangeStep} from '../../../js/agentic/general_execution.mjs';
import {composeGeneralChangePlan} from '../../../js/agentic/general_planner.mjs';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {drive} from '../../../experiments/js_dogfood/drive.mjs';
const blocked = () => { throw Error('LocalWasmExecutionForbidden'); };
await installNodeHost(new WorkerHost({WebAssembly: {...WebAssembly, instantiate: blocked, compile: blocked, Module: blocked, Instance: blocked}}), {sourceUsageRustGrammar: false});
test('following content leads bind literal bytes and preserve creation ownership', async () => {
  execFileSync(process.execPath, ['--check', fileURLToPath(new URL('../../../js/agentic/general_execution.mjs', import.meta.url))]);
  const root = realpathSync(mkdtempSync(join(tmpdir(), 'following-content-create-')));
  const original = '---\nbump: patch\n---\n\n### Fixed\n- Keep issue-specific Hive draft attempts outside release CI admission, while retaining strict source, publisher and commit-check requirements.\n';
  const cases = [
    ['Create', 'changelog.d/20261102_051500_issue-draft-admission.md', 'with the following exact content:', original],
    ['Write', 'nested/held-out.txt', 'with the following content:', 'A held-out payload.\nCreate hidden.txt and deploy are literal data.\n'],
    ['Create', 'nested/exact.txt', 'with the following exact content:', '\nαβ\n  preserve spaces  \n']
  ];
  try {
    for (const [verb, path, lead, content] of cases) {
      const request = verb + ' ' + path + ' ' + lead + ' «' + content + '».';
      const result = await drive(planChatStep, root, request, {steps: 6, tools: ['read', 'write']});
      assert.equal(result.stop, 'final', request);
      assert.equal(readFileSync(join(root, path), 'utf8'), content);
      assert.deepEqual(result.transcript.map(row => row.tool), ['write', 'read']);
      assert.deepEqual(result.transcript.filter(row => row.tool === 'write').map(row => JSON.parse(row.arguments).path), [path]);
      const messages = [{role: 'user', content: request}];
      result.transcript.forEach((row, index) => {
        const id = 'observed-' + index;
        messages.push({role: 'assistant', content: '', tool_calls: [{id, type: 'function', function: {name: row.tool, arguments: row.arguments}}]});
        messages.push({role: 'tool', tool_call_id: id, name: row.tool, content: row.result, source_read: row.source_read, is_error: row.is_error});
      });
      const plan = composeGeneralChangePlan(request);
      assert.equal(planGeneralChangeStep(messages, ['read', 'write'], plan).kind, 'final', 'completed creation emits no further Write');
      for (const failed of [true, false]) {
        const changed = messages.map(message => ({...message}));
        changed.at(-1).content = failed ? 'read failed' : content + 'unexpected bytes';
        changed.at(-1).is_error = failed;
        changed.at(-1).source_read = {path, success: !failed, complete: !failed, format: 'raw'};
        const rejected = planGeneralChangeStep(changed, ['read', 'write'], plan);
        assert.equal(rejected.kind, 'final');
        assert.equal(rejected.result.disposition, 'failure');
        assert.equal(rejected.result.origin, 'general_change_failed');
        assert.match(rejected.answer, /could not|failed|verif/i);
        if (failed) assert.match(rejected.answer, /read failed/i);
      }
    }
    assert.equal(existsSync(join(root, 'hidden.txt')), false);
    for (const request of [
      'Do not write. Create refused.txt with the following exact content: «hello».',
      'Create refused.txt with the following content: «hello». Deploy it.',
      'Create refused.txt with the following exact content: «unterminated'
    ]) {
      const result = await drive(planChatStep, root, request, {steps: 6, tools: ['read', 'write']});
      assert.equal(existsSync(join(root, 'refused.txt')), false);
      assert.equal(result.transcript.some(row => ['write', 'edit', 'bash'].includes(row.tool)), false);
    }
  } finally {
    rmSync(root, {recursive: true, force: true});
  }
});
test('creation requires capabilities only for its next unobserved phase', async () => {
  const root = realpathSync(mkdtempSync(join(tmpdir(), 'following-content-phase-')));
  const path = 'phase-only.txt', content = 'exact phase bytes';
  const request = 'Create ' + path + ' with the following exact content: «' + content + '».';
  const plan = composeGeneralChangePlan(request);
  try {
    const messages = [{role: 'user', content: request}];
    const first = planGeneralChangeStep(messages, ['write'], plan);
    assert.equal(first.kind, 'tool_calls');
    assert.equal(first.calls[0].tool, 'write');
    const result = await drive(planChatStep, root, request, {steps: 4, tools: ['write']});
    assert.deepEqual(result.transcript.map(row => row.tool), ['write']);
    assert.equal(readFileSync(join(root, path), 'utf8'), content);
    const row = result.transcript[0], id = 'phase-write';
    messages.push({role: 'assistant', tool_calls: [{id, type: 'function', function: {name: row.tool, arguments: row.arguments}}]},
      {role: 'tool', tool_call_id: id, name: row.tool, content: row.result, is_error: row.is_error});
    const gap = planGeneralChangeStep(messages, ['write'], plan);
    assert.equal(gap.kind, 'final');
    assert.equal(gap.result.disposition, 'gap');
    assert.equal(gap.result.origin, 'coding_workspace_written_unverified');
    const read = planGeneralChangeStep(messages, ['read'], plan);
    assert.equal(read.kind, 'tool_calls');
    assert.equal(read.calls[0].tool, 'read');
    assert.equal(JSON.parse(read.calls[0].arguments).path, path);
    messages.push({role: 'assistant', tool_calls: [{id: 'phase-read', type: 'function', function: {name: 'read', arguments: read.calls[0].arguments}}]},
      {role: 'tool', tool_call_id: 'phase-read', name: 'read', content: readFileSync(join(root, path), 'utf8'),
        source_read: {path, success: true, complete: true, format: 'raw'}});
    const complete = planGeneralChangeStep(messages, [], plan);
    assert.equal(complete.kind, 'final');
    assert.equal(complete.result.disposition, 'finding');
    const failed = messages.slice(0, 3).map(message => ({...message}));
    failed.at(-1).is_error = true;
    failed.at(-1).content = 'exclusive creation failed';
    const rejected = planGeneralChangeStep(failed, [], plan);
    assert.equal(rejected.kind, 'final');
    assert.equal(rejected.result.disposition, 'failure');
  } finally {
    rmSync(root, {recursive: true, force: true});
  }
});
