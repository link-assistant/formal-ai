// Real small Git fixtures and byte-mutated executable receipts; no native compilation.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {mkdtempSync,mkdirSync,readFileSync,writeFileSync,rmSync,cpSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
import {sealSource,importSource,nativeInputIdentity,SOURCE_FILE} from '../../../scripts/native-release-source.mjs';
import {expectedIdentity,writeExecutableReceipt,verifyExecutableReceipt,verifyArchivedExecutable,NATIVE_TARGETS,RECEIPT_FILE} from '../../../scripts/native-release-artifact.mjs';
const hash=x=>createHash('sha256').update(x).digest('hex');
const git=(cwd,...args)=>execFileSync('git',args,{cwd,encoding:'utf8',stdio:['ignore','pipe','pipe']}).trim();
const metadata={workspace_members:['formal'],packages:[{id:'formal',name:'formal-ai',version:'1.2.3',features:{default:['meta-language','doublets-native']},targets:[{name:'formal-ai',kind:['bin']}]}]};
const compiler=(host='x86_64-unknown-linux-gnu')=>'rustc 1.90.0\ncommit-hash: '+'a'.repeat(40)+'\nhost: '+host+'\nrelease: 1.90.0\n';
function fixture(context) {
 const root=mkdtempSync(join(tmpdir(),'native-release-contract-'));context.after(()=>rmSync(root,{recursive:true,force:true}));
 const repo=join(root,'repo');mkdirSync(repo);git(repo,'init','--initial-branch=main');git(repo,'config','user.name','Fixture');git(repo,'config','user.email','fixture@example.invalid');git(repo,'config','core.autocrlf','false');
 mkdirSync(join(repo,'rust/src'),{recursive:true});writeFileSync(join(repo,'rust/Cargo.toml'),'[package]\nname="formal-ai"\nversion="1.2.3"\n');writeFileSync(join(repo,'rust/Cargo.lock'),'version = 4\n');writeFileSync(join(repo,'rust/src/main.rs'),'fn main() {}\n');
 git(repo,'add','.');git(repo,'commit','-m','base');
 const original=git(repo,'rev-parse','HEAD');git(repo,'checkout','-b','pr');
 writeFileSync(join(repo,'rust/src/main.rs'),'fn main() { println!("PR"); }\n');git(repo,'add','.');git(repo,'commit','-m','pull request');
 const head=git(repo,'rev-parse','HEAD');git(repo,'checkout','main');writeFileSync(join(repo,'README.md'),'new base\n');git(repo,'add','.');git(repo,'commit','-m','fresh base');
 const base=git(repo,'rev-parse','HEAD');git(repo,'checkout','pr');
 const consumer=join(root,'consumer');git(root,'clone','-c','core.autocrlf=false','--no-hardlinks',repo,consumer);
 git(repo,'merge',base,'--no-edit');const selected=git(repo,'rev-parse','HEAD');
 const packet=join(root,'source');mkdirSync(packet);
 const sealed=sealSource(repo,packet,{head,base,run:'42',metadata,compiler:compiler()});
 return {root,repo,consumer,packet,sealed,head,base,selected,original};
}

test('one fresh merge imports through a minimal bundle with exact source/tree/input validation',context=>{
 const f=fixture(context);
 assert.notEqual(f.selected,f.head);assert.ok(f.sealed.record.bundle.bytes>0);
 assert.throws(()=>git(f.consumer,'cat-file','-e',f.selected+'^{commit}'));
 const record=importSource(f.consumer,f.packet,{head:f.head,run:'42',selectionSha:f.sealed.sha256});
 assert.equal(git(f.consumer,'rev-parse','HEAD'),f.selected);assert.equal(record.source_tree,git(f.repo,'rev-parse','HEAD^{tree}'));
 assert.deepEqual(record.root_default_features,['doublets-native','meta-language']);
 assert.equal(nativeInputIdentity(f.consumer).sha256,record.native_inputs_sha256);
});

test('selection refuses wrong run/head, altered JSON and damaged merge-bundle bytes',context=>{
 const f=fixture(context), options={head:f.head,run:'42',selectionSha:f.sealed.sha256};
 for(const change of [{head:f.base},{run:'43'},{selectionSha:'0'.repeat(64)}])assert.throws(()=>importSource(f.consumer,f.packet,{...options,...change}));
 const bundle=join(f.packet,'source.bundle'),old=readFileSync(bundle);writeFileSync(bundle,Buffer.concat([old,Buffer.from('altered')]));
 assert.throws(()=>importSource(f.consumer,f.packet,options));
});

test('tag selection has no synthetic bundle and refuses uncommitted compilation inputs',context=>{
 const f=fixture(context);git(f.repo,'tag','v1.2.3');
 const destination=join(f.root,'tag-source');mkdirSync(destination);
 const tag=sealSource(f.repo,destination,{head:f.selected,tag:'v1.2.3',run:'42',metadata,compiler:compiler()});
 assert.equal(tag.record.bundle,null);assert.equal(tag.record.release_tag,'v1.2.3');
 writeFileSync(join(f.repo,'rust/src/main.rs'),'uncommitted source');
 assert.throws(()=>nativeInputIdentity(f.repo),/uncommitted native compilation input/u);
});

test('exact native executable receipt rejects every independent build-contract mismatch',context=>{
 const f=fixture(context),target=Object.keys(NATIVE_TARGETS).find(t=>NATIVE_TARGETS[t].platform===process.platform&&NATIVE_TARGETS[t].architecture===process.arch);
 assert.ok(target,'test host requires an actual supported native target');
 const identity=expectedIdentity(f.sealed.record,target,f.sealed.sha256,'42'),directory=join(f.root,'binary');mkdirSync(directory);
 const binary=join(directory,identity.platform==='win32'?'formal-ai.exe':'formal-ai');writeFileSync(binary,'actual executable bytes\n');
 writeExecutableReceipt(directory,identity,binary,{compiler:compiler(identity.host),environment:{RUSTFLAGS:'-Dwarnings'},job:'native',attempt:'1'});
 assert.equal(verifyExecutableReceipt(directory,identity).binary,binary);
 for(const field of ['source_commit','source_tree','cargo_lock_sha256','native_inputs_sha256','target','profile','requested_features','rust_flags','compiler_release','compiler_commit','source_selection_sha256','producer_run']) {
  assert.throws(()=>verifyExecutableReceipt(directory,{...identity,[field]:'mismatched'}),field);
 }
 assert.throws(()=>verifyExecutableReceipt(directory,{...identity,root_default_features:[]}));
 writeFileSync(binary,'corrupt executable bytes\n');assert.throws(()=>verifyExecutableReceipt(directory,identity));
});

test('producer refuses unexpected flags, compiler, host or feature policy before recording success',context=>{
 const f=fixture(context),target=Object.keys(NATIVE_TARGETS).find(t=>NATIVE_TARGETS[t].platform===process.platform&&NATIVE_TARGETS[t].architecture===process.arch);
 const identity=expectedIdentity(f.sealed.record,target,f.sealed.sha256,'42'),directory=join(f.root,'binary');mkdirSync(directory);const binary=join(directory,'input');writeFileSync(binary,'bytes');
 const options={compiler:compiler(identity.host),environment:{RUSTFLAGS:'-Dwarnings'},job:'native',attempt:'1'};
 for(const environment of [{RUSTFLAGS:''},{RUSTFLAGS:'-Dwarnings',CARGO_ENCODED_RUSTFLAGS:'different'},{RUSTFLAGS:'-Dwarnings',CARGO_PROFILE_RELEASE_LTO:'true'}])assert.throws(()=>writeExecutableReceipt(directory,identity,binary,{...options,environment}));
 assert.throws(()=>writeExecutableReceipt(directory,identity,binary,{...options,compiler:compiler('wrong-host')}));
 assert.throws(()=>expectedIdentity({...f.sealed.record,requested_features:'all'},target,f.sealed.sha256,'42'));
 assert.throws(()=>expectedIdentity(f.sealed.record,'unknown',f.sealed.sha256,'42'));
});

test('altered compiler provenance, extra artifact files and a substituted executable name refuse',context=>{
 const f=fixture(context),target=Object.keys(NATIVE_TARGETS).find(t=>NATIVE_TARGETS[t].platform===process.platform&&NATIVE_TARGETS[t].architecture===process.arch);
 const identity=expectedIdentity(f.sealed.record,target,f.sealed.sha256,'42'),directory=join(f.root,'binary');mkdirSync(directory);const binary=join(directory,identity.platform==='win32'?'formal-ai.exe':'formal-ai');writeFileSync(binary,'bytes');
 const receipt=writeExecutableReceipt(directory,identity,binary,{compiler:compiler(identity.host),environment:{RUSTFLAGS:'-Dwarnings'},job:'native',attempt:'1'});
 const file=join(directory,RECEIPT_FILE);writeFileSync(file,JSON.stringify({...receipt,producer:{...receipt.producer,compiler:compiler('wrong-host')}}));assert.throws(()=>verifyExecutableReceipt(directory,identity));
 writeFileSync(file,JSON.stringify({...receipt,executable:{...receipt.executable,name:'another'}}));assert.throws(()=>verifyExecutableReceipt(directory,identity));
 writeFileSync(file,JSON.stringify(receipt));writeFileSync(join(directory,'unexpected'),'extra');assert.throws(()=>verifyExecutableReceipt(directory,identity));
});

test('ignored physical build inputs refuse before sealing any native source identity',context=>{
 const f=fixture(context), initial=nativeInputIdentity(f.repo);
 writeFileSync(join(f.repo,'.gitignore'),'rust/src/ignored.rs\ndata/seed/api-cache/\n.cargo/\nrust/.cargo/\n');
 git(f.repo,'add','.gitignore');git(f.repo,'commit','-m','ignore fixture outputs');
 const paths=['rust/src/ignored.rs','data/seed/api-cache/legacy.lino',
  '.cargo/config.toml','rust/.cargo/config.toml'];
 for(const path of paths) {
  const parts=path.split('/');parts.pop();
  mkdirSync(join(f.repo,...parts),{recursive:true});
  writeFileSync(join(f.repo,path),'unrecorded compiler input\n');
  assert.throws(()=>nativeInputIdentity(f.repo),/untracked native compilation input/u,path);
  rmSync(join(f.repo,path));
  assert.equal(nativeInputIdentity(f.repo).sha256,initial.sha256);
 }
});

test('tracked legacy seeds and Cargo configuration bind source identity and refuse dirty bytes',context=>{
 const f=fixture(context), initial=nativeInputIdentity(f.repo);
 for(const path of ['data/seed/api-cache/legacy.lino','.cargo/config.toml','rust/.cargo/config.toml']) {
  const parts=path.split('/');parts.pop();
  mkdirSync(join(f.repo,...parts),{recursive:true});
  writeFileSync(join(f.repo,path),'tracked compiler input\n');
  git(f.repo,'add',path);git(f.repo,'commit','-m','record build input');
  const next=nativeInputIdentity(f.repo);
  assert.notEqual(next.sha256,initial.sha256);
  writeFileSync(join(f.repo,path),'altered compiler input\n');
  assert.throws(()=>nativeInputIdentity(f.repo),/uncommitted native compilation input/u);
  git(f.repo,'-c','core.autocrlf=false','checkout-index','--all','--force');
 }
});

test('extracted archives preserve the verified executable name, length and SHA',context=>{
 const f=fixture(context);
 const target=Object.keys(NATIVE_TARGETS).find(t=>
  NATIVE_TARGETS[t].platform===process.platform&&NATIVE_TARGETS[t].architecture===process.arch);
 const identity=expectedIdentity(f.sealed.record,target,f.sealed.sha256,'42');
 const directory=join(f.root,'binary');mkdirSync(directory);
 const name=identity.platform==='win32'?'formal-ai.exe':'formal-ai';
 const binary=join(directory,name);writeFileSync(binary,'actual producer executable bytes\n');
 const receipt=writeExecutableReceipt(directory,identity,binary,{
  compiler:compiler(identity.host),environment:{RUSTFLAGS:'-Dwarnings'},job:'native',attempt:'1',
 });
 const extracted=join(f.root,'extracted');mkdirSync(extracted);
 const archiveBinary=join(extracted,name);cpSync(binary,archiveBinary);
 assert.equal(verifyArchivedExecutable(archiveBinary,receipt),archiveBinary);
 const wrongName=join(extracted,'substituted');cpSync(binary,wrongName);
 assert.throws(()=>verifyArchivedExecutable(wrongName,receipt));
 writeFileSync(archiveBinary,'different payload with changed bytes\n');
 assert.throws(()=>verifyArchivedExecutable(archiveBinary,receipt));
 writeFileSync(archiveBinary,'');
 assert.throws(()=>verifyArchivedExecutable(archiveBinary,receipt));
});
