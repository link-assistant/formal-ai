import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {recoverClientInstall,failedRegistryTarball,requireRegistryMetadata} from '../../../scripts/install-client-registry-recovery.mjs';
const fixture=JSON.parse(fs.readFileSync(new URL('../fixtures/client-install/registry-tarball-404.json',import.meta.url)));
const args=['add','-g','--ignore-scripts',fixture.client];
const candidate=failedRegistryTarball(fixture.stdout,fixture.stderr);
const record={name:'vscode-languageserver-types',version:'3.18.5',dist:{tarball:candidate.tarball,
  integrity:'sha512-'+Buffer.alloc(64,7).toString('base64')}};
const result=(status,stderr='',extra={})=>({status,signal:null,error:undefined,cleanup_complete:true,
  timed_out:false,stdout:Buffer.from('bun add\n'),stderr:Buffer.from(stderr),...extra});
async function replay({first=result(1,fixture.stderr),second=result(0),metadata=record,head=200,
  fetchFailure=false,advance=0,redirect=null}={}){
  let calls=0,clock=0;const fetches=[],outputs=[],records=[],timeouts=[],environment={AUTH_TOKEN:'owned-test-secret'};
  const final=await recoverClientInstall('bun',args,{seconds:60,cwd:'/owned-client',env:environment},{
    now:()=>clock,command:async(file,actual,options)=>{
      calls++;assert.equal(file,'bun');assert.deepEqual(actual,args);assert.equal(options.cwd,'/owned-client');
      assert.equal(options.env,environment);assert(options.timeout>0&&options.timeout<=60000);
      assert.equal(options.stdoutLimitBytes,4194304);assert.equal(options.stderrLimitBytes,4194304);
      timeouts.push(options.timeout);if(calls===1)clock+=advance;return calls===1?first:second;
    },fetch:async(url,options)=>{
      fetches.push({url,options});assert.equal(options.redirect,'error');assert(options.signal instanceof AbortSignal);
      if(fetchFailure)throw Error('transport');
      if(options.method==='HEAD')return {status:head,url:redirect??url,body:null};
      const bytes=Buffer.from(JSON.stringify(metadata));
      return {status:200,url:redirect??url,body:(async function*(){yield bytes;})()};
    },output:(stream,text)=>outputs.push({stream,text}),metadata:value=>records.push(value)
  });
  return {final,calls,fetches,outputs,records,timeouts};
}
test('captured actual tarball failure needs qualified metadata and HEAD before one identical retry',async()=>{
  assert.deepEqual(candidate,{name:record.name,version:record.version,tarball:record.dist.tarball});
  const observed=await replay();assert.equal(observed.final.status,0);assert.equal(observed.calls,2);
  assert.equal(observed.fetches.length,2);assert.equal(observed.fetches[1].options.method,'HEAD');
  assert.equal(observed.outputs.length,4);assert(observed.records.some(row=>row.phase==='head-qualified'));
});
test('persistent tarball failure gets at most two identical installer calls',async()=>{
  const observed=await replay({second:result(1,fixture.stderr)});
  assert.equal(observed.final.status,1);assert.equal(observed.calls,2);assert.equal(observed.fetches.length,2);
});
for(const [name,first] of Object.entries({success:result(0),authorization:result(1,'error: GET '+candidate.tarball+' - 401'),
  otherFatal:result(1,fixture.stderr+'error: permission denied\n'),
  capitalError:result(1,fixture.stderr+'Error: permission denied\n'),panic:result(1,fixture.stderr+'panic: other failure\n'),unknown:result(1,'unrecognized failure'),
  signal:result(1,fixture.stderr,{signal:'SIGTERM'}),timeout:result(1,fixture.stderr,{timed_out:true}),
  overflow:result(1,fixture.stderr,{error:Object.assign(Error('overflow'),{code:'ENOBUFS'})}),
  uncertainCleanup:result(1,fixture.stderr,{cleanup_complete:false})})){
  test(name+' does not probe or retry',async()=>{
    const observed=await replay({first});assert.equal(observed.calls,1);assert.equal(observed.fetches.length,0);
  });
}
for(const [name,options] of Object.entries({wrongName:{metadata:{...record,name:'another'}},
  wrongVersion:{metadata:{...record,version:'3.18.4'}},wrongTarball:{metadata:{...record,dist:{...record.dist,tarball:'https://other.invalid/x'}}},
  missingIntegrity:{metadata:{...record,dist:{tarball:candidate.tarball}}},
  unavailable:{head:404},unauthorized:{head:401},redirect:{redirect:'https://other.invalid/x'},transport:{fetchFailure:true}})){
  test(name+' refuses recovery and retains the original failed result',async()=>{
    const observed=await replay(options);assert.equal(observed.calls,1);assert.equal(observed.final.status,1);
    assert(observed.records.some(row=>row.phase==='recovery-refused'));
  });
}
test('failed install, probes and second call share one finite deadline',async()=>{
  const observed=await replay({advance:59000});assert.equal(observed.calls,2);assert.equal(observed.timeouts[1],1000);
  const exhausted=await replay({advance:60000});assert.equal(exhausted.calls,1);assert.equal(exhausted.final.status,1);
});
test('official scoped package URL parses, foreign and noncanonical URLs refuse',()=>{
  const line='error: GET https://registry.npmjs.org/@scope/name/-/name-1.2.3-next.1.tgz - 404';
  assert.equal(failedRegistryTarball('',line).name,'@scope/name');
  for(const value of [line.replace('registry.npmjs.org','registry.npmjs.org.evil'),
    line.replace('https:','http:'),line.replace('/name-1.2.3','/different-1.2.3'),line.replace('.tgz','.tgz?token=x'),
    line.replace('registry.npmjs.org/','registry.npmjs.org:443/'),line+'\nerror: second failure'])
    assert.equal(failedRegistryTarball('',value),null);
});
test('bounded replay redacts known credentials and metadata never includes arguments or environment',async()=>{
  const observed=await replay({first:result(1,fixture.stderr+'owned-test-secret\n')});
  assert(!observed.outputs.some(row=>row.text.includes('owned-test-secret')));
  assert(observed.outputs.some(row=>row.text.includes('[REDACTED]')));
  assert(!JSON.stringify(observed.records).includes(fixture.client));
  assert(!JSON.stringify(observed.records).includes('AUTH_TOKEN'));
  assert.throws(()=>requireRegistryMetadata(candidate,{...record,dist:{...record.dist,integrity:'sha1-abc'}}));
});

test('mixed-log channel reconstruction does not change failure classification',()=>{
  assert.deepEqual(failedRegistryTarball(fixture.stderr,fixture.stdout),candidate);
});
