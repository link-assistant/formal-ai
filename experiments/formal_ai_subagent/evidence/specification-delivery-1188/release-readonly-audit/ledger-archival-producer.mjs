import {readFileSync,writeFileSync,readdirSync,statSync,mkdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {join,relative} from 'node:path';
const source='/private/tmp/spec-recurrence-1188';
const target=join(process.cwd(),'experiments/formal_ai_subagent/evidence/specification-delivery-1188/release-readonly-audit');
for(const name of readdirSync(source).filter(name=>name.startsWith('T2939-')||name==='T2939.log'))writeFileSync(join(target,'tasks',name),readFileSync(join(source,name)));
writeFileSync(join(target,'ledger-T2933-T2940.md'),readFileSync(join(source,'release-ledger-T2933-T2940.draft.md')));
const observed=[];
for(let number=2933;number<=2939;number++){const value=JSON.parse(readFileSync(join(source,'T'+number+'-observation.json')));observed.push(value);}
observed.push({taskId:'T2940',calls:['bash'],provenance:'this archival command invocation; full external transcript finalized after process return'});
const calls=observed.flatMap(row=>row.calls);
const physical={tasks:8,toolCalls:calls.length,bash:calls.filter(value=>value==='bash').length,write:calls.filter(value=>value==='write').length,edit:calls.filter(value=>value==='edit').length,explicitWriteTasks:['T2935 (scratch producer)','T2937 (failed exact report)','T2938 (metric correction)'],repositoryArchivalBashTasks:['T2939','T2940'],scope:'Physical driver tool calls, not tokens, monetary costs, files changed or production coverage. Observational producer Bash tasks also write selected scratch raw evidence.',observed};
writeFileSync(join(target,'physical-call-audit.json'),JSON.stringify(physical,null,2)+'\n');
writeFileSync(join(target,'ledger-archival-producer.mjs'),readFileSync(new URL(import.meta.url)));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');const files=[];
function walk(dir){for(const name of readdirSync(dir).sort()){const path=join(dir,name);if(statSync(path).isDirectory())walk(path);else if(name!=='artifact-manifest.json'){const bytes=readFileSync(path);files.push({path:relative(target,path),bytes:bytes.length,sha256:hash(bytes)});}}}
walk(target);writeFileSync(join(target,'artifact-manifest.json'),JSON.stringify({archivalTask:'T2940',sourceScope:'T2934 network-failed and T2936 real retrieved observation',originalLiteralFailure:'T2937 remains open',files},null,2)+'\n');
console.log(JSON.stringify({artifacts:files.length,tasks:physical.tasks,toolCalls:physical.toolCalls,bash:physical.bash,write:physical.write,originalLiteralFailure:'OPEN',productionSourceWrites:0}));
