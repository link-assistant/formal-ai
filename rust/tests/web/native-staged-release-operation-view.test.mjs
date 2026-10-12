import test,{after} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdtempSync,mkdirSync,lstatSync,realpathSync,chmodSync,symlinkSync,rmSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {tmpdir} from 'node:os';
import {join,dirname,sep} from 'node:path';
const repositoryRoot=fileURLToPath(new URL('../../../',import.meta.url));
const privateDirectory=realpathSync(mkdtempSync(join(tmpdir(),'native-release-operation-controls-')));
const root=privateDirectory+sep;
const protocolEvidence='experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
const sourcePaths=[
 'package.json',
 '.github/workflows/release.yml',
 '.github/workflows/release-staged.yml',
 'scripts/native-staged-release-operation-view.mjs',
 'scripts/checked-release-operation-view.mjs',
 'scripts/maintained-staged-authority.mjs',
 'scripts/generate-staged-release.mjs',
 'scripts/lib/ci-speed-workflows.mjs',
 'scripts/staged-caller-authority.mjs',
 'scripts/release-source-freshness.mjs',
 'scripts/governed-github-command-provider.mjs',
 'rust/tests/unit/ci-cd/pinned_tool_images.rs',
 ...['original-release-workflow.yml','candidate-release-caller.yml','stage-source-coverage.json','candidate-output-bindings.json'].map(name=>protocolEvidence+name)
];
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
const sourceWitness=sourcePaths.map(relative=>{
 const source=join(repositoryRoot,relative),target=join(privateDirectory,relative);
 const status=lstatSync(source);
 assert.ok(status.isFile()&&!status.isSymbolicLink(),'regular immutable source fixture input required');
 const bytes=readFileSync(source),mode=status.mode&0o777;
 mkdirSync(dirname(target),{recursive:true});writeFileSync(target,bytes,{flag:'wx'});chmodSync(target,mode);
 assert.equal(realpathSync(target),target,'private fixture must not redirect to repository source');
 return {relative,sha256:digest(bytes),mode};
});
// Read-only installed dependencies are shared; all compiler/workflow source leaves are regular private copies.
symlinkSync(join(repositoryRoot,'node_modules'),join(privateDirectory,'node_modules'),'dir');
after(()=>{
 try {
  for(const witness of sourceWitness){
   const source=join(repositoryRoot,witness.relative);
   assert.equal(digest(readFileSync(source)),witness.sha256,'repository source bytes changed during private controls');
   assert.equal(lstatSync(source).mode&0o777,witness.mode,'repository source mode changed during private controls');
  }
 }finally{rmSync(privateDirectory,{recursive:true,force:true});}
});
const YAML=createRequire(root+'package.json')('yaml');
const evidence='experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
const run=args=>spawnSync(process.execPath,['scripts/native-staged-release-operation-view.mjs',...args],{cwd:root,encoding:'utf8',timeout:15000,maxBuffer:2*1024*1024});
test('native source bridge exposes checked original operations and actual permission/DAG surfaces',()=>{
 const result=run(['--source-view']);assert.equal(result.status,0,result.stderr);const view=JSON.parse(result.stdout);
 assert.equal(view.productionAuthority,false);assert.equal(view.inventory.operations,52);assert.equal(view.inventory.bindings,104);
 assert.equal(view.originalSource,readFileSync(root+evidence+'original-release-workflow.yml','utf8'));
 assert.deepEqual(view.physicalCaller,YAML.parse(readFileSync(root+'.github/workflows/release.yml','utf8')));
 assert.deepEqual(view.physicalStages,YAML.parse(readFileSync(root+'.github/workflows/release-staged.yml','utf8')));
});
test('native source bridge refuses unknown operation and caller-selected authority',()=>{assert.notEqual(run(['--source-view','foreign']).status,0);assert.notEqual(run(['--arbitrary-source']).status,0);});
for(const [name,file,mutate] of [
 ['caller Actions write','.github/workflows/release.yml',d=>{d.jobs['auto-release'].permissions.actions='write';}],
 ['prepare missing Actions read','.github/workflows/release-staged.yml',d=>{delete d.jobs['auto_prepare-source'].permissions.actions;}],
 ['deadline31','.github/workflows/release-staged.yml',d=>{d.jobs['manual_compile-release']['timeout-minutes']=31;}],
 ['Create missing predecessor','.github/workflows/release-staged.yml',d=>{d.jobs['auto_create-release'].needs.pop();}],
 ['duplicate descendant writer','.github/workflows/release-staged.yml',d=>{d.jobs['auto_compile-release'].concurrency={group:'formal-ai-repository-writes'};}],
 ['forged output source','.github/workflows/release-staged.yml',d=>{d.jobs['auto_create-release'].outputs.pages_sha='forged';}],
 ['prepare root credential removed','.github/workflows/release-staged.yml',document=>{document.jobs['auto_prepare-source'].steps[0].with['persist-credentials']=false;}],
 ['prepare root token spoofed','.github/workflows/release-staged.yml',document=>{document.jobs['manual_prepare-source'].steps[0].with.token='foreign';}],
 ['caller writer queue lost','.github/workflows/release.yml',document=>{delete document.jobs['manual-release'].concurrency.queue;}],
 ['prepare version operation changed','.github/workflows/release-staged.yml',document=>{document.jobs['auto_prepare-source'].steps.find(step=>step.id==='version').run='echo skipped';}],
])test('native source bridge refuses '+name,()=>{const original=readFileSync(root+file,'utf8');try{const document=YAML.parse(original);mutate(document);writeFileSync(root+file,YAML.stringify(document));assert.notEqual(run(['--source-view']).status,0);}finally{writeFileSync(root+file,original);}});

test('native source bridge conserves ordinary inline identity and its historical deadline floor',()=>{
 const file='.github/workflows/release.yml',original=readFileSync(root+file,'utf8');
 try {
  writeFileSync(root+file,readFileSync(root+evidence+'original-release-workflow.yml','utf8'));
  const result=run(['--source-view']);assert.equal(result.status,0,result.stderr);
  const view=JSON.parse(result.stdout);assert.equal(view.mode,'inline');
  for(const name of ['auto-release','manual-release'])assert.ok(view.physicalCaller.jobs[name]['timeout-minutes']>=60);
 }finally{writeFileSync(root+file,original);}
});

test('actual relocated image deadline and anonymous verifier retain operation-bound caps',()=>{
 const result=run(['--source-view']);assert.equal(result.status,0,result.stderr);const view=JSON.parse(result.stdout);
 for(const mode of ['auto','manual']) {
  const job=view.physicalStages.jobs[mode+'_publish-verify-images'];assert.equal(job['timeout-minutes'],30);
  const publish=job.steps.find(step=>step.name==='Publish Docker image to GHCR');assert.equal(publish['timeout-minutes'],21);
  assert.ok(publish['timeout-minutes']*100<=job['timeout-minutes']*70);
  for(const operand of ['TEST_BUDGET_ENFORCE=true','TEST_BUDGET_GRACE_SECONDS=5','TEST_BUDGET_POLL_SECONDS=1','run-with-budget-warning.sh 1250'])assert.ok(publish.run.includes(operand));
  const anonymous=job.steps.find(step=>step.name==='Verify anonymous access to the immutable published manifest');assert.equal(anonymous['timeout-minutes'],2);
  assert.ok(anonymous.run.includes('scripts/verify-anonymous-image-manifest.mjs'));
 }
});

test('checked logical writers retain source-bound physical root credential and ancestor lease',()=>{
 const result=run(['--source-view']);assert.equal(result.status,0,result.stderr);
 const view=JSON.parse(result.stdout);
 for(const [caller,producer] of [['auto-release','auto_prepare-source'],['manual-release','manual_prepare-source']]) {
  const owner=view.physicalCaller.jobs[caller];
  assert.equal(owner.uses,'./.github/workflows/release-staged.yml');
  assert.equal(owner.concurrency.group,'formal-ai-repository-writes');
  assert.equal(owner.concurrency.queue,'max');
  assert.equal(owner.permissions.contents,'write');
  const stage=view.physicalStages.jobs[producer];
  assert.equal(stage.concurrency,undefined);
  const checkout=stage.steps.find(step=>step.uses==='actions/checkout@v7' && !step.with?.path);
  assert.notEqual(checkout.with['persist-credentials'],false);
  assert.equal(checkout.with.token,'${{ secrets.GITHUB_TOKEN }}');
  assert.ok(stage.steps.find(step=>step.id==='version').run.includes('scripts/version-and-commit.rs'));
 }
 const native=readFileSync(root+'rust/tests/unit/ci-cd/pinned_tool_images.rs','utf8');
 assert.ok(native.includes('crate::ci_gates::staged_release_operations::release_operation_workflow()'));
 assert.ok(native.includes('crate::ci_gates::staged_release_operations::assert_release_delivery_budget(caller)'));
});
