import test, {before} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {host,installHost} from '../../../js/agentic/host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
before(async()=>{await installNodeHost(new WorkerHost());});
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {sourceCstTransformRequest,planSourceCstTransformStep} from '../../../js/agentic/source_cst_transform.mjs';
import {executeSourceCstTransform} from '../../../scripts/source-cst-transform-request.mjs';
const digest=value=>createHash('sha256').update(value).digest('hex');
const request='Rename parameter count of function combine to quantity in example.mjs';
function fixture(source='// count is supplied\nfunction combine(count) { return count + 1; }\n'){
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-qualified-'));
 execFileSync('git',['init','-q'],{cwd:root});fs.writeFileSync(path.join(root,'example.mjs'),source);execFileSync('git',['add','example.mjs'],{cwd:root});
 return {root,source,expected:digest(source)};
}
test('natural declaration-qualified request is structured while ambiguous/path escape tasks are not owned',()=>{
 assert.deepEqual(sourceCstTransformRequest(request),{owner:'example.mjs',declaration:'combine',binding:'count',target:'quantity'});
 for(const task of ['rename count everywhere',request.replace('example.mjs','../example.mjs'),request+' and delete other files'])assert.equal(sourceCstTransformRequest(task),null);
});
test('planner observes source before command and never includes source bytes in command',()=>{
 const first=planSourceCstTransformStep(request,[],['read_file','run_shell_command'],()=>{throw Error('premature command');});
 assert.equal(JSON.parse(first.calls[0].arguments).path,'example.mjs');
 const source='function combine(count) { return count; }\n';
 const messages=[{role:'assistant',tool_calls:[{id:'read-bound',function:{name:'read_file',arguments:'{"path":"example.mjs"}'}}]},{role:'tool',tool_call_id:'read-bound',content:source}];
 const next=planSourceCstTransformStep(request,messages,['read_file','run_shell_command'],(_messages,_tools,command)=>command);
 assert.ok(next.includes(digest(source)));assert.ok(!next.includes(source));assert.ok(next.includes('--request'));
});
test('actual tracked owner changes through observed preimage and persisted full CST, then exact inverse restores it',async()=>{
 const item=fixture();let evidence,restoration;
 try{
 const receipt=await executeSourceCstTransform({...item,request});evidence=receipt.evidenceDirectory;
 assert.equal(fs.readFileSync(path.join(item.root,'example.mjs'),'utf8'),'// count is supplied\nfunction combine(quantity) { return quantity + 1; }\n');
 assert.equal(receipt.physicalWriteVerified,true);assert.equal(receipt.verification.compilerValidity,'not-checked');
 assert.equal(receipt.verification.commentNeeds.length,1);assert.equal(receipt.authorshipCredit,0);
 for(const name of ['before.lino','after.lino','rule.lino','inverse.lino','intent.json','receipt.json'])assert.ok(fs.statSync(path.join(evidence,name)).size>0);
 const result=await executeSourceCstTransform({root:item.root,expected:receipt.afterSha256,inverseDirectory:evidence});restoration=result.evidenceDirectory;
 assert.equal(fs.readFileSync(path.join(item.root,'example.mjs'),'utf8'),item.source);
 }finally{for(const directory of [item.root,evidence,restoration])if(directory)fs.rmSync(directory,{recursive:true,force:true});}
});
test('stale physical source, generated ownership, capture and untracked owners refuse without mutation',async()=>{
 for(const kind of ['stale','generated','capture','untracked','hardlink','symlink']){
 const source=kind==='generated'?'// @generated\nfunction combine(count){return count;}':kind==='capture'?'function combine(count){const quantity=1;return count+quantity;}':undefined;
 const item=fixture(source);
 try{
 if(kind==='hardlink')fs.linkSync(path.join(item.root,'example.mjs'),path.join(item.root,'other.mjs'));
 if(kind==='symlink'){fs.renameSync(path.join(item.root,'example.mjs'),path.join(item.root,'other.mjs'));fs.symlinkSync('other.mjs',path.join(item.root,'example.mjs'));}
 if(kind==='stale')fs.appendFileSync(path.join(item.root,'example.mjs'),'// drift');
 if(kind==='untracked')execFileSync('git',['rm','--cached','-q','example.mjs'],{cwd:item.root});
 const before=fs.readFileSync(path.join(item.root,'example.mjs'),'utf8');
 await assert.rejects(executeSourceCstTransform({...item,request}));assert.equal(fs.readFileSync(path.join(item.root,'example.mjs'),'utf8'),before);
 }finally{fs.rmSync(item.root,{recursive:true,force:true});}
 }
});

test('wrong-owner and failed reads cannot provide the requested mutation preimage',()=>{
 const source='function combine(count){return count;}';
 for(const variant of ['wrong-owner','failed','conflicting-alias']){
 const argumentsValue=variant==='wrong-owner'?{path:'other.mjs'}:variant==='conflicting-alias'?{path:'example.mjs',filePath:'other.mjs'}:{path:'example.mjs'};
 const messages=[{role:'assistant',tool_calls:[{id:'read-other',function:{name:'read_file',arguments:JSON.stringify(argumentsValue)}}]},{role:'tool',tool_call_id:'read-other',content:source,is_error:variant==='failed'}];
 const result=planSourceCstTransformStep(request,messages,['read_file','run_shell_command'],()=>{throw Error('unbound source executed');});
 assert.equal(result.kind,'final');assert.match(result.answer,/Read observation/u);
 }
});
test('browser host without an installed executable capability explicitly refuses',()=>{
 const trusted=host();installHost({...trusted,sourceCstTransformCommand:undefined});
 try{const result=planSourceCstTransformStep(request,[],['read_file','run_shell_command'],()=>{throw Error('missing host executed');});assert.equal(result.kind,'final');assert.match(result.answer,/installed host/u);}
 finally{installHost(trusted);}
});
test('short writes complete every UTF8 byte before source replacement',async()=>{
 const item=fixture('function combine(count) { return "雪" + count; }\n');let evidence,calls=0;
 try{
 const receipt=await executeSourceCstTransform({...item,request,writeChunk(descriptor,bytes,offset,length,position){calls+=1;return fs.writeSync(descriptor,bytes,offset,Math.min(3,length),position);}});evidence=receipt.evidenceDirectory;
 assert.ok(calls>10);assert.equal(fs.readFileSync(path.join(item.root,'example.mjs'),'utf8'),'function combine(quantity) { return "雪" + quantity; }\n');
 }finally{for(const directory of [item.root,evidence])if(directory)fs.rmSync(directory,{recursive:true,force:true});}
});
test('zero invalid and failing write progress leave the original intact and record failure custody',async()=>{
 for(const variant of ['zero','negative','fraction','overflow','nan','throw-after-partial','swap-owner']){
 const item=fixture();let evidence,calls=0;
 try{
 await assert.rejects(executeSourceCstTransform({...item,request,writeChunk(descriptor,bytes,offset,length,position){
  calls+=1;
  if(variant==='swap-owner'&&calls===1){fs.renameSync(path.join(item.root,'example.mjs'),path.join(item.root,'moved.mjs'));fs.writeFileSync(path.join(item.root,'example.mjs'),item.source);}
  if(variant==='throw-after-partial'){if(calls===1)return fs.writeSync(descriptor,bytes,offset,2,position);throw Error('injected write failure');}
  if(variant==='swap-owner')return fs.writeSync(descriptor,bytes,offset,length,position);
  return variant==='zero'?0:variant==='negative'?-1:variant==='fraction'?0.5:variant==='overflow'?length+1:NaN;
 }}),error=>{evidence=error.evidenceDirectory;assert.ok(evidence);return true;});
 assert.equal(fs.readFileSync(path.join(item.root,'example.mjs'),'utf8'),item.source);
 assert.equal(JSON.parse(fs.readFileSync(path.join(evidence,'failure.json'),'utf8')).sourceUnchanged,true);
 assert.ok(!fs.readdirSync(item.root).some(name=>name.includes('.formal-ai-')));
 }finally{for(const directory of [item.root,evidence])if(directory)fs.rmSync(directory,{recursive:true,force:true});}
 }
});
test('installed planner command transforms a human TypeScript owner in another repository and observes its physical receipt',async()=>{
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-other-repository-'));let evidence;
 const source='// Human authored TypeScript\nexport function measure(amount: number): number { return amount * 2; }\n';
 const task='Rename parameter amount of function measure to total in hand-written.ts';
 try{
 execFileSync('git',['init','-q'],{cwd:root});fs.writeFileSync(path.join(root,'hand-written.ts'),source);execFileSync('git',['add','hand-written.ts'],{cwd:root});
 const messages=[{role:'user',content:task}],tools=['read_file','run_shell_command'];
 const read=await planChatStep(messages,tools);assert.equal(read.calls[0].tool,'read_file');
 messages.push({role:'assistant',tool_calls:[{id:'other-repo-read',function:{name:read.calls[0].tool,arguments:read.calls[0].arguments}}]},{role:'tool',tool_call_id:'other-repo-read',content:fs.readFileSync(path.join(root,'hand-written.ts'),'utf8')});
 const run=await planChatStep(messages,tools);assert.equal(run.calls[0].tool,'run_shell_command');
 const command=JSON.parse(run.calls[0].arguments).command;assert.ok(command.includes(process.execPath));assert.ok(!command.includes(source));
 const output=execFileSync('/bin/sh',['-c',command],{cwd:root,encoding:'utf8',maxBuffer:1024*1024,timeout:180000});
 const receipt=JSON.parse(output);evidence=receipt.evidenceDirectory;
 assert.equal(receipt.root,fs.realpathSync(root));assert.equal(receipt.owner,'hand-written.ts');assert.equal(receipt.physicalWriteVerified,true);
 assert.equal(fs.readFileSync(path.join(root,'hand-written.ts'),'utf8'),source.replaceAll('amount','total'));
 messages.push({role:'assistant',tool_calls:[{id:'other-repo-write',function:{name:run.calls[0].tool,arguments:run.calls[0].arguments}}]},{role:'tool',tool_call_id:'other-repo-write',content:output});
 assert.equal((await planChatStep(messages,tools)).kind,'final');
 }finally{for(const directory of [root,evidence])if(directory)fs.rmSync(directory,{recursive:true,force:true});}
});
