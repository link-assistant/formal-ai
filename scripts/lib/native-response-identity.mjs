import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {lstatSync,readdirSync,existsSync,openSync,fstatSync,readSync,closeSync} from 'node:fs';
import {join} from 'node:path';
import {nativeInputIdentity} from '../native-release-source.mjs';
// Synchronous diagnostics report phase state only; they never replace verification.
const NATIVE_TEST_PHASES=new Set(['build-identity','native-inputs','tracked-test-inputs',
 'tracked-inventory','tracked-test-tree','tracked-physical-bytes',
 'write-receipt','verify-receipt','list-test','run-test']);
export function observeNativeTestPhase(phase,action,observe=null){
 assert(NATIVE_TEST_PHASES.has(phase),'unknown native test phase');
 const started=performance.now();
 const emit=state=>{if(observe!==null)observe({phase,state,elapsedMs:Math.max(0,performance.now()-started)});};
 emit('started');
 try{const value=action();emit('completed');return value;}
 catch(error){emit('failed');throw error;}
}

const DEFAULT_CHUNK_IO=Object.freeze({openSync,fstatSync,readSync,closeSync});
function foldTrackedPhysicalFile(cwd,row,digest,buffer,io) {
 const file=join(cwd,row.name),before=lstatSync(file);
 assert.ok(before.isFile(),'test input must be a regular physical file');
 const descriptor=io.openSync(file,'r');
 try {
  const initial=io.fstatSync(descriptor);
  assert.ok(initial.isFile(),'test input descriptor must be regular');
  assert.ok(Number.isSafeInteger(initial.size)&&initial.size>=0,'finite physical test input size required');
  assert.equal(initial.size,before.size,'physical test input size changed before reading');
  assert.equal(initial.dev,before.dev,'physical test input device changed');
  assert.equal(initial.ino,before.ino,'physical test input inode changed');
  const object=createHash('sha1').update(Buffer.from('blob '+initial.size+'\0'));
  digest.update(row.name).update('\0').update(String(initial.size)).update('\0');
  let bytes=0;
  for(;;) {
   const count=io.readSync(descriptor,buffer,0,buffer.length,null);
   assert.ok(Number.isInteger(count)&&count>=0&&count<=buffer.length,'invalid physical read length');
   if(count===0)break;
   bytes+=count;assert.ok(bytes<=initial.size,'physical test input size changed while reading');
   const chunk=buffer.subarray(0,count);object.update(chunk);digest.update(chunk);
  }
  assert.equal(bytes,initial.size,'physical test input ended before its declared size');
  const final=io.fstatSync(descriptor);
  for(const key of ['dev','ino','size','mtimeMs','ctimeMs'])
   assert.equal(final[key],initial[key],'physical test input changed while reading');
  assert.equal(object.digest('hex'),row.object,'uncommitted physical test input: '+row.name);
  return bytes;
 } finally {io.closeSync(descriptor);}
}
/** Every committed workspace blob, physically checked; options are trusted diagnostic/test seams. */
export function trackedTestInputIdentity(cwd,{observe=null,chunkBytes=1024*1024,chunkIo=DEFAULT_CHUNK_IO}={}) {
 assert.ok(observe===null||typeof observe==='function');
 assert.ok(Number.isInteger(chunkBytes)&&chunkBytes>0&&chunkBytes<=1024*1024,'bounded physical read chunk required');
 for(const name of ['openSync','fstatSync','readSync','closeSync'])assert.equal(typeof chunkIo?.[name],'function');
 const rows=observeNativeTestPhase('tracked-inventory',()=>execFileSync('git',['ls-tree','-r','-z','HEAD'],{cwd,encoding:'utf8',maxBuffer:32*1024*1024}).split('\0').filter(Boolean).map(line=>{
  const match=/^(\d+) blob ([a-f0-9]{40})\t([\s\S]+)$/u.exec(line);assert.ok(match,'test input must be a committed blob');
  assert.notEqual(match[1],'120000','symlink test-input witness cannot certify physical source');return {name:match[3],object:match[2]};
 }).sort((a,b)=>a.name<b.name?-1:a.name>b.name?1:0),observe);
 const tracked=new Set(rows.map(row=>row.name));
 function inspect(relative){if(!existsSync(join(cwd,relative)))return;for(const entry of readdirSync(join(cwd,relative),{withFileTypes:true})){
  const name=relative+'/'+entry.name;assert.equal(entry.isSymbolicLink(),false,'symlink test source');
  if(entry.isDirectory())inspect(name);else assert.ok(tracked.has(name),'untracked test input: '+name);
 }}
 observeNativeTestPhase('tracked-test-tree',()=>inspect('rust/tests'),observe);
 return observeNativeTestPhase('tracked-physical-bytes',()=>{
  const digest=createHash('sha256'),buffer=Buffer.allocUnsafe(chunkBytes),started=performance.now();
  let files=0,bytes=0,last=started,events=0;
  const progress=()=>{if(observe!==null)observe({phase:'tracked-physical-bytes',state:'progress',
   elapsedMs:Math.max(0,performance.now()-started),files,bytes,totalFiles:rows.length});};
  for(const row of rows){assert.doesNotMatch(row.name,/[\r\n]/u);
   bytes+=foldTrackedPhysicalFile(cwd,row,digest,buffer,chunkIo);files++;
   assert.ok(Number.isSafeInteger(bytes),'finite total physical input size required');
   const now=performance.now();if(events<31&&now-last>=10000){progress();events++;last=now;}
  }
  progress();
  return {sha256:digest.digest('hex'),files:rows.length};
 },observe);
}
/** Metadata comes from the actual native test producer/compiler, not a release-default feature substitution. */
export function augmentNativeTestIdentity(cwd,identity,environment=process.env,observe=null) {
 const host=/^host: (\S+)$/mu.exec(identity.compiler)?.[1];assert.ok(host,'actual compiler host missing');
 const native=observeNativeTestPhase('native-inputs',()=>nativeInputIdentity(cwd),observe);
 const tests=observeNativeTestPhase('tracked-test-inputs',()=>trackedTestInputIdentity(cwd,{observe}),observe);
 const overrides=Object.fromEntries(Object.entries(environment).filter(([key])=>/^CARGO_PROFILE_[A-Z0-9_]+$/u.test(key)).sort(([a],[b])=>a.localeCompare(b,'en')));
 return {...identity,target:environment.CARGO_BUILD_TARGET||host,'encoded-rust-flags':environment.CARGO_ENCODED_RUSTFLAGS??'',
  'cargo-profile-overrides':overrides,'native-inputs-sha256':native.sha256,'native-input-files':native.files,
  'test-inputs-sha256':tests.sha256,'test-input-files':tests.files};
}
