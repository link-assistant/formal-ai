// Issue #1155 in the JavaScript root: a work-item read is validated before it
// becomes page evidence, and a failed read walks the fallbacks instead of
// ending the retrieval. Twin of rust/tests/unit/agentic-coding/issue_1155_read_validation.rs,
// driven through the JavaScript planner (`planChatStep`) the server runs, so
// the sentinel (R1155-1), the shape check (R1155-2), the fallback order
// (R1155-3) and the honest exhausted report (R1155-4) hold in both roots.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let planChatStep;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
});

const AGENT_TOOLS = ['bash', 'read', 'write', 'edit', 'webfetch', 'websearch'];
const CODEX_TOOLS = ['shell', 'apply_patch', 'update_plan', 'mcp__codex_apps__github_fetch', 'mcp__codex_apps__github_search'];
const ISSUE = 'https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/issues/1';
const PR = 'https://github.com/konard/test-hello-world-019fb331-c107-78c7-8ff6-9f127a3c593c/pull/2';
const GH_AUTH_BANNER =
  'To get started with GitHub CLI, please run:  gh auth login\nAlternatively, populate the GH_TOKEN environment variable with a GitHub API authentication token.';
const NOT_FOUND = '{"message":"Not Found","documentation_url":"https://docs.github.com/rest"}\n__formal_ai_exit=22\n';

const solvePrompt = (target, pr) =>
  `Issue to solve: ${target}\nYour prepared branch: issue-1-9f127a3c593c\n` +
  'Your prepared working directory: /tmp/hive-mind-workspace\n' +
  (pr ? `Your prepared Pull Request: ${pr}\n` : '') +
  '\nProceed.\n';

const ISSUE_BODY =
  'Implement Hello World in Rust\n\n## Task\nPlease implement a "Hello World" program in Rust.\n\n## Requirements\n' +
  '1. Create a file with the appropriate extension for Rust\n2. The program should print exactly: `Hello, World!`\n' +
  '3. **Create a GitHub Actions workflow that automatically runs and tests the program on every push and pull request**\n';

function commandOf(call) {
  try {
    const value = JSON.parse(call.arguments);
    for (const key of ['command', 'cmd', 'script']) if (typeof value[key] === 'string') return value[key];
  } catch {
    // Not JSON arguments: no command.
  }
  return '';
}

/** Drive a session; `results(tool, command)` is what the harness answers, echoed without a status. */
async function drive(tools, prompt, results, maxSteps) {
  const messages = [{ role: 'user', content: prompt }];
  const planned = [];
  for (let step = 0; step < maxSteps; step += 1) {
    const plan = await planChatStep(messages, tools);
    if (!plan) return { planned, answer: null };
    if (plan.kind === 'final') return { planned, answer: plan.answer };
    const call = plan.calls[0];
    const id = `c${step}`;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: id, name: call.tool, content: results(call.tool, commandOf(call)) });
    planned.push(call);
  }
  return { planned, answer: null };
}

describe('issue #1155 work-item read validation (JavaScript root)', () => {
  test('R1155-1: the planned read prints its own exit status as data', () => {
    const steps = readFileSync(new URL('../../../data/meta/work-item-steps.lino', import.meta.url), 'utf8');
    const view = steps.split('\n').find((line) => line.trim().startsWith('issue_view_command '));
    assert.ok(view && view.includes("printf '\\n__formal_ai_exit=%s\\n' $?"), view);
  });

  test('R1155-2/5: a status-less gh banner is a failed read and webfetch of the issue follows', async () => {
    const { planned } = await drive(AGENT_TOOLS, solvePrompt(ISSUE), (tool, command) =>
      tool === 'bash' && command.startsWith('gh issue view') ? GH_AUTH_BANNER : '', 2);
    assert.ok(planned.length >= 2, JSON.stringify(planned));
    assert.equal(planned[1].tool, 'webfetch');
    assert.ok(planned[1].arguments.includes(ISSUE), planned[1].arguments);
  });

  test('R1155-1/5: with the sentinel line the read is a failure and the fetch follows', async () => {
    const { planned } = await drive(AGENT_TOOLS, solvePrompt(ISSUE), (tool, command) =>
      tool === 'bash' && command.startsWith('gh issue view') ? `${GH_AUTH_BANNER}\n__formal_ai_exit=4\n` : '', 2);
    assert.equal(planned[1]?.tool, 'webfetch', JSON.stringify(planned));
  });

  test('R1155-1: a failing sentinel outranks a well-shaped output', async () => {
    const shaped = `Implement Hello World in Rust\n\n${GH_AUTH_BANNER}\n__formal_ai_exit=4\n`;
    const { planned } = await drive(AGENT_TOOLS, solvePrompt(ISSUE), (tool, command) =>
      tool === 'bash' && command.startsWith('gh issue view') ? shaped : '', 2);
    assert.equal(planned[1]?.tool, 'webfetch', JSON.stringify(planned));
  });

  test('R1155-3: the REST routes follow the fetch fallback in order', async () => {
    const { planned } = await drive(CODEX_TOOLS, solvePrompt(ISSUE), (tool, command) => {
      if (tool !== 'shell') return '';
      if (command.startsWith('gh issue view') || command.startsWith('gh api ')) return `${GH_AUTH_BANNER}\n__formal_ai_exit=1\n`;
      if (command.startsWith('curl ')) return NOT_FOUND;
      return '';
    }, 4);
    assert.ok(planned.length >= 4, JSON.stringify(planned));
    assert.equal(planned[1].tool, 'mcp__codex_apps__github_fetch');
    const rest = commandOf(planned[2]);
    assert.ok(rest.startsWith("curl -fsSL -H 'Accept: application/vnd.github.raw+json' https://api.github.com/repos/"), rest);
    assert.ok(rest.includes('/issues/1;'), rest);
    const api = commandOf(planned[3]);
    assert.ok(api.startsWith('gh api repos/') && api.includes('/issues/1 --jq .title'), api);
  });

  test('R1155-3: the REST fallback of a pull request reads the pulls endpoint', async () => {
    const { planned } = await drive(CODEX_TOOLS, solvePrompt(PR), (tool, command) =>
      tool === 'shell' && command.startsWith('gh pr view') ? `${GH_AUTH_BANNER}\n__formal_ai_exit=1\n` : '', 3);
    assert.ok(planned.length >= 3, JSON.stringify(planned));
    assert.ok(commandOf(planned[2]).includes('/pulls/2;'), commandOf(planned[2]));
  });

  test('R1155-3: the prepared pull request is the last read fallback', async () => {
    const { planned } = await drive(AGENT_TOOLS, solvePrompt(ISSUE, PR), (tool, command) => {
      if (tool === 'bash' && command.startsWith('gh ')) return `${GH_AUTH_BANNER}\n__formal_ai_exit=1\n`;
      if (tool === 'bash' && command.startsWith('curl ')) return '{"message":"Not Found"}\n__formal_ai_exit=22\n';
      return '';
    }, 5);
    assert.ok(planned.length >= 5, JSON.stringify(planned));
    assert.equal(planned[4].tool, 'webfetch');
    assert.ok(planned[4].arguments.includes(PR) && !planned[4].arguments.includes(ISSUE), planned[4].arguments);
  });

  test('R1155-4: an exhausted retrieval reports every read and writes no plan record', async () => {
    const { planned, answer } = await drive(CODEX_TOOLS, solvePrompt(ISSUE), (tool, command) => {
      if (tool !== 'shell') return '';
      if (command.startsWith('gh issue view') || command.startsWith('gh api ')) return `${GH_AUTH_BANNER}\n__formal_ai_exit=1\n`;
      if (command.startsWith('curl ')) return NOT_FOUND;
      return '';
    }, 10);
    assert.ok(answer, 'the exhausted retrieval closes with a report');
    for (const needle of ['could not be read', 'gh issue view', 'curl -fsSL', 'gh api ', 'exit status 1']) {
      assert.ok(answer.includes(needle), `${needle}: ${answer}`);
    }
    assert.equal(planned.filter((call) => call.tool === 'shell').length, 3, JSON.stringify(planned));
    for (const call of planned) assert.ok(!call.arguments.includes('.formal-ai/general-change-plan.lino'), call.arguments);
  });

  test('a successful curl read drives execution and the sentinel stays out of the text', async () => {
    const { planned } = await drive(CODEX_TOOLS, solvePrompt(ISSUE), (tool, command) => {
      if (tool !== 'shell') return '';
      if (command.startsWith('gh issue view')) return GH_AUTH_BANNER;
      if (command.startsWith('curl ')) return `${ISSUE_BODY}\n__formal_ai_exit=0\n`;
      return 'Hello, World!\n';
    }, 12);
    const patches = planned.filter((call) => call.tool === 'apply_patch');
    assert.ok(patches.length > 0, JSON.stringify(planned.map((call) => call.tool)));
    for (const call of patches) assert.ok(!call.arguments.includes('__formal_ai_exit'), call.arguments);
  });

  test('a sentinel success is page evidence and never replanned', async () => {
    const { planned } = await drive(CODEX_TOOLS, solvePrompt(ISSUE), (tool, command) => {
      if (tool !== 'shell') return '';
      if (command.startsWith('gh issue view')) return `${ISSUE_BODY}\n__formal_ai_exit=0\n`;
      return 'Hello, World!\n';
    }, 12);
    assert.ok(planned.some((call) => call.tool === 'apply_patch'), JSON.stringify(planned.map((call) => call.tool)));
    assert.equal(planned.filter((call) => commandOf(call).startsWith('gh issue view')).length, 1);
  });
});
