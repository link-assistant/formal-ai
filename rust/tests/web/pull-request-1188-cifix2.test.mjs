// PR #1188 CIFIX2: the routing regressions behind the Rust CI failures on
// 6e1c539fc (dogfood ledger rows T310-T319 in
// docs/case-studies/pull-request-1188/formal-ai-dogfood.md). Each request is
// planned by the planner the JS server runs (`planChatStep`). The Rust twin is
// rust/tests/unit/agentic-coding/pull_request_1188_cifix2.rs.

import { before, describe, test } from 'node:test';
import assert from 'node:assert/strict';

import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let planChatStep;

before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
});

/** The first planned call: `{tool, args}`. */
async function firstCall(prompt, tools) {
  const plan = await planChatStep([{ role: 'user', content: prompt }], tools);
  assert.equal(plan?.kind, 'tool_calls', `${prompt}: ${JSON.stringify(plan)}`);
  const [call] = plan.calls;
  return { tool: call.tool, args: JSON.parse(call.arguments) };
}

describe('a request that declares the file it writes creates it unread (T310)', () => {
  const tools = ['read_file', 'write_file', 'exec_command'];

  test('a file noun before the path and the content after it write first, whatever the verb', async () => {
    for (const prompt of [
      'add file note.txt containing hello',
      'add a new file src/lib.rs with content pub fn ready() {}',
      'new file: notes.txt, contents: hello',
      'record file note.txt containing hello',
      'добавь файл note.txt с текстом hello',
      'जोड़ो फ़ाइल note.txt सामग्री के साथ hello',
      '添加 文件 note.txt 内容为 hello',
      'añade archivo note.txt con el texto hello',
    ]) {
      const { tool } = await firstCall(prompt, tools);
      assert.equal(tool, 'write_file', prompt);
    }
  });

  test('a destination ahead of the file noun keeps the read-first guard', async () => {
    for (const prompt of ['add to the file note.txt containing hello', 'set note.txt containing hello']) {
      const { tool, args } = await firstCall(prompt, tools);
      assert.equal(tool, 'read_file', prompt);
      assert.equal(args.path, 'note.txt', prompt);
    }
  });
});

describe('an explicit grep runs as written; a search request is the workspace search (T311)', () => {
  test('execute grep TODO note.txt is the command itself', async () => {
    const { tool, args } = await firstCall('execute grep TODO note.txt', ['exec_command']);
    assert.equal(tool, 'exec_command');
    assert.equal(args.command, 'grep TODO note.txt');
  });

  test('a local search request greps the workspace for the whole word', async () => {
    const { args } = await firstCall('search for TODO in the code', ['exec_command']);
    assert.equal(args.command, "grep -rnHw --exclude-dir=.git -- 'TODO' '.'");
    const grep = await firstCall('search the code for RouteIntent', ['grep_search']);
    assert.equal(grep.args.pattern, '\\bRouteIntent\\b');
  });
});

describe('an article after a place preposition is no directory (T312)', () => {
  test('qué hay en la carpeta actual lists the current directory', async () => {
    const { tool, args } = await firstCall('qué hay en la carpeta actual', ['exec_command']);
    assert.equal(tool, 'exec_command');
    assert.equal(args.command, 'ls');
  });

  test('a named directory is still listed', async () => {
    const { args } = await firstCall('List the files in src.', ['exec_command']);
    assert.equal(args.command, "ls 'src'");
  });
});
