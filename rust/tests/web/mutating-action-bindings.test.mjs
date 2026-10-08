import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let verifiedRecipe;
let planStep;
before(async () => {
  await installNodeHost(new WorkerHost());
  ({ verifiedRecipe, planStep } = await import('../../../js/agentic/mutating_action.mjs'));
});

function workspace(run) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-effect-'));
  try { return run(directory); } finally { rmSync(directory, { recursive: true, force: true }); }
}
function execute(directory, command) {
  const recipe = verifiedRecipe(command);
  assert.ok(recipe, command);
  const steps = [];
  for (const step of recipe) {
    const result = spawnSync('/bin/sh', ['-c', step], { cwd: directory, encoding: 'utf8' });
    steps.push({ step, status: result.status, error: result.stderr });
    if (result.status !== 0) break;
  }
  return { recipe, steps };
}

test('G120: declared flags and value options bind the directory path', () => workspace((directory) => {
  const command = 'mkdir -p -m 700 -- "evidence rows/nested"';
  const result = execute(directory, command);
  assert.ok(result.steps.every((step) => step.status === 0));
  assert.deepEqual(result.recipe, ['test ! -e "evidence rows/nested" || test -d "evidence rows/nested"', command,
    'test -d "evidence rows/nested"']);
  assert.ok(execute(directory, command).steps.every((step) => step.status === 0));
}));

test('G120: reuse permits a directory and still blocks an occupied file', () => workspace((directory) => {
  writeFileSync(join(directory, 'occupied'), 'keep');
  const result = execute(directory, 'mkdir -p occupied');
  assert.equal(result.steps.length, 1);
  assert.equal(result.steps[0].status, 1);
  assert.equal(readFileSync(join(directory, 'occupied'), 'utf8'), 'keep');
}));

test('G121: each glob match gets its own destination and source checks', () => workspace((directory) => {
  mkdirSync(join(directory, 'logs'));
  mkdirSync(join(directory, 'evidence'));
  for (const name of ['one.log', 'two rows.log']) writeFileSync(join(directory, 'logs', name), name);
  const result = execute(directory, 'cp logs/*.log evidence/');
  assert.equal(result.steps.length, result.recipe.length);
  assert.ok(result.steps.every((step) => step.status === 0));
  for (const name of ['one.log', 'two rows.log']) {
    assert.equal(readFileSync(join(directory, 'evidence', name), 'utf8'), name);
    assert.equal(readFileSync(join(directory, 'logs', name), 'utf8'), name);
  }
}));

test('G121: an unmatched glob stops before preparation or action', () => workspace((directory) => {
  const result = execute(directory, 'cp absent-*.log evidence/');
  assert.equal(result.steps.length, 1);
  assert.equal(result.steps[0].status, 1);
}));

test('G121: occupied glob targets require explicit force consent', () => workspace((directory) => {
  mkdirSync(join(directory, 'evidence'));
  writeFileSync(join(directory, 'one.log'), 'new');
  writeFileSync(join(directory, 'evidence', 'one.log'), 'old');
  const blocked = execute(directory, 'cp *.log evidence/');
  assert.equal(blocked.steps.length, 2);
  assert.equal(blocked.steps[1].status, 1);
  assert.equal(readFileSync(join(directory, 'evidence', 'one.log'), 'utf8'), 'old');
  const forced = execute(directory, 'cp -f *.log evidence/');
  assert.ok(forced.steps.every((step) => step.status === 0));
  assert.equal(readFileSync(join(directory, 'evidence', 'one.log'), 'utf8'), 'new');
}));

test('quoted operands and multiple sources keep their shell boundaries', () => workspace((directory) => {
  writeFileSync(join(directory, 'first row'), 'first');
  writeFileSync(join(directory, 'second row'), 'second');
  mkdirSync(join(directory, 'archive rows'));
  const copied = execute(directory, "cp 'first row' 'second row' 'archive rows/'");
  assert.ok(copied.steps.every((step) => step.status === 0));
  const moved = execute(directory, "mv 'first row' 'deep rows/final row'");
  assert.ok(moved.steps.every((step) => step.status === 0));
  assert.equal(readFileSync(join(directory, 'deep rows', 'final row'), 'utf8'), 'first');
}));

test('failed postconditions report the attempted action instead of a skipped action', () => {
  const command = 'mkdir -p rows';
  const messages = [{ role: 'user', content: 'Run ' + command }];
  let plan;
  for (let index = 0; index < 3; index += 1) {
    plan = planStep(command, messages, ['bash'], 'Run ' + command);
    const call = plan.calls[0];
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id: 'c' + index, type: 'function', function: {
      name: 'bash', arguments: call.arguments,
    } }] }, { role: 'tool', tool_call_id: 'c' + index, content: index === 2 ? 'Output: \nError: failed\nExit Code: 1' : '' });
  }
  plan = planStep(command, messages, ['bash'], 'Run ' + command);
  assert.equal(plan.kind, 'final');
  assert.equal(plan.answer, 'I attempted `mkdir -p rows`. The step `test -d rows` exited 1, so completion is not verified.');
});
