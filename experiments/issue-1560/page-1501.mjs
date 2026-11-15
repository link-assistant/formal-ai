// Extract the issue title and body from a fetched GitHub issue page (issue #1560).
// Usage: node experiments/issue-1560/page-1501.mjs <page.txt>
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { workItemPageText } from '../../js/agentic/work_item_page.mjs';
await installNodeHost(new WorkerHost({}));
const out = workItemPageText(readFileSync(process.argv[2], 'utf8'));
console.log(out === null ? '<rejected>' : out);
