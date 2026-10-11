import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { mkdtempSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { collectionSummaryOwns } from '../../../js/agentic/planner/collection_summary.mjs';
import { goalLedger } from '../../../js/agentic/planner/owned_goals.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
const literal = target => ({ kind: 'literal_file', target });
const goals = (clause, targets) => [{ kind: 'unsupported', clause }, ...targets.map(literal)];
for (const [clause, targets] of [
  ['Two files.', ['a.txt', 'b.txt']],
  ['3 files.', ['a.txt', 'b.txt', 'c.txt']],
  ['Dos archivos.', ['a.txt', 'b.txt']],
  ['Два файла.', ['a.txt', 'b.txt']],
  ['दो फ़ाइलें।', ['a.txt', 'b.txt']],
  ['两文件。', ['a.txt', 'b.txt']],
]) test('seeded complete collection summary: ' + clause, () => {
  assert.equal(collectionSummaryOwns(goals(clause, targets)), true);
});
for (const [clause, targets] of [
  ['Two files.', ['a.txt']],
  ['Three files.', ['a.txt', 'b.txt']],
  ['Two files.', ['a.txt', './a.txt']],
  ['Two files.', ['A.txt', 'a.txt']],
  ['Two files.', ['a.txt', '../outside.txt']],
  ['Publish two files.', ['a.txt', 'b.txt']],
  ['Two files and deploy.', ['a.txt', 'b.txt']],
  ['"Two files."', ['a.txt', 'b.txt']],
  ['Two archivos.', ['a.txt', 'b.txt']],
]) test('unowned or contradictory summary: ' + clause + targets.join(','), () => {
  assert.equal(collectionSummaryOwns(goals(clause, targets)), false);
});
test('unsupported and edit obligations are never scheduled by a count', () => {
  for (const kind of ['unsupported', 'source_edit']) {
    const input = goals('Two files.', ['a.txt', 'b.txt']);
    input[2].kind = kind;
    assert.equal(collectionSummaryOwns(input), false);
  }
});
const actions = 'First, create file a.txt containing alpha. Second, create file b.txt containing beta.';
for (const request of [
  'Do not write. Two files. ' + actions,
  'Do not read. Two files. ' + actions,
  'Publish the unknown service. Two files. ' + actions,
  'Two files. ' + actions + ' Do not write.',
  'Two files. ' + actions + ' Read private.txt.',
  'Two files. ' + actions + ' Deploy the unknown service.',
  'Three files. ' + actions,
]) test('whole-request refusal before effects: ' + request, async context => {
  const workspace = mkdtempSync(join(tmpdir(), 'cardinal-summary-negative-'));
  context.after(() => rmSync(workspace, { recursive: true, force: true }));
  const result = await drive(planChatStep, workspace, request, { steps: 8 });
  assert.equal(result.stop, 'final');
  assert.equal(result.transcript.length, 0);
  assert.deepEqual(readdirSync(workspace), []);
  assert.match(result.answer, /no_artifact_in_clause/u);
});
test('authored policy-like bytes remain a complete literal obligation', () => {
  const request = 'Two files. First, create file a.txt containing «Do not write.».'
    + ' Second, create file b.txt containing «Read private.txt.».';
  const owned = goalLedger(request);
  assert.equal(collectionSummaryOwns(owned), true);
  assert.equal(owned[1].expected, 'Do not write.');
  assert.equal(owned[2].expected, 'Read private.txt.');
});
