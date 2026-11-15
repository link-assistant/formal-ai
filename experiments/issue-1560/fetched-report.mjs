// Prints the report for the issue-1069 fetched-task fixture (issue #1560 migration check).
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { planChatStepResolved } from '../../js/agentic/planner.mjs';
import { workItemRecordOf } from '../../js/agentic/work_item_report.mjs';
await installNodeHost(new WorkerHost({}));
const task = 'Solve https://github.com/link-assistant/formal-ai/issues/1069 in this checkout as one whole task.';
const prose = 'Produce a legitimate release with no fabricated evidence.\nAdding a bypass flag to check-self-development-release.rs is not acceptable.';
const tools = ['web_fetch', 'write_file', 'bash'];
const messages = [{ role: 'user', content: task }];
const plan = await planChatStepResolved(messages, tools);
const id = 'a1';
messages.push({ role: 'assistant', tool_calls: [{ id, type: 'function', function: { name: plan.calls[0].tool, arguments: plan.calls[0].arguments } }] },
  { role: 'tool', tool_call_id: id, content: 'Output: Issue #1069\n\n' + prose + '\n__formal_ai_exit=0\nExit Code: 0' });
const result = await planChatStepResolved(messages, tools);
console.log(result.answer);
const record = workItemRecordOf(result);
console.log(JSON.stringify({ ...record, observations: undefined, requirements: record?.requirements?.length }));
