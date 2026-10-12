import {readCheckedReleaseOperationView} from '../../../scripts/checked-release-operation-view.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { createRequire } from 'node:module';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../..');
const YAML = createRequire(path.join(root, 'package.json'))('yaml');
const evidence = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
function layout() {
  const directory = fs.mkdtempSync(path.join(tmpdir(), 'maintained-stage-controls-'));
  const files = ['scripts/maintained-staged-authority.mjs', 'scripts/staged-caller-authority.mjs', 'scripts/governed-github-command-provider.mjs', 'scripts/release-source-freshness.mjs', 'scripts/generate-staged-release.mjs', 'scripts/lib/ci-speed-workflows.mjs', '.github/workflows/release.yml', '.github/workflows/release-staged.yml', ...['stage-source-coverage.json', 'candidate-release-caller.yml', 'candidate-output-bindings.json', 'original-release-workflow.yml'].map(name => evidence + name)];
  for (const file of files) {
    fs.mkdirSync(path.dirname(path.join(directory, file)), {
      recursive: true
    });
    fs.copyFileSync(path.join(root, file), path.join(directory, file));
  }
  fs.copyFileSync(path.join(root, 'package.json'), path.join(directory, 'package.json'));
  fs.symlinkSync(path.join(root, 'node_modules'), path.join(directory, 'node_modules'));
  // Ordinary fixtures retain their original input; actual current deployment is checked first.
  fs.writeFileSync(path.join(directory,'.github/workflows/release.yml'),readCheckedReleaseOperationView().originalSource);
  return directory;
}
test('maintained source compiler conserves full release and rejects receipt and protocol drift', async () => {
  const directory = layout();
  try {
    const module = await import(pathToFileURL(path.join(directory, 'scripts/maintained-staged-authority.mjs')));
    const receipt = module.compileMaintainedStagedAuthority();
    const inventory = module.maintainedStagedInventory(receipt);
    assert.equal(inventory.operations, 52);
    assert.equal(inventory.bindings, 104);
    assert.equal(inventory.productionAuthority, false);
    await assert.rejects(() => module.observeCurrentMaintainedStagedContext(receipt, {
      caller: 'auto-release',
      ordinal: 0,
      environment: {}
    }));
    await assert.rejects(() => module.observeCurrentMaintainedStagedContext(receipt, {
      caller: 'foreign',
      ordinal: 0
    }));
    assert.throws(() => module.maintainedStagedInventory({
      ...receipt
    }));
    await assert.rejects(() => module.observeMaintainedStagedContext({
      ...receipt
    }, {}));
    for (const name of ['governed-github-command-provider.mjs', 'staged-caller-authority.mjs', 'release-source-freshness.mjs', 'generate-staged-release.mjs', 'lib/ci-speed-workflows.mjs', 'maintained-staged-authority.mjs']) {
      const file = path.join(directory, 'scripts', name),
        before = fs.readFileSync(file, 'utf8');
      fs.writeFileSync(file, before + '\n// changed source\n');
      await assert.rejects(() => module.observeMaintainedStagedContext(receipt, {}), /maintained source changed/);
      fs.writeFileSync(file, before);
    }
  } finally {
    fs.rmSync(directory, {
      recursive: true,
      force: true
    });
  }
});
test('checked complete projection rejects changed authority, source, operations and output bindings', async () => {
  const directory = layout();
  try {
    const module = await import(pathToFileURL(path.join(directory, 'scripts/maintained-staged-authority.mjs')));
    const file = path.join(directory, '.github/workflows/release-staged.yml');
    const before = fs.readFileSync(file, 'utf8');
    const mutations = [workflow => workflow.jobs['auto_prepare-source'].permissions.contents = 'read', workflow => workflow.jobs['auto_compile-release'].env.EXPECT_COMPILER = 'foreign', workflow => workflow.jobs['auto_compile-release'].env.EXPECT_SOURCE = 'foreign', workflow => workflow.jobs['auto_compile-release']['timeout-minutes'] = 31, workflow => workflow.jobs['auto_compile-release'].concurrency = {
      group: 'formal-ai-repository-writes'
    }, workflow => workflow.jobs['auto_compile-release'].steps.pop(), workflow => workflow.jobs['auto_compile-release'].steps.reverse(), workflow => workflow.jobs['auto_prepare-source'].outputs.source_sha = 'foreign'];
    for (const mutate of mutations) {
      const workflow = YAML.parse(before);
      mutate(workflow);
      fs.writeFileSync(file, YAML.stringify(workflow));
      assert.throws(() => module.compileMaintainedStagedAuthority());
      fs.writeFileSync(file, before);
    }
    for (const name of ['candidate-output-bindings.json', 'candidate-release-caller.yml']) {
      const target = path.join(directory, evidence, name),
        original = fs.readFileSync(target, 'utf8');
      fs.writeFileSync(target, original + '# unknown source\n');
      assert.throws(() => module.compileMaintainedStagedAuthority());
      fs.writeFileSync(target, original);
    }
    assert.equal(module.maintainedStagedInventory(module.compileMaintainedStagedAuthority()).bindings, 104);
  } finally {
    fs.rmSync(directory, {
      recursive: true,
      force: true
    });
  }
});

test('deployed caller is independently checked against retained original source and complete projection',async()=>{
  const directory=layout();
  try {
    const module=await import(pathToFileURL(path.join(directory,'scripts/maintained-staged-authority.mjs')));
    const caller=path.join(directory,'.github/workflows/release.yml');
    const before=fs.readFileSync(caller);
    assert.throws(()=>module.compileDeployedStagedAuthority());
    fs.copyFileSync(path.join(directory,evidence,'candidate-release-caller.yml'),caller);
    assert.throws(()=>module.compileMaintainedStagedAuthority());
    const receipt=module.compileDeployedStagedAuthority();
    assert.equal(module.maintainedStagedInventory(receipt).operations,52);
    assert.equal(module.maintainedStagedInventory(receipt).bindings,104);
    await assert.rejects(()=>module.observeCurrentMaintainedStagedContext(receipt,{caller:'auto-release',ordinal:0,environment:{}}));
    const changes=[
      ['original-release-workflow.yml',bytes=>bytes+'# changed retained source\n'],
      ['stage-source-coverage.json',bytes=>{const packet=JSON.parse(bytes);packet.workflowSha256='0'.repeat(64);return JSON.stringify(packet);}],
      ['stage-source-coverage.json',bytes=>{const packet=JSON.parse(bytes);packet.callers[0].steps[0].ordinal=99;return JSON.stringify(packet);}],
      ['stage-source-coverage.json',bytes=>{const packet=JSON.parse(bytes);packet.callers[0].crossStageOutputTransfers[0].output='foreign';return JSON.stringify(packet);}],
      ['candidate-output-bindings.json',bytes=>bytes+'# changed output binding\n']
    ];
    for(const [name,mutate] of changes) {
      const file=path.join(directory,evidence,name),original=fs.readFileSync(file,'utf8');
      fs.writeFileSync(file,mutate(original));assert.throws(()=>module.compileDeployedStagedAuthority());
      fs.writeFileSync(file,original);
    }
    const stage=path.join(directory,'.github/workflows/release-staged.yml'),original=fs.readFileSync(stage,'utf8');
    const mutations=[
      workflow=>workflow.jobs['auto_prepare-source'].permissions.contents='read',
      workflow=>workflow.jobs['auto_compile-release'].env.EXPECT_COMPILER='foreign',
      workflow=>workflow.jobs['auto_compile-release'].env.EXPECT_SOURCE='foreign',
      workflow=>workflow.jobs['auto_compile-release']['timeout-minutes']=31,
      workflow=>workflow.jobs['auto_compile-release'].steps.reverse()
    ];
    for(const mutate of mutations) {
      const workflow=YAML.parse(original);mutate(workflow);fs.writeFileSync(stage,YAML.stringify(workflow));
      assert.throws(()=>module.compileDeployedStagedAuthority());fs.writeFileSync(stage,original);
    }
    assert.equal(module.maintainedStagedInventory(module.compileDeployedStagedAuthority()).bindings,104);
    fs.writeFileSync(caller,before);assert.equal(module.maintainedStagedInventory(module.compileMaintainedStagedAuthority()).operations,52);
  } finally {fs.rmSync(directory,{recursive:true,force:true});}
});

test('actual subprocess refuses missing immutable protocol and supplied host authority',async()=>{
 const directory=layout();
 try {
  const {spawnSync}=await import('node:child_process');
  const event=path.join(directory,'event.json');fs.writeFileSync(event,JSON.stringify({repository:{full_name:'link-assistant/formal-ai'}}));
  const temporary=path.join(directory,'runner-temp');fs.mkdirSync(temporary);
  const invoke = environment => spawnSync(
    process.execPath,
    [path.join(directory, 'scripts/release-source-freshness.mjs'), 'before-sync'],
    {
      cwd: directory,
      env: {
        ...process.env,
        GITHUB_ACTIONS: 'true',
        GITHUB_EVENT_PATH: event,
        GITHUB_JOB: 'auto_prepare-source',
        GITHUB_SHA: 'a'.repeat(40),
        GITHUB_RUN_ID: '1',
        GITHUB_RUN_ATTEMPT: '1',
        RUNNER_TEMP: temporary,
        ...environment
      },
      encoding: 'utf8',
      timeout: 10000
    }
  );
  const missing=invoke({});assert.notEqual(missing.status,0);assert.match(missing.stderr,/_protocol/);for(const caller of ['auto-release','manual-release'])assert.equal(fs.existsSync(path.join(temporary,'formal-ai-release-freshness-1-1-'+caller+'.json')),false);
  const script=path.join(directory,'supplied-host.mjs');fs.writeFileSync(script,"import {main} from './scripts/release-source-freshness.mjs'; await main({...process.env});");
  const supplied=spawnSync(process.execPath,[script],{cwd:directory,env:{...process.env,GITHUB_ACTIONS:'true',GITHUB_EVENT_PATH:event,GITHUB_JOB:'auto_prepare-source'},encoding:'utf8',timeout:10000});
  assert.notEqual(supplied.status,0);assert.match(supplied.stderr,/actual current process host/);
  fs.mkdirSync(path.join(directory,'_protocol'));const malformed=invoke({});assert.notEqual(malformed.status,0);assert.match(malformed.stderr,/git/);
 }finally{fs.rmSync(directory,{recursive:true,force:true});}
});

test('separate current group proof rejects stale foreign duplicate and unsourced identities',async()=>{
 const directory=layout();try {
  const module=await import(pathToFileURL(path.join(directory,'scripts/staged-caller-authority.mjs')));
  const canonical=fs.readFileSync(path.join(directory,evidence,'original-release-workflow.yml'),'utf8');
  const callerDraft=fs.readFileSync(path.join(directory,evidence,'candidate-release-caller.yml'),'utf8');
  const staged=fs.readFileSync(path.join(directory,'.github/workflows/release-staged.yml'),'utf8');
  const packet=JSON.parse(fs.readFileSync(path.join(directory,evidence,'stage-source-coverage.json'),'utf8'));
  const contract=module.compileContract({canonical,callerDraft,staged,packet,YAML});const route=contract.routes[0];
  const expected={repository:'link-assistant/formal-ai',run:'1',attempt:'2',head:'a'.repeat(40),runnerName:'declared fixture runner'};
  const job={id:71,run_id:1,run_attempt:2,head_sha:expected.head,name:route.descendantName,runner_name:expected.runnerName,runner_id:81,status:'in_progress',started_at:'2026-10-10T10:00:00Z',completed_at:null,url:'https://api.github.com/repos/link-assistant/formal-ai/actions/jobs/71'};
  const jobs={body:{jobs:[job]},observedAt:'2026-10-10T10:00:01Z'};
  const group={status:200,observedAt:'2026-10-10T10:00:02Z',body:{group_name:contract.writerGroup,group_members:[{job_id:71,run_id:1,job_name:route.descendantName,status:'in_progress'}]}};
  assert.equal(module.requireCurrentGroupMembership(contract,expected,route,group,jobs),undefined,'pure proof helper grants no release receipt');
  const mutations=[
   (g,j)=>g.status=422,(g,j)=>g.body.group_name='foreign',
   (g,j)=>g.body.group_members[0].job_id=72,(g,j)=>g.body.group_members[0].run_id=3,
   (g,j)=>g.body.group_members[0].job_name='foreign',(g,j)=>g.body.group_members[0].status='pending',
   (g,j)=>g.body.group_members.push({...g.body.group_members[0]}),
   (g,j)=>j.body.jobs[0].run_attempt=3,(g,j)=>j.body.jobs[0].head_sha='b'.repeat(40),
   (g,j)=>j.body.jobs[0].runner_name='foreign',(g,j)=>j.body.jobs[0].runner_id=0,
   (g,j)=>j.body.jobs[0].status='completed',(g,j)=>j.body.jobs[0].completed_at='2026-10-10T10:00:02Z',
   (g,j)=>j.body.jobs[0].url='https://foreign.invalid/jobs/71',
   (g,j)=>g.observedAt='2026-10-10T10:01:02Z',(g,j)=>g.observedAt='2026-10-10T09:59:59Z',
   (g,j)=>j.body.jobs.push({...j.body.jobs[0]})
  ];
  for(const mutate of mutations){const g=structuredClone(group),j=structuredClone(jobs);mutate(g,j);assert.throws(()=>module.requireCurrentGroupMembership(contract,expected,route,g,j));}
  assert.throws(()=>module.requireCurrentGroupMembership({...contract},expected,route,group,jobs));
  assert.throws(()=>module.requireCurrentGroupMembership(contract,expected,{...route},group,jobs));
  for(const field of ['concurrency']) {
   const workflow=YAML.parse(staged);workflow[field]={group:contract.writerGroup};assert.throws(()=>module.compileContract({canonical,callerDraft,staged:YAML.stringify(workflow),packet,YAML}));
   const caller=YAML.parse(callerDraft);caller[field]={group:contract.writerGroup};assert.throws(()=>module.compileContract({canonical,callerDraft:YAML.stringify(caller),staged,packet,YAML}));
  }
 }finally{fs.rmSync(directory,{recursive:true,force:true});}
});
