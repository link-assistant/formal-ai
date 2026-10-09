import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { mkdtempSync, readFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { goalLedger } from '../../../js/agentic/planner/owned_goals.mjs';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';

before(async () => { await installNodeHost(new WorkerHost()); });
const affirmative = [
  ['bare payload', 'Write hello to x.txt', 'x.txt', 'hello'],
  ['closed payload then destination', 'Write «crème Ω» into beta.dat.', 'beta.dat', 'crème Ω'],
  ['seeded Spanish destination', 'Escribe «volcán Ω» en lectura.txt.', 'lectura.txt', 'volcán Ω'],
];
for (const [label, request, target, expected] of affirmative) {
  test('complete owned destination: ' + label, async () => {
    assert.equal(goalLedger(request), null);
    const directory = mkdtempSync(join(tmpdir(), 'formal-ai-owned-destination-'));
    try {
      const out = await drive(planChatStep, directory, request, { steps: 8 });
      assert.equal(readFileSync(join(directory, target), 'utf8'), expected);
      assert.equal(out.stop, 'final');
      assert.ok(out.transcript.some(entry => entry.tool === 'write' && JSON.parse(entry.arguments).path === target));
      assert.ok(out.transcript.some(entry => entry.tool === 'bash' && JSON.parse(entry.arguments).command === 'cat ' + target));
    } finally { rmSync(directory, { recursive: true, force: true }); }
  });
}

const ambiguous = [
  'Write «observed» and implement another transform to receipt.dat.',
  'Write «observed» and run an unrequested command to receipt.dat.',
  'Write «observed» to receipt.dat and explain the algorithm.',
  'Write observed to receipt.dat and implement a normalizer.',
  'Write «observed» and remove another file into receipt.dat.',
];
for (const request of ambiguous) {
  test('unowned action is never consumed as destination grammar: ' + request, async () => {
    const directory = mkdtempSync(join(tmpdir(), 'formal-ai-unowned-destination-'));
    try {
      const out = await drive(planChatStep, directory, request, { steps: 8 });
      assert.equal(out.transcript.some(entry => ['write', 'edit', 'bash'].includes(entry.tool)), false);
      assert.equal(existsSync(join(directory, 'receipt.dat')), false);
      assert.equal(out.stop, 'final');
    } finally { rmSync(directory, { recursive: true, force: true }); }
  });
}

test('the complete original failed repair request cannot authorize its embedded write example', async () => {
  const request = [
    'Repair js/agentic/planner/owned_goals.mjs and its Rust twin so whole literal writes with an already-owned destination after the payload, including Write hello to x.txt, remain complete.',
    'Prove the destination cue and exact target span through existing typed write bindings; reject any semantic clause before, between or after payload/destination and preserve all independent Goals/Needs.',
    'Do not accept arbitrary trailing text, raise ceilings, change original tests or run native locally.',
    'Add heldout bare/quoted, renamed, multilingual/Unicode and hidden-action controls; run the unchanged G15 and original ownership controls.',
  ].join(' ');
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-embedded-example-'));
  try {
    const out = await drive(planChatStep, directory, request, { steps: 8 });
    assert.equal(out.transcript.length, 0);
    assert.equal(existsSync(join(directory, 'x.txt')), false);
    assert.equal(out.stop, 'final');
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test('both whole literal clauses retain independently owned paths and exact bodies', () => {
  const request = 'Write «first Ω» to one.dat. Write «second λ» into two.dat.';
  const goals = goalLedger(request);
  assert.equal(goals.length, 2);
  assert.deepEqual(goals.map(goal => [goal.kind, goal.target, goal.expected]), [
    ['literal_file', 'one.dat', 'first Ω'], ['literal_file', 'two.dat', 'second λ'],
  ]);
  for (const goal of goals) assert.equal(request.slice(goal.span.start, goal.span.end), goal.clause);
});

test('a Unicode trailing goal is retained with exact UTF16 and UTF8 source positions', () => {
  const request = 'Write «Ω sample» to output.dat. Implement an unrelated λ normalizer.';
  const goals = goalLedger(request);
  assert.equal(goals.length, 2);
  assert.equal(goals[0].kind, 'literal_file');
  assert.equal(goals[1].kind, 'unsupported');
  const missing = goals[1];
  assert.equal(request.slice(missing.span.start, missing.span.end), missing.clause);
  assert.deepEqual(missing.byteSpan, [
    Buffer.byteLength(request.slice(0, missing.span.start)),
    Buffer.byteLength(request.slice(0, missing.span.end)),
  ]);
});
