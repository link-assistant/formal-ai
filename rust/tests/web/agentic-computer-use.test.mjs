// The JavaScript port of rust/src/computer_use (planner, seed, lexicon,
// induction, synthesis). Expectations mirror rust/tests/issue_707_*.rs and
// rust/tests/unit/issue_1066_ladder_capability.rs.

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { handleMcp } from '../../../js/server/mcp.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { computerUsePlanAgenticStep, toolForPrimitive } from '../../../js/agentic/crate/computer_use_planner.mjs';
import { capabilityGapForRequest, planRequest } from '../../../js/agentic/crate/computer_use.mjs';
import { parseBenchmarkTasks, planForPrompt } from '../../../js/agentic/crate/computer_use_seed.mjs';
import { induce, linksNotation } from '../../../js/agentic/crate/computer_use_induction.mjs';
import { synthesizeWith } from '../../../js/agentic/crate/computer_use_synthesis.mjs';
import { capabilityGapCue, normalize, operationCues, resourceCue } from '../../../js/agentic/crate/computer_use_lexicon.mjs';

const CORPUS = 'rust/tests/fixtures/computer-use/tasks.lino';
const ROOT = new URL('../../../', import.meta.url);
const TOOLS = ['fs.read', 'fs.write', 'fs.list', 'fs.move', 'shell.run', 'http.fetch', 'http.post',
  'dom.query', 'dom.extract', 'archive.pack', 'archive.unpack', 'process.status'];

const user = (content) => ({ role: 'user', content, tool_calls: [], tool_call_id: null, name: null });
const toolResult = (id, name, content) => ({ role: 'tool', content, tool_calls: [], tool_call_id: id, name });
const benchmarkTasks = () => parseBenchmarkTasks(readFileSync(new URL(CORPUS, ROOT), 'utf8'));
const learned = () => induce(benchmarkTasks());
const taskRequest = (prompt, steps) => prompt + '\n```computer-use\n'
  + JSON.stringify(steps.map(({ primitive, arguments: args }) => ({ primitive, arguments: args })))
  + '\n```';
const customerRequest = (prompt = 'Filter active customers into a report') =>
  taskRequest(prompt, planForPrompt(benchmarkTasks(), prompt).steps);
const assistantCalls = (calls) => ({ role: 'assistant', content: '', tool_calls: calls, tool_call_id: null, name: null });

before(async () => {
  await installNodeHost(new WorkerHost());
});

test('ten seeded tasks, four prompts each, conditions name the permission', () => {
  const tasks = benchmarkTasks();
  assert.equal(tasks.length, 10);
  for (const task of tasks) {
    assert.equal(task.prompts.length, 4, task.id);
    assert.ok(task.steps.length >= 2, task.id);
    for (const step of task.steps) {
      assert.ok(step.precondition.includes(`tool:computer:${step.primitive}`), step.id);
      assert.ok(step.postcondition.trim(), step.id);
    }
  }
});

test('a seed-recorded plan: first step and its arguments', () => {
  assert.equal(planRequest('Filter active customers into a report'), null);
  const prompt = customerRequest();
  const id = planRequest(prompt).id;
  const plan = computerUsePlanAgenticStep([user(prompt)], TOOLS);
  assert.equal(plan.kind, 'tool_calls');
  assert.equal(plan.calls.length, 1);
  assert.equal(plan.calls[0].tool, 'fs.write');
  assert.equal(plan.calls[0].arguments,
    '{"confirmed":true,"content":"name,status\\nAda,active\\nLin,inactive\\nMei,active\\n",'
    + `"path":"input/customers.csv","plan_id":"${id}",`
    + '"postcondition":"output file bytes match the reported sha256",'
    + '"precondition":"agent mode active; tool:computer:fs.write granted; all paths are confined to workspace",'
    + `"step_id":"${id}-01"}`);
});

test('every seeded plan is emitted step by step and completes', () => {
  for (const task of benchmarkTasks()) {
    const prompt = task.prompts.find(([locale]) => locale === 'en')[1];
    const explicit = taskRequest(prompt, task.steps);
    const resolved = planRequest(explicit);
    const messages = [user(explicit)];
    for (const expected of resolved.steps) {
      const plan = computerUsePlanAgenticStep(messages, TOOLS);
      assert.equal(plan.kind, 'tool_calls', `${task.id} finished before ${expected.id}`);
      assert.equal(plan.calls[0].tool, expected.primitive);
      const args = JSON.parse(plan.calls[0].arguments);
      assert.equal(args.plan_id, resolved.id);
      assert.equal(args.step_id, expected.id);
      assert.equal(args.precondition, expected.precondition);
      assert.equal(args.postcondition, expected.postcondition);
      messages.push(toolResult(`call-${expected.id}`, plan.calls[0].tool, '{"verified":true}'));
    }
    const done = computerUsePlanAgenticStep(messages, TOOLS);
    assert.equal(done.kind, 'final');
    assert.equal(done.answer,
      `computer_use_complete: verified plan ${resolved.id} with ${resolved.steps.length} steps.`);
  }
});

test('progress is scoped to the latest user turn; a failed verification halts', () => {
  const prompt = customerRequest();
  const id = planRequest(prompt).id;
  const messages = [
    user('An earlier unrelated request'),
    toolResult('old-call', 'fs.read', '{"verified":true}'),
    user(prompt),
  ];
  assert.equal(computerUsePlanAgenticStep(messages, TOOLS).calls[0].tool, 'fs.write');
  messages.push(toolResult('current-call', 'fs.write', '{"verified":false}'));
  assert.deepEqual(computerUsePlanAgenticStep(messages, TOOLS), {
    kind: 'final',
    answer: `computer_use_incomplete: verification failed for plan ${id}, step `
      + `${id}-01 (fs.write); no later effects were scheduled.`,
  });
});

test('a name-less OpenAI tool result resolves through its call id and advances', () => {
  const prompt = customerRequest();
  const id = planRequest(prompt).id;
  const messages = [user(prompt)];
  const first = computerUsePlanAgenticStep(messages, TOOLS);
  messages.push(assistantCalls([{ id: 'call-01', type: 'function',
    function: { name: 'formal_ai_fs_write', arguments: first.calls[0].arguments } }]));
  messages.push(toolResult('call-01', null, '{"verified":true}'));
  const next = computerUsePlanAgenticStep(messages, TOOLS);
  assert.equal(next.kind, 'tool_calls');
  assert.equal(next.calls[0].tool, 'shell.run');
  assert.equal(JSON.parse(next.calls[0].arguments).step_id, `${id}-02`);
});

test('localized messages come from the seed', () => {
  const prompt = customerRequest('Отфильтруй активных клиентов в отчёт');
  const id = planRequest(prompt).id;
  const messages = [user(prompt), toolResult('c', 'fs.read', '{"verified":true}')];
  assert.equal(computerUsePlanAgenticStep(messages, TOOLS).answer,
    `computer_use_incomplete: проверка плана ${id}, шага ${id}-01 (fs.write) `
    + 'не пройдена; последующие действия не запускались.');
});

test('capability gap: seeded cue and recognised phrasing in every language', () => {
  assert.deepEqual(computerUsePlanAgenticStep([user('Take a screenshot of the rendered page')], TOOLS), {
    kind: 'final',
    answer: 'capability_gap: gui_rendering requires a rendering or vision tool; no visual result was claimed.',
  });
  assert.equal(capabilityGapForRequest('Take a screenshot of the rendered page').locale, 'en');
  for (const [locale, prompt] of [
    ['en', 'Take a screenshot of the rendered customers dashboard'],
    ['ru', 'Сделай снимок отрисованной страницы с клиентами'],
    ['hi', 'ग्राहकों के rendered page का screenshot लो'],
    ['zh', '截图渲染后的客户页面'],
  ]) {
    assert.equal(planRequest(prompt), null, locale);
    const gap = capabilityGapForRequest(prompt);
    assert.equal(gap.capability, 'gui_rendering', locale);
    assert.equal(gap.locale, locale);
    assert.ok(gap.response.includes('capability_gap'), locale);
  }
  for (const prompt of [
    'Please take a screenshot of the rendered dashboard',
    'Сделай снимок отрисованной страницы отчёта',
    'इस page का screenshot लेकर भेजो',
    '请截图渲染后的页面并发送',
  ]) {
    assert.equal(capabilityGapCue(normalize(prompt)), 'gui_rendering', prompt);
  }
});

test('a client that advertises no computer-use primitive gets null; a partial one is told the gap', () => {
  const prompt = 'Count the sub-tasks of the customer import rewrite and save the result in `counts.md`.';
  assert.equal(computerUsePlanAgenticStep([user(prompt)], ['write', 'read', 'bash']), null);
  assert.equal(computerUsePlanAgenticStep([user(prompt)], ['fs.read']), null);
  const explicit = customerRequest();
  const id = planRequest(explicit).id;
  assert.deepEqual(computerUsePlanAgenticStep([user(explicit)], ['fs.read']), {
    kind: 'final',
    answer: `capability_gap: required primitive fs.write was not advertised for plan ${id}, step ${id}-01.`,
  });
  assert.equal(toolForPrimitive(['read', 'mcp__formal_ai__fs_write'], 'fs.write'), 'mcp__formal_ai__fs_write');
  assert.equal(toolForPrimitive(['read'], 'fs.write'), null);
});

test('literal payload is not read as a computer-use plan', () => {
  const task = 'Create file data/seed/learned-program-rules.lino containing\n'
    + 'substitution_rules\n  id "learned_program_plan_rules"\n  rule "learned_reverse"\n    order "90"\n'
    + '    replace "request:task -> list_files_arg"\n';
  assert.equal(planRequest(task), null);
  assert.equal(capabilityGapForRequest(task), null);
});

test('lexicon: operation order and most specific resource', () => {
  assert.deepEqual(operationCues(normalize('Pack and unpack a document, then verify it')).map((cue) => cue.slug),
    ['computer_use_pack_archive', 'computer_use_unpack_archive']);
  assert.equal(resourceCue(normalize('Move the inbox note into the processed folder')).slug,
    'computer_use_resource_inbox_note');
});

test('the committed schema snapshot matches a fresh induction', () => {
  const committed = readFileSync(new URL('docs/case-studies/issue-707/learned-schemas.lino', ROOT), 'utf8');
  assert.equal(linksNotation(learned(), CORPUS), committed);
  assert.equal(learned().operations.size, 12);
  assert.deepEqual(learned().rejected, []);
});

function generalizationSuite() {
  const text = readFileSync(new URL('data/benchmarks/computer-use-generalization.lino', ROOT), 'utf8');
  const cases = [];
  for (const line of text.split('\n')) {
    const trimmed = line.trim();
    if (trimmed.startsWith('case ')) {
      cases.push({ id: trimmed.slice(5), resource: '', operations: [], prompts: [] });
      continue;
    }
    const current = cases[cases.length - 1];
    if (!current) continue;
    if (trimmed.startsWith('resource ')) current.resource = trimmed.slice(9);
    else if (trimmed.startsWith('operation ')) current.operations.push(trimmed.slice(10));
    else if (trimmed.startsWith('prompt ')) {
      const rest = trimmed.slice(7);
      const at = rest.indexOf(' ');
      current.prompts.push([rest.slice(0, at), rest.slice(at + 1).trim().replace(/^"+|"+$/g, '')]);
    }
  }
  return cases;
}

test('held-out requests synthesize the expected resource and operations; locales agree', () => {
  const cases = generalizationSuite();
  assert.ok(cases.length >= 12);
  for (const entry of cases) {
    let first = null;
    for (const [locale, prompt] of entry.prompts) {
      const synthesis = synthesizeWith(prompt, learned());
      assert.ok(synthesis, `${entry.id} [${locale}] produced no plan`);
      assert.equal(synthesis.resource, entry.resource, `${entry.id} [${locale}]`);
      assert.deepEqual(synthesis.operations, entry.operations, `${entry.id} [${locale}]`);
      assert.equal(planRequest(prompt), null);
      const plan = synthesis.plan;
      const identity = JSON.stringify([plan.id, plan.steps]);
      first ??= identity;
      assert.equal(identity, first, entry.id);
    }
  }
});

test('a synthesized plan is planned, advanced and completed', () => {
  const prompt = 'Count the lines in the notes and archive the result';
  const synthesis = synthesizeWith(prompt, learned());
  assert.equal(synthesis.plan.id,
    'synthesized-computer_use_resource_notes-computer_use_count_lines-computer_use_pack_archive');
  assert.deepEqual(synthesis.plan.steps.map((step) => [step.primitive, step.arguments]), [
    ['fs.write', { path: 'input/notes.txt', content: 'alpha\nbeta\ngamma\n', confirmed: true }],
    ['shell.run', { operation: 'count_lines', input: 'input/notes.txt', output: 'reports/count_lines-notes.txt',
      confirmed: true }],
    ['archive.pack', { paths: ['reports/count_lines-notes.txt'], archive: 'out/notes.fai', confirmed: true }],
    ['fs.read', { path: 'out/notes.fai' }],
  ]);
  const explicit = taskRequest(prompt, synthesis.plan.steps);
  const resolved = planRequest(explicit);
  const messages = [user(explicit)];
  for (const expected of resolved.steps) {
    const plan = computerUsePlanAgenticStep(messages, TOOLS);
    assert.equal(plan.calls[0].tool, expected.primitive);
    assert.equal(JSON.parse(plan.calls[0].arguments).step_id, expected.id);
    messages.push(toolResult(`call-${expected.id}`, expected.primitive, '{"verified":true}'));
  }
  assert.equal(computerUsePlanAgenticStep(messages, TOOLS).answer,
    `computer_use_complete: verified plan ${resolved.id} with 4 steps.`);
  messages.push(toolResult('extra', 'fs.read', '{"verified":true}'));
  assert.equal(computerUsePlanAgenticStep(messages, TOOLS).answer,
    `computer_use_incomplete: verification failed for plan ${resolved.id}, step unexpected-result (fs.read); `
    + 'no later effects were scheduled.');
});

test('malformed plans fail before scheduling the first effect', () => {
  const first = { primitive: 'fs.write', arguments: { path: 'first', content: 'caller bytes', confirmed: true } };
  const badSteps = [
    { primitive: 'shell.run', arguments: { operation: 'filter_csv', input: 'a', output: 'b', confirmed: true } },
    { primitive: 'fs.write', arguments: { path: 'b', content: 'extra', confirmed: 'true' } },
    { primitive: 'fs.read', arguments: { path: 'a', postcondition: 'override' } },
  ];
  for (const second of badSteps) {
    const prompt = taskRequest('Run the plan.', [first, second]);
    assert.equal(planRequest(prompt), null);
    assert.match(computerUsePlanAgenticStep([user(prompt)], TOOLS).answer, /^computer_use_input_required/);
  }
  const valid = taskRequest('Run the plan.', [first]);
  assert.ok(planRequest(valid));
  assert.equal(planRequest(valid + '\ntrailing payload\n```'), null);
  assert.equal(planRequest('```text\nexample\n```\n' + valid), null);
});

test('chat schedules the caller operation and preserves quotes in its data', async () => {
  const content = `The caller's file contains "double quotes".\n`;
  const request = taskRequest('Run the plan.', [
    { primitive: 'fs.write', arguments: { path: 'note.txt', content, confirmed: true } },
  ]);
  const plan = await planChatStep([user(request)], TOOLS);
  assert.equal(plan.kind, 'tool_calls');
  assert.equal(plan.calls[0].tool, 'fs.write');
  assert.equal(JSON.parse(plan.calls[0].arguments).content, content);
});

test('MCP observes caller HTTP bytes and verifies origin, status and confirmation', async () => {
  const script = fileURLToPath(new URL('rust/tests/fixtures/computer-use/serve.py', ROOT));
  const publisher = spawn('python3', [script], { stdio: ['ignore', 'pipe', 'inherit'] });
  const exited = once(publisher, 'exit');
  const original = process.env.FORMAL_AI_COMPUTER_USE_HTTP_ORIGIN;
  try {
    let announcement = '';
    for await (const data of publisher.stdout) {
      announcement += data;
      if (announcement.includes('\n')) break;
    }
    const origin = announcement.trim();
    assert.match(origin, /^http:\/\/127\.0\.0\.1:\d+$/u);
    const call = async (name, url, fields = {}) => {
      const response = await handleMcp({ agentMode: true }, { body: JSON.stringify({
        jsonrpc: '2.0', id: 1, method: 'tools/call', params: {
          name, arguments: {
            plan_id: 'caller-http-tests', step_id: 'test', precondition: 'granted',
            postcondition: 'verified bytes', url, save_as: 'capture', ...fields,
          },
        },
      }) });
      return JSON.parse(response.body).result.structuredContent;
    };
    delete process.env.FORMAL_AI_COMPUTER_USE_HTTP_ORIGIN;
    const denied = await call('http.fetch', origin + '/caller-data');
    assert.equal(denied.output.error, 'computer_use_http_origin_required');
    process.env.FORMAL_AI_COMPUTER_USE_HTTP_ORIGIN = origin;
    const fetched = await call('http.fetch', origin + '/caller-data');
    assert.equal(fetched.verified, true);
    assert.equal(fetched.output.body, 'caller α\n');
    assert.equal(fetched.output.headers['content-type'], 'text/plain; charset=utf-8');
    assert.equal(fetched.output.sha256, createHash('sha256').update('caller α\n').digest('hex'));
    const redirect = await call('http.fetch', origin + '/redirect');
    assert.equal(redirect.output.error, 'computer_use_http_status:302');
    const foreign = await call('http.fetch', 'https://example.invalid/data');
    assert.equal(foreign.output.error, 'computer_use_http_origin_mismatch');
    const refused = await call('http.post', origin + '/submit', { body: 'token=fixture-token' });
    assert.match(refused.output.error, /^confirmation_required/u);
    const posted = await call('http.post', origin + '/submit', { body: 'token=fixture-token', confirmed: true });
    assert.equal(posted.verified, true);
    assert.equal(JSON.parse(posted.output.body).accepted, true);
  } finally {
    if (original === undefined) delete process.env.FORMAL_AI_COMPUTER_USE_HTTP_ORIGIN;
    else process.env.FORMAL_AI_COMPUTER_USE_HTTP_ORIGIN = original;
    publisher.kill();
    await exited;
  }
});
