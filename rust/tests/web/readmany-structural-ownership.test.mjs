import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installDefaultNodeSourceHost} from '../../../js/server/default-node-source-bootstrap.mjs';
import {planNamedCapabilityStep} from '../../../js/agentic/capability_router.mjs';

before(async () => installDefaultNodeSourceHost(new WorkerHost()));
const plan = (prompt, tools) => planNamedCapabilityStep(prompt, [], tools);
const operands = result => JSON.parse(result.calls[0].arguments);

test('ReadMany preserves structurally owned multiple reads in both transports', () => {
  for (const names of [['one.txt', 'two.txt'], ['alpha.mjs', 'omega.rs']]) {
    const prompt = `Read ${names[0]} and ${names[1]}`;
    const direct = plan(prompt, ['read_many_files']);
    assert.deepEqual(operands(direct).paths, names);
    assert.deepEqual(operands(direct).file_paths, names);
    assert.equal(operands(plan(prompt, ['bash'])).command,
      `cat '${names[0]}' '${names[1]}'`);
  }
});

test('ReadMany does not infer read authority from proposed output filenames', () => {
  for (const tools of [['read_many_files'], ['bash']]) {
    const result = plan('Read one.txt and two.txt. Proposed source owners new.mjs and test.mjs.', tools);
    assert.ok(result);
    const args = operands(result);
    assert.deepEqual(args.paths ?? args.command, tools[0] === 'bash'
      ? "cat 'one.txt' 'two.txt'" : ['one.txt', 'two.txt']);
  }
});

test('ReadMany preserves original NoRead policy before transport lowering', () => {
  for (const tools of [['read_many_files'], ['bash']]) {
    assert.equal(plan('Read one.txt and two.txt. Do not read one.txt.', tools), null);
    assert.equal(plan('Do not read one.txt and two.txt.', tools), null);
    assert.equal(plan('Read one.txt and two.txt. Do not read private.txt.', tools), null);
  }
});

test('ReadMany cannot settle an unbound independent create or write clause', () => {
  for (const tools of [['read_many_files'], ['bash']]) {
    for (const prompt of [
      'Read one.txt and two.txt then write new.txt.',
      'Read one.txt and two.txt only after creating new.txt.',
    ]) assert.equal(plan(prompt, tools), null, prompt);
  }
});


test('An independent authoring clause remains a prerequisite read, with no write or final credit', () => {
  for (const tools of [['read_many_files'], ['bash']]) {
    const result = plan('Implement new.mjs and new.test.mjs. Read one.txt and two.txt.', tools);
    assert.equal(result.kind, 'tool_calls');
    const args = operands(result);
    assert.deepEqual(args.paths ?? args.command, tools[0] === 'bash'
      ? "cat 'one.txt' 'two.txt'" : ['one.txt', 'two.txt']);
    assert.equal(result.calls.length, 1);
    assert.ok(!JSON.stringify(args).includes('new.mjs'));
    assert.ok(!JSON.stringify(args).includes('new.test.mjs'));
  }
});
