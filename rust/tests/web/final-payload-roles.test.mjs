import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import test from 'node:test';
import { FinalDisposition, resolvedFinalAnswer, canDeliverFinal } from '../../../js/agentic/final_result.mjs';
import { FinalPayloadRole } from '../../../js/agentic/final_payload.mjs';

test('an audit finding satisfies a report operand but cannot author source', () => {
  const plan = resolvedFinalAnswer('Line 29: const tokens = [];', FinalDisposition.Finding, 'observed_audit', FinalPayloadRole.AuditReport);
  assert.equal(canDeliverFinal(plan), true);
  assert.equal(canDeliverFinal(plan, FinalPayloadRole.SourceModule), false);
});

test('source delivery requires an actual complete-payload syntax receipt', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'source-role-'));
  try {
    const content = 'export function identity(value) { return value; }\n';
    const file = path.join(dir, 'module.mjs');
    fs.writeFileSync(file, content);
    execFileSync(process.execPath, ['--check', file]);
    const contentId = createHash('sha256').update(content).digest('hex');
    const artifact = { kind: 'observed-complete', content, contentId, syntax: { content, contentId, exitCode: 0 } };
    const plan = resolvedFinalAnswer(content, FinalDisposition.Finding, 'observed_syntax_check', FinalPayloadRole.SourceModule, artifact);
    assert.equal(canDeliverFinal(plan, FinalPayloadRole.SourceModule), true);
    for (const bad of [{ ...artifact, syntax: null }, { ...artifact, syntax: { ...artifact.syntax, exitCode: 1 } },
      { ...artifact, content: 'Line 2: ' + content }, { ...artifact, syntax: { ...artifact.syntax, contentId: 'stale' } }]) {
      assert.equal(canDeliverFinal(resolvedFinalAnswer(content, FinalDisposition.Finding, 'probe', FinalPayloadRole.SourceModule, bad), FinalPayloadRole.SourceModule), false);
    }
    fs.writeFileSync(file, 'Line 29: const tokens = [];');
    assert.throws(() => execFileSync(process.execPath, ['--check', file], { stdio: 'pipe' }));
  } finally { fs.rmSync(dir, { recursive: true, force: true }); }
});


test('source authoring operands reject audit text while ordinary reports retain findings', async () => {
  const { WorkerHost } = await import('../../../js/server/worker-host.mjs');
  const { installNodeHost } = await import('../../../js/agentic/node-host.mjs');
  const { planChatStep } = await import('../../../js/agentic/planner.mjs');
  const { drive } = await import('../../../experiments/js_dogfood/drive.mjs');
  await installNodeHost(new WorkerHost());
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'artifact-operand-'));
  try {
    fs.writeFileSync(path.join(dir, 'input.mjs'), 'export const tokens = [];\n');
    const sourceRequest = 'Add a source module to destination.any. Read input.mjs and report missing primitives.';
    const sourceResult = await drive(planChatStep, dir, sourceRequest, { steps: 12 });
    assert.equal(fs.existsSync(path.join(dir, 'destination.any')), false);
    assert.equal(sourceResult.transcript.some((call) => call.tool === 'write'), false);
    const reportResult = await drive(planChatStep, dir, 'Read input.mjs. Write a report about source in report.any.', { steps: 12 });
    assert.equal(fs.existsSync(path.join(dir, 'report.any')), true);
    assert.ok(fs.readFileSync(path.join(dir, 'report.any'), 'utf8').includes('export const tokens = [];'));
    assert.ok(reportResult.answer);
  } finally { fs.rmSync(dir, { recursive: true, force: true }); }
});


test('explicit successful stdout observations preserve compact JSON and whitespace', async () => {
  const { observedPayload, normalizedPayload } = await import('../../../js/agentic/tool_result.mjs');
  for (const content of ['{"z":"failed: recorded attempt","a":"fixture"}\n', '  seed\n    bundle\n', 'Error: literal example.\n']) {
    const raw = 'Output: ' + content + '\nExit Code: 0';
    assert.equal(observedPayload(raw), content);
    assert.equal(observedPayload(raw.replace('Exit Code: 0', 'Exit Code: 1')), null);
  }
  assert.notEqual(normalizedPayload('Output: {"z":2,"a":1}\nExit Code: 0'), '{"z":2,"a":1}');
});
