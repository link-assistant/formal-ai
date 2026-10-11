import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { ownsAdditiveScope } from '../../../js/agentic/workspace_change/additive_scope.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
import { mkdtempSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
before(async () => { await installNodeHost(new WorkerHost()); });
const movement = "Move the line 'δ = 7' to the top of settings.toml.";
const positive = [
  movement,
  "Move the line 'δ = 7' to the end of settings.toml.",
  "Move the line 'δ = 7' after the line 'β = 2' in settings.toml.",
  "Move the line 'δ = 7' before the line 'β = 2' in settings.toml.",
  "Swap the lines 'δ = 7' and 'β = 2' in settings.toml.",
  "Append these lines to tree.lino:\n  branch y\n    leaf 8",
  "Append these lines to tree.lino:\n  Do not write.\n  Read secret.txt.",
  "Append these lines to tree.lino:\n  a dangling quote ' is authored data",
];
for (const request of positive) test('complete seeded scope: ' + request, () => {
  assert.equal(ownsAdditiveScope(request), true);
});
const negative = [
  'Do not write. ' + movement,
  'Do not read. ' + movement,
  'Publish remotely. ' + movement,
  movement + ' Publish remotely.',
  'Read secret.txt. ' + movement,
  "Create Move the line 'δ = 7' to the top of settings.toml.",
  "Move the line 'δ = 7' to the top and end of settings.toml.",
  "Move the line 'δ = 7' to the top of settings.toml and other.toml.",
  "Publish remotely. Append these lines to tree.lino:\n  branch y",
  "Do not write. Append these lines to tree.lino:\n  branch y",
];
for (const request of negative) test('unconsumed or contradictory scope: ' + request, () => {
  assert.equal(ownsAdditiveScope(request), false);
});
for (const request of negative.slice(0, 4)) test('no operation for unsupported movement: ' + request, async () => {
  const plan = await planChatStep([{ role: 'user', content: request }], ['read', 'edit', 'bash', 'write']);
  assert.ok(plan === null || plan.kind === 'final');
  assert.equal(plan?.calls?.length ?? 0, 0);
});

test('an explicit Read prerequisite remains Read, without a movement effect', async () => {
  const request = negative[4];
  const plan = await planChatStep([{ role: 'user', content: request }], ['read', 'edit', 'bash', 'write']);
  assert.equal(plan.kind, 'tool_calls');
  assert.equal(plan.calls.length, 1);
  assert.equal(plan.calls[0].tool, 'read');
  const args = JSON.parse(plan.calls[0].arguments);
  assert.equal(args.file_path ?? args.path, 'secret.txt');
});

test('a movement cannot discard an introduced body as additive data', () => {
  const request = "Move the line 'δ = 7' to the top of settings.toml:\n  Publish remotely.";
  assert.equal(ownsAdditiveScope(request), false);
});

test('quoted action-looking data cannot authorize discarding a movement body', () => {
  const request = "Move the line 'write now' to the top of settings.toml:\n  authored body";
  assert.equal(ownsAdditiveScope(request), false);
});

const leftQuote = String.fromCodePoint(0xab), rightQuote = String.fromCodePoint(0xbb);
const physicalRefusals = [
  'Move the line ' + leftQuote + 'β😀' + rightQuote + ' to the end of sample.txt:\n  authored-looking bytes',
  "Move the line 'write now' to the top of sample.txt:\n  authored body",
  'Move the line ' + leftQuote + 'β😀' + rightQuote + ' after the line ' + leftQuote + 'α' + rightQuote
    + ' in sample.txt. An unavailable dependency must remain mandatory.',
];
for (const request of physicalRefusals) test('the real composer refuses an unconsumed movement request: ' + request, async () => {
  const workspace = mkdtempSync(join(tmpdir(), 'scope-operation-'));
  const source = 'α\nβ😀\nγ\n';
  try {
    writeFileSync(join(workspace, 'sample.txt'), source);
    const result = await drive(planChatStep, workspace, request, { steps: 6 });
    assert.equal(result.transcript.length, 0);
    assert.equal(readFileSync(join(workspace, 'sample.txt'), 'utf8'), source);
    assert.equal(result.stop, 'final');
    assert.equal(result.answer, 'Missing source authoring contract: complete request remains unbound.');
  } finally {
    rmSync(workspace, { recursive: true, force: true });
  }
});
