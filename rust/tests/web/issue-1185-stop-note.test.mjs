// Issue #1185 R4/R5/R6 parity: a stopped repair loop reports why it stopped
// and carries the attempt chain, as `stop_note` does in
// rust/tests/unit/agentic-coding/issue_1185_error_repair_loop.rs.

import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { MAX_REPAIR_RUNGS, failedStep, repairStep, stopNote } from '../../../js/agentic/repair_loop.mjs';

before(async () => {
  await installNodeHost(new WorkerHost());
});

const userTurn = () => ({ role: 'user', content: 'run the generated program and verify it' });
const call = (id, name, args) => ({
  role: 'assistant',
  content: '',
  tool_calls: [{ id, type: 'function', function: { name, arguments: args } }],
});
const result = (id, name, content) => ({ role: 'tool', content, tool_call_id: id, name });
const rustcFailure = () => failedStep(
  'rust',
  'error[E0308]: mismatched types\n --> src/main.rs:6:33\n  |\n6 |     let total = left + right;\n  |                          ^ expected `f64`, found `i32`\n',
  { exit_code: 1, failed_command: 'rustc --edition 2021 src/main.rs', artifact_path: 'src/main.rs' },
);
const TOOLS = ['web_search', 'web_fetch', 'write_file', 'bash'];

test('stopNote names the unresolved need and carries the attempt chain', () => {
  const messages = [
    userTurn(),
    call('c1', 'web_search', '{ "query": "rust E0308 mismatched types" }'),
    result('c1', 'web_search', 'https://example.org/gardening'),
    call('c2', 'web_fetch', '{ "url": "https://example.org/gardening" }'),
    result('c2', 'web_fetch', 'A page about raised beds and compost. Nothing about types.'),
  ];
  const outcome = repairStep(messages, TOOLS, rustcFailure(), 0, MAX_REPAIR_RUNGS);
  assert.deepEqual(outcome, { kind: 'stop', reason: 'no_match' });
  const note = stopNote(messages, rustcFailure(), outcome.reason, 0, MAX_REPAIR_RUNGS);
  assert.match(note, /unresolved/u);
  assert.match(note, /E0308/u);
  assert.match(note, /repair_attempts\n {2}attempt_count 1/u);
  assert.match(note, /candidate_fix false/u);
});

test('stopNote reports the spent ladder and is silent without a rung', () => {
  const exhausted = stopNote([userTurn()], rustcFailure(), 'exhausted', MAX_REPAIR_RUNGS, MAX_REPAIR_RUNGS);
  assert.match(exhausted, /rung 3/u);
  assert.match(exhausted, /repair_attempts/u);
  for (const reason of ['no_diagnostic', 'no_tools']) {
    assert.equal(stopNote([userTurn()], rustcFailure(), reason, 0, MAX_REPAIR_RUNGS), '');
  }
});
