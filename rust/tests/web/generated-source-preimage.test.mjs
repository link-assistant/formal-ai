import assert from 'node:assert/strict';
import {before,test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
let planGeneratedSourceStep,rustSourceForTask;
before(async()=>{await installNodeHost(new WorkerHost());({planGeneratedSourceStep,rustSourceForTask}=await import('../../../js/agentic/code_task.mjs'));});
const task='Create rust/src/held_out.rs containing a public Rust function named held_out_value returning 29.';
const messages=()=>[{role:'user',content:task}];
const observed=(raw)=>[...messages(),{role:'assistant',tool_calls:[{id:'preimage',function:{name:'read_file',arguments:JSON.stringify({path:'rust/src/held_out.rs'})}}]},{role:'tool',tool_call_id:'preimage',name:'read_file',content:raw}];
test('generated source observes its exact target before any write',()=>{
 const plan=planGeneratedSourceStep(task,messages(),['read_file','write_file']);assert.equal(plan.calls[0].tool,'read_file');assert.equal(JSON.parse(plan.calls[0].arguments).path,'rust/src/held_out.rs');
});
test('existing and unreadable sources remain unchanged without explicit replacement',()=>{
 for(const raw of ['pub fn existing() {}\n','Error: permission denied','{"error":"cannot read"}']){const plan=planGeneratedSourceStep(task,observed(raw),['read_file','write_file']);assert.equal(plan.kind,'final');assert(!plan.answer.includes('marked the change complete'));}
});
test('actual absence, empty bytes and exact desired preimage authorize source creation',()=>{
 for(const raw of ['Error: ENOENT: no such file or directory','',rustSourceForTask(task).content]){const plan=planGeneratedSourceStep(task,observed(raw),['read_file','write_file']);assert.equal(plan.calls[0].tool,'write_file');assert.equal(JSON.parse(plan.calls[0].arguments).content,rustSourceForTask(task).content);}
});
test('unobserved read capability produces a typed gap without source mutation',()=>{
 const plan=planGeneratedSourceStep(task,messages(),['write_file']);assert.equal(plan.kind,'final');assert.match(plan.answer,/not observed/);
});
test('numbers inside identifiers never become requested Rust values',()=>{
 for(const id of ['R1188','abc923xyz','snake_829','123suffix','x12_y']){assert.equal(rustSourceForTask('Create rust/src/value.rs with a public Rust function named numeric_guard returning the value described by '+id+'.'),null);}
 assert.match(rustSourceForTask('Create rust/src/value.rs with a public Rust function named numeric_guard returning 41 for R1188.').content,/41/);
});
