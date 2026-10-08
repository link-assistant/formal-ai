// PR #1188: Formal AI works as a subagent on its own requirements from the
// repository folder experiments/formal_ai_subagent/ (README.md there), where
// the coding agents keep their claims, the gaps found by probing and the task
// prompts. These are the coordination edits Formal AI is asked to make there
// with the repository root as its workspace; a regression here takes Formal AI
// out of its own improvement loop. The Rust twin is
// rust/tests/unit/pull_request_1188_subagent_folder.rs.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

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
    if (call.tool === 'read') result = file;
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
    const command = 'node experiments/formal_ai_subagent/local-gates.mjs --only check_file_size';
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
    assert.match(row('check_file_size'), /^run {3}check_file_size +node scripts\/check-file-size\.mjs$/);
  });

  test('a gate that needs a Rust build is left to CI', () => {
    assert.match(row('run_clippy'), /^skip .*\[needs a Rust build \(CI only\)\]$/);
  });

  test('a workflow step given only the base ref runs with it', () => {
    assert.match(listed.find((line) => line.includes('check-changelog-fragment.')), /^run /);
  });
});
