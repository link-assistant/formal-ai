import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {host} from '../../../js/agentic/host.mjs';
import {sourceFromLinks,readSourceLinks,sourceOwner} from '../../../js/meta/readable-source.js';
import {TARGET_MODULE_PATH,astTask,AST_PATH} from '../../../js/agentic/self_ast.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {drive,execute} from '../../../experiments/js_dogfood/drive.mjs';
import {parseChatMessage} from '../../../js/server/chat-request.mjs';

test('default Node self inspection emits actual complete source through the actual chat planner and tool effects without an SDK', async () => {
  const previous=process.env.FORMAL_AI_META_LANGUAGE;
  const workspace=fs.mkdtempSync(path.join(os.tmpdir(),'default-self-cst-'));
  try {
    delete process.env.FORMAL_AI_META_LANGUAGE;
    await installNodeHost(new WorkerHost());
    // Keep the general driver's ownership refusal intact; this is the actual
    // chat planner protocol used by the independently executed Agent tools.
    const guarded=await drive(planChatStep,workspace,astTask(),{steps:6});
    assert.equal(guarded.stop,'unowned-create-request');
    assert.equal(fs.existsSync(path.join(workspace,AST_PATH)),false);
    const messages=[parseChatMessage({role:'user',content:astTask()})],effects=[];
    let answer=null;
    for(let turn=0;turn<4;turn++){
      const plan=await planChatStep(messages,['write_file','run_command']);
      if(plan?.kind==='final'){answer=plan.answer;break;}
      assert.equal(plan?.kind,'tool_calls');assert.equal(plan.calls.length,1);
      const call=plan.calls[0],args=JSON.parse(call.arguments);
      let adapter;
      if(effects.length===0){assert.equal(call.tool,'write_file');assert.equal(args.path,AST_PATH);adapter='write';}
      else{assert.equal(effects.length,1);assert.equal(call.tool,'run_command');assert.equal(args.command,'cat '+AST_PATH);adapter='bash';}
      const result=execute(workspace,{...call,tool:adapter}),id='self_cst_'+turn;
      messages.push(parseChatMessage({role:'assistant',content:'',tool_calls:[{id,type:'function',function:{name:call.tool,arguments:call.arguments}}]}));
      messages.push(parseChatMessage({role:'tool',tool_call_id:id,name:call.tool,content:result}));
      effects.push(call.tool);
    }
    assert.equal(typeof answer,'string');
    assert.deepEqual(effects,['write_file','run_command']);
    const document=fs.readFileSync(path.join(workspace,AST_PATH),'utf8');
    const model=readSourceLinks(document);
    assert.equal(sourceOwner(model),TARGET_MODULE_PATH);
    assert.equal(sourceFromLinks(model),fs.readFileSync(new URL('../../../rust/'+TARGET_MODULE_PATH,import.meta.url),'utf8'));
    assert.ok(!answer.includes('Unknown:'));
    await assert.rejects(host().sourceDocument('../Cargo.toml'),/IncompleteOwnedSourceInventory/);
    await assert.rejects(host().sourceDocument('src/not-owned.rs'),/IncompleteOwnedSourceInventory/);
  } finally {
    if(previous===undefined)delete process.env.FORMAL_AI_META_LANGUAGE;else process.env.FORMAL_AI_META_LANGUAGE=previous;
    fs.rmSync(workspace,{recursive:true,force:true});
  }
});

test('explicit rejected SDK never silently uses the default CST engine', async()=>{
  await installNodeHost(new WorkerHost(),{sourceModelDirectory:'/nonexistent/formal-ai-rejected-sdk'});
  await assert.rejects(host().sourceDocument(TARGET_MODULE_PATH));
  await installNodeHost(new WorkerHost(),{sourceModelDirectory:42});
  await assert.rejects(host().sourceDocument(TARGET_MODULE_PATH),/PinnedSourceModelSdkUnavailable/);
});
