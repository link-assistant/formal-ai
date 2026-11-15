// Issue #1558 (comment on hive-mind PR 4040): a work item of another repository
// names an existing file of the checkout (`src/solve.results.lib.mjs`) and asks
// for a change in it. Runs the recorded session in a throwaway git workspace
// holding that file and prints every planned action.
// Usage: node experiments/issue-1558/simulate-hive-mind-4033.mjs
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, realpathSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import os from 'node:os';
import path from 'node:path';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../js/server/node-host-install.mjs';
import { runSession } from '../../rust/tests/web/helpers/issue-1558-session.mjs';

const root = realpathSync(mkdtempSync(path.join(os.tmpdir(), 'issue-1558-')));
try {
  for (const file of ['src/solve.results.lib.mjs', 'src/pull-request-changes.lib.mjs']) {
    mkdirSync(path.join(root, path.dirname(file)), { recursive: true });
    writeFileSync(path.join(root, file), `// ${file}\nexport const placeholder = true;\n`);
  }
  execFileSync('git', ['init', '-q'], { cwd: root });
  execFileSync('git', ['add', '.'], { cwd: root });
  const worker = new WorkerHost();
  await ensureNodeHost({ worker }, { sourceUsageRustGrammar: false });
  const { agenticOutcome, chatCompletionFromPlan } = await import('../../js/server/agentic.mjs');
  const { parseChatRequest } = await import('../../js/server/chat-request.mjs');
  const issueText = readFileSync('data/benchmarks/fixtures/issue-1558/hive-mind-4033.txt', 'utf8');
  const session = await runSession({ worker, agenticOutcome, chatCompletionFromPlan, parseChatRequest }, 4033, root, { issueText, repository: 'link-assistant/hive-mind' });
  console.log('stop:', session.stop);
  for (const call of session.calls) console.log(' ', call.tool, call.arguments.replaceAll(root, '<ws>'), call.answer ? (call.answer.failed ? 'FAILED' : `${call.answer.content.length} bytes`) : 'UNEMULATED');
  console.log(String(session.final ?? '').replaceAll(root, '<ws>').slice(0, 1500));
} finally {
  rmSync(root, { recursive: true, force: true });
}
