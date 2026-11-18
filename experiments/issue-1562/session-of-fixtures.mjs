// Issue #1562: drive a recorded Agent CLI session (the PR #1493 request
// prefix) on each fixture text and print every planned call and the final.
// Usage: node experiments/issue-1562/session-of-fixtures.mjs [file...]
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../js/server/node-host-install.mjs';
import { runSession } from '../../rust/tests/web/helpers/issue-1558-session.mjs';

const worker = new WorkerHost();
await ensureNodeHost({ worker }, { sourceUsageRustGrammar: false });
const { agenticOutcome, chatCompletionFromPlan } = await import('../../js/server/agentic.mjs');
const { parseChatRequest } = await import('../../js/server/chat-request.mjs');
const files = process.argv.slice(2);
for (const file of files.length ? files : [4031, 4034, 4035].map(n => `data/benchmarks/fixtures/issue-1562/hive-mind-${n}.txt`)) {
  const issueText = readFileSync(file, 'utf8');
  const session = await runSession({ worker, agenticOutcome, chatCompletionFromPlan, parseChatRequest }, 1, process.cwd(),
    { issueText, repository: 'link-assistant/hive-mind' });
  console.log(`=== ${file}: ${session.stop} after ${session.calls.length} calls`);
  for (const call of session.calls) console.log('  ', call.tool, call.arguments.slice(0, 160));
  if (session.final) console.log(session.final.slice(0, 1500));
}
process.exit(0);
