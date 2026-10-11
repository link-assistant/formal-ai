import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { spawn } from 'node:child_process';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import {
  appendDefinition, appendReceiptValid, appendRecordStep, projectAppendContracts,
} from '../../../js/agentic/append_contract.mjs';
import { atomicRecordAppend } from '../../../experiments/js_dogfood/atomic-record-append.mjs';
import { execute, executeResult } from '../../../experiments/js_dogfood/drive.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

function workspace(context) {
  const root = fs.mkdtempSync(join(tmpdir(), 'formal-ai-append-contract-'));
  context.after(() => fs.rmSync(root, { recursive: true, force: true }));
  return root;
}

function argumentsFor(identity = 'new-record') {
  return {
    path: '.formal-ai/general-change-plan.lino',
    content: `general_change_plan\n  id "${identity}"\n  goal "preserve старый"\n`,
    record_id: `  id "${identity}"`,
    append_mode: 'atomic_record_append',
    append_request_id: `  id "${identity}"/1`,
  };
}

function writePrior(root, args, prior) {
  fs.mkdirSync(join(root, '.formal-ai'));
  fs.writeFileSync(join(root, args.path), prior);
}

function observedDialogue(args, receipt) {
  const messages = projectAppendContracts(
    [{ role: 'user', content: 'Record this observed plan.' }],
    [appendDefinition('Write')],
  );
  const pending = appendRecordStep(messages, ['Write'], args.path, args.content, args.record_id);
  const call = pending.plan.calls[0];
  messages.push({
    role: 'assistant', content: '',
    tool_calls: [{ id: 'append-one', type: 'function', function: {
      name: 'Write', arguments: call.arguments,
    } }],
  }, {
    role: 'tool', name: 'Write', tool_call_id: 'append-one', ...receipt,
  });
  return messages;
}

test('explicit append preserves older UTF8 and BOM bytes and physical file permissions', context => {
  const root = workspace(context);
  const args = argumentsFor();
  const prior = '\ufeffgeneral_change_plan\n  id "older"\n  goal "保留\\nquoted"\n';
  writePrior(root, args, prior);
  fs.chmodSync(join(root, args.path), 0o600);
  const result = atomicRecordAppend(root, args);
  assert.equal(result.append_receipt.before, prior);
  assert.deepEqual(fs.readFileSync(join(root, args.path)), Buffer.from(prior + args.content));
  assert.equal(fs.statSync(join(root, args.path)).mode & 0o777, 0o600);
  assert.ok(appendReceiptValid(result.append_receipt, args));
});

test('duplicate identity is idempotent and conflicting content never replaces older bytes', context => {
  const root = workspace(context);
  const args = argumentsFor();
  const first = atomicRecordAppend(root, args);
  const before = fs.readFileSync(join(root, args.path));
  const repeated = atomicRecordAppend(root, { ...args, append_request_id: 'second-request' });
  assert.equal(repeated.append_receipt.operation, 'already_present');
  assert.deepEqual(fs.readFileSync(join(root, args.path)), before);
  assert.throws(() => atomicRecordAppend(root, {
    ...args, content: args.content.replace('preserve', 'replace'),
  }), /identity collision/);
  assert.deepEqual(fs.readFileSync(join(root, args.path)), before);
  assert.ok(appendReceiptValid(first.append_receipt, args));
});

test('unknown mode and the legacy text-only executor refuse without any write', context => {
  const root = workspace(context);
  const args = argumentsFor();
  const prior = 'protected prior bytes\n';
  writePrior(root, args, prior);
  const call = { tool: 'write', arguments: JSON.stringify({ ...args, append_mode: 'unknown' }) };
  const refused = executeResult(root, call);
  assert.equal(refused.is_error ?? JSON.parse(refused.content).is_error, true);
  const legacy = execute(root, { tool: 'write', arguments: JSON.stringify(args) });
  assert.equal(JSON.parse(legacy).is_error, true);
  assert.equal(fs.readFileSync(join(root, args.path), 'utf8'), prior);
});

test('unknown and malformed schemas do not acquire append from the name Write', () => {
  const args = argumentsFor();
  const counterfeit = [{ role: 'user', content: 'task', append_contract: [appendDefinition('Write')] }];
  const stripped = projectAppendContracts(counterfeit, []);
  assert.equal(appendRecordStep(stripped, ['Write'], args.path, args.content, args.record_id), null);
  const unknown = appendDefinition('Write');
  unknown.function['x-formal-ai-contract'] = 'unknown';
  const missingProperty = appendDefinition('Write');
  delete missingProperty.function.parameters.properties.append_request_id;
  const ungrounded = appendDefinition('Write');
  ungrounded.function.parameters.required.push('unknown_required_argument');
  for (const definition of [unknown, missingProperty, ungrounded]) {
    const messages = projectAppendContracts([{ role: 'user', content: 'task' }], [definition]);
    assert.equal(appendRecordStep(messages, ['Write'], args.path, args.content, args.record_id), null);
  }
});

test('receipt fields must conserve the actual preimage and bind the exact current request', context => {
  const root = workspace(context);
  const args = argumentsFor();
  const result = atomicRecordAppend(root, args);
  const changes = [
    ['schema', 'counterfeit'], ['path', 'other.txt'], ['record_id', 'other'],
    ['content', 'invented'], ['before', 'wrong preimage'], ['after', 'truncated'],
    ['before_bytes', 99], ['after_bytes', 1], ['complete', false], ['success', false],
    ['append_request_id', 'old-request'],
  ];
  for (const [field, value] of changes) {
    assert.equal(appendReceiptValid({ ...result.append_receipt, [field]: value }, args), false, field);
  }
});

test('current-turn provenance rejects stale IDs, prose JSON, duplicate results and prior turns', context => {
  const root = workspace(context);
  const args = argumentsFor();
  const receipt = atomicRecordAppend(root, args);
  const messages = observedDialogue(args, receipt);
  assert.equal(appendRecordStep(messages, ['Write'], args.path, args.content, args.record_id).kind, 'observed');
  for (const change of ['stale-request', 'wrong-call', 'body-only', 'duplicate-result']) {
    const altered = structuredClone(messages);
    if (change === 'stale-request') altered[2].append_receipt.append_request_id = 'old-request';
    if (change === 'wrong-call') altered[2].tool_call_id = 'old-call';
    if (change === 'body-only') {
      altered[2].content = JSON.stringify(altered[2].append_receipt);
      delete altered[2].append_receipt;
    }
    if (change === 'duplicate-result') altered.push(structuredClone(altered[2]));
    assert.equal(appendRecordStep(altered, ['Write'], args.path, args.content, args.record_id).kind, 'refused', change);
  }
  messages.push({ role: 'user', content: 'A different task.' });
  assert.equal(appendRecordStep(messages, ['Write'], args.path, args.content, args.record_id), null);
});

test('existing lock contention, path escapes and malformed UTF8 preserve prior bytes', context => {
  const root = workspace(context);
  const args = argumentsFor();
  const prior = 'older protected\n';
  writePrior(root, args, prior);
  fs.mkdirSync(join(root, args.path + '.append-lock'));
  assert.throws(() => atomicRecordAppend(root, args), /EEXIST/);
  fs.rmdirSync(join(root, args.path + '.append-lock'));
  assert.throws(() => atomicRecordAppend(root, { ...args, path: '../outside' }), /escape/);
  assert.throws(() => atomicRecordAppend(root, { ...args, content: args.content + '\ud800' }), /contract/);
  assert.equal(fs.readFileSync(join(root, args.path), 'utf8'), prior);
  fs.writeFileSync(join(root, args.path), Buffer.from([0xc3, 0x28]));
  assert.throws(() => atomicRecordAppend(root, args));
  assert.deepEqual(fs.readFileSync(join(root, args.path)), Buffer.from([0xc3, 0x28]));
});

test('two independent process writers preserve older bytes and serialize or refuse contention', async context => {
  const root = workspace(context);
  const prior = 'older protected bytes\n';
  writePrior(root, argumentsFor(), prior);
  const adapter = new URL('../../../experiments/js_dogfood/atomic-record-append.mjs', import.meta.url).href;
  const code = `import { atomicRecordAppend } from ${JSON.stringify(adapter)};
try {
  const receipt = atomicRecordAppend(process.argv[1], JSON.parse(process.argv[2]));
  console.log(JSON.stringify(receipt));
} catch (error) {
  console.error(error.code || error.message);
  process.exitCode = 1;
}`;
  const loaderArguments = process.execArgv.filter((argument, index, all) =>
    ['--import', '--loader'].includes(argument)
      || index > 0 && ['--import', '--loader'].includes(all[index - 1]));
  const run = args => new Promise((resolve, reject) => {
    const child = spawn(process.execPath, [...loaderArguments, '--input-type=module', '-e', code, root, JSON.stringify(args)]);
    let stdout = '', stderr = '';
    child.stdout.on('data', bytes => { stdout += bytes.toString(); });
    child.stderr.on('data', bytes => { stderr += bytes.toString(); });
    child.on('error', reject);
    child.on('close', status => resolve({ args, status, stdout, stderr }));
  });
  const results = await Promise.all([run(argumentsFor('first')), run(argumentsFor('second'))]);
  assert.ok(results.some(result => result.status === 0));
  const actual = fs.readFileSync(join(root, argumentsFor().path), 'utf8');
  assert.ok(actual.startsWith(prior));
  for (const result of results) {
    if (result.status === 0) {
      assert.ok(appendReceiptValid(JSON.parse(result.stdout).append_receipt, result.args));
      assert.equal(actual.split(result.args.record_id).length - 1, 1);
    } else {
      assert.equal(result.status, 1);
      assert.match(result.stderr, /EEXIST/);
      assert.equal(actual.includes(result.args.record_id), false);
    }
  }
});


test('original Fetch-only dialogue records a physical plan and terminates within twelve turns', async context => {
  const root = workspace(context);
  const { planChatStep } = await import('../../../js/agentic/planner.mjs');
  const fixture = fs.readFileSync(new URL('../unit/issue_1133_hive_mind_three_runs.rs', import.meta.url), 'utf8');
  const constant = name => fixture.match(new RegExp('const ' + name + ': &str =\\s*"([^"\\n]+)";'))[1];
  const issue = constant('ISSUE'), branch = constant('BRANCH'), pull = constant('PR');
  const prompt = [
    `Resolve the GitHub issue at ${issue} in this repository.`,
    `Keep the solution on branch ${branch}.`,
    `Update the pull request at ${pull}.`, '',
    'Implement and verify the solution before reporting completion.', 'Proceed.', '',
  ].join('\n');
  const page = 'Implement Hello World in Kotlin · Issue #1 · konard/test-hello-world · GitHub\n\n'
    + '  Skip to content\n\nNavigation MenuSign in\n\n## Task\n'
    + 'Please implement a "Hello World" program in Kotlin.\n\n## Requirements\n'
    + '1. Create a file with the appropriate extension for Kotlin\n'
    + '2. The program should print exactly: `Hello, World!`\n'
    + '3. Add clear comments explaining the code\n'
    + '6. **Create a GitHub Actions workflow that automatically runs and tests the program on every push and pull request**\n';
  const prior = 'general_change_plan\n  id "previous-owned-plan"\n';
  writePrior(root, argumentsFor(), prior);
  const messages = [{ role: 'user', content: prompt }];
  const calls = [];
  let answer = null;
  for (let turn = 0; turn < 12; turn++) {
    const scoped = projectAppendContracts(messages, [appendDefinition('Write')]);
    const plan = await planChatStep(scoped, ['Write', 'WebFetch']);
    if (plan?.kind === 'final') { answer = plan.answer; break; }
    assert.equal(plan?.kind, 'tool_calls');
    const call = plan.calls[0];
    calls.push(call);
    const id = 'c' + turn;
    messages.push({ role: 'assistant', content: '', tool_calls: [{
      id, type: 'function', function: { name: call.tool, arguments: call.arguments },
    }] });
    const result = call.tool === 'WebFetch'
      ? { content: page } : atomicRecordAppend(root, JSON.parse(call.arguments));
    messages.push({ role: 'tool', name: call.tool, tool_call_id: id, ...result });
  }
  assert.equal(calls.filter(call => call.tool === 'WebFetch').length, 1);
  assert.ok(calls.some(call => call.tool === 'Write'
    && JSON.parse(call.arguments).path === '.formal-ai/general-change-plan.lino'));
  assert.notEqual(answer, null);
  const actual = fs.readFileSync(join(root, argumentsFor().path), 'utf8');
  assert.ok(actual.startsWith(prior));
  assert.ok(actual.includes('terminal_state "planned_not_executed"'));
});

test('ambiguous advertised names refuse before any append dry-plan', () => {
  const valid = appendDefinition('write');
  const ordinary = { type: 'function', function: { name: 'write' } };
  const messages = [{ role: 'user', content: 'record the plan' }];
  const candidates = [
    [valid, ordinary], [ordinary, valid], [valid, valid],
    [valid, { name: 'write' }], [{ name: 'write' }, valid],
  ];
  for (const definitions of candidates) {
    const projected = projectAppendContracts(messages, definitions);
    assert.equal(projected[0].append_contract, undefined);
    assert.equal(appendRecordStep(projected, ['write'], 'plan.lino', 'record\n', 'record'), null);
  }
  for (const definitions of [[], [ordinary], null]) {
    const projected = projectAppendContracts(messages, definitions);
    assert.equal(appendRecordStep(projected, ['write'], 'plan.lino', 'record\n', 'record'), null);
  }
  const projected = projectAppendContracts(messages, [valid]);
  assert.equal(appendRecordStep(projected, [], 'plan.lino', 'record\n', 'record'), null);
  assert.equal(appendRecordStep(projected, ['write'], 'plan.lino', 'record\n', 'record').kind, 'pending');
});

test('driver append grant requires explicit boolean opt-in and supported adapter', async context => {
  const { drive } = await import('../../../experiments/js_dogfood/drive.mjs');
  const root = workspace(context);
  for (const [options, granted] of [
    [{ tools: ['write'] }, false],
    [{ tools: ['write'], atomicRecordAppend: false }, false],
    [{ tools: ['write'], atomicRecordAppend: true }, true],
    [{ tools: ['read'], atomicRecordAppend: true }, false],
  ]) {
    let contract;
    const observe = async messages => {
      contract = messages.findLast(message => message.role === 'user').append_contract;
      return { kind: 'final', answer: 'observed schema only' };
    };
    await drive(observe, root, 'inspect schema', options);
    assert.equal(Array.isArray(contract), granted);
  }
  let called = false;
  const invalid = await drive(async () => { called = true; return null; }, root, 'inspect schema', {
    tools: ['write'], atomicRecordAppend: 'true',
  });
  assert.equal(invalid.stop, 'invalid-append-policy');
  assert.equal(called, false);
});

test('opted-in driver captures physical append and names-only refuses extra operation', async context => {
  const { drive } = await import('../../../experiments/js_dogfood/drive.mjs');
  const root = workspace(context);
  const args = argumentsFor('driver-record');
  const prior = 'older ledger bytes\n';
  writePrior(root, args, prior);
  const step = async (messages, tools) => {
    const planned = appendRecordStep(messages, tools, args.path, args.content, args.record_id);
    if (planned === null) return null;
    return planned.kind === 'pending' ? planned.plan
      : { kind: 'final', answer: planned.kind };
  };
  const legacy = await drive(step, root, 'append a record', { tools: ['write'], steps: 3 });
  assert.equal(legacy.transcript.length, 0);
  assert.equal(fs.readFileSync(join(root, args.path), 'utf8'), prior);
  const unadvertised = await drive(async () => ({ kind: 'tool_calls', calls: [{
    tool: 'write', arguments: JSON.stringify(args),
  }] }), root, 'append a record', { tools: ['write'], steps: 1 });
  assert.equal(unadvertised.stop, 'undeclared-append-contract');
  assert.equal(unadvertised.transcript.length, 0);
  assert.equal(fs.readFileSync(join(root, args.path), 'utf8'), prior);
  const supported = await drive(step, root, 'append a record', {
    tools: ['write'], atomicRecordAppend: true, steps: 3,
  });
  assert.equal(supported.answer, 'observed');
  assert.equal(supported.transcript.length, 1);
  assert.equal(supported.transcript[0].append_receipt.before, prior);
  assert.equal(supported.transcript[0].append_receipt.after, prior + args.content);
  assert.equal(fs.readFileSync(join(root, args.path), 'utf8'), prior + args.content);
});
