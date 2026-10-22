import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const root=fileURLToPath(new URL('../../../',import.meta.url));
// GitHub runner configure-apt-sources.sh blob b3f5b2b180df8eb39fcbba034f616e03a69e775c.
// c28 job119691094928: first Coq archive starts with ~5s left; second same-mirror attempt times out.
const upstream=fs.readFileSync(new URL('../fixtures/apt/hosted-ubuntu-mirrors.txt',import.meta.url),'utf8');
function run({optin='1',mode='timeout',text=upstream,missing=false,symlink=false,direct=false}={}){
  const directory=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-apt-mirrors-')));
  const mirror=path.join(directory,'apt-mirrors.txt'),target=path.join(directory,'foreign.txt');
  const sources=path.join(directory,'ubuntu.sources');
  const sourceText='Types: deb\nURIs: mirror+file:'+mirror+'\nSuites: noble noble-updates\nComponents: universe\nSigned-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg\n';
  fs.writeFileSync(sources,sourceText);
  if(symlink){fs.writeFileSync(target,text);fs.symlinkSync(target,mirror);}
  else if(!missing)fs.writeFileSync(mirror,text,{mode:0o640});
  const fake=path.join(directory,'apt-get'),calls=path.join(directory,'calls'),count=path.join(directory,'count');
  fs.writeFileSync(fake,`#!/usr/bin/env bash
set -eu
printf '%s\\n' "$*" >> "$FAKE_APT_CALLS"
for operand in "$@"; do
  if [ "$operand" = install ]; then
    seen=0; [ ! -f "$FAKE_APT_COUNT" ] || seen="$(cat "$FAKE_APT_COUNT")"
    seen=$((seen + 1)); printf '%s' "$seen" > "$FAKE_APT_COUNT"
    case "$FAKE_APT_MODE" in
      auth) echo 'repository signature could not be verified' >&2; exit 100 ;;
      persistent) exit 124 ;;
      actual-timeout) [ "$seen" -ne 1 ] || sleep 30 ;;
      timeout) [ "$seen" -ne 1 ] || exit 124 ;;
    esac
  fi
done
`,{mode:0o755});
  try{
    const arguments_=direct?['scripts/prepare-ubuntu-apt-mirror.sh',mirror]:['scripts/apt-install-with-retry.sh','coq','zstd'];
    const result=spawnSync('bash',arguments_,{
      cwd:root,encoding:'utf8',timeout:15000,maxBuffer:1048576,
      env:{...process.env,FORMAL_AI_APT_OFFICIAL_MIRROR_FALLBACK:optin,
        FORMAL_AI_APT_MIRROR_LIST:mirror,FORMAL_AI_APT_GET:fake,FORMAL_AI_APT_PRIVILEGE:'',
        FORMAL_AI_APT_ATTEMPTS:'2',FORMAL_AI_APT_ATTEMPT_SECONDS:'3',
        FORMAL_AI_APT_RETRY_DELAY_SECONDS:'0',TEST_BUDGET_SECONDS:'6',
        FAKE_APT_CALLS:calls,FAKE_APT_COUNT:count,FAKE_APT_MODE:mode}});
    assert.ifError(result.error);assert.equal(result.signal,null);
    assert.equal(fs.readFileSync(sources,'utf8'),sourceText,'signed source stanzas remain byte-exact');
    const observed=fs.existsSync(mirror)?fs.readFileSync(mirror,'utf8'):null;
    const invocations=fs.existsSync(calls)?fs.readFileSync(calls,'utf8').trim().split('\n'):[];
    assert(invocations.every(line=>line==='-o DPkg::Lock::Timeout=60 update -q'
      ||line==='-o DPkg::Lock::Timeout=60 install -y -q coq zstd'));
    assert.deepEqual(fs.readdirSync(directory).filter(name=>/\.before\.|\.next\./u.test(name)),[]);
    if(direct&&result.status===0){
      const inode=fs.statSync(mirror).ino;
      const repeated=spawnSync('bash',arguments_,{cwd:root,encoding:'utf8',timeout:5000,maxBuffer:1048576});
      assert.ifError(repeated.error);assert.equal(repeated.status,0);assert.equal(fs.statSync(mirror).ino,inode);
      assert.equal(fs.readFileSync(mirror,'utf8'),observed,'already prepared mirror is unchanged');
    }
    return {...result,observed,invocations,installs:fs.existsSync(count)?Number(fs.readFileSync(count,'utf8')):0,
      mode:missing||symlink?null:fs.statSync(mirror).mode&0o777,foreign:symlink?fs.readFileSync(target,'utf8'):null};
  }finally{fs.rmSync(directory,{recursive:true,force:true});}
}
test('second existing deadline-failed attempt selects exact official alternate, preserving signed sources/mode/cache',()=>{
  assert.equal(Buffer.byteLength(upstream),144);
  const result=run();assert.equal(result.status,0,result.stdout+result.stderr);
  assert.equal(result.installs,2);assert.equal(result.invocations.length,4);assert.equal(result.mode,0o640);
  assert.equal(result.observed,upstream.replace('http://azure.archive.ubuntu.com/ubuntu/','https://archive.ubuntu.com/ubuntu/'));
});
test('default callers retain their mirror bytes and existing attempts',()=>{
  const result=run({optin:'0'});assert.equal(result.status,0);assert.equal(result.observed,upstream);assert.equal(result.installs,2);
});
test('first successful attempt never changes the opted-in mirror list',()=>{
  const result=run({mode:'success'});assert.equal(result.status,0);assert.equal(result.installs,1);assert.equal(result.observed,upstream);
});
test('signature/authentication failure never opens mirror failover or bypasses apt verdict',()=>{
  const result=run({mode:'auth'});assert.equal(result.status,100);assert.equal(result.installs,2);assert.equal(result.observed,upstream);
});
test('persistent deadline failure remains nonzero with no third attempt',()=>{
  const result=run({mode:'persistent'});assert.equal(result.status,124);assert.equal(result.installs,2);
});
test('real child deadline reaches the existing alternate without another retry',()=>{
  const result=run({mode:'actual-timeout'});assert.equal(result.status,0);assert.equal(result.installs,2);
  assert.match(result.stderr,/Attempt 1 exited 124/u);assert.match(result.stdout,/official Ubuntu archive HTTPS alternate/u);
});
test('unterminated mirror input refuses before normalization can change unrelated bytes',()=>{
  const text=upstream.trimEnd();const result=run({text});assert.equal(result.status,2);
  assert.equal(result.installs,1);assert.equal(result.observed,text);
});
test('foreign hosts, Azure lookalikes, priorities and comments are not rewritten',()=>{
  const text='https://other.invalid/ubuntu/\tpriority:1\nhttp://azure.archive.ubuntu.com/ubuntu-extra/\tpriority:2\n# http://azure.archive.ubuntu.com/ubuntu/\n';
  const result=run({text});assert.equal(result.status,0);assert.equal(result.observed,text);
});
for(const options of [{missing:true},{symlink:true}])test('absent or symlink mirror list refuses alternate without replacing foreign bytes '+JSON.stringify(options),()=>{
  const result=run(options);assert.equal(result.status,2);assert.equal(result.installs,1);
  if(options.symlink)assert.equal(result.foreign,upstream);else assert.equal(result.observed,null);
});
test('direct browser preparation shares the exact signed-source-preserving mirror transform without APT calls',()=>{
  const result=run({direct:true});assert.equal(result.status,0,result.stdout+result.stderr);
  assert.equal(result.observed,upstream.replace('http://azure.archive.ubuntu.com/ubuntu/','https://archive.ubuntu.com/ubuntu/'));
  assert.equal(result.mode,0o640);assert.equal(result.installs,0);assert.deepEqual(result.invocations,[]);
});
for(const options of [{missing:true},{symlink:true},{text:upstream.trimEnd()}])test('direct browser mirror preparation fails closed '+JSON.stringify(options),()=>{
  const result=run({...options,direct:true});assert.equal(result.status,2);assert.equal(result.installs,0);
  if(options.symlink)assert.equal(result.foreign,upstream);
});
