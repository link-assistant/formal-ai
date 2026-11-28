// Issue #1191: two main-only failures of main run 43730270231, replayed in the
// fast PR lane through the JavaScript twins, without compiling Rust.
//
// 1. `issue_1558_reference_resolution`: Rust read a `gh issue view` result with
//    `normalized_payload`, which drops a body that quotes "exit 5" or "HTTP 403"
//    (issue 1488), while the JavaScript twin used `observedPayload`. The pull
//    request ran only the JavaScript side, so it was green and main failed.
// 2. `agentic_general_planner` (issue-698 driver): since #1560 a fetched issue
//    body no longer refuses reads, so the planner reads
//    data/benchmarks/industry-suite.lino after the fetch. The JavaScript planner
//    did that on the pull request; the Rust expectation was stale and only main
//    noticed.
//
// Both replays also pin the Rust side to the JavaScript behaviour, so a pull
// request that lets the two drift fails here first.

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const read = relative => readFileSync(join(ROOT, relative), 'utf8');

await installNodeHost(new WorkerHost());
const { normalizedPayload, observedPayload } = await import('../../../js/agentic/tool_result.mjs');
const { planChatStepResolved } = await import('../../../js/agentic/planner.mjs');

/** The Rust gh work-item read branch of `fn progress` (from `work_item_read_url` to the next `run_outputs.push`). */
function rustWorkItemReadBranch() {
  const source = read('rust/src/agentic_coding/progress.rs');
  const start = source.indexOf('&& let Some(url) = work_item_read_url(command)');
  assert.ok(start >= 0, 'the Rust gh work-item read branch is still found by its guard');
  const end = source.indexOf('run_outputs.push(raw);', start);
  assert.ok(end > start);
  // Comments may quote either helper's name; only code counts.
  return source.slice(start, end).split('\n').filter(line => !line.trim().startsWith('//')).join('\n');
}

test('a work-item body that quotes errors is observed, not normalized away (issue 1488 fixture)', () => {
  const raw = read('data/benchmarks/fixtures/issue-1558/issue-1488.txt');
  assert.equal(normalizedPayload(raw), null, 'the error-shaped body is what normalized_payload drops');
  const observed = observedPayload(raw);
  assert.equal(typeof observed, 'string');
  assert.ok(observed.trim().length > 0);
  for (const issue of [1484, 1485, 1487, 1507]) {
    assert.equal(typeof observedPayload(read(`data/benchmarks/fixtures/issue-1558/issue-${issue}.txt`)), 'string', `issue ${issue}`);
  }
});

test('the Rust gh work-item read uses observed_payload like js/agentic/progress.mjs', () => {
  const branch = rustWorkItemReadBranch();
  assert.match(branch, /super::tool_result::observed_payload\(&raw\)/u);
  assert.doesNotMatch(branch, /normalized_payload/u);
  const js = read('js/agentic/progress.mjs');
  const jsBranch = js.slice(js.indexOf('const url = command !== null ? workItemReadUrl(command) : null;'));
  assert.match(jsBranch.slice(0, jsBranch.indexOf('progress.run_outputs.push(raw);')), /observedPayload\(raw\)/u);
});

test('replay of the issue-698 driver plans run_command, web_fetch, read_file as the Rust test expects', async () => {
  const source = read('rust/tests/unit/agentic-coding/agentic_general_planner.rs');
  const rustString = value => JSON.parse(value.replace(/\\\r?\n[ \t]*/gu, '').replace(/\r?\n/gu, '\\n'));
  const declared = source.match(/const ISSUE_698_WORK_ITEM: &str = ("(?:\\[\s\S]|[^"\\])*");/u);
  assert.ok(declared, 'the Rust test still declares ISSUE_698_WORK_ITEM');
  const body = read('data/benchmarks/issue-requirements/issue-698.md');
  const tools = ['web_search', 'web_fetch', 'write_file', 'run_command', 'translate', 'read_file'];
  const messages = [{ role: 'user', content: rustString(declared[1]) }];
  const planned = [];
  const reads = [];
  let answer = null;
  for (let turn = 0; turn < 12 && answer === null; turn += 1) {
    const step = await planChatStepResolved(messages, tools);
    if (step.kind !== 'tool_calls') {
      answer = String(step.answer ?? '');
      break;
    }
    const call = step.calls[0];
    planned.push(call.tool);
    const args = JSON.parse(call.arguments);
    let content;
    let sourceRead;
    let isError = true;
    if (call.tool === 'run_command') {
      content = `run_command produced no result for ${JSON.stringify(args.command)}`;
    } else if (call.tool === 'web_fetch' && args.url === 'https://github.com/link-assistant/formal-ai/issues/698') {
      content = body;
      isError = false;
    } else if (call.tool === 'read_file') {
      reads.push(args.path);
      content = 'No such file or directory (os error 2)';
      sourceRead = { path: args.path, success: false, complete: false, format: 'raw', error_code: 'ENOENT' };
    } else {
      content = `unemulated ${call.tool}`;
    }
    const id = `c${turn}`;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    const result = { role: 'tool', tool_call_id: id, name: call.tool, content };
    if (isError) result.is_error = true;
    if (sourceRead) result.source_read = sourceRead;
    messages.push(result);
  }
  assert.deepEqual(planned, ['run_command', 'web_fetch', 'read_file']);
  assert.deepEqual(reads, ['data/benchmarks/industry-suite.lino']);
  assert.match(answer ?? '', /No such file or directory/u);

  // The Rust unit test must expect the same sequence and read path.
  const expected = source.match(/"run_command",\s*"web_fetch",\s*"read_file"/u);
  assert.ok(expected, 'agentic_general_planner.rs expects ["run_command", "web_fetch", "read_file"]');
  assert.ok(source.includes('"data/benchmarks/industry-suite.lino"'), 'the Rust test expects the industry-suite read');
});
