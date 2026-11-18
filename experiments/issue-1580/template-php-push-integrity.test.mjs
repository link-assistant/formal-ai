// Run the pinned PHP release entrypoint against bounded local Git remotes.
// Original push failures and a disposable candidate are exercised without network.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {cpSync,existsSync,mkdirSync,mkdtempSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const fixture=fileURLToPath(new URL('./fixtures/php-template/',import.meta.url));
const provenance=JSON.parse(readFileSync(join(fixture,'source.json'),'utf8'));
for(const item of provenance.files){
  const raw=readFileSync(join(fixture,item.path));
  assert.equal(createHash('sha1').update(`blob ${raw.length}\0`).update(raw).digest('hex'),item.gitBlob);
  assert.equal(createHash('sha256').update(raw).digest('hex'),item.sha256);
}

function releaseFixture(t,{mode,rejection,repaired}){
  const temporary=mkdtempSync(join(tmpdir(),'issue-1580-php-release-'));
  t.after(()=>rmSync(temporary,{recursive:true,force:true}));
  const cwd=join(temporary,'checkout');mkdirSync(cwd);
  const remote=join(temporary,'remote.git');
  const hooks=join(temporary,'hooks');mkdirSync(hooks);
  const env={...process.env};
  for(const name of Object.keys(env))if(name.startsWith('GIT_'))delete env[name];
  Object.assign(env,{GIT_CONFIG_NOSYSTEM:'1',GIT_CONFIG_GLOBAL:'/dev/null',GIT_TERMINAL_PROMPT:'0',
    GITHUB_OUTPUT:join(temporary,'outputs'),GITHUB_ACTIONS:'',GITHUB_REPOSITORY:''});
  const git=(...args)=>{
    const result=spawnSync('git',args,{cwd,env,encoding:'utf8',timeout:5000,maxBuffer:131072});
    assert.equal(result.error,undefined,String(result.error));
    assert.equal(result.status,0,result.stderr);
    return result.stdout.trim();
  };
  cpSync(join(fixture,'scripts'),join(cwd,'scripts'),{recursive:true});
  mkdirSync(join(cwd,'changelog.d'));
  writeFileSync(join(cwd,'composer.json'),JSON.stringify({name:'fixture/package',version:'1.2.3'})+'\n');
  writeFileSync(join(cwd,'CHANGELOG.md'),'# Changelog\n\n<!-- changelog-insert-here -->\n');
  writeFileSync(join(cwd,'changelog.d','fix.md'),'---\nbump: patch\n---\n### Fixed\n- Release integrity fixture.\n');
  if(repaired){
    const path=join(cwd,'scripts/src/Git.php');
    const source=readFileSync(path,'utf8');
    const boundary="return Process::run(['git', 'push',";
    assert.equal(source.split(boundary).length,3,'both push status boundaries');
    writeFileSync(path,source.replaceAll(boundary,"return Process::mustRun(['git', 'push',"));
  }
  git('init','--quiet','--initial-branch=fixture');
  git('config','user.email','fixture@example.invalid');git('config','user.name','Release fixture');
  git('config','commit.gpgsign','false');git('config','tag.gpgsign','false');
  git('config','core.autocrlf','false');git('config','core.hooksPath',hooks);
  git('add','.');git('commit','--quiet','-m','release input');
  git('init','--quiet','--bare',remote);
  git('remote','add','origin',remote);git('push','--quiet','--set-upstream','origin','fixture');
  const before=git('rev-parse','HEAD');
  if(rejection){
    const rejected=rejection==='branch'?'refs/heads/*':'refs/tags/*';
    writeFileSync(join(remote,'hooks','update'),`#!/bin/sh\ncase "$1" in ${rejected}) echo "fixture rejects ${rejection}" >&2; exit 23;; esac\nexit 0\n`,{mode:0o755});
  }
  const args=['-d','memory_limit=32M',join(cwd,'scripts/version-and-commit.php'),`--mode=${mode}`];
  if(mode==='instant')args.push('--bump=patch');
  const result=spawnSync('php',args,{cwd,env,encoding:'utf8',timeout:5000,maxBuffer:131072});
  assert.equal(result.error,undefined,String(result.error));
  const output=existsSync(env.GITHUB_OUTPUT)?readFileSync(env.GITHUB_OUTPUT,'utf8'):'';
  const remoteHead=git('--git-dir',remote,'rev-parse','refs/heads/fixture');
  const remoteTags=git('ls-remote','--tags','origin');
  return {cwd,remote,git,before,result,output,remoteHead,remoteTags};
}

for(const mode of ['changeset','instant'])for(const rejection of ['branch','tag']){
  test(`PHP ${mode} release reports success after rejected ${rejection} publication`,{timeout:20000},t=>{
    const f=releaseFixture(t,{mode,rejection,repaired:false});
    assert.equal(f.result.status,0,f.result.stderr);
    assert.match(f.output,/^new_version=1\.2\.4$/mu);
    assert.match(f.output,/^version_committed=true$/mu);
    const head=f.git('rev-parse','HEAD');
    assert.notEqual(head,f.before,'release commit exists locally');
    if(rejection==='branch'){
      assert.equal(f.remoteHead,f.before,'branch update was rejected');
      assert.match(f.remoteTags,/refs\/tags\/v1\.2\.4/u,'tag push still ran after branch rejection');
      assert.equal(f.git('--git-dir',f.remote,'rev-parse','v1.2.4^{}'),head);
    }else{
      assert.equal(f.remoteHead,head,'branch update succeeded');
      assert.equal(f.remoteTags,'','release tag was rejected');
    }
  });

  test(`PHP ${mode} candidate propagates rejected ${rejection} publication`,{timeout:20000},t=>{
    const f=releaseFixture(t,{mode,rejection,repaired:true});
    assert.notEqual(f.result.status,0,f.result.stdout);
    assert.match(f.result.stderr,/Command failed.*git push/us);
    assert.doesNotMatch(f.output,/^version_committed=true$/mu);
    assert.equal(f.remoteTags,'','no release tag reached the remote');
    if(rejection==='branch')assert.equal(f.remoteHead,f.before);
    else assert.equal(f.remoteHead,f.git('rev-parse','HEAD'),'earlier branch push remains visible');
  });
}

test('PHP successful branch and tag publication works before and after the candidate',{timeout:30000},t=>{
  for(const mode of ['changeset','instant'])for(const repaired of [false,true]){
    const f=releaseFixture(t,{mode,repaired});
    assert.equal(f.result.status,0,f.result.stderr);
    assert.match(f.output,/^version_committed=true$/mu);
    const head=f.git('rev-parse','HEAD');
    assert.notEqual(head,f.before);
    assert.equal(f.remoteHead,head);
    assert.equal(f.git('--git-dir',f.remote,'rev-parse','v1.2.4^{}'),head);
    assert.equal(JSON.parse(f.git('--git-dir',f.remote,'show','v1.2.4:composer.json')).version,'1.2.4');
    if(mode==='changeset')assert.equal(existsSync(join(f.cwd,'changelog.d','fix.md')),false);
  }
});
