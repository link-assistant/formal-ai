import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {dockerRunner} from '../../../scripts/release-image-factory.mjs';

test('Docker progress reaches both host streams before the runner completes',()=>{
 const moduleUrl=new URL('../../../scripts/release-image-factory.mjs',import.meta.url).href;
 const childScript=`import {spawnSync} from 'node:child_process';import {dockerRunner} from ${JSON.stringify(moduleUrl)};
 const fake=(command,args,options)=>spawnSync(process.execPath,['-e',"console.log('fake-build-stdout');console.error('fake-build-stderr');"],options);
 dockerRunner(fake,process.cwd(),process.env)(['buildx','build']);
 console.log('runner-completed');console.error('runner-completed');`;
 const result=spawnSync(process.execPath,['--input-type=module','-e',childScript],{encoding:'utf8',timeout:10000});
 assert.equal(result.status,0,result.stderr);assert.equal(result.signal,null);
 assert.ok(result.stdout.indexOf('fake-build-stdout')>=0);assert.ok(result.stdout.indexOf('fake-build-stdout')<result.stdout.indexOf('runner-completed'));
 assert.ok(result.stderr.indexOf('fake-build-stderr')>=0);assert.ok(result.stderr.indexOf('fake-build-stderr')<result.stderr.indexOf('runner-completed'));
});

test('Docker inspect JSON stays captured and failed push cannot be certified',()=>{
 const saved=console.error,phases=[];console.error=value=>phases.push(JSON.parse(value));
 try {
  const run=(command,args,options)=>spawnSync(process.execPath,['-e',args[0]==='image'?"process.stdout.write('[{\"Id\":\"observed-config\"}]');process.stderr.write('inspect diagnostic');":"process.exit(7);"],options);
  const invoke=dockerRunner(run,process.cwd(),{PRIVATE_FIXTURE_TOKEN:'must-not-appear'});
  assert.deepEqual(JSON.parse(invoke(['image','inspect','fixture'])),[{Id:'observed-config'}]);assert.equal(phases.length,0);
  assert.throws(()=>invoke(['push','fixture']),/docker push.*failed/);
  assert.equal(phases.length,2);assert.equal(phases[0].event,'start');assert.equal(phases[1].event,'end');assert.equal(phases[1].status,7);
  assert.ok(JSON.stringify(phases).length<1024);assert.ok(!JSON.stringify(phases).includes('must-not-appear'));
 } finally {console.error=saved;}
});

test('Docker child timeout preserves the original error and bounded phase metadata',()=>{
 const saved=console.error,phases=[];console.error=value=>phases.push(JSON.parse(value));
 try {
  const timeout=Object.assign(Error('controlled child timeout'),{code:'ETIMEDOUT'});
  const invoke=dockerRunner(()=>({error:timeout,status:null,signal:'SIGTERM'}),process.cwd(),{PRIVATE_FIXTURE_TOKEN:'must-not-appear'});
  assert.throws(()=>invoke(['buildx','build']),error=>error===timeout);
  assert.equal(phases.length,2);assert.equal(phases[1].errorCode,'ETIMEDOUT');assert.equal(phases[1].signal,'SIGTERM');
  assert.ok(JSON.stringify(phases).length<1024);assert.ok(!JSON.stringify(phases).includes('must-not-appear'));
 } finally {console.error=saved;}
});

function capturedRuntime(run,check) {
 const saved=console.error,events=[];console.error=value=>events.push(JSON.parse(value));
 try {check(dockerRunner(run,process.cwd(),{PRIVATE_FIXTURE_TOKEN:'must-not-appear'}),events);}
 finally {console.error=saved;}
}
const runtimeArgs=['run','--rm','--privileged','fixture@sha256:abc','verify-formal-ai-dind'];
test('privileged runtime stdout-only failure is visible with bounded output',()=>{
 capturedRuntime(()=>({status:7,signal:null,stdout:'runtime-stdout-failure '+ 'x'.repeat(9000),stderr:''}),(invoke,events)=>{
  assert.throws(()=>invoke(runtimeArgs),/docker privileged-runtime.*failed/);
  assert.deepEqual(events.map(x=>x.event),['start','failure-output','end']);
  assert.ok(events[1].stdout.text.startsWith('runtime-stdout-failure'));assert.equal(events[1].stdout.truncated,true);
  assert.ok(Buffer.byteLength(events[1].stdout.text)<=4096);assert.equal(events[2].status,7);
  assert.ok(!JSON.stringify(events).includes('must-not-appear'));assert.ok(JSON.stringify(events).length<6000);
 });
});
test('privileged runtime signal retains stdout stderr and refusal',()=>{
 capturedRuntime(()=>({status:null,signal:'SIGTERM',stdout:'interrupted stdout',stderr:'interrupted stderr'}),(invoke,events)=>{
  assert.throws(()=>invoke(runtimeArgs),/interrupted/);assert.equal(events[1].stdout.text,'interrupted stdout');assert.equal(events[1].stderr.text,'interrupted stderr');
  assert.equal(events[2].signal,'SIGTERM');assert.equal(events[2].status,null);
 });
});
test('privileged runtime spawn error preserves original error and diagnostics',()=>{
 const error=Object.assign(Error('controlled runtime timeout'),{code:'ETIMEDOUT'});
 capturedRuntime(()=>({error,status:null,signal:'SIGTERM',stdout:'partial runtime output',stderr:'partial runtime error'}),(invoke,events)=>{
  assert.throws(()=>invoke(runtimeArgs),actual=>actual===error);assert.equal(events[1].errorCode,'ETIMEDOUT');assert.equal(events[1].stdout.text,'partial runtime output');assert.equal(events[2].errorCode,'ETIMEDOUT');
 });
});
test('successful privileged runtime keeps captured receipt bytes and timing',()=>{
 const stdout='verified runtime receipt\n';
 capturedRuntime((command,args,options)=>{assert.equal(options.stdio,undefined);assert.equal(options.timeout,1500000);assert.equal(options.maxBuffer,16*1024*1024);return{status:0,signal:null,stdout,stderr:'successful diagnostic'};},(invoke,events)=>{
  assert.equal(invoke(runtimeArgs),stdout);assert.deepEqual(events.map(x=>x.event),['start','end']);assert.equal(events[0].progress,'captured-receipt');assert.ok(events[1].elapsedMs>=0);assert.equal(events[1].status,0);
  assert.ok(!JSON.stringify(events).includes(stdout));assert.ok(!JSON.stringify(events).includes('must-not-appear'));
 });
});
