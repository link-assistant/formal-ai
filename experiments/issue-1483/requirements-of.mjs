// Prints the requirements a task splits into. Usage: node requirements-of.mjs <fixture.json | text>
import {readFileSync,existsSync} from 'node:fs';
import {WorkerHost} from '../../js/server/worker-host.mjs';
import {installNodeHost} from '../../js/agentic/node-host.mjs';
import {workItemRequirements} from '../../js/agentic/work_item_requirements.mjs';
await installNodeHost(new WorkerHost({}));
const input=process.argv[2];
let task=input;
if(existsSync(input)){const f=JSON.parse(readFileSync(input,'utf8'));task=f.issue.title+'\n\n'+f.issue.body;}
for(const [text,span] of workItemRequirements(task))console.log(span,'-',text.replace(/\n/g,' ').slice(0,150));
