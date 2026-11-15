// Probes the host's observed-source-usage operation on one file, to see why
// the 1455 documentation candidate omits the source usage paragraph.
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../js/server/worker-host.mjs';
import { installNodeHost } from '../../js/agentic/node-host.mjs';
import { host } from '../../js/agentic/host.mjs';
await installNodeHost(new WorkerHost({}));
const path = process.argv[2] ?? 'js/web-search-component-entry.js';
const source = readFileSync(path, 'utf8');
try {
  const usage = host().observedSourceUsage({ owner: path, source, language: path.endsWith('.rs') ? 'Rust' : 'JavaScript' });
  console.log(JSON.stringify(usage, null, 1).slice(0, 3000));
} catch (error) {
  console.log('threw', error);
}
