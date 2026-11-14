// Render the work-item evidence terminal for a refused read and an
// evidence gap, and show the answer is prose while the record is kept on
// the result (issue #1560).
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost({}));
const { planWorkItemEvidence } = await import('../../js/agentic/work_item_evidence.mjs');
const { workItemRecordOf, parsesAsJsonObject } = await import('../../js/agentic/work_item_report.mjs');
const task = 'Fix the parser.\n\n- Update `src/lib.rs` so empty input is rejected.\n- Add a test in `tests/parser.rs`.\n\nDo not read src/lib.rs.';
const messages = [{ role: 'user', content: task }];
for (const tools of [['read'], []]) {
  const plan = planWorkItemEvidence(task, messages, tools, 'owner/repo#1');
  console.log('---', JSON.stringify(tools), plan?.kind);
  console.log(plan?.answer);
  console.log('json answer?', parsesAsJsonObject(plan?.answer ?? ''), 'record reason:', workItemRecordOf(plan)?.reason, 'refused:', JSON.stringify(workItemRecordOf(plan)?.refused));
}
