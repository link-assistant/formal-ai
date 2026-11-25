// Lists PR-triggered workflows with their jobs, needs and if conditions.
import fs from 'node:fs';
import YAML from 'yaml';
import {lightweightWorkflows} from '../scripts/lib/documentation-paths.mjs';
const dir = '.github/workflows';
for (const f of fs.readdirSync(dir).filter((n) => n.endsWith('.yml')).sort()) {
  const d = YAML.parse(fs.readFileSync(`${dir}/${f}`, 'utf8'));
  const on = d.on;
  if (!on || typeof on !== 'object' || !('pull_request' in on) || lightweightWorkflows.has(f)) continue;
  console.log(`== ${f} types=${JSON.stringify(on.pull_request?.types ?? null)} perms=${JSON.stringify(d.permissions)}`);
  for (const [name, job] of Object.entries(d.jobs)) {
    console.log(`  ${name} needs=${JSON.stringify(job.needs ?? null)} uses=${job.uses ?? ''} if=${JSON.stringify(job.if ?? null).slice(0, 140)}`);
  }
}
