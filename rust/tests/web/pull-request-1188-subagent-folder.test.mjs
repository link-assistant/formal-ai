// PR #1188: Formal AI works as a subagent on its own requirements from the
// repository folder experiments/formal_ai_subagent/ (README.md there), where
// the coding agents keep their claims, the gaps found by probing and the task
// prompts. These are the coordination edits Formal AI is asked to make there
// with the repository root as its workspace; a regression here takes Formal AI
// out of its own improvement loop. The Rust twin is
// rust/tests/unit/agentic-coding/pull_request_1188_subagent_folder.rs.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync, spawnSync } from 'node:child_process';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { completeOwnedSourceReadFrame } from '../../../js/agentic/tool_result.mjs';

let planChatStep;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
});

const GAPS = 'experiments/formal_ai_subagent/gaps.md';
const GAP_LIST = '# Gaps\n\n- G1 set overwrote a key\n- G2 change from to\n';

/** Run `prompt` over one file at `path`; the tools act on it as the Agent CLI's do. */
async function drive(prompt, path, source) {
  let file = source;
  const messages = [{ role: 'user', content: prompt }];
  const calls = [];
  for (let step = 0; step < 6; step += 1) {
    const plan = await planChatStep(messages, ['read', 'edit', 'bash', 'write']);
    if (!plan || plan.kind === 'final') return { calls, file, answer: plan ? plan.answer : null };
    const [call] = plan.calls;
    const args = JSON.parse(call.arguments);
    let result = '';
    if (call.tool === 'read') result = completeOwnedSourceReadFrame(file);
    if (call.tool === 'edit') file = file.replace(args.oldString, () => args.newString);
    if (call.tool === 'bash') result = `${createHash('sha256').update(file).digest('hex')}  ${path}\n`;
    calls.push([call.tool, args.filePath ?? args.command]);
    const id = `call_${step}`;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, content: result });
  }
  return { calls, file, answer: null };
}

describe('Formal AI edits its own coordination files', () => {
  test('a gap entry is appended to the gap list', async () => {
    const { calls, file, answer } = await drive(`Append the line '- G3 swap planned nothing' to ${GAPS}.`, GAPS, GAP_LIST);
    assert.deepEqual(calls.map(([tool]) => tool), ['read', 'edit', 'bash']);
    assert.equal(calls[0][1], GAPS);
    assert.equal(file, `${GAP_LIST}- G3 swap planned nothing\n`);
    assert.equal(answer, `Appended \`- G3 swap planned nothing\` to the end of \`${GAPS}\` and observed the result.`);
  });

  test('an entry holding backticks is echoed as one code span (G30)', async () => {
    const { file, answer } = await drive(`Append the line '- G3 the \`x\` flag' to ${GAPS}.`, GAPS, GAP_LIST);
    assert.equal(file, `${GAP_LIST}- G3 the \`x\` flag\n`);
    assert.equal(answer, `Appended \`\`- G3 the \`x\` flag\`\` to the end of \`${GAPS}\` and observed the result.`);
  });

  test('an entry is placed after the line it names', async () => {
    const { file } = await drive(`Insert the line '- G1b set in ru' after the line containing 'G1 set' in ${GAPS}.`, GAPS, GAP_LIST);
    assert.equal(file, '# Gaps\n\n- G1 set overwrote a key\n- G1b set in ru\n- G2 change from to\n');
  });

  test('the probe tool runs as asked', async () => {
    const plan = await planChatStep(
      [{ role: 'user', content: `Run node experiments/formal_ai_subagent/probe.mjs path "Read 'notes.txt'."` }],
      ['read', 'edit', 'bash', 'write'],
    );
    assert.equal(plan?.kind, 'tool_calls');
    assert.deepEqual(plan.calls.map((call) => [call.tool, JSON.parse(call.arguments).command]),
      [['bash', `node experiments/formal_ai_subagent/probe.mjs path "Read 'notes.txt'."`]]);
  });

  test('the local gate runner runs as asked', async () => {
    const command = 'node experiments/formal_ai_subagent/local-gates.mjs --only check-file-size';
    const plan = await planChatStep([{ role: 'user', content: `Run ${command}` }], ['read', 'edit', 'bash', 'write']);
    assert.deepEqual(plan.calls.map((call) => [call.tool, JSON.parse(call.arguments).command]), [['bash', command]]);
  });
});

describe('the local gate runner reads the gates CI runs', () => {
  const listed = execFileSync('node', ['experiments/formal_ai_subagent/local-gates.mjs', '--list'], {
    cwd: new URL('../../../', import.meta.url),
    encoding: 'utf8',
  }).split('\n');
  const row = (name) => listed.find((line) => line.split(/\s+/)[1] === name);

  test('a registry gate runs locally', () => {
    assert.match(row('check-file-size'), /^run {3}check[-_]file[-_]size +node scripts\/check-file-size\.mjs$/);
  });

  test('a gate that needs a Rust build is left to CI', () => {
    assert.match(row('run_clippy'), /^skip .*\[needs a Rust build \(CI only\)\]$/);
  });

  test('a workflow step given only the base ref runs with it', () => {
    assert.match(listed.find((line) => line.includes('check-changelog-fragment.')), /^run /);
  });
});

describe('local gate selection cannot silently verify zero gates', () => {
  const invoke = (...args) => spawnSync(process.execPath,
    ['experiments/formal_ai_subagent/local-gates.mjs', ...args], {
      cwd: new URL('../../../', import.meta.url), encoding: 'utf8',
      timeout: 10000, maxBuffer: 65536,
    });

  test('a misspelled gate fails before any selected gate executes', {timeout:15000}, () => {
    const result = invoke('--only', 'run_clippy,check_file_size');
    assert.equal(result.status, 1, result.stdout + result.stderr);
    assert.match(result.stderr, /Unknown gate name.*check_file_size/u);
    assert.doesNotMatch(result.stdout, /SKIP|PASS|0\/0/u);
  });

  test('an unmatched filter and an empty selection are errors', {timeout:15000}, () => {
    for (const args of [['--match', 'unmatched-gate-selection'], ['--only', '']]) {
      const result = invoke(...args);
      assert.equal(result.status, 1, result.stdout + result.stderr);
      assert.match(result.stderr, /No gates selected|--only requires/u);
      assert.doesNotMatch(result.stdout, /0\/0/u);
    }
  });

  test('a selected native gate stays an explicit successful skip', {timeout:15000}, () => {
    const result = invoke('--only', 'run_clippy');
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /SKIP run_clippy .*needs a Rust build/u);
    assert.match(result.stdout, /1\/1 passed or skipped/u);
  });
});
