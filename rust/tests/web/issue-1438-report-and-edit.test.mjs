import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {planCodeArtifactStep} from '../../../js/agentic/code_artifact.mjs';
import {planReportFlow} from '../../../js/agentic/report_issue.mjs';
import {completeOwnedSourceReadFrame} from '../../../js/agentic/tool_result.mjs';
await installNodeHost(new WorkerHost({WebAssembly:{instantiate(){throw Error('LocalWasmExecutionForbidden');},compile(){throw Error('LocalWasmExecutionForbidden');}}}));
const source='// kept 雪\r\nfn main() {\r\n    println!("Penger!");\r\n}\r\n';
const prompt='Change the output to "I told you!"';
const call=(id,name,args)=>({id,type:'function',function:{name,arguments:JSON.stringify(args)}});
const assistant=toolCall=>({role:'assistant',content:'',tool_calls:[toolCall]});
const reply=(toolCall,content,failed=false)=>({role:'tool',name:toolCall.function.name,tool_call_id:toolCall.id,content,is_error:failed});
const tools=['Read','Edit','Write'];
const history=()=>{const write=call('created','Write',{file_path:'main.rs',content:source});return [
 {role:'user',content:'Write Rust program that will print Penger! msg in the console'},assistant(write),reply(write,'File created.'),
 {role:'assistant',content:'Created main.rs.'},{role:'user',content:prompt}];};
const framed=text=>text.split('\n').map((row,index)=>String(index+1).padStart(6)+'\t'+row).join('\n');
function append(messages,planned,content,failed=false){const c=call('step-'+messages.length,planned.tool,JSON.parse(planned.arguments));messages.push(assistant(c),reply(c,content,failed));}
test('captured contextual replacement uses a literal edit and preserves physical source bytes',context=>{
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-numbered-read-'));context.after(()=>fs.rmSync(directory,{recursive:true,force:true}));
 const file=path.join(directory,'main.rs');fs.writeFileSync(file,source);
 const messages=history();let plan=planCodeArtifactStep(prompt,messages,tools);
 assert.equal(plan.calls[0].tool,'Read');append(messages,plan.calls[0],framed(source));
 plan=planCodeArtifactStep(prompt,messages,tools);assert.equal(plan.calls[0].tool,'Edit');
 const args=JSON.parse(plan.calls[0].arguments),old=args.old_string??args.oldString,next=args.new_string??args.newString;
 assert.equal(old,'Penger!');assert.equal(next,'I told you!');
 fs.writeFileSync(file,fs.readFileSync(file,'utf8').replace(old,()=>next));append(messages,plan.calls[0],'Edit applied.');
 plan=planCodeArtifactStep(prompt,messages,tools);assert.equal(plan.calls[0].tool,'Read');append(messages,plan.calls[0],framed(fs.readFileSync(file,'utf8')));
 plan=planCodeArtifactStep(prompt,messages,tools);assert.equal(plan.kind,'final');assert.match(plan.answer,/observed the requested result/u);
 assert.deepEqual(fs.readFileSync(file),Buffer.from(source.replace('Penger!','I told you!')));
});
test('an opaque numbered read cannot authorize a whole-file Write when Edit is unavailable',()=>{
 const messages=history();const read=planCodeArtifactStep(prompt,messages,['Read','Write']).calls[0];append(messages,read,framed(source));
 const plan=planCodeArtifactStep(prompt,messages,['Read','Write']);assert.equal(plan.kind,'final');assert.match(plan.answer,/no whole-file replacement/u);
});
test('failed edits and failed verification reads stop without another mutation or success claim',()=>{
 for(const stage of ['edit','verification']){
  const messages=history();let plan=planCodeArtifactStep(prompt,messages,tools);append(messages,plan.calls[0],framed(source));
  plan=planCodeArtifactStep(prompt,messages,tools);append(messages,plan.calls[0],stage==='edit'?'Error: preimage did not match':'Edit applied.',stage==='edit');
  plan=planCodeArtifactStep(prompt,messages,tools);
  if(stage==='verification'){assert.equal(plan.calls[0].tool,'Read');append(messages,plan.calls[0],'Error: read denied',true);plan=planCodeArtifactStep(prompt,messages,tools);}
  assert.equal(plan.kind,'final');assert.match(plan.answer,/Could not independently verify/u);
 }
});
test('a different file read does not supply contextual source authority',()=>{
 const messages=history(),read=call('foreign','Read',{file_path:'other.rs'});messages.push(assistant(read),reply(read,framed(source)));
 const plan=planCodeArtifactStep(prompt,messages,tools);assert.equal(plan.calls[0].tool,'Read');assert.equal(JSON.parse(plan.calls[0].arguments).file_path,'main.rs');
});
test('provider-owned complete reads retain general substitution semantics without an Edit tool',()=>{
 for(const [file,before,task,expected] of [
  ['notes.custom-format','alpha beta alpha',"Transform 'alpha' into 'omega'.",'omega beta alpha'],
  ['main.rs','fn main() {\n    println!("Hello, world!");\n}\n','(("Hello, world!")) ((terminal: "Hello 2"))','fn main() {\n    println!("Hello 2");\n}\n'],
  ['notes.txt','body','() ((terminal: "prefix "))','prefix body']
 ]){
  const created=call('created','Write',{file_path:file,content:before});
  const messages=[{role:'user',content:'Create the active workspace artifact.'},assistant(created),reply(created,'Created.'),{role:'assistant',content:'Created the artifact.'},{role:'user',content:task}];
  let plan=planCodeArtifactStep(task,messages,['Read','Write']);assert.equal(plan.calls[0].tool,'Read');
  append(messages,plan.calls[0],completeOwnedSourceReadFrame(before));
  plan=planCodeArtifactStep(task,messages,['Read','Write']);assert.equal(plan.calls[0].tool,'Write');
  const written=JSON.parse(plan.calls[0].arguments);assert.equal(written.content,expected);assert.equal(written.file_path??written.filePath??written.path,file);
 }
});
const reportHistory=()=>[{role:'user',content:'The source file gained line numbers.'},{role:'user',content:'report'},
 {role:'user',content:'GitHub issue'},{role:'user',content:'Both logs'}];
test('reporting emits one explicit named submission, with the selected full context',()=>{
 const plan=planReportFlow(reportHistory(),['Bash']),command=JSON.parse(plan.calls[0].arguments).command;
 assert.match(command,/^formal-ai report create-issue /u);assert.match(command,/--source both/u);assert.match(command,/--separate-context-links/u);
 assert.match(command,/--repo 'link-assistant\/formal-ai'/u);assert(!command.includes('set -eu'));assert(!command.includes('mktemp'));assert(!command.includes('gh issue create'));
});
test('the captured host approval refusal terminates honestly and never retries submission',()=>{
 const messages=reportHistory(),plan=planReportFlow(messages,['Bash']);
 append(messages,plan.calls[0],'Error: Auto mode could not evaluate this action and is blocking it for safety. This is not a judgment that the action is unsafe.',true);
 const stopped=planReportFlow(messages,['Bash']);assert.equal(stopped.kind,'final');assert(!stopped.answer.includes('created an issue'));assert.match(stopped.answer,/Auto mode/u);
});
test('report success requires an actual returned issue URL',()=>{
 const messages=reportHistory(),plan=planReportFlow(messages,['Bash']);append(messages,plan.calls[0],'https://github.com/link-assistant/formal-ai/issues/42');
 assert.match(planReportFlow(messages,['Bash']).answer,/issues\/42/u);
});

test('a foreign, zero or nonnumeric issue URL never establishes report success',()=>{
 for(const url of ['https://github.com/other/repo/issues/42','https://github.com/link-assistant/formal-ai/issues/0','https://github.com/link-assistant/formal-ai/issues/not-a-number']){
  const messages=reportHistory(),plan=planReportFlow(messages,['Bash']);append(messages,plan.calls[0],url);const final=planReportFlow(messages,['Bash']);assert.equal(final.kind,'final');assert(final.answer.startsWith("I couldn't file the issue"));
 }
});
