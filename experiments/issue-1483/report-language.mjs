// Report language of each fixture objective: whole-text `detect` (what the
// work-item report used) against prose-proportion `detectProse` (issue #1557).
//   node experiments/issue-1483/report-language.mjs
import {readFileSync} from 'node:fs';
import {WorkerHost} from '../../js/server/worker-host.mjs';
import {installNodeHost} from '../../js/agentic/node-host.mjs';
import {detect,detectProse,detectWorkItem} from '../../js/agentic/crate/language.mjs';
await installNodeHost(new WorkerHost({}));
const root=new URL('../../',import.meta.url).pathname;
const {issues}=JSON.parse(readFileSync(root+'data/benchmarks/fixtures/work-item-1483/issues.json','utf8'));
for(const [number,issue] of Object.entries(issues)){
  const task=`${issue.title}\n\n${issue.body}`;
  console.log(`#${number} detect=${detect(task)} detectProse=${detectProse(task)} detectWorkItem=${detectWorkItem(task)}`);
}
