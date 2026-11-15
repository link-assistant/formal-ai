// Issue #1571 A6: replay the Rust driver loop of run_agentic_task(ISSUE_698_WORK_ITEM) through the JS planner.
import fs from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());
const { planChatStepResolved } = await import('../../js/agentic/planner.mjs');
const source = fs.readFileSync('rust/tests/unit/agentic-coding/agentic_general_planner.rs', 'utf8');
const rustString = value => JSON.parse(value.replace(/\\\r?\n[ \t]*/gu, '').replace(/\r?\n/gu, '\\n'));
const task = rustString(source.match(/const ISSUE_698_WORK_ITEM: &str = ("(?:\\[\s\S]|[^"\\])*");/u)[1]);
const body = fs.readFileSync('rust/embedded/data/benchmarks/issue-requirements/issue-698.md', 'utf8');
const tools = ['web_search', 'web_fetch', 'write_file', 'run_command', 'translate', 'read_file'];
const messages = [{ role: 'user', content: task }];
for (let turn = 0; turn < 12; turn++) {
  const step = await planChatStepResolved(messages, tools);
  if (step.kind !== 'tool_calls') { console.log('FINAL', JSON.stringify(step.result ?? null), '\n', String(step.answer ?? '').slice(0, 1500)); break; }
  const call = step.calls[0];
  console.log('CALL', call.tool, call.arguments);
  const args = JSON.parse(call.arguments);
  let content, isError = false, sourceRead;
  if (call.tool === 'run_command') { content = `run_command produced no result for ${JSON.stringify(args.command)}`; isError = true; }
  else if (call.tool === 'web_fetch') { content = args.url === 'https://github.com/link-assistant/formal-ai/issues/698' ? body : `web_fetch error: 404 not found for ${args.url}`; isError = content !== body; }
  else if (call.tool === 'read_file') { content = 'No such file or directory (os error 2)'; isError = true; sourceRead = { path: args.path, success: false, complete: false, format: 'raw', error_code: 'ENOENT' }; }
  else { content = 'unemulated'; isError = true; }
  const id = `c${turn}`;
  messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
  const result = { role: 'tool', tool_call_id: id, name: call.tool, content };
  if (isError) result.is_error = true;
  if (sourceRead) result.source_read = sourceRead;
  messages.push(result);
}
process.exit(0);
