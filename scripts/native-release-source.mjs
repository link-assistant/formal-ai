// One immutable source selection for every native release producer and consumer.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {existsSync,readdirSync,readFileSync,writeFileSync,lstatSync,openSync,readSync,closeSync,mkdirSync} from 'node:fs';
import {join,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {authorizePublishedStableSource,validateSelectedSourceAuthority} from './native-release-trust.mjs';

export const SOURCE_FILE='source-selection.json';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const git=(cwd,args)=>execFileSync('git',args,{cwd,encoding:'utf8',timeout:30000,maxBuffer:32*1024*1024}).trim();
const commit=value=>assert.match(value,/^[a-f0-9]{40}$/u);

export const nativeCompilationInputRoots = Object.freeze(['rust/src','rust/embedded','rust/Cargo.toml','rust/Cargo.lock','rust/build.rs',
 'data/seed/api-cache','.cargo','rust/.cargo']);

/** Hash only tracked native compilation inputs, and refuse an uncommitted input. */
export function nativeInputIdentity(cwd) {
 const listing=execFileSync('git',['ls-tree','-r','-z','HEAD','--',...nativeCompilationInputRoots],{cwd,encoding:'utf8'}).split('\0').filter(Boolean);
 const records=listing.map(line=>{const match=/^\d+ blob ([a-f0-9]{40})\t([\s\S]+)$/u.exec(line);assert.ok(match,'native input must be a tracked blob');return {object:match[1],name:match[2]};}).sort((a,b)=>a.name<b.name?-1:a.name>b.name?1:0);
 const names=records.map(r=>r.name);assert.ok(names.includes('rust/Cargo.toml')&&names.includes('rust/Cargo.lock'));
 for(const name of names)assert.doesNotMatch(name,/[\r\n]/u);
 const tracked=new Set(names);
 function inspect(directory,legacyOnly=false) {
  if(!existsSync(join(cwd,directory)))return;
  for(const entry of readdirSync(join(cwd,directory),{withFileTypes:true})) {
   const name=directory+'/'+entry.name;
   assert.equal(entry.isSymbolicLink(),false,'symlink native compilation input: '+name);
   if(entry.isDirectory())inspect(name,legacyOnly);
   else if(!legacyOnly||name.endsWith('.lino')) {
    assert.ok(tracked.has(name),'untracked native compilation input: '+name);
   }
  }
 }
 for(const directory of ['rust/src','rust/embedded','.cargo','rust/.cargo'])inspect(directory);
 inspect('data/seed/api-cache',true);
 const objects=execFileSync('git',['hash-object','--no-filters','--stdin-paths'],{cwd,encoding:'utf8',input:names.join('\n')+'\n'}).trim().split('\n');
 assert.equal(objects.length,records.length);
 const digest=createHash('sha256');
 for(let index=0;index<records.length;index++) {
  const {name,object}=records[index];assert.equal(objects[index],object,'uncommitted native compilation input: '+name);
  const bytes=readFileSync(join(cwd,name));
  digest.update(name);digest.update('\0');digest.update(String(bytes.length));digest.update('\0');digest.update(bytes);
 }
 return {sha256:digest.digest('hex'),files:names.length};
}

/** Complete only this selected commit's ancestry, not every branch and tag.
 * A fresh, unfiltered checkout keeps current source usable with the remote offline. */
export function hydrateSourceHistory(cwd) {
 const head=git(cwd,['rev-parse','HEAD']);commit(head);
 const shallow=git(cwd,['rev-parse','--is-shallow-repository'])==='true';
 let partial='';try{partial=git(cwd,['config','--get','remote.origin.promisor']);}catch(error){if(error.status!==1)throw error;}
 assert.notEqual(partial,'true','native source must not inherit a promisor object database');
 if(shallow)execFileSync('git',['-c','protocol.version=2','fetch','--progress','--no-tags','--unshallow','origin',head],{
  cwd,stdio:'inherit',timeout:240000,killSignal:'SIGKILL'});
 assert.equal(git(cwd,['rev-parse','HEAD']),head);
 assert.equal(git(cwd,['rev-parse','--is-shallow-repository']),'false','selected source ancestry remains incomplete');
 return {head,completedHistory:shallow,sourceTree:git(cwd,['rev-parse','HEAD^{tree}']),refspec:'exact selected commit only'};
}

/** Root default features are derived from Cargo's actual package metadata. */
export function packageContract(metadata) {
 const packages=metadata.packages.filter(p=>metadata.workspace_members.includes(p.id));
 const root=packages.find(p=>p.name==='formal-ai');assert.ok(root,'formal-ai package missing');
 assert.ok(root.targets.some(t=>t.name==='formal-ai'&&t.kind.includes('bin')));
 return {version:root.version,default_features:[...(root.features.default??[])].sort()};
}

export function compilerContract(text) {
 const release=/^release: (\S+)$/mu.exec(text)?.[1], hash=/^commit-hash: ([a-f0-9]{40})$/mu.exec(text)?.[1];
 assert.ok(release&&hash,'incomplete actual compiler identity');return {release,commit:hash};
}

/** Export only a new merge's objects, with the already checked-out heads as prerequisites. */
export function sealSource(cwd,directory,{head,base=null,tag=null,run,metadata,compiler}) {
 commit(head);if(base!==null)commit(base);assert.match(String(run),/^\d+$/u);
 const selected=git(cwd,['rev-parse','HEAD']),tree=git(cwd,['rev-parse','HEAD^{tree}']);commit(selected);commit(tree);
 if(tag!==null)assert.equal(git(cwd,['rev-parse',tag+'^{commit}']),selected);
 else {
  assert.equal(git(cwd,['merge-base',selected,head]),head);
  assert.ok(base,'pull-request source requires a pinned base');
  assert.equal(git(cwd,['merge-base',selected,base]),base);
 }
 const native=nativeInputIdentity(cwd), pkg=packageContract(metadata), rust=compilerContract(compiler);
 let bundle=null;
 if(selected!==head) {
  assert.ok(base,'a synthetic source requires both prerequisites');
  const file=join(directory,'source.bundle');
  execFileSync('git',['bundle','create',file,'HEAD','^'+head,'^'+base],{cwd,stdio:'pipe'});
  const bytes=readFileSync(file);bundle={name:'source.bundle',bytes:bytes.length,sha256:sha(bytes)};
 }
 const record={version:1,source_commit:selected,source_tree:tree,selected_head:head,pinned_base:base,
  release_tag:tag,producer_run:String(run),cargo_lock_sha256:sha(readFileSync(join(cwd,'rust/Cargo.lock'))),
  native_inputs_sha256:native.sha256,native_input_files:native.files,package_version:pkg.version,
  requested_features:'default',root_default_features:pkg.default_features,
  profile:'release',rust_flags:'-Dwarnings',compiler_release:rust.release,compiler_commit:rust.commit,bundle};
 const bytes=JSON.stringify(record,null,2)+'\n';writeFileSync(join(directory,SOURCE_FILE),bytes);
 return {record,sha256:sha(bytes)};
}

/** Verify independent workflow outputs before accepting any selected tree or bundle. */
function readSourceSelection(directory,{head,run,selectionSha}) {
 const bytes=readFileSync(join(directory,SOURCE_FILE));assert.equal(sha(bytes),selectionSha);
 const record=JSON.parse(bytes);assert.equal(record.version,1);assert.equal(record.selected_head,head);
 assert.equal(record.producer_run,String(run));commit(record.source_commit);commit(record.source_tree);
 return record;
}

const HISTORY_BUNDLE='source-history.bundle',HISTORY_MANIFEST='source-history.json';
const HISTORY_LIMIT=2*1024*1024*1024;
function historyDigest(file) {
 const before=lstatSync(file);assert(before.isFile()&&!before.isSymbolicLink());
 assert(before.size>0&&before.size<=HISTORY_LIMIT,'source history exceeds explicit transfer budget');
 const descriptor=openSync(file,'r'),buffer=Buffer.alloc(65536),digest=createHash('sha256');
 try{for(let bytes;(bytes=readSync(descriptor,buffer,0,buffer.length,null))>0;)digest.update(buffer.subarray(0,bytes));}
 finally{closeSync(descriptor);}
 const after=lstatSync(file);assert.equal(after.size,before.size);assert.equal(after.mtimeMs,before.mtimeMs);
 return {bytes:before.size,sha256:digest.digest('hex')};
}
function historyCommand(cwd,args,deadline) {
 const remaining=deadline-Date.now();assert(remaining>0,'source history deadline exhausted');
 execFileSync('git',args,{cwd,stdio:'inherit',timeout:Math.min(180000,remaining),killSignal:'SIGKILL',
  env:{...process.env,GIT_LFS_SKIP_SMUDGE:'1',GIT_TERMINAL_PROMPT:'0'}});
}
/** One complete selected ancestry supplies CLI jobs without repeated cold GitHub transfers. */
export function exportSourceHistory(cwd,directory,outputDirectory=directory) {
 const bytes=readFileSync(join(directory,SOURCE_FILE)),record=JSON.parse(bytes);
 commit(record.source_commit);commit(record.source_tree);
 assert.equal(git(cwd,['rev-parse','HEAD']),record.source_commit);
 assert.equal(git(cwd,['rev-parse','HEAD^{tree}']),record.source_tree);
 assert.equal(git(cwd,['rev-parse','--is-shallow-repository']),'false');
 if(!existsSync(outputDirectory))mkdirSync(outputDirectory);
 const file=resolve(outputDirectory,HISTORY_BUNDLE);assert(!existsSync(file));
 historyCommand(cwd,['bundle','create','--progress',file,'HEAD'],Date.now()+240000);
 const manifest={schema:'native-source-history/v1',sourceCommit:record.source_commit,sourceTree:record.source_tree,
  selectionSha256:sha(bytes),bundle:HISTORY_BUNDLE,...historyDigest(file)};
 writeFileSync(join(outputDirectory,HISTORY_MANIFEST),JSON.stringify(manifest,null,2)+'\n',{flag:'wx'});
 return manifest;
}
/** Restore only an authenticated same-run source into a fresh, unfiltered repository. */
export function restoreSourceHistory(cwd,directory,options) {
 assert(!existsSync(join(cwd,'.git')),'offline source restore requires a fresh workspace');
 const record=readSourceSelection(directory,options),manifestFile=join(directory,HISTORY_MANIFEST);
 const metadata=lstatSync(manifestFile);assert(metadata.isFile()&&!metadata.isSymbolicLink()&&metadata.size<=65536);
 const manifest=JSON.parse(readFileSync(manifestFile,'utf8'));
 assert.equal(manifest.schema,'native-source-history/v1');assert.equal(manifest.bundle,HISTORY_BUNDLE);
 assert.equal(manifest.sourceCommit,record.source_commit);assert.equal(manifest.sourceTree,record.source_tree);
 assert.equal(manifest.selectionSha256,options.selectionSha);
 assert(Number.isSafeInteger(manifest.bytes)&&manifest.bytes>0&&manifest.bytes<=HISTORY_LIMIT);
 const file=resolve(directory,HISTORY_BUNDLE);
 assert.deepEqual(historyDigest(file),{bytes:manifest.bytes,sha256:manifest.sha256});
 const repository=options.repository;assert.match(repository,/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
 const deadline=Date.now()+240000;
 for(const args of [['init','--initial-branch=source-artifact'],['config','core.autocrlf','false'],
  ['config','core.longpaths','true'],['bundle','verify',file],['fetch','--progress','--no-tags',file,'HEAD'],
  ['checkout','--detach',record.source_commit],['remote','add','origin','https://github.com/'+repository+'.git']])
  historyCommand(cwd,args,deadline);
 assert.equal(git(cwd,['rev-parse','HEAD^{tree}']),record.source_tree);
 assert.equal(git(cwd,['rev-parse','--is-shallow-repository']),'false');
 assert.deepEqual(historyDigest(file),{bytes:manifest.bytes,sha256:manifest.sha256});
 return record;
}

export function importSource(cwd,directory,options) {
 const record=readSourceSelection(directory,options);
 if(record.bundle!==null) {
  assert.equal(record.bundle.name,'source.bundle');const file=join(directory,record.bundle.name),payload=readFileSync(file);
  assert.equal(payload.length,record.bundle.bytes);assert.equal(sha(payload),record.bundle.sha256);
  execFileSync('git',['bundle','verify',file],{cwd,stdio:'pipe'});
  execFileSync('git',['fetch',file,'HEAD'],{cwd,stdio:'pipe'});
 }
 execFileSync('git',['diff','--quiet'],{cwd,stdio:'pipe'});
 execFileSync('git',['checkout','--detach',record.source_commit],{cwd,stdio:'pipe'});
 execFileSync('git',['-c','core.autocrlf=false','checkout-index','--all','--force'],{cwd,stdio:'pipe'});
 assert.equal(git(cwd,['rev-parse','HEAD^{tree}']),record.source_tree);
 assert.equal(sha(readFileSync(join(cwd,'rust/Cargo.lock'))),record.cargo_lock_sha256);
 assert.equal(nativeInputIdentity(cwd).sha256,record.native_inputs_sha256);
 return record;
}

if(import.meta.url===pathToFileURL(process.argv[1]??'').href) {
 const [mode,directory,destination]=process.argv.slice(2),cwd=process.cwd();
 if(mode==='hydrate-source') {
  assert.equal(directory,undefined);console.log(JSON.stringify(hydrateSourceHistory(cwd)));
 } else if(mode==='seal') {
  const result=sealSource(cwd,directory,{head:process.env.NATIVE_SELECTED_HEAD,
   base:process.env.NATIVE_BASE_COMMIT||null,tag:process.env.NATIVE_RELEASE_TAG||null,run:process.env.GITHUB_RUN_ID,
   metadata:JSON.parse(readFileSync(process.env.NATIVE_METADATA_FILE,'utf8')),
   compiler:readFileSync(process.env.NATIVE_COMPILER_FILE,'utf8')});
  validateSelectedSourceAuthority(cwd,result.record,{event:process.env.NATIVE_RELEASE_EVENT,purpose:process.env.NATIVE_RELEASE_PURPOSE,
   authority:process.env.NATIVE_RELEASE_EVENT==='pull_request'||process.env.NATIVE_RELEASE_PURPOSE==='validation'?null:{
    tag:process.env.NATIVE_RELEASE_TAG,releaseCommit:process.env.NATIVE_AUTHORIZED_RELEASE_COMMIT,
    defaultCommit:process.env.NATIVE_AUTHORIZED_DEFAULT_COMMIT}});
  if(process.env.GITHUB_OUTPUT) {
   writeFileSync(process.env.GITHUB_OUTPUT,
    'commit='+result.record.source_commit+'\ntree='+result.record.source_tree
    +'\ntoolchain='+result.record.compiler_release+'\ncompiler-commit='+result.record.compiler_commit
    +'\nselected-head='+result.record.selected_head+'\nselection-sha256='+result.sha256+'\n',{flag:'a'});
  }
  console.log(JSON.stringify(result));
 } else if(mode==='export-history') {
  console.log(JSON.stringify(exportSourceHistory(cwd,directory,destination)));
 } else if(mode==='import'||mode==='import-offline') {
  const options={head:process.env.NATIVE_SELECTED_HEAD,run:process.env.GITHUB_RUN_ID,
   selectionSha:process.env.NATIVE_SELECTION_SHA256,repository:process.env.NATIVE_REPOSITORY};
  if(mode==='import-offline')restoreSourceHistory(cwd,directory,options);
  const record=importSource(cwd,directory,options);
  const authority=process.env.NATIVE_RELEASE_EVENT==='pull_request'||process.env.NATIVE_RELEASE_PURPOSE==='validation'?null:
   authorizePublishedStableSource(cwd,{tag:record.release_tag,repository:process.env.NATIVE_REPOSITORY});
  console.log(JSON.stringify(validateSelectedSourceAuthority(cwd,record,{
   event:process.env.NATIVE_RELEASE_EVENT,purpose:process.env.NATIVE_RELEASE_PURPOSE,authority})));
 } else assert.fail('unknown source-selection mode');
}
