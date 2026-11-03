import { createHash } from 'node:crypto';
import { trackedTestInputIdentity } from '../../../scripts/lib/native-response-identity.mjs';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { join } from 'node:path';
import { tmpdir } from 'node:os';
import { execFileSync } from 'node:child_process';
import { test } from 'node:test';
import { nativeInputIdentity } from '../../../scripts/native-release-source.mjs';
import { immutableSourceWitness, nativeDerivationRows, verifyPhysicalDerivationInputs } from '../../../scripts/lib/qualified-native-data-collector.mjs';
function fixture() {
  const directory = fs.mkdtempSync(join(tmpdir(), 'formal-ai-native-root-'));
  const env = {
    ...process.env,
    GIT_CONFIG_NOSYSTEM: '1',
    GIT_CONFIG_GLOBAL: '/dev/null',
    GIT_TERMINAL_PROMPT: '0'
  };
  const git = (...args) => execFileSync('git', args, {
    cwd: directory,
    env,
    encoding: 'utf8',
    timeout: 5000
  }).trim();
  const files = {
    'rust/Cargo.toml': '[package]\nname="heldout"\nversion="0.1.0"\n',
    'rust/Cargo.lock': '# held-out lock data\n',
    'rust/src/lib.rs': 'pub fn heldout() {}\n',
    'rust/embedded/heldout.lino': 'heldout compilation data\n',
    'rust/build.rs': 'fn main() {}\n',
    'data/seed/api-cache/heldout.lino': 'heldout cached data\n',
    '.cargo/config.toml': '[build]\njobs=1\n',
    'rust/.cargo/config.toml': '[net]\nretry=1\n',
    'data/meta/source-model/heldout.grammar': 'heldout CST grammar\n',
    'scripts/aggregate-source-content.mjs': 'heldout aggregation source\n',
    'scripts/mirror-package-data.mjs': 'heldout mirror source\n',
    'js/crypto/blake3.mjs': 'heldout digest helper\n',
    'js/agentic/crate/rust_str.mjs': 'heldout string helper\n',
    'other/unrelated.txt': 'whole-source witness only\n'
  };
  for (const [name, content] of Object.entries(files)) {
    fs.mkdirSync(join(directory, name, '..'), {
      recursive: true
    });
    fs.writeFileSync(join(directory, name), content);
  }
  git('init', '-q');
  git('config', 'user.name', 'Source fixture');
  git('config', 'user.email', 'fixture@example.invalid');
  git('config', 'commit.gpgSign', 'false');
  const hooks = join(directory, 'empty-hooks');
  fs.mkdirSync(hooks);
  git('config', 'core.hooksPath', hooks);
  git('add', '-A');
  git('commit', '-qm', 'held-out source');
  return {
    directory,
    head: git('rev-parse', 'HEAD'),
    close: () => fs.rmSync(directory, {
      recursive: true,
      force: true
    })
  };
}
test('independent immutable native witness agrees with actual compiler-input producer without CST-only helpers', async () => {
  const item = fixture();
  try {
    const witness = await immutableSourceWitness(item.directory, item.head);
    assert.deepEqual(witness.native, nativeInputIdentity(item.directory));
    assert(witness.whole.files > witness.native.files);
    fs.writeFileSync(join(item.directory, 'scripts/aggregate-source-content.mjs'), 'changed CST-only helper\n');
    assert.deepEqual(nativeInputIdentity(item.directory), witness.native, 'CST helper edits do not change compilation identity');
    assert.throws(() => verifyPhysicalDerivationInputs(item.directory, nativeDerivationRows(witness.rows, [])), /physical derivation source drift/u);
  } finally {
    item.close();
  }
});
test('expanded CST derivation custody still checks every helper and native bytes', async () => {
  const item = fixture();
  try {
    const witness = await immutableSourceWitness(item.directory, item.head),
      rows = nativeDerivationRows(witness.rows, []);
    for (const file of ['data/meta/source-model/heldout.grammar', 'scripts/aggregate-source-content.mjs', 'scripts/mirror-package-data.mjs', 'js/crypto/blake3.mjs', 'js/agentic/crate/rust_str.mjs', 'rust/src/lib.rs']) assert(rows.some(row => row.path === file));
    assert(!rows.some(row => row.path === 'other/unrelated.txt'));
    assert.equal(verifyPhysicalDerivationInputs(item.directory, rows).length, rows.length);
    fs.writeFileSync(join(item.directory, 'rust/src/lib.rs'), 'changed actual compilation input\n');
    assert.throws(() => nativeInputIdentity(item.directory), /uncommitted native compilation input/u);
    assert.throws(() => verifyPhysicalDerivationInputs(item.directory, rows), /physical derivation source drift/u);
    assert.throws(() => nativeDerivationRows(witness.rows, ['missing/producer.rs']), /declared derivation source missing/u);
  } finally {
    item.close();
  }
});


// Independent tiny physical fixtures only; no Rust compiler, product or checkout-wide identity.
function physicalFixture() {
  const item = fixture();
  const environment = {...process.env,GIT_CONFIG_NOSYSTEM:'1',GIT_CONFIG_GLOBAL:'/dev/null',GIT_TERMINAL_PROMPT:'0'};
  const git = (...arguments_) => execFileSync('git',arguments_,{cwd:item.directory,env:environment,encoding:'utf8',timeout:5000,maxBuffer:1024*1024});
  const extra = new Map([
    ['physical-input/empty.bin',Buffer.alloc(0)],
    ['physical-input/ascii-secret.txt',Buffer.from('physical fixture secret\n')],
    ['physical-input/路径-λ.txt',Buffer.from('Unicode 🙂\n')],
    ['physical-input/payload-λ.bin',Buffer.from(Array.from({length:257},(_,index)=>(index*37)%256))],
  ]);
  for(const [name,body] of extra){fs.mkdirSync(join(item.directory,name,'..'),{recursive:true});fs.writeFileSync(join(item.directory,name),body);}
  git('add','-A');git('commit','-qm','independent physical byte fixtures');
  const names=git('ls-tree','-r','--name-only','-z','HEAD').split('\0').filter(Boolean).sort((left,right)=>left<right?-1:left>right?1:0);
  const retained=new Map(names.map(name=>[name,fs.readFileSync(join(item.directory,name))]));
  const digest=createHash('sha256');let bytes=0;
  for(const [name,body]of retained){digest.update(Buffer.from(name+'\0'+String(body.length)+'\0'));digest.update(body);bytes+=body.length;}
  return {...item,git,retained,bytes,expected:{sha256:digest.digest('hex'),files:names.length}};
}
function observedChunkIo(onRead=null){
  const open=new Map(),lengths=[];let closed=0;
  const io={
    openSync(...args){const fd=fs.openSync(...args);open.set(fd,args[0]);return fd;},
    fstatSync(...args){return fs.fstatSync(...args);},
    readSync(...args){lengths.push(args[3]);return onRead?onRead(args,open):fs.readSync(...args);},
    closeSync(fd){try{return fs.closeSync(fd);}finally{assert(open.delete(fd));closed++;}},
  };
  return {io,open,lengths,closed:()=>closed};
}
test('physical tracked digest preserves canonical ASCII Unicode empty binary and bounded multi-chunk bytes',()=>{
  const item=physicalFixture(),calls=observedChunkIo();
  try{
    assert.deepEqual(trackedTestInputIdentity(item.directory,{chunkBytes:17,chunkIo:calls.io}),item.expected);
    assert.deepEqual(trackedTestInputIdentity(item.directory),item.expected,'legacy default API and protocol stay identical');
    assert(calls.lengths.length>item.expected.files,'binary input actually spans multiple physical reads');
    assert(calls.lengths.every(length=>Number.isInteger(length)&&length>0&&length<=17));
    assert.equal(calls.open.size,0);assert.equal(calls.closed(),item.expected.files);
  }finally{item.close();}
});
test('tracked physical stale and missing blobs cannot share their committed identity',()=>{
  for(const mode of ['same-length-stale','missing']){
    const item=physicalFixture();
    try{
      const file=join(item.directory,'physical-input/ascii-secret.txt');
      if(mode==='missing')fs.unlinkSync(file);else{const original=fs.readFileSync(file);const changed=Buffer.from(original);changed[0]^=1;fs.writeFileSync(file,changed);}
      assert.throws(()=>trackedTestInputIdentity(item.directory),mode==='missing'?undefined:/uncommitted|physical|blob/iu);
    }finally{item.close();}
  }
});
test('tracked symlinks physical symlink substitution and untracked test files remain refused',()=>{
  for(const mode of ['tracked-symlink','physical-symlink','untracked-test']){
    const item=physicalFixture();
    try{
      if(mode==='untracked-test'){fs.mkdirSync(join(item.directory,'rust/tests/unit'),{recursive:true});fs.writeFileSync(join(item.directory,'rust/tests/unit/untracked.rs'),'untracked fixture');}
      else{
        const file=join(item.directory,'physical-input/ascii-secret.txt'),copy=join(item.directory,'other/same-bytes.txt');
        fs.copyFileSync(file,copy);fs.unlinkSync(file);fs.symlinkSync('../other/same-bytes.txt',file);
        if(mode==='tracked-symlink'){item.git('add','-A');item.git('commit','-qm','committed symlink refusal fixture');}
      }
      assert.throws(()=>trackedTestInputIdentity(item.directory));
    }finally{item.close();}
  }
});
test('physical chunk size drift and read failures close every descriptor and refuse identity',()=>{
  for(const mode of ['append-after-read','read-error']){
    const item=physicalFixture();let altered=false;
    const calls=observedChunkIo((args,open)=>{
      if(mode==='read-error')throw new Error('controlled fixture read failure');
      const count=fs.readSync(...args);
      if(!altered&&count>0&&String(open.get(args[0])).endsWith('/physical-input/payload-λ.bin')){altered=true;fs.appendFileSync(open.get(args[0]),Buffer.from([9]));}
      return count;
    });
    try{
      assert.throws(()=>trackedTestInputIdentity(item.directory,{chunkBytes:17,chunkIo:calls.io}),mode==='read-error'?/controlled fixture read failure/u:/size|length|changed|EOF|physical|committed/iu);
      if(mode==='append-after-read')assert.equal(altered,true,'actual tiny physical file changed during the fold');
      assert.equal(calls.open.size,0);assert(calls.closed()>0);
    }finally{item.close();}
  }
});
test('physical identity diagnostics are bounded aggregate counters without filenames or contents',()=>{
  const item=physicalFixture(),events=[];
  try{
    assert.deepEqual(trackedTestInputIdentity(item.directory,{chunkBytes:17,observe:event=>events.push(event)}),item.expected);
    const progress=events.filter(event=>event.phase==='tracked-physical-bytes'&&event.state==='progress');
    assert(progress.length>=1&&progress.length<=32);
    let files=0,bytes=0,elapsed=0;
    for(const event of progress){
      assert.deepEqual(Object.keys(event).sort(),['bytes','elapsedMs','files','phase','state','totalFiles']);
      assert(Number.isFinite(event.elapsedMs)&&event.elapsedMs>=elapsed);elapsed=event.elapsedMs;
      assert(Number.isInteger(event.files)&&event.files>=files&&event.files<=item.expected.files);files=event.files;
      assert(Number.isInteger(event.bytes)&&event.bytes>=bytes&&event.bytes<=item.bytes);bytes=event.bytes;
      assert.equal(event.totalFiles,item.expected.files);
    }
    assert.equal(files,item.expected.files);assert.equal(bytes,item.bytes);
    const serialized=JSON.stringify(events);assert(!serialized.includes(item.directory));
    for(const name of item.retained.keys())assert(!serialized.includes(name));
    assert(!serialized.includes('physical fixture secret'));assert(!serialized.includes('Unicode 🙂'));
    for(const phase of ['tracked-inventory','tracked-test-tree','tracked-physical-bytes']){
      assert(events.some(event=>event.phase===phase&&event.state==='started'));
      assert(events.some(event=>event.phase===phase&&event.state==='completed'));
    }
  }finally{item.close();}
});
