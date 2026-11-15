// Captured #1501 requests, actual tiny-workspace effects, and held-out controls.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import vm from 'node:vm';
import {drive} from '../../../experiments/js_dogfood/drive.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {fileCommandContract} from '../../../js/agentic/file_command_contract.mjs';
import {quoteShellWord} from '../../../js/agentic/portable/string_contracts.mjs';
import {normalizedPayload,reportedExitCode} from '../../../js/agentic/tool_result.mjs';
const denied=()=>{throw Error('LocalProductWasmForbidden');};
await installNodeHost(new WorkerHost({WebAssembly:{...WebAssembly,compile:denied,instantiate:denied,Module:denied,Instance:denied}}),{sourceUsageRustGrammar:false});
const originals=[
 ['print list of files in current directory','listing',['ls']],
 ['print list of files in corrent directory','listing',['ls']],
 ['delete all files','unsupported',null],
 ['search for TODO in all files','search',['grep','-rFn','--','TODO','.']],
 ['how many files are in the current directory','count',null],
 ['show the first 5 lines of README.md','head',['head','-n','5','--','README.md']],
 ['ls; rm -rf ~','unsupported',null]
];
for(const [prompt,operation,argumentsList] of originals)test('unchanged reported request: '+prompt,async()=>{
 const contract=fileCommandContract(prompt);assert.equal(contract.operation,operation);
 const plan=await planChatStep([{role:'user',content:prompt}],['bash']);
 if(operation==='unsupported'){
  assert.equal(plan.kind,'final');const result=JSON.parse(plan.answer);
  assert.equal(result.operation,'unsupported');assert.equal(result.complete,false);assert.equal(result.verified,false);return;
 }
 assert.equal(plan.kind,'tool_calls');assert.equal(plan.calls.length,1);
 const command=JSON.parse(plan.calls[0].arguments).command;
 if(argumentsList)assert.deepEqual(contract.arguments,argumentsList);
 assert.equal(command,[contract.arguments[0],...contract.arguments.slice(1).map(quoteShellWord)].join(' '));
 if(operation==='count'){assert.equal(contract.arguments[0],'node');assert.equal(contract.arguments[1],'-e');assert.equal(contract.arguments.at(-1),'.');}
});
for(const prompt of ['ls && touch marker','ls | cat','ls\nrm marker','ls $(touch marker)','ls "$(touch marker)"','`ls; rm marker`'])test('bare shell composition is refused: '+JSON.stringify(prompt),async()=>{
 const plan=await planChatStep([{role:'user',content:prompt}],['bash']);
 assert.equal(plan.kind,'final');assert.equal(JSON.parse(plan.answer).reason,'CompoundShellInputUnsupported');
});
test('a completed unrelated ls cannot complete a deletion request',async()=>{
 const plan=await planChatStep([{role:'user',content:'delete all files'},
  {role:'assistant',content:'',tool_calls:[{id:'prior',type:'function',function:{name:'bash',arguments:'{"command":"ls"}'}}]},
  {role:'tool',tool_call_id:'prior',content:'<untrusted_context>\nCommand: ls\nOutput: a.txt\nExit Code: 0\n</untrusted_context>'}],['bash']);
 assert.equal(plan.kind,'final');assert.equal(JSON.parse(plan.answer).reason,'UnsupportedBulkDeletion');
});
test('held-out numbers paths and literal queries bind to typed arguments',()=>{
 assert.deepEqual(fileCommandContract('Display the first 12 lines from "notes file.txt".').arguments,['head','-n','12','--','notes file.txt']);
 assert.deepEqual(fileCommandContract('Find the-token in src').arguments,['grep','-rFn','--','the-token','src']);
 assert.equal(fileCommandContract('show first 0 lines of README.md').reason,'FileCommandLineLimit');
 assert.equal(fileCommandContract('show first 1001 lines of README.md').reason,'FileCommandLineLimit');
 assert.equal(fileCommandContract('show first 3 lines of ../private.txt').reason,'FileCommandPathUnsupported');
 assert.equal(fileCommandContract('show first 3 lines of -flags').reason,'FileCommandPathUnsupported');
 assert.equal(fileCommandContract('print list of files in "corrent directory"'),null);
 assert.equal(fileCommandContract('Document dependencies in docs/guide.md using package.json.'),null);
});
test('actual read-only executions return listings search hits line limits and an observed count',async context=>{
 const root=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-file-command-')));
 context.after(()=>fs.rmSync(root,{recursive:true,force:true}));
 const files=new Map([['README.md',Array.from({length:7},(_,index)=>'line '+(index+1)).join('\n')+'\n'],
  ['notes.txt','TODO retained\nneedle\'; touch injected.txt; echo \'\n'],['.hidden','hidden\n'],['src/child.txt','TODO nested\n']]);
 for(const [owner,source] of files){fs.mkdirSync(path.dirname(path.join(root,owner)),{recursive:true});fs.writeFileSync(path.join(root,owner),source);}
 const names=fs.readdirSync(root).sort();
 for(const prompt of originals.filter(row=>row[1]!=='unsupported').map(row=>row[0])){
  const contract=fileCommandContract(prompt);
  assert.equal(contract.command,[contract.arguments[0],...contract.arguments.slice(1).map(quoteShellWord)].join(' '));
  const result=await drive(planChatStep,root,prompt,{steps:4,tools:['bash'],allowedCommands:[contract.command]});
  assert.equal(result.stop,'final');assert.equal(result.transcript.length,1);
  const receipt=result.transcript[0].result;assert.equal(reportedExitCode(receipt),0);
  const output=normalizedPayload(receipt);assert.equal(typeof output,'string');
  if(contract.operation==='listing'){assert.ok(output.includes('README.md'));assert.ok(output.includes('notes.txt'));}
  if(contract.operation==='search'){assert.ok(output.includes('notes.txt:1:TODO retained'));assert.ok(output.includes('src/child.txt:1:TODO nested'));}
  if(contract.operation==='head')assert.equal(output.trimEnd(),Array.from({length:5},(_,index)=>'line '+(index+1)).join('\n'));
  if(contract.operation==='count'){assert.equal(output.trim(),'3');assert.equal(result.answer.trim(),'3');}
 }
 const prompt="search for needle'; touch injected.txt; echo ' in all files",contract=fileCommandContract(prompt);
 assert.equal(contract.command,[contract.arguments[0],...contract.arguments.slice(1).map(quoteShellWord)].join(' '));
 const literal=await drive(planChatStep,root,prompt,{steps:4,tools:['bash'],allowedCommands:[contract.command]});
 assert.equal(reportedExitCode(literal.transcript[0].result),0);
 assert.ok(normalizedPayload(literal.transcript[0].result).includes('notes.txt:2:'));
 assert.equal(fs.existsSync(path.join(root,'injected.txt')),false);
 assert.deepEqual(fs.readdirSync(root).sort(),names);
 for(const [owner,source] of files)assert.equal(fs.readFileSync(path.join(root,owner),'utf8'),source);
});
test('file counting stops at its explicit directory budget and closes the iterator',()=>{
 const program=fs.readFileSync(new URL('../../../data/meta/file-count-program.cjs',import.meta.url),'utf8');
 let visited=0,closed=0,printed=0;
 assert.throws(()=>vm.runInNewContext(program,{process:{argv:['node','.']},console:{log(){printed++;}},
  require(name){assert.equal(name,'node:fs');return {opendirSync(){return {readSync(){if(++visited>5000)throw Error('OracleStop');return {isFile:()=>true};},closeSync(){closed++;}};}};}},{timeout:1000}),/FileCountLimit/u);
 assert.equal(visited,4097);assert.equal(closed,1);assert.equal(printed,0);
});

test('shared native operation vectors cover original requests and held-out operands',()=>{
 const fixture=JSON.parse(fs.readFileSync(new URL('../fixtures/file-command-cases.json',import.meta.url),'utf8'));
 for(const entry of fixture.cases){const contract=fileCommandContract(entry.task);
  if(entry.unclaimed){assert.equal(contract,null,entry.task);continue;}
  assert.equal(contract.operation,entry.operation,entry.task);
  if(entry.reason)assert.equal(contract.reason,entry.reason);
  if(entry.arguments)assert.deepEqual(contract.arguments,entry.arguments);
  if(entry.countPath){assert.equal(contract.arguments[0],'node');assert.equal(contract.arguments.at(-1),entry.countPath);}
 }
});

test('captured native-tool precedence survives shell-only command support',async()=>{
 const read=await planChatStep([{role:'user',content:'Show the first 2 lines of n.txt.'}],['read','bash']);
 assert.equal(read.calls[0].tool,'read');
 const search=await planChatStep([{role:'user',content:"Grep for 'return a' in lib/"}],['bash']);
 assert.equal(JSON.parse(search.calls[0].arguments).command,"grep -rnHF --exclude-dir=.git -- 'return a' 'lib'");
});

test('captured desktop folder discovery retains filename search',async()=>{
 const plan=await planChatStep([{role:'user',content:'Find sample-target folder in desktop'}],['bash']);
 assert.equal(plan.kind,'tool_calls');assert.ok(JSON.parse(plan.calls[0].arguments).command.startsWith('find '));
});

test('relocated native shared-vector modules resolve every embedded input',()=>{
 for(const name of ['file_command_contract','source_import_binding']){
  const owner=new URL('../unit/agentic-coding/'+name+'.rs',import.meta.url);
  const source=fs.readFileSync(owner,'utf8');
  const inputs=[...source.matchAll(/\binclude_str!\s*\(\s*"([^"\n]+)"\s*\)/gu)];
  assert.ok(inputs.length>0,name);
  for(const [,relative] of inputs){
   const content=fs.readFileSync(new URL(relative,owner),'utf8');
   if(relative.endsWith('.json'))assert.ok(Array.isArray(JSON.parse(content).cases),relative);
   else assert.equal(content,fs.readFileSync(new URL('../../../data/meta/file-count-program.cjs',import.meta.url),'utf8'));
  }
 }
});
