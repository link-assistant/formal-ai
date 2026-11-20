import assert from 'node:assert/strict';
import {mkdtempSync, readFileSync, writeFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import YAML from 'yaml';
import {boundedCommand} from '../../js/server/bounded-command.mjs';

const actionPath=process.env.FORMAL_AI_BUILDKIT_ACTION??'.github/actions/setup-buildx-resilient/action.yml';
const action=()=>YAML.parse(readFileSync(actionPath,'utf8'));
const defaultImage=()=>action().inputs['buildkit-image'].default;
const immutable=/^moby\/buildkit@sha256:[a-f0-9]{64}$/u;

async function shell(t,step,env,{outputIsDirectory=false}={}) {
  const directory=mkdtempSync(join(tmpdir(),'formal-ai-buildkit-'));
  t.after(()=>rmSync(directory,{recursive:true,force:true}));
  const output=join(directory,'output');writeFileSync(output,'');
  const result=await boundedCommand('bash',['-c',step.run],{
    cwd:process.cwd(),timeout:10000,maxBuffer:128*1024,
    env:{...process.env,RUNNER_DEBUG:'0',VERBOSE:'false',...env,GITHUB_OUTPUT:outputIsDirectory?directory:output},
  });
  assert.equal(result.error,undefined,result.stderr?.toString());
  assert.equal(result.cleanup_complete,true);
  const outputs=Object.fromEntries(readFileSync(output,'utf8').trim().split('\n').filter(Boolean)
    .map(line=>{const at=line.indexOf('=');return [line.slice(0,at),line.slice(at+1)];}));
  return {status:result.status,outputs,stdout:result.stdout.toString(),stderr:result.stderr.toString()};
}

// Only the remote setup action's outcome is mocked. Execute the actual local
// shell blocks. A hosted probe separately verifies real Docker/BuildKit boot.
async function setup(t,{canonicalWorks=true,mirrorWorks=true,mirror='mirror.gcr.io',image=defaultImage()}={}) {
  const steps=action().runs.steps;
  const references=steps.find(step=>step.id==='references');assert.ok(references,'validate before Docker IO');
  const checked=await shell(t,references,{BUILDKIT_IMAGE:image,REGISTRY_MIRROR:mirror});
  if(checked.status!==0)return {...checked,setups:[]};
  const canonical=steps.find(step=>step.id==='canonical');
  assert.equal(canonical?.uses,'docker/setup-buildx-action@v4');
  assert.equal(canonical['continue-on-error'],true);
  const setups=[checked.outputs.canonical];
  if(canonicalWorks)return {status:0,setups,image:checked.outputs.canonical};
  const notice=steps.find(step=>step.name==='Report registry fallback');
  assert.equal(notice.if,"steps.canonical.outcome == 'failure'");
  const result=await shell(t,notice,{REGISTRY_MIRROR:mirror,MIRROR_IMAGE:checked.outputs.mirror??''});
  if(result.status!==0)return {...result,setups};
  const fallback=steps.find(step=>step.id==='mirror');
  assert.equal(fallback?.uses,canonical.uses);
  assert.equal(fallback.if,"steps.canonical.outcome == 'failure' && inputs.registry-mirror != ''");
  setups.push(checked.outputs.mirror);
  return {status:mirrorWorks?0:1,setups,image:checked.outputs.mirror};
}

test('every BuildKit bootstrap selects the same immutable multi-platform image',()=>{
  const document=action(),image=defaultImage();assert.match(image,immutable);
  const [references,canonical,notice,mirror]=document.runs.steps;
  assert.equal(references.id,'references');assert.equal(canonical.id,'canonical');assert.equal(mirror.id,'mirror');
  assert.equal(canonical.with['driver-opts'],'image=${{ steps.references.outputs.canonical }}');
  assert.equal(mirror.with['driver-opts'],'image=${{ steps.references.outputs.mirror }}');
  assert.equal(document.inputs.cleanup.default,'true');
  for(const step of [canonical,mirror])assert.equal(step.with.cleanup,'${{ inputs.cleanup }}');
  assert.equal(document.outputs.image.value,"${{ steps.mirror.outcome == 'success' && steps.references.outputs.mirror || steps.references.outputs.canonical }}");
  assert.equal(document.outputs.name.value,"${{ steps.mirror.outcome == 'success' && steps.mirror.outputs.name || steps.canonical.outputs.name }}");
  assert.doesNotMatch(document.runs.steps.filter(step=>step.run).map(step=>step.run).join('\n'),/docker (?:pull|tag)|\bsleep\b/u);
  assert.match(notice.run,/RUNNER_DEBUG/u);assert.match(notice.run,/set -x/u);
  const workflow=YAML.parse(readFileSync('.github/workflows/container-images.yml','utf8'));
  const direct=Object.values(workflow.jobs).flatMap(job=>job.steps??[])
    .filter(step=>step.uses==='docker/setup-buildx-action@v4');
  assert.equal(direct.length,4);
  for(const step of direct){assert.equal(step.with['driver-opts'],'image='+image);assert.equal(step.with.cleanup,false);}
});
test('canonical success skips actual mirror setup',async t=>{
  assert.deepEqual(await setup(t),{status:0,setups:[defaultImage()],image:defaultImage()});
});
test('canonical setup failure retries actual setup with the identical digest at the mirror',async t=>{
  const image=defaultImage();
  assert.deepEqual(await setup(t,{canonicalWorks:false}),{status:0,setups:[image,'mirror.gcr.io/'+image],image:'mirror.gcr.io/'+image});
});
test('failure of both actual setup actions remains fatal',async t=>{
  const image=defaultImage(),result=await setup(t,{canonicalWorks:false,mirrorWorks:false});
  assert.equal(result.status,1);assert.deepEqual(result.setups,[image,'mirror.gcr.io/'+image]);
});
test('disabled mirrors preserve failure and never invoke a fallback',async t=>{
  const image=defaultImage(),result=await setup(t,{canonicalWorks:false,mirror:''});
  assert.equal(result.status,1);assert.deepEqual(result.setups,[image]);assert.match(result.stdout,/mirror is disabled/u);
});
test('explicit caller references remain supported',async t=>{
  const image='moby/buildkit:v0.27.0';
  assert.deepEqual(await setup(t,{image}),{status:0,setups:[image],image});
});
test('invalid multiline or empty references fail before Docker setup or output records',async t=>{
  for(const options of [{image:''},{image:'moby/buildkit:tag'+String.fromCharCode(10)+'image=forged'},
    {mirror:'mirror.gcr.io'+String.fromCharCode(13)+'image=forged'}]) {
    const result=await setup(t,options);assert.equal(result.status,1);
    assert.deepEqual(result.setups,[]);assert.deepEqual(result.outputs,{});
  }
});
test('reference output write failure is fatal before the official setup action',async t=>{
  const step=action().runs.steps.find(row=>row.id==='references');
  const result=await shell(t,step,{BUILDKIT_IMAGE:defaultImage(),REGISTRY_MIRROR:'mirror.gcr.io'},{outputIsDirectory:true});
  assert.notEqual(result.status,0);assert.deepEqual(result.outputs,{});
});

test('hosted real-daemon probe retains read-only authority and immutable image observations',()=>{
  const workflow=YAML.parse(readFileSync('.github/workflows/workflows.yml','utf8'));
  assert.equal(workflow.permissions.contents,'read');
  for(const event of ['push','pull_request'])assert(workflow.on[event].paths.includes('experiments/issue-1580/buildkit-bootstrap-probe.py'));
  const job=workflow.jobs['buildkit-bootstrap'];
  assert.equal(job['runs-on'],'ubuntu-24.04');assert.equal(job['timeout-minutes'],12);
  const checkout=job.steps[0];assert.equal(checkout.with['persist-credentials'],false);
  for(const path of ['/.github/actions/setup-buildx-resilient/','/experiments/issue-1580/buildkit-bootstrap-probe.py'])
    assert(checkout.with['sparse-checkout'].split('\n').includes(path));
  const setups=job.steps.filter(step=>step.uses==='./.github/actions/setup-buildx-resilient');
  assert.deepEqual(setups.map(step=>step.id),['initial','fallback']);
  for(const step of setups){assert.equal(step.with.cleanup,'false');assert.equal(step['timeout-minutes'],4);}
  const faultIndex=job.steps.findIndex(step=>step.run?.includes(' install-fault '));
  assert(faultIndex>job.steps.findIndex(step=>step.run?.includes(' initial ')));
  assert(faultIndex<job.steps.findIndex(step=>step.id==='fallback'));
  const verification=job.steps.find(step=>step.run?.includes(' mirror '));
  assert.equal(verification.env.IMAGE,'${{ steps.fallback.outputs.image }}');
  assert.equal(verification.env.BUILDER,'${{ steps.fallback.outputs.name }}');
  const upload=job.steps.at(-1);assert.equal(upload.uses,'actions/upload-artifact@v7');
  assert.equal(upload.if,'always()');assert.equal(upload.with['if-no-files-found'],'error');
});
test('hosted outage injector delegates every unrelated Docker command unchanged',async()=>{
  const result=await boundedCommand('python3',['experiments/issue-1580/buildkit-bootstrap-probe.py','self-test'],{
    cwd:process.cwd(),timeout:10000,maxBuffer:128*1024,env:{...process.env,PYTHONDONTWRITEBYTECODE:'1'},
  });
  assert.equal(result.error,undefined,result.stderr?.toString());assert.equal(result.status,0,result.stderr?.toString());
  assert.equal(result.cleanup_complete,true);assert.match(result.stdout.toString(),/rejects only the exact canonical pull/u);
});
