// Issue #1562: replay the recorded Agent CLI session with the texts of
// hive-mind issues 4031, 4034 and 4035 in an empty git workspace and print
// every planned action. None of the three asks for a program.
// Usage: node experiments/issue-1562/replay-hive-mind.mjs [4031|4034|4035]
import { mkdtempSync, readFileSync, rmSync, realpathSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import os from 'node:os';
import path from 'node:path';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../js/server/node-host-install.mjs';
import { runSession } from '../../rust/tests/web/helpers/issue-1558-session.mjs';

const worker = new WorkerHost();
await ensureNodeHost({ worker }, { sourceUsageRustGrammar: false });
const { agenticOutcome, chatCompletionFromPlan } = await import('../../js/server/agentic.mjs');
const { parseChatRequest } = await import('../../js/server/chat-request.mjs');
const issues = process.argv.slice(2).length ? process.argv.slice(2).map(Number) : [4031, 4034, 4035];
for (const issue of issues) {
  const root = realpathSync(mkdtempSync(path.join(os.tmpdir(), 'issue-1562-')));
  try {
    execFileSync('git', ['init', '-q'], { cwd: root });
    const issueText = readFileSync(`data/benchmarks/fixtures/issue-1562/hive-mind-${issue}.txt`, 'utf8');
    const session = await runSession({ worker, agenticOutcome, chatCompletionFromPlan, parseChatRequest }, issue, root, { issueText, repository: 'link-assistant/hive-mind' });
    console.log(`=== hive-mind ${issue}: stop ${session.stop}`);
    for (const call of session.calls) console.log(' ', call.tool, call.arguments.replaceAll(root, '<ws>').slice(0, 300), call.answer ? (call.answer.failed ? 'FAILED' : `${call.answer.content.length} bytes`) : 'UNEMULATED');
    console.log(String(session.final ?? '').replaceAll(root, '<ws>').slice(0, 1500));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
process.exit(0);
