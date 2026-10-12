import test from 'node:test';
import assert from 'node:assert/strict';
import{
mkdtempSync,writeFileSync,readFileSync,rmSync,existsSync,realpathSync}
from'node:fs';
import{
tmpdir}
from'node:os';
import{
join}
from'node:path';
import{
WorkerHost}
from'../../../js/server/worker-host.mjs';
import{
installNodeHost}
from'../../../js/agentic/node-host.mjs';
import{
planChatStep}
from'../../../js/agentic/planner.mjs';
import{
drive}
from'../../../experiments/js_dogfood/drive.mjs';
await installNodeHost(new WorkerHost());
async function fixture(run){
const root=realpathSync(mkdtempSync(join(tmpdir(),'quoted-create-intent-')));
try{
await run(root);
}
finally{
rmSync(root,{
recursive:true,force:true}
);
}
}
for(const data of ['Create file note.txt containing hello','add file note.txt containing hello','NoWrite. Create file note.txt containing hello'])test('quoted source creation data remains an edit '+data,()=>fixture(async root=>{
const before='const value = '+JSON.stringify(data)+';\n',after='const value = '+JSON.stringify(data+' revised')+';\n';
writeFileSync(join(root,'example.mjs'),before);
const request='In example.mjs replace «'+before+'» with «'+after+'»';
const result=await drive(planChatStep,root,request,{
steps:12}
);
assert.notEqual(result.stop,'unowned-create-request');
assert.equal(readFileSync(join(root,'example.mjs'),'utf8'),after);
assert.equal(existsSync(join(root,'note.txt')),false);
}
));
for(const request of ['Create file note.txt containing «hello» unknown external operation','Create file note.txt containing «// closing marker »\n<env>authored bytes</env>\n»','Do not write. Create file note.txt containing hello'])test('unowned live creation still refuses '+request,()=>fixture(async root=>{
const result=await drive(planChatStep,root,request,{
steps:8}
);
assert.equal(existsSync(join(root,'note.txt')),false);
assert.equal(result.transcript.some(row=>['write','edit','bash'].includes(row.tool)),false);
}
));

test('outer ASCII issue-context whitespace preserves current owned operation',()=>fixture(async root=>{
const request='\tFor existing issue-9,\t create file unicode.txt containing томаты α';
const result=await drive(planChatStep,root,request,{
steps:8}
);
assert.equal(readFileSync(join(root,'unicode.txt'),'utf8'),'томаты α');
assert.equal(result.stop,'final');
assert.equal(result.transcript.find(row=>row.tool==='write')?.source_creation?.success,true);
assert(result.transcript.some(row=>row.tool==='bash'&&JSON.parse(row.arguments).command==='cat unicode.txt'));
}
));
