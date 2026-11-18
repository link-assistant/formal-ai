// Probe: the typed clause checks and meaning of a task body (default: #1450).
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());
const { specifyTask } = await import('../../js/agentic/task_acceptance.mjs');
const files = process.argv.slice(2);
for (const path of files.length ? files : [new URL('./issue-1450-body.md', import.meta.url)]) {
  const specification = specifyTask(readFileSync(path, 'utf8'));
  console.log('==', String(path), specification.language, specification.task_identity);
  for (const clause of specification.clauses) console.log(clause.span.join(':'), clause.required ? '' : '(optional)', JSON.stringify(clause.text.slice(0, 50)), JSON.stringify(clause.checks));
  console.log('meaning', specification.meaning);
}
