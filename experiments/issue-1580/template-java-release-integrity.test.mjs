// Execute pinned upstream scripts against disposable Git repositories and local
// bare remotes. No hosted workflow, GitHub API, Maven build or network is used.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const fixture=fileURLToPath(new URL('./fixtures/java-template/',import.meta.url));
const provenance=JSON.parse(readFileSync(join(fixture,'source.json'),'utf8'));
for(const item of provenance.files){
  const bytes=readFileSync(join(fixture,item.path));
  assert.equal(createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex'),item.gitBlob);
  assert.equal(createHash('sha256').update(bytes).digest('hex'),item.sha256);
}

function releaseFixture(t,{rejectCommit,repaired,mode}){
  const temporary=mkdtempSync(join(tmpdir(),'issue-1580-java-release-'));
  t.after(()=>rmSync(temporary,{recursive:true,force:true}));
  const cwd=join(temporary,'checkout');mkdirSync(cwd);
  const remote=join(temporary,'remote.git');
  const hooks=join(temporary,'hooks');mkdirSync(hooks);
  const env={...process.env};
  for(const name of Object.keys(env))if(name.startsWith('GIT_'))delete env[name];
  Object.assign(env,{
    GIT_CONFIG_NOSYSTEM:'1',GIT_CONFIG_GLOBAL:'/dev/null',GIT_TERMINAL_PROMPT:'0',
    GITHUB_OUTPUT:join(temporary,'outputs'),GITHUB_ACTIONS:'',GITHUB_REPOSITORY:'',
  });
  const git=(...args)=>{
    const result=spawnSync('git',args,{cwd,env,encoding:'utf8',timeout:5000,maxBuffer:131072});
    assert.equal(result.error,undefined,String(result.error));
    assert.equal(result.status,0,result.stderr);
    return result.stdout.trim();
  };
  cpSync(join(fixture,'scripts'),join(cwd,'scripts'),{recursive:true});
  mkdirSync(join(cwd,'.changeset'));
  const javaDirectory=join(cwd,'src/main/java/com/linkfoundation/mypackage');
  mkdirSync(javaDirectory,{recursive:true});
  writeFileSync(join(cwd,'pom.xml'),'<project><version>1.2.3</version></project>\n');
  writeFileSync(join(javaDirectory,'MyPackage.java'),'class MyPackage { static final String VERSION = "1.2.3"; }\n');
  writeFileSync(join(cwd,'CHANGELOG.md'),'# Changelog\n\n## [1.2.3]\n\nPrevious release.\n');
  writeFileSync(join(cwd,'.changeset','fix.md'),"---\n'my-package': patch\n---\n\nRelease integrity fixture.\n");
  const script=join(cwd,'scripts/version-and-commit.mjs');
  if(repaired){
    const source=readFileSync(script,'utf8');
    const boundary='exec(`git commit -m "${commitMessage}" || echo "Nothing to commit"`);';
    assert.equal(source.split(boundary).length,2,'one swallowed commit failure boundary');
    writeFileSync(script,source.replace(boundary,'exec(`git commit -m "${commitMessage}"`);'));
  }
  git('init','--quiet','--initial-branch=fixture');
  git('config','user.email','fixture@example.invalid');
  git('config','user.name','Release fixture');
  git('config','commit.gpgsign','false');
  git('config','tag.gpgsign','false');
  git('config','core.autocrlf','false');
  git('config','core.hooksPath',hooks);
  git('add','.');git('commit','--quiet','-m','release input');
  git('init','--quiet','--bare',remote);
  git('remote','add','origin',remote);git('push','--quiet','--set-upstream','origin','fixture');
  const before=git('rev-parse','HEAD');
  if(rejectCommit)writeFileSync(join(hooks,'pre-commit'),'#!/bin/sh\nprintf "fixture rejects release commit\\n" >&2\nexit 23\n',{mode:0o755});
  const args=['--max-old-space-size=64',script,'--mode',mode];
  if(mode==='instant')args.push('--bump-type','patch');
  const result=spawnSync(process.execPath,args,{cwd,env,encoding:'utf8',timeout:5000,maxBuffer:131072});
  assert.equal(result.error,undefined,String(result.error));
  const output=existsSync(env.GITHUB_OUTPUT)?readFileSync(env.GITHUB_OUTPUT,'utf8'):'';
  return {cwd,remote,git,before,result,output};
}

for(const mode of ['changeset','instant']){
  test(`Java ${mode} release suppresses a failed commit and publishes a tag on the old version`,{timeout:15000},t=>{
    const f=releaseFixture(t,{mode,rejectCommit:true,repaired:false});
    assert.equal(f.result.status,0,f.result.stderr);
    assert.match(f.result.stdout,/Nothing to commit/u);
    assert.match(f.output,/^released=true$/mu);
    assert.equal(f.git('rev-parse','HEAD'),f.before,'no release commit was created');
    assert.equal(f.git('rev-parse','v1.2.4^{}'),f.before,'new version tag points at the previous commit');
    assert.match(readFileSync(join(f.cwd,'pom.xml'),'utf8'),/<version>1\.2\.4<\/version>/u);
    assert.match(f.git('show','v1.2.4:pom.xml'),/<version>1\.2\.3<\/version>/u);
    assert.equal(f.git('--git-dir',f.remote,'rev-parse','v1.2.4^{}'),f.before,'the wrong tag reached the local remote');
    assert.match(f.git('--git-dir',f.remote,'show','v1.2.4:pom.xml'),/<version>1\.2\.3<\/version>/u);
  });

  test(`Java ${mode} release propagates commit failure after the candidate repair`,{timeout:15000},t=>{
    const f=releaseFixture(t,{mode,rejectCommit:true,repaired:true});
    assert.equal(f.result.status,1,f.result.stdout);
    assert.doesNotMatch(f.output,/^released=true$/mu);
    assert.equal(f.git('rev-parse','HEAD'),f.before);
    assert.equal(f.git('tag','--list'),'','failed commit must not create a release tag');
    assert.equal(f.git('ls-remote','--tags','origin'),'','failed commit must not publish a release tag');
    assert.equal(f.git('--git-dir',f.remote,'rev-parse','refs/heads/fixture'),f.before);
  });
}

test('Java successful commit still publishes the updated version before and after candidate repair',{timeout:15000},t=>{
  for(const mode of ['changeset','instant'])for(const repaired of [false,true]){
    const f=releaseFixture(t,{mode,repaired,rejectCommit:false});
    assert.equal(f.result.status,0,`${f.result.stdout}\n${f.result.stderr}`);
    assert.match(f.output,/^released=true$/mu);
    const head=f.git('rev-parse','HEAD');
    assert.notEqual(head,f.before);
    assert.equal(f.git('rev-parse','v1.2.4^{}'),head);
    assert.equal(f.git('--git-dir',f.remote,'rev-parse','v1.2.4^{}'),head);
    assert.match(f.git('--git-dir',f.remote,'show','v1.2.4:pom.xml'),/<version>1\.2\.4<\/version>/u);
    assert.equal(existsSync(join(f.cwd,'.changeset','fix.md')),false);
  }
});
