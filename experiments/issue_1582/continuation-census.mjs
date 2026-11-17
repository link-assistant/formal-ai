// Supplied exact measurement of the reviewed request-recognizer delta.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import {parseLedger,sourceFiles,audit} from '../../scripts/check-minimal-core-boundary.mjs';
const file='data/meta/core-boundary-ledger.lino';
let text=fs.readFileSync(file,'utf8');
const ledger=parseLedger(text), files=sourceFiles(process.cwd());
const active=ledger.entries.filter(e=>['promote','migrate'].includes(e.disposition));
const changed=active.filter(e=>e.baselineLines!==files.get(e.path));
assert.deepEqual(changed.map(e=>e.path),['rust/src/solver_handler_how.rs']);
for(const entry of changed){
 const start=text.indexOf('  source '+entry.path+'\n');
 const end=text.indexOf('\n  source ',start+3);
 const stop=end<0?text.length:end;
 const row=text.slice(start,stop).replace('baseline_lines '+entry.baselineLines,'baseline_lines '+files.get(entry.path));
 text=text.slice(0,start)+row+text.slice(stop);
}
const outside=active.filter(e=>e.disposition==='migrate');
const sum=entries=>entries.reduce((n,e)=>n+files.get(e.path),0);
for(const [key,value]of Object.entries({source_lines_max:sum(active),outside_core_lines_max:sum(outside)}))text=text.replace(new RegExp('(  '+key+' )\\d+'),'$1'+value);
text=text.replace('  boundary ','  note "Issue #1582 continuation measures the shared direct-request recognizer in solver_handler_how.rs; quoted permission text remains owned by its enclosing task. All dispositions are retained."\n  boundary ');
assert.deepEqual(audit(parseLedger(text),files),[]);
fs.writeFileSync(file,text);
const budget='data/meta/worker-line-budget/formal_ai_worker_capabilities_and_runtime_rules.lino';
const lines=fs.readFileSync('js/worker/formal_ai_worker_capabilities_and_runtime_rules.js','utf8').split('\n').length-1;
fs.writeFileSync(budget,fs.readFileSync(budget,'utf8').replace(/ceiling \d+/u,'ceiling '+lines).replace(/"\n$/u,' Request ownership is anchored to the start of a seeded direct request so quoted task data cannot dispatch permission guidance."\n'));
const inventory='data/benchmarks/fast-pull-request-javascript-tests.json', data=JSON.parse(fs.readFileSync(inventory,'utf8'));
const test='rust/tests/web/issue-1582-permissions.test.mjs';
if(!data.files.includes(test))data.files.splice(data.files.indexOf('rust/tests/web/issue-1582-defaults.test.mjs')+1,0,test);
fs.writeFileSync(inventory,JSON.stringify(data,null,2)+'\n');
console.log(JSON.stringify({changed:changed.map(e=>({path:e.path,before:e.baselineLines,after:files.get(e.path)})),workerLines:lines}));
