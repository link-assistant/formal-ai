// Prints how Progress.scan classifies each recorded tool observation.
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { Progress } from '../../js/agentic/progress.mjs';
await installNodeHost(new WorkerHost({}));
const body = JSON.parse(readFileSync(process.argv[2], 'utf8'));
const progress = Progress.scan(body.messages);
console.log(JSON.stringify({
  attempts: progress.attempts.map(a => ({ capability: a.capability, succeeded: a.succeeded, detail: String(a.detail).slice(0, 160) })),
  workItemReads: progress.attempted_work_item_reads,
  failed: progress.failed_work_item_reads.map(([u, r]) => [u, String(r).slice(0, 200)]),
  pages: progress.fetched_pages.map(([u, t]) => [u, t.length]),
}, null, 1));
