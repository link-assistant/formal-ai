// Drives a Hive Mind session on issue 1486, 1489 or 1501 to its end with the
// recorded issue text and this checkout's files (issue #1560 acceptance).
// Usage: node experiments/issue-1560/session-outcome.mjs <number> [steps]
import { existsSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { planChatStepResolved } from '../../js/agentic/planner.mjs';
import { workItemRecordOf, parsesAsJsonObject } from '../../js/agentic/work_item_report.mjs';
import { completeOwnedSourceReadFrame } from '../../js/agentic/tool_result.mjs';
await installNodeHost(new WorkerHost({}));
const root = new URL('../../', import.meta.url).pathname;
const fixtures = join(root, 'data/benchmarks/fixtures/work-item-1560');
const number = process.argv[2];
const issue = JSON.parse(readFileSync(join(fixtures, 'issues.json'), 'utf8')).issues[number];
const prompt = readFileSync(join(fixtures, 'hive-mind-prompt.txt'), 'utf8').split('{number}').join(number).split('{pull}').join(String(Number(number) + 1)).split('{branch}').join('0123456789ab');
const tools = ['bash', 'read', 'glob', 'grep', 'list', 'edit', 'write', 'task', 'webfetch', 'websearch', 'codesearch', 'batch', 'todowrite', 'todoread'];
const messages = [{ role: 'user', content: prompt }];
for (let step = 0; step < Number(process.argv[3] ?? 40); step += 1) {
  const plan = await planChatStepResolved(messages, tools);
  if (plan?.kind !== 'tool_calls') {
    console.log('FINAL json=' + parsesAsJsonObject(plan?.answer ?? '') + ' record.reason=' + workItemRecordOf(plan)?.reason);
    console.log(String(plan?.answer).slice(0, 1500));
    break;
  }
  const call = plan.calls[0];
  const args = JSON.parse(call.arguments);
  const path = args.filePath ?? args.file_path ?? args.path;
  let content = '';
  if (call.tool === 'bash' && /gh (issue|api) /.test(args.command ?? '')) content = issue.title + '\n\n' + issue.body + '\n\n__formal_ai_exit=0\n';
  else if (call.tool === 'read') {
    const full = join(root, String(path));
    content = existsSync(full) && statSync(full).isFile() ? completeOwnedSourceReadFrame(readFileSync(full, 'utf8')) : 'Error: File not found: ' + path;
  }
  console.log(step, call.tool, JSON.stringify(args).slice(0, 160));
  const id = 'c' + step;
  messages.push({ role: 'assistant', content: '', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] },
    { role: 'tool', tool_call_id: id, name: call.tool, content });
}
