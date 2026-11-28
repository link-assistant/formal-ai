// Observe the unchanged pinned audit entrypoint; candidate changes stay disposable.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdirSync,mkdtempSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const fixture=fileURLToPath(new URL('./fixtures/python-template/',import.meta.url));
const provenance=JSON.parse(readFileSync(join(fixture,'source.json'),'utf8'));
const entry=provenance.files.find(item=>item.path==='scripts/audit-npm-locks.sh');
assert(entry,'retain the full upstream audit source identity');
const bytes=readFileSync(join(fixture,entry.path));
assert.equal(createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex'),entry.gitBlob);
assert.equal(createHash('sha256').update(bytes).digest('hex'),entry.sha256);

function auditFixture(t,{candidate=false,mode='real'}={}){
  const cwd=mkdtempSync(join(tmpdir(),'issue-1580-python-audit-'));
  t.after(()=>rmSync(cwd,{recursive:true,force:true}));
  const bin=join(cwd,'bin');mkdirSync(bin);
  const calls=join(cwd,'audit-calls');writeFileSync(calls,'');
  const env={...process.env,PATH:bin+':'+process.env.PATH,AUDIT_FIXTURE_CALLS:calls,
    AUDIT_FIXTURE_GIT_MODE:mode,GIT_CONFIG_NOSYSTEM:'1',GIT_CONFIG_GLOBAL:'/dev/null'};
  for(const name of ['GIT_DIR','GIT_COMMON_DIR','GIT_INDEX_FILE','GIT_WORK_TREE'])delete env[name];
  const git=(...args)=>{
    const result=spawnSync('git',args,{cwd,env,encoding:'utf8',timeout:5000,maxBuffer:65536});
    assert.ifError(result.error);assert.equal(result.status,0,result.stderr);
  };
  git('init','--quiet','--initial-branch=main');
  const locks=['.github/actions/artifact/package-lock.json','second action/package-lock.json'];
  for(const lock of [...locks,'untracked/package-lock.json']){
    mkdirSync(dirname(join(cwd,lock)),{recursive:true});writeFileSync(join(cwd,lock),'{}\n');
  }
  if(mode==='real')git('add','--',...locks);
  if(mode==='unavailable'||mode==='partial')writeFileSync(join(bin,'git'),String.raw`#!/usr/bin/env bash
if [[ "$AUDIT_FIXTURE_GIT_MODE" == partial ]]; then printf '.github/actions/artifact/package-lock.json\n'; fi
echo 'fatal: fixture Git inventory failed' >&2
exit 128
`,{mode:0o755});
  writeFileSync(join(bin,'npm'),String.raw`#!/usr/bin/env bash
printf '%s\t%s\n' "$PWD" "$*" >> "$AUDIT_FIXTURE_CALLS"
if [[ "$AUDIT_FIXTURE_NPM_STATUS" != 0 ]]; then
  echo 'fixture advisory verdict' >&2
  exit 1
fi
echo 'fixture clean audit verdict'
`,{mode:0o755});
  let source=bytes.toString('utf8');
  if(candidate){
    const boundary="mapfile -t locks < <(git ls-files -- ':(glob)**/package-lock.json')";
    assert.equal(source.split(boundary).length,2);
    source=source.replace(boundary,"tracked_locks=\"$(git ls-files -- ':(glob)**/package-lock.json')\"\nmapfile -t locks < <(printf '%s' \"$tracked_locks\")");
  }
  const script=join(cwd,'audit-npm-locks.sh');writeFileSync(script,source);
  const invoke=({list=false,advisory=false,level='moderate'}={})=>{
    writeFileSync(calls,'');
    const result=spawnSync('bash',[script,...(list?['--list']:[])],{cwd,
      env:{...env,NPM_AUDIT_LEVEL:level,AUDIT_FIXTURE_NPM_STATUS:advisory?'1':'0'},
      encoding:'utf8',timeout:5000,maxBuffer:65536});
    assert.ifError(result.error);assert.equal(result.signal,null);
    return {...result,calls:readFileSync(calls,'utf8')};
  };
  return {cwd,locks,invoke};
}

test('Python pinned npm audit accepts failed or partial Git inventory as a successful audit',{timeout:10000},t=>{
  for(const mode of ['unavailable','partial']){
    const f=auditFixture(t,{mode});
    for(const list of [false,true]){
      const result=f.invoke({list});
      assert.equal(result.status,0,result.stderr);
      assert.match(result.stderr,/fatal: fixture Git inventory failed/u);
      if(list)assert.equal(result.calls,'');
      else if(mode==='unavailable'){
        assert.match(result.stdout,/No package-lock.json is tracked; nothing to audit/u);
        assert.equal(result.calls,'');
      }else{
        assert.match(result.stdout,/Audited 1 npm lockfile\(s\)/u);
        assert.equal(result.calls.split('\n').filter(Boolean).length,1);
      }
    }
  }
});

test('Python candidate rejects failed and partial inventory before listing or auditing',{timeout:10000},t=>{
  for(const mode of ['unavailable','partial']){
    const f=auditFixture(t,{candidate:true,mode});
    for(const list of [false,true]){
      const result=f.invoke({list});
      assert.equal(result.status,128,result.stdout+result.stderr);
      assert.match(result.stderr,/fatal: fixture Git inventory failed/u);
      assert.equal(result.calls,'');assert.equal(result.stdout,'');
    }
  }
});

test('Python audit retains every tracked lock, spaces, advisory failures and real empty inventory',{timeout:15000},t=>{
  for(const candidate of [false,true]){
    const f=auditFixture(t,{candidate});
    const list=f.invoke({list:true});assert.equal(list.status,0,list.stderr);
    assert.deepEqual(list.stdout.trimEnd().split('\n'),f.locks);assert.equal(list.calls,'');
    for(const advisory of [false,true]){
      const result=f.invoke({advisory,level:'high'});
      assert.equal(result.status,advisory?1:0,result.stdout+result.stderr);
      assert.deepEqual(result.calls.trimEnd().split('\n'),f.locks.map(lock=>
        join(f.cwd,dirname(lock))+'\taudit --package-lock-only --audit-level=high'));
      if(advisory)assert.match(result.stdout,/::error file=second action\/package-lock.json/u);
      else assert.match(result.stdout,/Audited 2 npm lockfile\(s\)/u);
    }
    const empty=auditFixture(t,{candidate,mode:'empty'}).invoke();
    assert.equal(empty.status,0,empty.stderr);assert.equal(empty.calls,'');
    assert.match(empty.stdout,/No package-lock.json is tracked; nothing to audit/u);
  }
});
