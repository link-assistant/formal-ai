// Issue #1154 in the JavaScript root: one shell-argument reading for the
// whole agentic module (R1154-1), the repeat invariant (R1154-2), and the
// fetched issue driving execution (R1154-3). Twin of
// rust/tests/unit/agentic-coding/issue_1154_progress_arguments.rs, driven through the
// JavaScript planner (`planChatStep`) and the server's agentic entry
// (`agenticOutcome`), which projects the request's tool schemas onto the
// transcript before planning.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let planChatStep;
let commandArgument;
let commandArgumentKey;
let projectDeclaredCommandKeys;
let agenticOutcome;
let parseChatRequest;

const worker = new WorkerHost();

before(async () => {
  await installNodeHost(worker);
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
  ({ commandArgument, commandArgumentKey, projectDeclaredCommandKeys } = await import(
    '../../../js/agentic/tool_result.mjs'
  ));
  ({ agenticOutcome } = await import('../../../js/server/agentic.mjs'));
  ({ parseChatRequest } = await import('../../../js/server/chat-request.mjs'));
});

const CODEX_TOOLS = ['shell', 'apply_patch', 'update_plan', 'mcp__codex_apps__github_fetch', 'mcp__codex_apps__github_search'];
const ISSUE = 'https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/issues/1';
const READ = `gh issue view '${ISSUE}' --json title --jq .title && echo && gh issue view '${ISSUE}' --json body --jq .body`;
const ISSUE_TEXT =
  'Implement Hello World in Rust\n\n## Task\nPlease implement a "Hello World" program in Rust.\n\n## Requirements\n' +
  '1. Create a file with the appropriate extension for Rust\n2. The program should print exactly: `Hello, World!`\n' +
  '3. **Create a GitHub Actions workflow that automatically runs and tests the program on every push and pull request**\n';

const solvePrompt = () =>
  `Issue to solve: ${ISSUE}\nYour prepared branch: issue-1-9f127a3c593c\n` +
  'Your prepared working directory: /tmp/hive-mind-workspace\n\nProceed.\n';

/** The transcript after one read under `argumentsObject`, optionally repeated. */
function transcript(tool, argumentsObject, times = 1) {
  const messages = [{ role: 'user', content: solvePrompt() }];
  for (let index = 0; index < times; index += 1) {
    const id = `fc_${index}`;
    messages.push({
      role: 'assistant',
      content: '',
      tool_calls: [{ id, type: 'function', function: { name: tool, arguments: JSON.stringify(argumentsObject) } }],
    });
    messages.push({ role: 'tool', tool_call_id: id, name: tool, content: ISSUE_TEXT });
  }
  return messages;
}

function plannedCommands(plan) {
  if (!plan || plan.kind !== 'tool_calls') return [];
  return plan.calls.map((call) => commandArgument(call.arguments) ?? call.arguments);
}

/** A shell tool whose schema names its command property `shell_command`. */
const CUSTOM_SHELL = {
  type: 'function',
  function: {
    name: 'shell',
    description: 'Run a shell command',
    parameters: {
      type: 'object',
      properties: {
        shell_command: { type: 'string', description: 'The shell command to execute' },
        workdir: { type: 'string', description: 'The working directory to run the command in' },
      },
      required: ['shell_command'],
    },
  },
};

describe('issue #1154 one shell-argument reading (JavaScript root)', () => {
  test('R1154-1: every fallback spelling reads the same command', () => {
    for (const args of [{ command: READ }, { cmd: READ }, { script: READ }]) {
      assert.equal(commandArgument(JSON.stringify(args)), READ);
    }
    assert.equal(commandArgument(JSON.stringify({ command: ['gh', 'issue', 'view', ISSUE] })), `gh issue view ${ISSUE}`);
  });

  test('R1154-1: the command key comes from the tool schema, not the working-directory property', () => {
    assert.equal(commandArgumentKey(CUSTOM_SHELL), 'shell_command');
    const codex = { type: 'function', name: 'exec_command', parameters: { properties: { cmd: { type: 'string', description: 'The bash command to execute' } } } };
    assert.equal(commandArgumentKey(codex), 'cmd');
  });

  test('R1154-1: the projection carries the declared key under `command` and leaves other calls alone', () => {
    const messages = transcript('shell', { shell_command: READ, workdir: '/tmp' });
    const projected = projectDeclaredCommandKeys(messages, [CUSTOM_SHELL]);
    assert.equal(commandArgument(projected[1].tool_calls[0].function.arguments), READ);
    assert.equal(commandArgument(messages[1].tool_calls[0].function.arguments), null, 'the client messages are not mutated');
    const fallback = transcript('shell', { cmd: READ });
    assert.equal(projectDeclaredCommandKeys(fallback, [CUSTOM_SHELL])[1], fallback[1], 'a call already readable is unchanged');
    assert.equal(projectDeclaredCommandKeys(messages, []), messages);
  });

  test('R1154-1/3: a schema-declared key is read exactly as the `command` key is', async () => {
    const canonical = await planChatStep(transcript('shell', { command: READ }), CODEX_TOOLS);
    assert.ok(canonical, 'the fetched issue must drive a step');
    for (const command of plannedCommands(canonical)) {
      assert.ok(!command.startsWith('gh issue view'), `re-planned the read: ${command}`);
    }
    const messages = transcript('shell', { shell_command: READ });
    const unprojected = await planChatStep(messages, CODEX_TOOLS);
    assert.notDeepEqual(unprojected, canonical, 'without the schema the read is invisible to the work-item path');
    const projected = await planChatStep(projectDeclaredCommandKeys(messages, [CUSTOM_SHELL]), CODEX_TOOLS);
    assert.deepEqual(projected, canonical);
  });

  test('R1154-1: the server entry applies the projection before planning', async () => {
    const body = { model: 'formal-ai', messages: transcript('shell', { shell_command: READ }), tools: [CUSTOM_SHELL] };
    const outcome = await agenticOutcome({ worker, agentMode: true }, parseChatRequest(JSON.stringify(body)), () => ({}));
    const canonical = await planChatStep(transcript('shell', { command: READ }), ['shell']);
    assert.equal(outcome.kind, 'planned', JSON.stringify(outcome));
    assert.deepEqual(outcome.plan, canonical);
  });

  test('R1154-3: the Codex `cmd` read drives execution instead of repeating', async () => {
    const plan = await planChatStep(transcript('exec_command', { cmd: READ }), CODEX_TOOLS);
    assert.ok(plan, 'the fetched issue must drive a step');
    for (const command of plannedCommands(plan)) {
      assert.ok(!command.startsWith('gh issue view'), `re-planned the read: ${command}`);
    }
  });

  test('R1154-2: a twice-completed read is reported, never planned a third time', async () => {
    const plan = await planChatStep(transcript('exec_command', { cmd: READ }, 2), CODEX_TOOLS);
    assert.ok(plan, 'the twice-read work item must still be planned');
    for (const command of plannedCommands(plan)) {
      assert.ok(!command.startsWith('gh issue view'), `a third identical read was planned: ${command}`);
    }
  });

  test('R1154-2: a completed call is the loop signature; byte-identical or same-command repeats are recognised', async () => {
    const { Progress } = await import('../../../js/agentic/progress.mjs');
    const progress = Progress.scan(transcript('exec_command', { cmd: READ }));
    const repeat = progress.repeatedCall({ tool: 'exec_command', arguments: JSON.stringify({ cmd: READ }) });
    assert.ok(repeat, 'the byte-identical call repeats the completed read');
    assert.ok(progress.repeatedCall({ tool: 'exec_command', arguments: JSON.stringify({ command: READ }) }),
      'the same command under another key is the same call');
    assert.equal(progress.repeatedCall({ tool: 'exec_command', arguments: JSON.stringify({ cmd: 'ls' }) }), null);
  });
});
