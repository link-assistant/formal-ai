// Real append persistence for general change events, including read-back evidence.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { execFile, execFileSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { composeGeneralChangePlan, planLinksNotation, PLAN_PATH } from '../../../js/agentic/general_planner.mjs';
import { planGeneralChangeStep } from '../../../js/agentic/general_execution.mjs';
import { reportedExitCode, harnessReportedFailure, observedPayload } from '../../../js/agentic/tool_result.mjs';

before(async () => { await installNodeHost(new WorkerHost()); });
const request = 'Set the contents of result.txt to «hello»';
function result(messages, call, output) {
  const id = `event-${messages.length}`;
  messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
  messages.push({ role: 'tool', tool_call_id: id, content: output });
}
function readWriteSession(files, options = {}) {
  const plan = composeGeneralChangePlan(request);
  const messages = [{ role: 'user', content: request }];
  const tools = ['read', 'write', 'run_command'];
  const calls = [];
  for (let turn = 0; turn < 12; turn += 1) {
    const step = planGeneralChangeStep(messages, tools, plan);
    if (step.kind === 'final') return { calls, answer: step.answer, event: planLinksNotation(plan) };
    const [call] = step.calls;
    const args = JSON.parse(call.arguments);
    calls.push({ tool: call.tool, args });
    const path = args.path ?? args.filePath;
    let output;
    if (call.tool === 'read') {
      output = files.has(path) ? files.get(path) : `Error: File not found: ${path}`;
      if (path === PLAN_PATH && options.denied) output = 'Error: Permission denied';
      if (path === PLAN_PATH && options.corrupt && calls.some((entry) => entry.tool === 'write' && entry.args.path === PLAN_PATH)) output = planLinksNotation(plan);
    } else if (call.tool === 'write') {
      files.set(path, args.content); output = '';
    } else { assert.equal(args.command, 'cat result.txt'); output = files.get('result.txt'); }
    result(messages, call, output);
  }
  assert.fail('append session exhausted its turn budget');
}

test('read clients preserve earlier bytes, verify the stream and deduplicate a replay', () => {
  const earlier = 'general_change_plan\n  id "earlier"\n  target "before.txt"';
  const files = new Map([[PLAN_PATH, earlier]]);
  const first = readWriteSession(files);
  assert.equal(files.get(PLAN_PATH), earlier + '\n' + first.event);
  assert.equal(files.get('result.txt'), 'hello');
  assert.deepEqual(first.calls.filter((call) => call.args.path === PLAN_PATH).map((call) => call.tool), ['read', 'write', 'read']);
  const second = readWriteSession(files);
  assert.equal(files.get(PLAN_PATH), earlier + '\n' + first.event);
  assert.equal(second.calls.filter((call) => call.tool === 'write' && call.args.path === PLAN_PATH).length, 0);
  assert.match(second.answer, /Completed the general change request/u);
});

test('an absent log is created, while a permission failure never overwrites it', () => {
  const files = new Map();
  const created = readWriteSession(files);
  assert.equal(files.get(PLAN_PATH), created.event);
  const deniedFiles = new Map([[PLAN_PATH, 'retained']]);
  const denied = readWriteSession(deniedFiles, { denied: true });
  assert.equal(deniedFiles.get(PLAN_PATH), 'retained');
  assert.equal(denied.calls.filter((call) => call.tool === 'write').length, 0);
  assert.match(denied.answer, /Permission denied/u);
});

test('read-back that loses earlier events cannot authorize the requested write', () => {
  const files = new Map([[PLAN_PATH, 'earlier\n']]);
  const run = readWriteSession(files, { corrupt: true });
  assert.ok(!files.has('result.txt'));
  assert.match(run.answer, /could not verify/u);
});

test('run-only clients execute a quoted append and preserve history on retries', () => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-events-'));
  try {
    mkdirSync(join(directory, '.formal-ai'));
    const earlier = 'earlier-event-without-final-newline';
    writeFileSync(join(directory, PLAN_PATH), earlier);
    const plan = composeGeneralChangePlan('Set the contents of result.txt to «quote \' and $HOME and `echo nope`»');
    const event = planLinksNotation(plan);
    for (let replay = 0; replay < 2; replay += 1) {
      const messages = [{ role: 'user', content: plan.goal }];
      const first = planGeneralChangeStep(messages, ['bash', 'read', 'write'], plan);
      assert.equal(first.kind, 'tool_calls');
      const call = first.calls[0];
      assert.equal(call.tool, 'bash');
      const output = execFileSync('/bin/sh', ['-c', JSON.parse(call.arguments).command], { cwd: directory, encoding: 'utf8', timeout: 5000 });
      result(messages, call, output);
      const next = planGeneralChangeStep(messages, ['bash', 'read', 'write'], plan);
      assert.equal(next.calls[0].tool, 'write');
      assert.equal(JSON.parse(next.calls[0].arguments).content, plan.content);
      assert.equal(readFileSync(join(directory, PLAN_PATH), 'utf8'), earlier + '\n' + event);
    }
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test('write-only clients cannot overwrite an unobserved event stream', () => {
  const plan = composeGeneralChangePlan(request);
  const step = planGeneralChangeStep([{ role: 'user', content: request }], ['write'], plan);
  assert.equal(step.kind, 'final');
  assert.match(step.answer, /not executed/u);
});


test('simultaneous retries take the bounded lock and retain exactly one event', async () => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-events-concurrent-'));
  try {
    mkdirSync(join(directory, '.formal-ai'));
    const before = 'prior-event';
    writeFileSync(join(directory, PLAN_PATH), before);
    const plan = composeGeneralChangePlan(request);
    const step = planGeneralChangeStep([{ role: 'user', content: request }], ['bash', 'read', 'write'], plan);
    const command = JSON.parse(step.calls[0].arguments).command;
    const run = () => new Promise((resolve, reject) => execFile('/bin/sh', ['-c', command], { cwd: directory, encoding: 'utf8', timeout: 5000 }, (error, output) => error ? reject(error) : resolve(output)));
    const results = await Promise.all([run(), run()]);
    const event = planLinksNotation(plan);
    assert.equal(readFileSync(join(directory, PLAN_PATH), 'utf8'), before + '\n' + event);
    assert.ok(results.every((output) => output === event));
    mkdirSync(join(directory, PLAN_PATH + '.lock'));
    assert.throws(() => execFileSync('/bin/sh', ['-c', command], { cwd: directory, encoding: 'utf8', timeout: 5000 }), (error) => error.status === 75);
    assert.equal(readFileSync(join(directory, PLAN_PATH), 'utf8'), before + '\n' + event);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test('repository-root shell events use the same sandbox as file tools', async () => {
  const { execute, REPOSITORY_ROOT, PLAN_EVENTS_SANDBOX } = await import('../../../experiments/js_dogfood/drive.mjs');
  const committed = join(REPOSITORY_ROOT, PLAN_PATH);
  const before = readFileSync(committed, 'utf8');
  const plan = composeGeneralChangePlan('Set the contents of mapper-proof.txt to «sandboxed plan event»');
  const step = planGeneralChangeStep([{ role: 'user', content: plan.goal }], ['bash', 'read', 'write'], plan);
  const output = execute(REPOSITORY_ROOT, step.calls[0]);
  assert.equal(reportedExitCode(output), 0);
  assert.equal(harnessReportedFailure(output), false);
  assert.equal(observedPayload(output), planLinksNotation(plan));
  assert.equal(readFileSync(committed, 'utf8'), before);
  assert.ok(readFileSync(join(PLAN_EVENTS_SANDBOX, 'general-change-plan.lino'), 'utf8').includes(planLinksNotation(plan)));
});


test('cached event readback excludes the separator before a later canonical record', async () => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-events-middle-'));
  try {
    mkdirSync(join(directory, '.formal-ai'));
    const first = composeGeneralChangePlan(request);
    const later = composeGeneralChangePlan('Set the contents of later.txt to «second event»');
    const before = planLinksNotation(first) + '\n' + planLinksNotation(later);
    writeFileSync(join(directory, PLAN_PATH), before);
    const step = planGeneralChangeStep([{ role: 'user', content: request }], ['bash', 'read', 'write'], first);
    const command = JSON.parse(step.calls[0].arguments).command;
    const output = execFileSync('/bin/sh', ['-c', command], { cwd: directory, encoding: 'utf8', timeout: 5000 });
    assert.equal(output, planLinksNotation(first));
    assert.equal(readFileSync(join(directory, PLAN_PATH), 'utf8'), before);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
