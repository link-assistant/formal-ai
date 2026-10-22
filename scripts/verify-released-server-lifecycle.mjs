// Hosted observation of the immutable native image; explicit PID1, not dind entrypoint acceptance.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import http from 'node:http';
import net from 'node:net';
import {performance} from 'node:perf_hooks';
import {createHash,randomUUID} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import {boundedCommand} from '../js/server/bounded-command.mjs';
import {parseLinksNotation} from '../js/server/memory-store.mjs';
import {makeRedactor} from '../js/server/structured-log.mjs';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const configured='/memory/memory.lino';
function ordinary(file){
  const stat=fs.lstatSync(file);assert(stat.isFile()&&!stat.isSymbolicLink()&&stat.size<=1048576);
  return fs.readFileSync(file);
}
function request(port,body,timeout){
  return new Promise((resolve,reject)=>{
    const bytes=body===null?null:Buffer.from(JSON.stringify(body));
    const req=http.request({host:'127.0.0.1',port,path:bytes?'/api/openai/v1/chat/completions':'/health',
      method:bytes?'POST':'GET',headers:bytes?{'content-type':'application/json','content-length':bytes.length}:{}},res=>{
      const chunks=[];let size=0;
      res.on('data',chunk=>{size+=chunk.length;if(size>65536)res.destroy(Error('HealthResponseLimit'));else chunks.push(chunk);});
      res.on('error',reject);res.on('aborted',()=>reject(Error('IncompleteHttpResponse')));
      res.on('end',()=>{
        try{assert.equal(res.statusCode,200);resolve(JSON.parse(Buffer.concat(chunks)));}catch(error){reject(error);}
      });
    });
    const timer=setTimeout(()=>req.destroy(Error('HttpDeadline')),timeout);
    req.on('close',()=>clearTimeout(timer));req.on('error',reject);req.end(bytes);
  });
}
function hold(port,timeout){
  return new Promise((resolve,reject)=>{
    const socket=net.connect({host:'127.0.0.1',port});
    const timer=setTimeout(()=>socket.destroy(Error('HeldConnectionDeadline')),timeout);
    socket.on('error',reject);socket.on('close',()=>{clearTimeout(timer);reject(Error('HeldConnectionClosed'));});
    socket.once('connect',()=>socket.write('POST /v1/chat/completions HTTP/1.1\r\nHost: localhost\r\nContent-Length: 1000\r\n\r\n{',error=>{
      if(error){reject(error);return;}
      resolve({isOpen:()=>!socket.destroyed,close:()=>socket.destroy()});
    }));
  });
}
export function requireMemoryHealth(value,active){
  assert.equal(value.status,'ok');assert.equal(value.memory.configured_path,configured);
  assert.equal(value.memory.active_path,active);assert.equal(value.memory.schema_version,2);
  assert.equal(value.memory.compatible,true);assert.equal(value.memory.migration_required,false);
}
export function requireSuccessfulStop(state,elapsed){
  assert(elapsed<5000,'NativeShutdownDeadline');assert.equal(state.Running,false);
  assert.equal(state.ExitCode,0);assert.equal(state.OOMKilled,false);assert.equal(state.Error,'');
}
export async function observeReleasedLifecycle({runtime,source,fixture1,fixture2,directory},injected={}){
  assert.match(source,/^[a-f0-9]{40}$/u);assert.equal(runtime.source,source);
  assert.match(runtime.image,/^ghcr\.io\/link-assistant\/formal-ai@sha256:[a-f0-9]{64}$/u);
  assert.match(runtime.binarySha256,/^[a-f0-9]{64}$/u);
  assert(fixture1.equals(Buffer.from(fixture2.toString().replace('  schema_version "2"\n',''))),'CanonicalFixturePair');
  const ops={command:boundedCommand,request,hold,now:()=>performance.now(),
    pause:ms=>new Promise(done=>setTimeout(done,ms)),...injected};
  const deadline=ops.now()+180000,redact=makeRedactor();
  const receipt={schema:'formal-ai-released-lifecycle/v1',source,image:runtime.image,
    binarySha256:runtime.binarySha256,entrypoint:'explicit-native-PID1',defaultDindEntrypointTested:false,context:{dreaming:false,liveFetch:false,chatRecording:true},
    fixtures:{supported:hash(fixture1),current:hash(fixture2)},cycles:[],complete:false,cleanupComplete:false};
  const remaining=(limit=30000)=>{const left=deadline-ops.now();assert(left>0,'LifecycleDeadline');return Math.max(1,Math.min(limit,left));};
  async function docker(args,timeout=30000,cleanup=false){
    const bounded=cleanup?Math.min(timeout,10000):remaining(timeout);
    const result=await ops.command('docker',args,{timeout:bounded,stdoutLimitBytes:1048576,stderrLimitBytes:65536});
    if(result.error||result.cleanup_complete!==true||result.signal!==null||result.status!==0){
      const stderr=Buffer.from(result.stderr??'');
      receipt.commandFailure={operation:args[0],status:result.status,signal:result.signal,
        errorCode:result.error?.code??null,timedOut:result.timed_out===true,
        cleanupComplete:result.cleanup_complete===true,stderrBytes:stderr.length,
        stderrSha256:hash(stderr),stderr:String(redact(stderr.toString()))};
    }
    assert.equal(result.error,undefined);assert.equal(result.cleanup_complete,true);
    assert.equal(result.signal,null);assert.equal(result.status,0,'DockerObservationFailed');return result.stdout.toString().trim();
  }
  const unknown=Buffer.from(fixture2.toString().replace('schema_version "2"','schema_version "99"'));
  let owned=null,held=null;
  try{
    const image=JSON.parse(await docker(['image','inspect',runtime.image]));
    assert.equal(image.length,1);assert(image[0].RepoDigests.includes(runtime.image));
    for(const scenario of ['supported','incompatible']){
      const volume=path.join(directory,scenario);fs.mkdirSync(volume,{mode:0o700});
      const original=scenario==='supported'?fixture1:unknown;
      fs.writeFileSync(path.join(volume,'memory.lino'),original,{flag:'wx',mode:0o600});
      const active=scenario==='supported'?configured:configured+'.schema-2.'+hash(original)+'.lino';
      let baseline=null,recordedAnswer=null;
      for(let pass=1;pass<=2;pass++){
        const name='formal-ai-lifecycle-'+randomUUID();owned=name;
        await docker(['create','--pull=never','--name',name,'--user',process.getuid()+':'+process.getgid(),
          '--env','HOME=/memory','--env','FORMAL_AI_MEMORY_PATH='+configured,
          '--env','FORMAL_AI_DREAMING=0','--env','FORMAL_AI_RECORD_CHAT=true',
          '--env','FORMAL_AI_DIALOG_LOG_DIR=/memory/dialog','--env','FORMAL_AI_LIVE_FETCH=false',
          '--publish','127.0.0.1::8080','--mount','type=bind,src='+volume+',dst=/memory',
          '--entrypoint','/usr/local/bin/formal-ai',runtime.image,'serve','--host','0.0.0.0','--port','8080']);
        await docker(['start',name]);
        const inspect=JSON.parse(await docker(['inspect',name]))[0];assert.equal(inspect.Image,image[0].Id);
        const bindings=inspect.NetworkSettings.Ports['8080/tcp'];assert.equal(bindings.length,1);
        assert.equal(bindings[0].HostIp,'127.0.0.1');const port=Number(bindings[0].HostPort);
        assert(Number.isSafeInteger(port)&&port>0&&port<=65535);
        const pidDigest=await docker(['exec',name,'sha256sum','/proc/1/exe']);
        assert.equal(pidDigest,runtime.binarySha256+'  /proc/1/exe','NativePid1BinaryMismatch');
        let health=null;
        for(let poll=0;poll<20;poll++){
          try{health=await ops.request(port,null,remaining(1000));break;}catch(error){
            if(poll===19)throw error;await ops.pause(Math.min(100,remaining()));
          }
        }
        requireMemoryHealth(health,active);
        let answer=null;
        if(scenario==='incompatible'&&pass===1){
          const response=await ops.request(port,{model:'formal-ai',stream:false,
            messages:[{role:'user',content:'2 + 2'}]},remaining(30000));
          answer=response.choices?.[0]?.message?.content;assert.equal(typeof answer,'string');assert.match(answer,/\b4\b/u);
          recordedAnswer=answer;
        }
        held=await ops.hold(port,remaining(10000));assert(held.isOpen(),'HeldConnectionNotOpen');
        // A second answered health request proves responsiveness while the first connection is held.
        requireMemoryHealth(await ops.request(port,null,remaining(1000)),active);
        assert(held.isOpen(),'HeldConnectionClosedBeforeSignal');
        const stop=ops.now();
        await docker(['kill','--signal=TERM',name],Math.min(remaining(),4999));
        const wait=await docker(['wait',name],Math.max(1,Math.min(remaining(),4999-(ops.now()-stop))));assert.equal(wait,'0');
        const stopped=JSON.parse(await docker(['inspect',name],Math.max(1,Math.min(remaining(),4999-(ops.now()-stop)))))[0];
        const shutdownMs=ops.now()-stop;
        requireSuccessfulStop(stopped.State,shutdownMs);held.close();held=null;
        const file=path.join(volume,path.basename(active)),bytes=ordinary(file);
        const events=parseLinksNotation(bytes.toString());
        let backupSha256=null,chatWritten=false;
        if(scenario==='supported'){
          assert(bytes.equals(fixture2),'SupportedMigrationLoss');
          const backups=fs.readdirSync(volume).filter(member=>member.endsWith('.backup'));assert.equal(backups.length,1);
          const backup=ordinary(path.join(volume,backups[0]));
          assert(backup.equals(original),'SupportedBackupMismatch');backupSha256=hash(backup);
          assert.deepEqual(events,parseLinksNotation(fixture1.toString()),'SupportedEventLoss');
        }else{
          assert(ordinary(path.join(volume,'memory.lino')).equals(original),'IncompatibleOriginalChanged');
          assert(bytes.toString().startsWith('demo_memory\n  schema_version "2"\n'));
          assert(events.length>=2,'FallbackDidNotRecordEvents');
          assert(events.some(event=>event.role==='user'&&event.content==='2 + 2'),'FallbackDidNotRecordPrompt');
          assert(events.some(event=>event.role==='assistant'&&event.content===recordedAnswer),'FallbackDidNotRecordAnswer');
          chatWritten=true;
        }
        if(baseline!==null)assert(bytes.equals(baseline),'RestartChangedMemory');baseline=bytes;
        receipt.cycles.push({scenario,pass,pid1BinarySha256:runtime.binarySha256,
          activePath:active,configuredPath:configured,memorySha256:hash(bytes),exitCode:0,
          heldConnection:true,shutdownMs,originalMemorySha256:hash(original),
          configuredMemorySha256:hash(ordinary(path.join(volume,'memory.lino'))),
          eventsSha256:hash(Buffer.from(JSON.stringify(events))),eventCount:events.length,backupSha256,
          supportedMigrationVerified:scenario==='supported',
          incompatibleOriginalPreserved:scenario==='incompatible',chatWritten,
          userContentSha256:chatWritten?hash(Buffer.from('2 + 2')):null,
          assistantContentSha256:chatWritten?hash(Buffer.from(recordedAnswer)):null});
        await docker(['rm',name]);owned=null;
      }
    }
    receipt.cleanupComplete=true;receipt.complete=true;
  }catch(error){
    receipt.failure=error.code??error.message;
    receipt.failureDetail=String(redact(String(error.message))).slice(0,8192);
  }
  finally{
    held?.close();
    if(owned!==null){
      try{await docker(['rm','--force',owned],10000,true);receipt.cleanupComplete=true;}
      catch{receipt.cleanupComplete=false;receipt.cleanupFailure='ContainerCleanupFailed';}
    }
  }
  return receipt;
}
export async function main(environment=process.env){
  assert.equal(environment.GITHUB_ACTIONS,'true','HostedLifecycleOnly');
  assert(environment.HIVE_PUBLISHED_RUNTIME_FILE,'PublishedRuntimeRequired');
  const identity=JSON.parse(ordinary(path.join(environment.E2E_LOG_DIR,'source-binary-identity.json')));
  const selection=JSON.parse(ordinary(environment.HIVE_PUBLISHED_RUNTIME_FILE));
  assert.equal(selection.admitted,true);assert.equal(selection.producer.run,environment.GITHUB_RUN_ID);
  assert.equal(selection.producer.attempt,environment.GITHUB_RUN_ATTEMPT);
  assert.equal(selection.selected.source,environment.HIVE_ADMISSION_SOURCE);
  assert.equal(identity.source,selection.selected.source);assert.deepEqual(identity.publishedRuntime,selection.publishedRuntime);
  assert.equal(identity.binarySha256,selection.publishedRuntime.binarySha256);
  const directory=fs.mkdtempSync(path.join(environment.RUNNER_TEMP??os.tmpdir(),'formal-ai-lifecycle-'));
  try{
    const receipt=await observeReleasedLifecycle({runtime:selection.publishedRuntime,source:selection.selected.source,
      fixture1:ordinary(new URL('../rust/tests/fixtures/memory/schema-1.lino',import.meta.url)),
      fixture2:ordinary(new URL('../rust/tests/fixtures/memory/schema-2.lino',import.meta.url)),directory});
    fs.writeFileSync(path.join(environment.E2E_LOG_DIR,'released-lifecycle.json'),JSON.stringify(receipt,null,2)+'\n',{flag:'wx'});
    assert.equal(receipt.complete,true,'ReleasedLifecycleIncomplete');return receipt;
  }finally{fs.rmSync(directory,{recursive:true,force:true});}
}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href)main().catch(error=>{
  console.error(error.message);process.exitCode=1;
});
