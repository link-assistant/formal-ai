// Issue #1558: run the recorded Agent CLI session for each fixture issue against
// this checkout and print the actions planned and the final answer.
// Usage: node experiments/issue-1558/simulate-session.mjs [issue ...]
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../js/server/node-host-install.mjs';
import { ISSUES, runSession } from '../../rust/tests/web/helpers/issue-1558-session.mjs';

const worker = new WorkerHost();
await ensureNodeHost({ worker }, { sourceUsageRustGrammar: false });
const { agenticOutcome, chatCompletionFromPlan } = await import('../../js/server/agentic.mjs');
const { parseChatRequest } = await import('../../js/server/chat-request.mjs');
const issues = process.argv.length > 2 ? process.argv.slice(2).map(Number) : ISSUES;
for (const issue of issues) {
  const session = await runSession({ worker, agenticOutcome, chatCompletionFromPlan, parseChatRequest }, issue, process.cwd());
  console.log(`== issue ${issue}: ${session.stop}`);
  for (const call of session.calls) console.log(' ', call.tool, call.arguments, call.answer ? (call.answer.failed ? 'FAILED' : `${call.answer.content.length} bytes`) : 'UNEMULATED');
  if (session.final) {
    let report = null;
    try { report = JSON.parse(session.final); } catch { report = null; }
    console.log(report ? JSON.stringify({ reason: report.reason, observations: report.observations?.map(o => [o.path, o.reference, o.resolvedBy, o.readFailed]), unresolved: report.unresolved, terms: report.terms }, null, 1) : session.final.slice(0, 800));
  }
}
