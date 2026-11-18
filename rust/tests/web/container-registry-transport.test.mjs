import assert from 'node:assert/strict';
import {readFileSync,existsSync,mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import test from 'node:test';
const workflow=readFileSync(new URL('../../../.github/workflows/container-images.yml',import.meta.url),'utf8');
const start=workflow.indexOf('    - name: Compile the full image independently on each native architecture\n');
assert.ok(start>=0);
const section=workflow.slice(start,workflow.indexOf('    - uses: actions/upload-artifact',start));
const script=section.slice(section.indexOf('        set -euo pipefail\n'),section.indexOf('        version=$(')).split('\n').map(line=>line.slice(8)).join('\n');
const rate='ERROR: failed to build: failed to solve: konard/box-dind:2.10.2: failed to resolve source metadata: unexpected status from HEAD request to https://registry-1.docker.io/v2/konard/box-dind/manifests/2.10.2: 429 Too Many Requests';
const expected=['buildx','build','--load','--provenance=false','--platform','linux/arm64','--file','Dockerfile','--build-arg','BINARY_SOURCE=compile','--tag','formal-ai:independent-full','--label','org.opencontainers.image.revision=source-sha','--metadata-file','standalone-build.json','.'];
function run(steps,teeFail=false){
 const directory=mkdtempSync(join(tmpdir(),'formal-ai-container-transport-')),bin=join(directory,'bin');mkdirSync(bin);
 const fake=`#!${process.execPath}\nimport fs from 'node:fs';const p=process.env.TRANSPORT_STATE;let s=JSON.parse(fs.readFileSync(p));const step=s.steps[s.calls.length];if(!step)throw Error('unbounded attempt');s.calls.push(process.argv.slice(2));fs.writeFileSync(p,JSON.stringify(s));console.error(step.output??'');process.exit(step.status);\n`;
 writeFileSync(join(bin,'docker'),fake,{mode:0o755});
 writeFileSync(join(bin,'sleep'),`#!${process.execPath}\nimport fs from 'node:fs';const p=process.env.TRANSPORT_STATE,s=JSON.parse(fs.readFileSync(p));s.waits.push(process.argv[2]);fs.writeFileSync(p,JSON.stringify(s));\n`,{mode:0o755});
 if(teeFail)writeFileSync(join(bin,'tee'),'#!/bin/sh\ncat >/dev/null\nexit 7\n',{mode:0o755});
 const state=join(directory,'state.json');writeFileSync(state,JSON.stringify({steps,calls:[],waits:[]}));
 try{
    const result=spawnSync('bash',
    ['-c',
    script],
    {
    cwd:directory,
    env:{
    ...process.env,
    PATH:bin+':'+process.env.PATH,
    RUNNER_TEMP:directory,
    TRANSPORT_STATE:state,
    CONTAINER_ARCH:'arm64',
    CONTAINER_SOURCE_COMMIT:'source-sha'}
    ,
    encoding:'utf8',
    timeout:8000}
    );
    assert.ok(!result.error,
    result.error?.message);
    const receipt=JSON.parse(readFileSync(state));
    for(const args of receipt.calls)assert.deepEqual(args,
    expected,
    'every original build operand preserved');
    return{
    status:result.status,
    retainedLog:existsSync(join(directory,'standalone-build.log'))?readFileSync(join(directory,'standalone-build.log'),'utf8'):null,
    outcome:JSON.parse(readFileSync(join(directory,'standalone-build-outcome.json'),'utf8')),
    ...receipt}
    ;
    }
    finally{
    rmSync(directory,
    {
    recursive:true,
    force:true}
    );
    }

}
test('429-only retry preserves original command, finite attempts and existing deadline',()=>{
 assert.match(section,/timeout-minutes: 25/u);
 for(const [steps,status,waits]of [
  [[{status:0}],0,[]],
  [[{status:41,output:rate},{status:0}],0,['30']],
  [[{status:41,output:rate},{status:42,output:rate},{status:0}],0,['30','60']],
  [[{status:41,output:rate},{status:42,output:rate},{status:43,output:rate}],43,['30','60']],
  [[{status:53,output:'ERROR: failed to build: Rust compilation failed'}],53,[]],
  [[{status:54,output:rate.replace('429 Too Many Requests','401 Unauthorized')}],54,[]],
  [[{status:55,output:rate.replace('registry-1.docker.io','ghcr.io')}],55,[]],
  [[{status:56,output:'429 Too Many Requests\nERROR: failed to build: actual compiler error'}],56,[]]
 ]){const r=run(steps);assert.equal(r.status,status);assert.deepEqual(r.waits,waits);assert.equal(r.calls.length,steps.length);}
});
test('a failed build log pipe cannot turn a producer failure into retry or success',()=>{
 const r=run([{status:0}],true);assert.equal(r.status,7);assert.equal(r.calls.length,1);assert.deepEqual(r.waits,[]);
 assert.equal(r.outcome.consoleComplete,false);assert.equal(r.outcome.logExitCode,7);
});

test('complete compiler output survives every attempt and a final non-transport failure',()=>{
 const first=rate+'\nfirst attempt diagnostic',second='ERROR: failed to build: real compiler failure\nsecond attempt diagnostic';
 const result=run([{status:41,output:first},{status:53,output:second}]);
 assert.equal(result.status,53);assert.equal(result.calls.length,2);assert.deepEqual(result.waits,['30']);
 assert.deepEqual(result.outcome,{state:'finished',attempt:2,exitCode:53,logExitCode:0,consoleComplete:true});
 for(const fragment of ['Build attempt 1/3',first,'Build attempt 2/3',second])assert(result.retainedLog.includes(fragment),fragment);
 assert(result.retainedLog.indexOf(first)<result.retainedLog.indexOf(second));
});
