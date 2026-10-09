import assert from 'node:assert/strict';
import test from 'node:test';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {execFileSync} from 'node:child_process';
import {checkReleaseFreshness,releaseContext} from '../../../scripts/release-source-freshness.mjs';
import {resolvePackageRelease} from '../../../scripts/resolve-package-release.mjs';
function fixture() {
 const directory=mkdtempSync(join(tmpdir(),'release-freshness-'));
 const remote=join(directory,'origin.git'),cwd=join(directory,'work'),state=join(directory,'state');mkdirSync(state);
 const run=(location,args)=>execFileSync('git',['-c','core.hooksPath=/dev/null',...args],{cwd:location,encoding:'utf8',stdio:['ignore','pipe','pipe']}).trim();
 run(directory,['init','--bare',remote]);run(directory,['clone',remote,cwd]);run(cwd,['checkout','-b','main']);run(cwd,['config','user.name','Fixture']);run(cwd,['config','user.email','fixture@example.invalid']);
 mkdirSync(join(cwd,'rust'));writeFileSync(join(cwd,'rust/Cargo.toml'),'[package]\nname = \"formal-ai\"\nversion = \"1.0.0\"\n');writeFileSync(join(cwd,'source.txt'),'original compiled source\n');run(cwd,['add','source.txt','rust/Cargo.toml']);run(cwd,['commit','-m','original']);run(cwd,['push','-u','origin','main']);
 const head=run(cwd,['rev-parse','HEAD']);
 const environment={GITHUB_ACTIONS:'true',GITHUB_REPOSITORY:'owner/repo',GITHUB_REF:'refs/heads/main',GITHUB_REF_NAME:'main',GITHUB_SHA:head,GITHUB_RUN_ID:'123',GITHUB_RUN_ATTEMPT:'2',GITHUB_WORKFLOW_REF:'owner/repo/.github/workflows/release.yml@refs/heads/main',GITHUB_EVENT_NAME:'push',GITHUB_JOB:'auto-release',RUNNER_TEMP:state};
 const event={ref:'refs/heads/main',after:head,repository:{full_name:'owner/repo'}};
 // Only transport remapping is injected: every source/tree/remote identity comes from actual Git.
 const gitRun=args=>args.join(' ')==='remote get-url origin'?'https://github.com/owner/repo.git':run(cwd,args);
 const check=phase=>checkReleaseFreshness({cwd,environment,event,phase,gitRun});
 const child=()=>{writeFileSync(join(cwd,'rust/Cargo.toml'),'[package]\nname = \"formal-ai\"\nversion = \"1.2.3\"\n');run(cwd,['add','rust/Cargo.toml']);check('before-commit');run(cwd,['commit','-m','chore: release v1.2.3']);return run(cwd,['rev-parse','HEAD']);};
 return {directory,cwd,remote,state,head,environment,event,run,gitRun,check,child,close:()=>rmSync(directory,{recursive:true,force:true})};
}
test('the full original resolver rejects a release rebased onto newer main; no ancestor fallback',()=>{
 const f=fixture();try {f.run(f.cwd,['checkout','-b','other']);writeFileSync(join(f.cwd,'source.txt'),'new source not tested by original run\n');f.run(f.cwd,['commit','-am','new source']);const newer=f.run(f.cwd,['rev-parse','HEAD']);writeFileSync(join(f.cwd,'version.txt'),'1.2.3\n');f.run(f.cwd,['add','version.txt']);f.run(f.cwd,['commit','-m','rebased version child']);const released=f.run(f.cwd,['rev-parse','HEAD']);
 const environment={EVENT:'workflow_run',REPOSITORY:'owner/repo',RUN_HEAD:f.head,RUN_BRANCH:'main',RUN_REPOSITORY:'owner/repo',RUN_CONCLUSION:'success'};
 const result=resolvePackageRelease(environment,path=>path.includes('/releases?')?[{tag_name:'v1.2.3',draft:false,prerelease:false}]:{sha:released,parents:[{sha:newer}]});assert.deepEqual(result,{tag:'',publish:false,build:false});
 }finally{f.close();}
});
test('physical exact main and recorded own version child survive a harmless retry or already completed push',()=>{
 const f=fixture();try {f.check('before-sync');const child=f.child();assert.equal(f.check('before-push').child,child);assert.equal(f.check('push-retry').child,child);f.run(f.cwd,['push','origin','HEAD:main']);assert.equal(f.check('push-retry').child,child);assert.equal(f.run(f.cwd,['rev-parse','HEAD^']),f.head);}finally{f.close();}
});
test('physical stale main refuses before mutation, before commit and before retry; changed child/tree refuse',()=>{
 for(const phase of ['before-sync','before-commit','push-retry']){const f=fixture();try {
 if(phase!=='before-sync')f.check('before-sync');if(phase==='push-retry'){f.child();f.check('before-push');}
 const other=join(f.directory,'other');f.run(f.directory,['clone',f.remote,other]);f.run(other,['checkout','main']);f.run(other,['config','user.name','Fixture']);f.run(other,['config','user.email','fixture@example.invalid']);writeFileSync(join(other,'source.txt'),'different main source\n');f.run(other,['commit','-am','advanced main']);f.run(other,['push','origin','main']);
 if(phase==='before-commit'){writeFileSync(join(f.cwd,'version.txt'),'1.2.3\n');f.run(f.cwd,['add','version.txt']);}assert.throws(()=>f.check(phase),/main advanced|main changed/);
 }finally{f.close();}}
 for(const mutation of ['tree','child','unrecorded']){const f=fixture();try{f.check('before-sync');f.child();
 if(mutation!=='unrecorded')f.check('before-push');
 if(mutation==='tree'){writeFileSync(join(f.cwd,'source.txt'),'spoof executable source\n');f.run(f.cwd,['commit','-am','changed source']);}
 else if(mutation==='child')f.run(f.cwd,['commit','--amend','-m','arbitrary replacement child']);
 assert.throws(()=>f.check('push-retry'),/tree|child|recorded/);}finally{f.close();}}
});
test('spoofed caller, identity, event, origin and recorded source fail closed',()=>{
 const f=fixture();try{
 const changes=[['GITHUB_ACTIONS','false'],['GITHUB_REF','refs/heads/other'],['GITHUB_REF_NAME','other'],['GITHUB_SHA','x'],['GITHUB_RUN_ID','0'],['GITHUB_RUN_ATTEMPT','0'],['GITHUB_REPOSITORY','fork/repo'],['GITHUB_WORKFLOW_REF','owner/repo/.github/workflows/other.yml@refs/heads/main'],['GITHUB_JOB','build'],['GITHUB_EVENT_NAME','pull_request']];
 for(const[key,value]of changes)assert.throws(()=>releaseContext({...f.environment,[key]:value},f.event));
 assert.throws(()=>releaseContext(f.environment,{...f.event,after:'f'.repeat(40)}));assert.throws(()=>releaseContext(f.environment,{...f.event,repository:{full_name:'fork/repo'}}));
 const dispatch={...f.environment,GITHUB_EVENT_NAME:'workflow_dispatch',GITHUB_JOB:'manual-release'};assert.equal(releaseContext(dispatch,{ref:'refs/heads/main',repository:{full_name:'owner/repo'},inputs:{release_mode:'instant'}}).head,f.head);assert.throws(()=>releaseContext(dispatch,{ref:'refs/heads/main',repository:{full_name:'owner/repo'},inputs:{release_mode:'checks'}}));
 assert.throws(()=>checkReleaseFreshness({cwd:f.cwd,environment:f.environment,event:f.event,phase:'before-sync',gitRun:args=>args.join(' ')==='remote get-url origin'?'https://github.com/fork/repo':f.gitRun(args)}));
 f.check('before-sync');const file=join(f.state,'formal-ai-release-freshness-123-2-auto-release.json');const record=JSON.parse(readFileSync(file));record.context.head='f'.repeat(40);writeFileSync(file,JSON.stringify(record));assert.throws(()=>f.check('before-commit'));
 }finally{f.close();}
});
test('original Rust producer invokes guards before mutations and keeps rebase confined to non-CI CLI',()=>{
 const source=readFileSync(new URL('../../../scripts/version-and-commit.rs',import.meta.url),'utf8');
 assert.ok(source.indexOf('guard_ci_release_source("before-sync")')<source.indexOf('// Get current version'));
 assert.match(source,/if !guarded_release\s*&& let Err\(e\) = sync_with_remote/);
 assert.ok(source.indexOf('guard_ci_release_source("before-commit")')<source.indexOf('exec("git", &["commit", "-m", &commit_msg])'));
 assert.ok(source.indexOf('guard_ci_release_source("before-push")')<source.indexOf('let max_push_attempts'));
 assert.match(source,/if guarded_release \{[\s\S]*?guard_ci_release_source\("push-retry"\)[\s\S]*?continue;\n\s+\}/);
 assert.ok(source.indexOf('guard_ci_release_source("push-retry")')<source.indexOf('exec("git", &["pull", "--rebase"'));
 assert.match(source,/env::var\("GITHUB_ACTIONS"\).as_deref\(\) != Ok\("true"\)/);
});

test('staged executable source or Cargo dependency drift cannot be recorded as the own version child',()=>{
 for(const kind of ['source','manifest']){const f=fixture();try{f.check('before-sync');writeFileSync(join(f.cwd,'rust/Cargo.toml'),'[package]\nname = "formal-ai"\nversion = "1.2.3"\n'+(kind==='manifest'?'[dependencies]\nevil = "1"\n':''));f.run(f.cwd,['add','rust/Cargo.toml']);if(kind==='source'){writeFileSync(join(f.cwd,'source.txt'),'different executable source\n');f.run(f.cwd,['add','source.txt']);}assert.throws(()=>f.check('before-commit'),/undeclared|manifest/);}finally{f.close();}}
});
