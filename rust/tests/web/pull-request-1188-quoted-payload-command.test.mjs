// PR #1188 G32: the words of a quoted payload never choose a shell command.
// `Add the line '- run tests' at the end of the section '## Usage' in
// README.md.` ran the workspace test suite (`bun test`), because the shell
// intent table matched its cue `run tests` inside the quotes. A request about
// text inside a file (`editsInsideAFile`) now reads the intent cues outside its
// quotes only, while a quoted command (`Run 'npm test'.`) still runs. Intent
// cues match whole words (G66: the `move` cue inside `removed` planned `mv`). The Rust
// twin is rust/tests/unit/pull_request_1188_quoted_payload_command.rs.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

const TOOLS = ['read', 'edit', 'bash', 'write'];
let planChatStep;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
});

const commands = async (prompt) => {
  const plan = await planChatStep([{ role: 'user', content: prompt }], TOOLS);
  return plan?.kind === 'tool_calls'
    ? plan.calls.filter((call) => call.tool === 'bash').map((call) => JSON.parse(call.arguments).command)
    : [];
};

describe('a quoted payload never chooses a shell command', () => {
  test('a line to add that names a test run runs nothing', async () => {
    assert.deepEqual(await commands("Add the line '- run tests' at the end of the section '## Usage' in README.md."), []);
  });

  test('an insert whose line names a test run reads the file to edit it', async () => {
    const plan = await planChatStep([{ role: 'user', content: "Insert the line 'run the tests' after the line 'x' in notes.txt." }], TOOLS);
    assert.deepEqual(plan.calls.map((call) => [call.tool, JSON.parse(call.arguments).filePath]), [['read', 'notes.txt']]);
  });

  test('a cue matches whole words only: move is not inside removed (G66)', async () => {
    const planned = await commands("Append the line 'a' then removed from f.md to g2.md.");
    assert.ok(!planned.some((command) => /^mv\b/.test(command)), planned.join('; '));
  });

  test('a quoted command still runs', async () => {
    assert.deepEqual(await commands("Run 'npm test'."), ['npm test']);
  });
});
