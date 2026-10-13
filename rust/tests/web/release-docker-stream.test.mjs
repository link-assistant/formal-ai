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
