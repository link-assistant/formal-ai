// Keep guarded release source within the exact main CI run and its own recorded version child.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {readFileSync,writeFileSync,existsSync,realpathSync} from 'node:fs';
import {join,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
const commit = value => assert.match(value, /^[a-f0-9]{40}$/u);
export function releaseContext(environment, event) {
  assert.equal(environment.GITHUB_ACTIONS, 'true');
  assert.match(environment.GITHUB_REPOSITORY, /^[\w.-]+\/[\w.-]+$/u);
  assert.equal(environment.GITHUB_REF, 'refs/heads/main');
  assert.equal(environment.GITHUB_REF_NAME, 'main');
  commit(environment.GITHUB_SHA);
  assert.match(environment.GITHUB_RUN_ID, /^[1-9][0-9]*$/u);
  assert.match(environment.GITHUB_RUN_ATTEMPT, /^[1-9][0-9]*$/u);
  assert.equal(environment.GITHUB_WORKFLOW_REF, environment.GITHUB_REPOSITORY + '/.github/workflows/release.yml@refs/heads/main');
  const job = environment.GITHUB_JOB;
  if (environment.GITHUB_EVENT_NAME === 'push') {
    assert.equal(job, 'auto-release');
    assert.equal(event.ref, 'refs/heads/main');
    assert.equal(event.after, environment.GITHUB_SHA);
    assert.equal(event.repository.full_name, environment.GITHUB_REPOSITORY);
  } else {
    assert.equal(environment.GITHUB_EVENT_NAME, 'workflow_dispatch');
    assert.equal(job, 'manual-release');
    assert.equal(event.inputs.release_mode, 'instant');
    assert.ok(['main','refs/heads/main'].includes(event.ref),'dispatch ref must name main');
    assert.equal(event.repository.full_name, environment.GITHUB_REPOSITORY);
  }
  return {repository:environment.GITHUB_REPOSITORY,head:environment.GITHUB_SHA,run:environment.GITHUB_RUN_ID,attempt:environment.GITHUB_RUN_ATTEMPT,job};
}
export function checkReleaseFreshness({cwd,environment,event,phase,stateDirectory=environment.RUNNER_TEMP,gitRun=null}) {
  const context=releaseContext(environment,event);
  assert.ok(['before-sync','before-commit','before-push','push-retry'].includes(phase));
  assert.ok(stateDirectory && resolve(stateDirectory)!==resolve(cwd),'private runner state required');
  const git = args => (gitRun ?? ((values)=>execFileSync('git',values,{cwd,encoding:'utf8',timeout:30000})))(args).trim();
  assert.equal(git(['remote','get-url','origin']).replace(/\.git$/u,''),'https://github.com/'+context.repository,'release repository differs');
  const refs=git(['ls-remote','origin','refs/heads/main']).split('\n').filter(Boolean).map(line=>line.split(/\s+/u));
  assert.equal(refs.length,1); assert.equal(refs[0][1],'refs/heads/main');
  const remote=refs[0][0];commit(remote);
  const head=git(['rev-parse','HEAD']);commit(head);
  const tree=git(['rev-parse','HEAD^{tree}']);commit(tree);
  const file=join(stateDirectory,`formal-ai-release-freshness-${context.run}-${context.attempt}-${context.job}.json`);
  if(phase==='before-sync') {
    assert.equal(head,context.head,'checkout differs from tested main run');
    assert.equal(remote,context.head,'main advanced beyond tested release source; fresh main run must release');
    assert.equal(git(['status','--porcelain','--untracked-files=no']),'','release checkout has tracked modifications');
    assert.equal(existsSync(file),false,'release state already exists');
    const record={schema:'guarded-release-source/v1',context,originalTree:tree,expectedTree:null,child:null};
    writeFileSync(file,JSON.stringify(record)+'\n',{flag:'wx',mode:0o600});return record;
  }
  const record=JSON.parse(readFileSync(file,'utf8'));
  assert.equal(record.schema,'guarded-release-source/v1');assert.deepEqual(record.context,context);commit(record.originalTree);
  if(phase==='before-commit') {
    assert.equal(head,context.head,'version parent drifted');assert.equal(tree,record.originalTree);
    assert.equal(remote,context.head,'main advanced before version commit');
    assert.equal(record.expectedTree,null);assert.equal(record.child,null);
    const changed=git(['diff','--cached','--name-only','-z']).split('\0').filter(Boolean);
    const fixed=new Set(['rust/Cargo.toml','rust/Cargo.lock','CHANGELOG.md','data/meta/self-hosting-ledger.lino','docs/status.md','docs/benchmarks.md','README.md','docs/case-studies/issue-711/fragment-release-map.tsv']);
    assert.ok(changed.includes('rust/Cargo.toml'),'version child must update actual Cargo manifest');
    for(const path of changed)assert.ok(fixed.has(path)||/^changelog\.d\/[^/]+\.md$/u.test(path)||/^docs\/changelog\/[^/]+\.md$/u.test(path),'undeclared version-child source change: '+path);
    const originalManifest=git(['show',context.head+':rust/Cargo.toml']);
    const nextManifest=git(['show',':rust/Cargo.toml']);
    const version=/^(version\s*=\s*")[^"]+(".*)$/mu;
    const beforeVersion=version.exec(originalManifest),afterVersion=version.exec(nextManifest);
    assert.ok(beforeVersion&&afterVersion,'actual Cargo package version missing');
    assert.match(afterVersion[0],/^version\s*=\s*"(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)"/u);
    assert.equal(originalManifest.replace(version,'$1<VERSION>$2'),nextManifest.replace(version,'$1<VERSION>$2'),'Cargo manifest may change only the original version projection');
    if(changed.includes('rust/Cargo.lock')) {
      const pattern=/(\[\[package\]\]\s*\nname\s*=\s*"formal-ai"\s*\nversion\s*=\s*")[^"]+(")/u;
      const originalLock=git(['show',context.head+':rust/Cargo.lock']),nextLock=git(['show',':rust/Cargo.lock']);
      assert.ok(pattern.test(originalLock)&&pattern.test(nextLock));
      assert.equal(originalLock.replace(pattern,'$1<VERSION>$2'),nextLock.replace(pattern,'$1<VERSION>$2'),'Cargo lock dependencies may not change in version child');
    }
    record.expectedTree=git(['write-tree']);commit(record.expectedTree);
    assert.notEqual(record.expectedTree,record.originalTree,'version child must change recorded tree');
  } else {
    commit(record.expectedTree);assert.equal(tree,record.expectedTree,'version child differs from independently recorded index');
    const parents=git(['rev-list','--parents','-n','1','HEAD']).split(/\s+/u);
    assert.deepEqual(parents,[head,context.head],'only own single version child of tested source may be pushed');
    assert.equal(git(['status','--porcelain','--untracked-files=no']),'','version child has tracked modifications');
    if(record.child!==null)assert.equal(head,record.child,'recorded own version child changed');
    if(phase==='push-retry')assert.notEqual(record.child,null,'initial own version child was not recorded');
    assert.ok(remote===context.head || (record.child!==null && remote===record.child),'main changed during release push; do not rebase or tag');
    record.child=head;
  }
  writeFileSync(file,JSON.stringify(record)+'\n',{mode:0o600});return record;
}
export async function main(environment=process.env) {
  const event=JSON.parse(readFileSync(environment.GITHUB_EVENT_PATH,'utf8'));
  let checkedEnvironment=environment;
  if(environment.GITHUB_ACTIONS==='true' && !['auto-release','manual-release'].includes(environment.GITHUB_JOB)) {
    assert.ok(environment===process.env,'staged context requires actual current process host');
    const trusted=resolve(process.cwd(),'_protocol');
    assert.equal(execFileSync('git',['-C',trusted,'rev-parse','HEAD'],{encoding:'utf8',timeout:30000}).trim(),environment.GITHUB_SHA,'immutable protocol checkout differs');
    execFileSync('git',['-C',trusted,'diff','--exit-code','HEAD'],{encoding:'utf8',timeout:30000});
    for(const file of ['scripts/maintained-staged-authority.mjs','scripts/staged-caller-authority.mjs','scripts/governed-github-command-provider.mjs','scripts/generate-staged-release.mjs','scripts/lib/ci-speed-workflows.mjs','scripts/release-source-freshness.mjs','package.json','bun.lock','.bun-version']) {
      const entry=execFileSync('git',['-C',trusted,'ls-tree','HEAD','--',file],{encoding:'utf8',timeout:5000}).trim();
      const [header,declaredPath]=entry.split('\t');assert.match(header,/^100644 blob [a-f0-9]{40}$/u);assert.equal(declaredPath,file);
      const immutable=execFileSync('git',['-C',trusted,'show','HEAD:'+file],{timeout:5000,maxBuffer:4194304});
      assert.deepEqual(readFileSync(join(trusted,file)),immutable,'trusted protocol tracked bytes differ');
    }
    const adapter=await import(pathToFileURL(join(trusted,'scripts/maintained-staged-authority.mjs')).href);
    const receipt=adapter.compileDeployedStagedAuthority();
    const context=await adapter.observeCurrentStepContext(receipt,'version');
    checkedEnvironment={...environment,GITHUB_JOB:context.job};
  }
  const record=checkReleaseFreshness({cwd:process.cwd(),environment:checkedEnvironment,event,phase:process.argv[2]});
  console.log(JSON.stringify({schema:record.schema,context:record.context,originalTree:record.originalTree,expectedTree:record.expectedTree,child:record.child,publication:false}));
}
if(process.argv[1] && import.meta.url===pathToFileURL(realpathSync(process.argv[1])).href)main().catch(error=>{console.error(error);process.exitCode=1;});
