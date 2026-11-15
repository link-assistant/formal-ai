// Issue #1560: the qualification oracle recovers a work-item record by
// replaying the captured final request; the answer itself is a report.
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { planChatStepResolved } from '../../js/agentic/planner.mjs';
import { workItemRecordOf } from '../../js/agentic/work_item_report.mjs';
import { explicitUnknownTerminal, replayedWorkItemRecord } from '../../scripts/work-item-qualification/trace-oracle.mjs';
await installNodeHost(new WorkerHost({}));
const task = process.argv[2] ?? 'Investigate package.json before implementing the requested integration.';
const tools = (process.argv[3] ?? 'write').split(',');
const messages = [{ role: 'user', content: task }];
const plan = await planChatStepResolved(messages, tools);
if (!plan) { console.log("no plan"); process.exit(0); }
console.log(plan.kind, plan.answer ?? JSON.stringify(plan.calls));
const exchange = { request_body: JSON.stringify({ messages }), request_tools: tools };
const record = await replayedWorkItemRecord(exchange, plan.answer, planChatStepResolved, workItemRecordOf);
console.log('record', record?.schema, record?.reason, workItemRecordOf(plan) !== null);
console.log('terminal', JSON.stringify(explicitUnknownTerminal(plan.answer, record)));
console.log('without record', JSON.stringify(explicitUnknownTerminal(plan.answer)));
