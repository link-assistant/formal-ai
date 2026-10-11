import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {readFileSync,lstatSync,readdirSync,existsSync} from 'node:fs';
import {join} from 'node:path';
import {nativeInputIdentity} from '../native-release-source.mjs';
/** Conservative test-input witness: every committed workspace blob, with actual physical bytes. */
export function trackedTestInputIdentity(cwd) {
 const rows=execFileSync('git',['ls-tree','-r','-z','HEAD'],{cwd,encoding:'utf8',maxBuffer:32*1024*1024}).split('\0').filter(Boolean).map(line=>{
  const match=/^(\d+) blob ([a-f0-9]{40})\t([\s\S]+)$/u.exec(line);assert.ok(match,'test input must be a committed blob');
  assert.notEqual(match[1],'120000','symlink test-input witness cannot certify physical source');return {name:match[3],object:match[2]};
 }).sort((a,b)=>a.name<b.name?-1:a.name>b.name?1:0);
 const tracked=new Set(rows.map(row=>row.name));
 function inspect(relative){if(!existsSync(join(cwd,relative)))return;for(const entry of readdirSync(join(cwd,relative),{withFileTypes:true})){
  const name=relative+'/'+entry.name;assert.equal(entry.isSymbolicLink(),false,'symlink test source');
  if(entry.isDirectory())inspect(name);else assert.ok(tracked.has(name),'untracked test input: '+name);
 }}
 inspect('rust/tests');
 const digest=createHash('sha256');
 for(const row of rows){assert.doesNotMatch(row.name,/[\r\n]/u);assert.ok(lstatSync(join(cwd,row.name)).isFile());
  const bytes=readFileSync(join(cwd,row.name));
  const object=createHash('sha1').update(Buffer.from('blob '+bytes.length+'\0')).update(bytes).digest('hex');
  assert.equal(object,row.object,'uncommitted physical test input: '+row.name);
  digest.update(row.name).update('\0').update(String(bytes.length)).update('\0').update(bytes);
 }
 return {sha256:digest.digest('hex'),files:rows.length};
}
/** Metadata comes from the actual native test producer/compiler, not a release-default feature substitution. */
export function augmentNativeTestIdentity(cwd,identity,environment=process.env) {
 const host=/^host: (\S+)$/mu.exec(identity.compiler)?.[1];assert.ok(host,'actual compiler host missing');
 const native=nativeInputIdentity(cwd),tests=trackedTestInputIdentity(cwd);
 const overrides=Object.fromEntries(Object.entries(environment).filter(([key])=>/^CARGO_PROFILE_[A-Z0-9_]+$/u.test(key)).sort(([a],[b])=>a.localeCompare(b,'en')));
 return {...identity,target:environment.CARGO_BUILD_TARGET||host,'encoded-rust-flags':environment.CARGO_ENCODED_RUSTFLAGS??'',
  'cargo-profile-overrides':overrides,'native-inputs-sha256':native.sha256,'native-input-files':native.files,
  'test-inputs-sha256':tests.sha256,'test-input-files':tests.files};
}
