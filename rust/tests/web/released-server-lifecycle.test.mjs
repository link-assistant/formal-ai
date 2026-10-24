import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash} from 'node:crypto';
import {observeReleasedLifecycle,requireSuccessfulStop,main} from '../../../scripts/verify-released-server-lifecycle.mjs';
import {SyncStore} from '../../../js/server/memory-store.mjs';
const fixture1=fs.readFileSync(new URL('../fixtures/memory/schema-1.lino',import.meta.url));
const fixture2=fs.readFileSync(new URL('../fixtures/memory/schema-2.lino',import.meta.url));
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
const runtime={source:'a'.repeat(40),image:'ghcr.io/link-assistant/formal-ai@sha256:'+'b'.repeat(64),binarySha256:'c'.repeat(64)};
async function run(mode='healthy',answer='4'){
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-lifecycle-control-'));
  let volume=null,active=null,clock=0,held=false,removed=0,forced=0,creates=0;
  const command=async(file,args,options)=>{
    assert(options.timeout>0&&options.timeout<=30000);
    assert.equal(file,'docker');clock+=10;let stdout='';
    if(args[0]==='image')stdout=JSON.stringify([{Id:'image-id',RepoDigests:[runtime.image]}]);
    else if(args[0]==='create'){
      creates++;if(mode==='create-observation-failure')throw Error('CreateObservationLost');
      assert(args.includes('--pull=never'));assert(args.includes('/usr/local/bin/formal-ai'));
      const mount=args[args.indexOf('--mount')+1];volume=mount.slice('type=bind,src='.length,-',dst=/memory'.length);
      const original=fs.readFileSync(path.join(volume,'memory.lino'));
      const unsupported=original.includes(Buffer.from('schema_version "99"'));
      active=unsupported?'memory.lino.schema-2.'+digest(original)+'.lino':'memory.lino';
      if(unsupported){
        if(!fs.existsSync(path.join(volume,active)))fs.writeFileSync(path.join(volume,active),'demo_memory\n  schema_version "2"\n');
        if(mode==='incompatible-overwrite')fs.writeFileSync(path.join(volume,'memory.lino'),'lost');
      }else if(!original.equals(fixture2)){
        fs.writeFileSync(path.join(volume,'memory.lino.schema-1.'+digest(original).slice(0,12)+'.backup'),original);
        fs.writeFileSync(path.join(volume,active),mode==='migration-loss'?fixture2.toString().replace('keep me','lost'):fixture2);
      }
      if(mode==='restart-mutation'&&creates===2)fs.appendFileSync(path.join(volume,active),'  event "unexpected"\n');
    }else if(args[0]==='inspect')stdout=JSON.stringify([{Image:'image-id',State:{Running:false,ExitCode:0,OOMKilled:false,Error:''},
      NetworkSettings:{Ports:{'8080/tcp':[{HostIp:'127.0.0.1',HostPort:'12345'}]}}}]);
    else if(args[0]==='exec')stdout=(['wrong-binary','no-pid1'].includes(mode)?'d'.repeat(64):runtime.binarySha256)+'  /proc/1/exe';
    else if(args[0]==='kill'){
      assert(held,'a held connection must precede signal');
      if(mode==='stop-deadline')clock+=5000;
      if(mode==='signal-error')return {error:Error('signal failed'),status:1,signal:null,cleanup_complete:true,stdout:Buffer.alloc(0)};
    }else if(args[0]==='wait')stdout='0';
    else if(args[0]==='rm'){
      removed++;if(args.includes('--force')){forced++;if(mode==='cleanup-error')throw Error('cleanup failed');}
      else if(mode==='cleanup-error')throw Error('cleanup failed');
    }
    return {stdout:Buffer.from(stdout),status:0,signal:null,error:undefined,cleanup_complete:true};
  };
  const request=async(port,body)=>{
    assert.equal(port,12345);
    if(mode==='no-health')throw Error('NoHealth');
    if(body){
      if(mode==='no-fallback-write')return {choices:[{message:{content:answer}}]};
      // Exercise the production writer: assistant task answers live in outputs,
      // with an evidence link to the user message, not in message.content.
      const file=path.join(volume,active);
      const store=SyncStore.open({FORMAL_AI_RECORD_CHAT:'true'},file);
      assert.equal(store.recordChatExchangeWithTools('2 + 2',answer),2);
      const task=store.events.find(event=>event.kind==='task');
      assert.equal(task.content,null);
      if(mode==='wrong-recorded-answer')task.outputs='5';
      if(mode==='wrong-recorded-prompt')task.inputs='3 + 1';
      if(mode==='unlinked-recorded-answer')task.evidence=['missing-user'];
      if(mode==='message-instead-of-task'){task.kind='message';task.content=answer;task.outputs=null;}
      store.persist();
      return {choices:[{message:{content:answer}}]};
    }
    return {status:'ok',memory:{configured_path:'/memory/memory.lino',active_path:'/memory/'+active,
      schema_version:2,compatible:true,migration_required:false}};
  };
  try{
    const result=await observeReleasedLifecycle({runtime,source:runtime.source,fixture1,fixture2,directory},
      {command,request,now:()=>clock,pause:async ms=>{clock+=ms;},hold:async()=>{
        held=mode!=='no-held-connection';return {isOpen:()=>held,close:()=>{held=false;}};
      }});
    return {result,removed,forced};
  }finally{fs.rmSync(directory,{recursive:true,force:true});}
}
test('released lifecycle observes both fixture restarts and native PID1 shutdown with held connection',async()=>{
  const {result,removed,forced}=await run();assert.equal(result.complete,true);assert.equal(result.cleanupComplete,true);
  assert.equal(result.cycles.length,4);assert.equal(removed,4);assert.equal(forced,0);
  assert.equal(result.defaultDindEntrypointTested,false);
  assert.equal(result.currentObservation,undefined);
  for(const kind of ['supported','incompatible']){
    const pair=result.cycles.filter(row=>row.scenario===kind);assert.equal(pair.length,2);
    assert.equal(pair[0].memorySha256,pair[1].memorySha256);
  }
});
test('fallback recording preserves the exact wire answer including whitespace and Unicode across restart',async()=>{
  const {result}=await run('healthy',' 4 — चार\n');
  assert.equal(result.complete,true,result.failureDetail);
  for(const cycle of result.cycles.filter(row=>row.scenario==='incompatible'))
    assert.equal(cycle.assistantContentSha256,digest(Buffer.from(' 4 — चार\n')));
});
for(const mode of ['wrong-binary','no-pid1','no-held-connection','stop-deadline','no-fallback-write','wrong-recorded-answer','wrong-recorded-prompt','unlinked-recorded-answer','message-instead-of-task','restart-mutation','create-observation-failure','no-health','migration-loss','incompatible-overwrite','signal-error']){
  test('released lifecycle refuses '+mode+' and cleans its owned container',async()=>{
    const {result,forced}=await run(mode);assert.equal(result.complete,false);assert.equal(result.cleanupComplete,true);
    assert.equal(forced,1);assert.equal(typeof result.failure,'string');
  });
}
test('failed fallback receipt retains bounded evidence from the failing pass without publishing answer text',async()=>{
  const {result}=await run('wrong-recorded-answer',' 4 — चार\n');
  assert.equal(result.complete,false);assert.equal(result.cleanupComplete,true);
  assert.match(result.failureDetail,/FallbackDidNotRecordLinkedAnswer/u);
  const observation=result.currentObservation;
  assert.equal(observation.scenario,'incompatible');assert.equal(observation.pass,1);
  assert.equal(observation.stage,'memory');assert.equal(observation.exitCode,0);
  assert.equal(observation.health.memory.compatible,true);
  assert.equal(observation.wireAnswer.sha256,digest(Buffer.from(' 4 — चार\n')));
  assert.equal(observation.events.length,2);
  const task=observation.events.find(event=>event.kind==='task');
  assert.equal(task.content,null);assert.equal(task.outputs.sha256,digest(Buffer.from('5')));
  assert.equal(task.evidence[0],observation.events.find(event=>event.role==='user').id);
  assert(!JSON.stringify(result).includes('चार'));
});
test('failed container cleanup cannot produce complete acceptance',async()=>{
  const {result}=await run('cleanup-error');assert.equal(result.complete,false);assert.equal(result.cleanupComplete,false);
  assert.equal(result.cleanupFailure,'ContainerCleanupFailed');
});
test('killed, running, or timed-out native PID1 is never a successful stop',()=>{
  const state={Running:false,ExitCode:0,OOMKilled:false,Error:''};
  for(const changed of [{ExitCode:137},{Running:true},{OOMKilled:true},{Error:'failure'}])
    assert.throws(()=>requireSuccessfulStop({...state,...changed},100));
  assert.throws(()=>requireSuccessfulStop(state,5000));
});

test('local lifecycle CLI refuses before reading identities or invoking effects',async()=>{
  await assert.rejects(main({GITHUB_ACTIONS:'false'}),/HostedLifecycleOnly/u);
});
