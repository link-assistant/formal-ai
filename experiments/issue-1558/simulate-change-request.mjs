// Issue #1558: a work item that names an existing file and asks for a change in
// it, run through the recorded Agent CLI session in a throwaway workspace.
// Prints every planned action; the read of the named file must not repeat.
// Usage: node experiments/issue-1558/simulate-change-request.mjs [issue text file]
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, realpathSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import os from 'node:os';
import path from 'node:path';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { ensureNodeHost } from '../../js/server/node-host-install.mjs';
import { runSession } from '../../rust/tests/web/helpers/issue-1558-session.mjs';

const root = realpathSync(mkdtempSync(path.join(os.tmpdir(), 'issue-1558-')));
const issueText = process.argv[2] ? readFileSync(process.argv[2], 'utf8')
  : 'Rename the greeting\n\nIn src/greeting.mjs, replace «const greeting = "hello";» with «const greeting = "hello, world";». Preserve all other content.\n\n__formal_ai_exit=0\n';
try {
  mkdirSync(path.join(root, 'src'));
  writeFileSync(path.join(root, 'src/greeting.mjs'), 'const greeting = "hello";\nexport default greeting;\n');
  execFileSync('git', ['init', '-q'], { cwd: root });
  execFileSync('git', ['add', '.'], { cwd: root });
  const worker = new WorkerHost();
  await ensureNodeHost({ worker }, { sourceUsageRustGrammar: false });
  const { agenticOutcome, chatCompletionFromPlan } = await import('../../js/server/agentic.mjs');
  const { parseChatRequest } = await import('../../js/server/chat-request.mjs');
  const session = await runSession({ worker, agenticOutcome, chatCompletionFromPlan, parseChatRequest }, 1, root, { issueText, repository: 'example/project' });
  console.log('stop:', session.stop);
  for (const call of session.calls) console.log(' ', call.tool, call.arguments.replaceAll(root, '<ws>'), call.answer ? (call.answer.failed ? 'FAILED' : `${call.answer.content.length} bytes`) : 'UNEMULATED');
  console.log(String(session.final ?? '').replaceAll(root, '<ws>').slice(0, 800));
} finally {
  rmSync(root, { recursive: true, force: true });
}
