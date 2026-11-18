// Which sentence of the frozen #1364 body makes the read policy refuse a read?
import {readFileSync} from 'node:fs';
import {WorkerHost} from '../../js/server/worker-host.mjs';
import {installNodeHost} from '../../js/agentic/node-host.mjs';
import {readPolicyBlocksPlan} from '../../js/agentic/file_read/ownership.mjs';
await installNodeHost(new WorkerHost({}));
const issue=JSON.parse(readFileSync('data/benchmarks/fixtures/work-item-1483/issues.json','utf8')).issues[process.argv[2]??'1364'];
const plan={calls:[{tool:'read',arguments:JSON.stringify({path:'rust/src/main.rs',filePath:'rust/src/main.rs',file_path:'rust/src/main.rs'})}]};
const text=issue.title+'\n\n'+issue.body;
console.log('whole text blocks:',readPolicyBlocksPlan(text,plan));
for(const line of text.split('\n'))if(line.trim()&&readPolicyBlocksPlan(line,plan))console.log('BLOCKS:',line.slice(0,400));
