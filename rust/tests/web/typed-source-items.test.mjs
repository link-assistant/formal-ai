import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import test from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStepResolved} from '../../../js/agentic/planner.mjs';
import {rustSourceForTask,verifiedSourceDescription} from '../../../js/agentic/code_task.mjs';
import {sourceDescriptionContract} from '../../../js/agentic/code_task/source_contract.mjs';
await installNodeHost(new WorkerHost());
const tools=['read_file','write_file','run_command'];
const cases=[
  {task:'Create a new file rust/src/echo.rs in this repository containing a single public Rust constant ECHO_TOKEN of type &str with the value echo.',file:'rust/src/echo.rs',content:'pub const ECHO_TOKEN: &str = "echo";\n',kind:'string-constant'},
  {task:'Create a new file rust/tests/unit/parity.rs in this repository containing a single Rust test named parity_check that asserts -7 equals -7.',file:'rust/tests/unit/parity.rs',content:'#[test]\nfn parity_check() {\n    assert_eq!(-7, -7);\n}\n',kind:'equality-test'},
  {task:'Create a Rust test file rust/tests/integration/balance.rs in this repository containing one Rust test named balance_check asserting -9 plus 5 equals -4.',file:'rust/tests/integration/balance.rs',content:'#[test]\nfn balance_check() {\n    assert_eq!(-9 + 5, -4);\n}\n',kind:'addition-test'}
];
for(const fixture of cases){
  test('a typed source item binds every source span and physically verifies its bytes: '+fixture.kind,async()=>{
    const contract=verifiedSourceDescription(fixture.task);
    assert(contract);assert.equal(contract.kind,fixture.kind);assert.equal(contract.wholeRequestConsumed,true);
    assert.equal(contract.compilation,'pending');
    assert.deepEqual(contract.unknownEffects,['module_initialization','tool_execution']);
    for(const capture of contract.captures)assert.equal(fixture.task.slice(...capture.span),capture.text);
    const directory=fs.mkdtempSync(path.join(os.tmpdir(),'typed-source-items-'));
    const messages=[{role:'user',content:fixture.task}];const observed=[];
    try{
      for(let turn=0;turn<5;turn++){
        const plan=await planChatStepResolved(messages,tools);
        if(plan.kind==='final'){
          assert.equal(fs.readFileSync(path.join(directory,fixture.file),'utf8'),fixture.content);
          assert.equal(observed.filter(item=>item.tool==='write_file').length,1);
          assert(observed.some(item=>item.tool==='run_command'&&item.status===0));
          assert(plan.answer.toLowerCase().includes('observed'));return;
        }
        assert.equal(plan.kind,'tool_calls');assert.equal(plan.calls.length,1);
        const call=plan.calls[0],args=JSON.parse(call.arguments),id='physical-'+turn;let result;
        if(call.tool==='read_file'){
          assert.equal(args.path,fixture.file);
          try{result=fs.readFileSync(path.join(directory,args.path),'utf8');}catch(error){assert.equal(error.code,'ENOENT');result='Error: ENOENT: no such file or directory';}
        }else if(call.tool==='write_file'){
          assert.equal(args.path,fixture.file);assert.equal(args.content,fixture.content);
          fs.mkdirSync(path.dirname(path.join(directory,args.path)),{recursive:true});fs.writeFileSync(path.join(directory,args.path),args.content);result='created';
        }else{
          assert.equal(call.tool,'run_command');assert.equal(args.command,'cat '+fixture.file);
          const process=spawnSync('/bin/sh',['-c',args.command],{cwd:directory,encoding:'utf8',timeout:3000});
          assert.equal(process.status,0);result=JSON.stringify({stdout:process.stdout,stderr:process.stderr,exit_code:process.status});observed.push({tool:call.tool,status:process.status});
        }
        if(call.tool!=='run_command')observed.push({tool:call.tool});
        messages.push({role:'assistant',tool_calls:[{id,type:'function',function:{name:call.tool,arguments:call.arguments}}]},{role:'tool',tool_call_id:id,name:call.tool,content:result});
      }
      assert.fail('source transaction did not finish');
    }finally{fs.rmSync(directory,{recursive:true,force:true});}
  });
  for(const suffix of [' Then deploy it.',' Do not write any files.',' Read README.md first.',' Also register this module.',' Preserve unknown requirements.',' And create another file.']){
    test('typed source cannot consume an independent obligation: '+fixture.kind+suffix,async()=>{
      const request=fixture.task+suffix;assert.equal(verifiedSourceDescription(request),null);
      const plan=await planChatStepResolved([{role:'user',content:request}],tools);
      assert.equal(plan.kind,'final');assert.equal(plan.result.disposition,'gap');
    });
  }
  test('typed source refuses extra declaration bytes and changed owned slots: '+fixture.kind,()=>{
    const artifact=rustSourceForTask(fixture.task);assert(artifact);
    for(const content of [artifact.content+'pub fn unrelated() {}\n',artifact.content.replace(fixture.kind==='string-constant'?'"echo"':fixture.kind==='equality-test'?'(-7, -7)':'(-9 + 5, -4)',fixture.kind==='string-constant'?'"other"':fixture.kind==='equality-test'?'(-7, 7)':'(-9 - 5, -4)')])assert.equal(sourceDescriptionContract(fixture.task,{...artifact,content}),null);
  });
}
test('technical string type authority remains exact and integer test literals fit their Rust domain',()=>{
  assert.equal(verifiedSourceDescription(cases[0].task.replace('&str','&STR')),null);
  assert.equal(verifiedSourceDescription(cases[1].task.replaceAll('-7','2147483648')),null);
  assert.equal(verifiedSourceDescription(cases[2].task.replace('-4','-2147483649')),null);
});
