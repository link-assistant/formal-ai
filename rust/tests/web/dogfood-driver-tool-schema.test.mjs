// Actual adapters and declared availability bound every execution, including replays.
// Native twin: rust/tests/unit/agentic-coding/issue_686_agent_cli.rs.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,existsSync,readFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {drive,AGENT_CLI_TOOLS} from '../../../experiments/js_dogfood/drive.mjs';
const write={tool:'write',arguments:JSON.stringify({filePath:'result.txt',content:'authored\n'})};
const call=(...calls)=>({kind:'tool_calls',calls});
const sandbox=()=>mkdtempSync(join(tmpdir(),'driver-tools-'));
test('selected actual adapter list is recorded and executes its requested effect',async()=>{
 const dir=sandbox();try{
  let turn=0;const result=await drive(()=>turn++?{kind:'final',answer:'done'}:call(write),dir,'author',{tools:['write']});
  assert.deepEqual(result.toolsAdvertised,['write']);assert.equal(readFileSync(join(dir,'result.txt'),'utf8'),'authored\n');assert.equal(result.transcript.length,1);assert.equal(result.transcript[0].result,'');
 }finally{rmSync(dir,{recursive:true,force:true});}
});
test('unavailable or duplicated adapter declarations stop before planner or effects',async()=>{
 const dir=sandbox();try{
  for(const tools of [['unsupported'],['write','write']]){
   let called=false;const result=await drive(()=>{called=true;return call(write);},dir,'author',{tools});
   assert.equal(called,false);assert.equal(result.stop,'invalid-tools');assert.deepEqual(result.transcript,[]);assert.equal(existsSync(join(dir,'result.txt')),false);
  }
 }finally{rmSync(dir,{recursive:true,force:true});}
});
test('an undeclared call vetoes the complete batch before its first side effect',async()=>{
 const dir=sandbox();try{
  const result=await drive(()=>call(write,{tool:'bash',arguments:'{}'}),dir,'author',{tools:['write']});
  assert.equal(result.stop,'undeclared-tool');assert.deepEqual(result.transcript,[]);assert.deepEqual(result.toolsAdvertised,['write']);assert.equal(existsSync(join(dir,'result.txt')),false);
 }finally{rmSync(dir,{recursive:true,force:true});}
});
test('caller and planner mutations cannot fabricate advertised availability',async()=>{
 const dir=sandbox();try{
  const original=['read'];const result=await drive((_messages,tools)=>{
   original.push('write');assert.throws(()=>tools.push('write'),TypeError);return call(write);
  },dir,'author',{tools:original});
  assert.deepEqual(result.toolsAdvertised,['read']);assert.equal(result.stop,'undeclared-tool');assert.equal(existsSync(join(dir,'result.txt')),false);
 }finally{rmSync(dir,{recursive:true,force:true});}
});
test('a final answer with no tool availability records an empty actual schema',async()=>{
 const result=await drive(()=>({kind:'final',answer:'answer'}),tmpdir(),'answer',{tools:[]});
 assert.deepEqual(result.toolsAdvertised,[]);assert.equal(result.answer,'answer');assert.deepEqual(result.transcript,[]);assert.ok(AGENT_CLI_TOOLS.includes('read'));
});
