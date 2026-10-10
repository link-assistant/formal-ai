#!/usr/bin/env node
// Workflow-owned tooling remains separate from the selected historical release source.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {existsSync,mkdirSync,readFileSync,writeFileSync,lstatSync,realpathSync,chmodSync} from 'node:fs';
import {dirname,join,relative,resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
export const RELEASE_PROTOCOL_FILES=Object.freeze({
 engine:Object.freeze(['scripts/release-protocol.mjs','scripts/publish-browser-engine.mjs',
    'scripts/build-selected-wasm.mjs','scripts/browser-engine-deadline.mjs','scripts/browser-engine-cold-install.mjs']),
 'source-networks':Object.freeze(['scripts/release-protocol.mjs','scripts/check-source-networks.mjs','scripts/verify-source-network-distribution.mjs','scripts/translate-js-rust.mjs','scripts/lib/source-network-packets.mjs','scripts/lib/source-network-workers.mjs','scripts/lib/source-network-worker.mjs','scripts/lib/translation-blockers.mjs','scripts/lib/translation-lowering.mjs','scripts/lib/translation-workarounds.mjs','scripts/run-with-budget-warning.sh','scripts/check-sccache-write-health.sh','.github/workflows/layered-ci.yml']),
});
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
const git=(root,args)=>execFileSync('git',['-C',root,...args],{maxBuffer:16*1024*1024});
const commit=value=>{assert.match(value,/^[a-f0-9]{40}$/u,'a full independent workflow commit is required');return value;};
function name(value){assert.equal(typeof value,'string');assert.match(value,/^[A-Za-z0-9_.-]+(?:\/[A-Za-z0-9_.-]+)*$/u);assert.ok(!value.split('/').some(part=>['.','..','.git'].includes(part)),'unsafe protocol path');return value;}
function blob(root,revision,path){
 name(path);commit(revision);
 const entry=git(root,['ls-tree','-z',revision,'--',path]).toString('utf8');
 assert.match(entry,/^100(?:644|755) blob [a-f0-9]{40}\t[^\0]+\0$/u,'protocol input must be one ordinary committed file');
 assert.equal(entry.slice(entry.indexOf('\t')+1,-1),path);
 return git(root,['show',revision+':'+path]);
}
export function protocolUpstreamCommit(text){
 const pins=[...text.matchAll(/repository: link-foundation\/meta-language\n\s+ref: ([a-f0-9]{40})/gu)].map(match=>match[1]);
 assert.equal(new Set(pins).size,1,'workflow serializer must have one exact shared commit');return pins[0];
}
export function captureReleaseProtocol(root,directory,revision,paths){
 root=realpathSync(root);directory=resolve(directory);commit(revision);
 assert.equal(git(root,['rev-parse','HEAD']).toString('utf8').trim(),revision,'capture must run on the independent workflow source');
 assert.ok(Array.isArray(paths)&&paths.length>0);assert.equal(new Set(paths).size,paths.length,'duplicate protocol path');
 assert.equal(existsSync(directory),false,'a fresh protocol directory is required');
 const files=[...paths].sort().map(path=>{
  name(path);const original=blob(root,revision,path),physical=join(root,path);
  assert.ok(lstatSync(physical).isFile());assert.equal(relative(root,realpathSync(physical)),path,'symlink protocol input');
  assert.ok(readFileSync(physical).equals(original),'protocol input differs from workflow commit: '+path);
  const mode=git(root,['ls-tree',revision,'--',path]).toString('utf8').slice(0,6);
  return {path,mode,bytes:original.length,sha256:digest(original),original};
 });
 const upstream=files.find(file=>file.path==='.github/workflows/layered-ci.yml');
 const record={schema:'release-protocol-snapshot/v1',commit:revision,tree:git(root,['rev-parse',revision+'^{tree}']).toString('utf8').trim(),
  ...(upstream?{upstreamCommit:protocolUpstreamCommit(upstream.original.toString('utf8'))}:{}),
  files:files.map(({original,...file})=>file)};
 mkdirSync(directory,{recursive:true});
 for(const file of files){const destination=join(directory,file.path);mkdirSync(dirname(destination),{recursive:true});writeFileSync(destination,file.original);chmodSync(destination,file.mode==='100755'?0o755:0o644);}
 const bytes=Buffer.from(JSON.stringify(record,null,2)+'\n');writeFileSync(join(directory,'protocol-selection.json'),bytes);
 assert.equal(git(root,['rev-parse','HEAD']).toString('utf8').trim(),revision,'workflow source changed during capture');
 return {...record,sha256:digest(bytes),directory};
}
export function verifyReleaseProtocol(root,directory,revision,expectedDigest){
 root=realpathSync(root);directory=resolve(directory);commit(revision);
 const bytes=readFileSync(join(directory,'protocol-selection.json')),record=JSON.parse(bytes);
 if(expectedDigest!==undefined){assert.match(expectedDigest,/^[a-f0-9]{64}$/u);assert.equal(digest(bytes),expectedDigest,'independently recorded protocol digest differs');}
 assert.equal(record.schema,'release-protocol-snapshot/v1');assert.equal(record.commit,revision);
 assert.equal(record.tree,git(root,['rev-parse',revision+'^{tree}']).toString('utf8').trim());
 assert.ok(Array.isArray(record.files)&&record.files.length>0);assert.equal(new Set(record.files.map(file=>file.path)).size,record.files.length);
 for(const file of record.files){const original=blob(root,revision,file.path),physical=join(directory,name(file.path));
  assert.ok(lstatSync(physical).isFile());assert.equal(relative(realpathSync(directory),realpathSync(physical)),file.path,'symlink captured protocol');
  const actual=readFileSync(physical);assert.ok(actual.equals(original),'captured protocol differs from workflow source: '+file.path);
  assert.equal(file.mode,git(root,['ls-tree',revision,'--',file.path]).toString('utf8').slice(0,6));
  assert.equal(file.bytes,original.length);assert.equal(file.sha256,digest(original));
 }
 const upstream=record.files.find(file=>file.path==='.github/workflows/layered-ci.yml');
 if(upstream)assert.equal(record.upstreamCommit,protocolUpstreamCommit(readFileSync(join(directory,upstream.path),'utf8')));
 else assert.equal(Object.hasOwn(record,'upstreamCommit'),false);
 return {...record,sha256:digest(bytes),directory};
}
/** Package metadata is derived only from the selected source's manifest and Cargo version. */
export function verifySelectedPackage(root,revision,path='packages/formal-ai-engine/package.json',built=false){
 root=realpathSync(root);commit(revision);name(path);
 assert.equal(git(root,['rev-parse','HEAD']).toString('utf8').trim(),revision,'selected package source changed');
 const original=blob(root,revision,path),cargo=blob(root,revision,'rust/Cargo.toml');
 assert.ok(readFileSync(join(root,'rust/Cargo.toml')).equals(cargo),'selected Cargo metadata changed');
 const physical=join(root,path);assert.equal(relative(root,realpathSync(physical)),path,'symlink package metadata');
 const actual=readFileSync(physical),manifest=JSON.parse(original),output=JSON.parse(actual);
 assert.equal(typeof manifest.name,'string');assert.equal(typeof manifest.version,'string');
 const version=/^version\s*=\s*"([^"]+)"/mu.exec(cargo.toString('utf8'))?.[1];assert.ok(version,'selected Cargo version missing');
 if(built)assert.deepEqual(output,{...manifest,version},'built package metadata differs beyond the selected Cargo version');
 else assert.ok(actual.equals(original),'selected package metadata differs before build');
 return {schema:'selected-release-package/v1',sourceCommit:revision,sourceTree:git(root,['rev-parse',revision+'^{tree}']).toString('utf8').trim(),
  packagePath:path,originalSha256:digest(original),originalVersion:manifest.version,derivedVersion:version,
  observedSha256:digest(actual),observedVersion:output.version,cargoSha256:digest(cargo),built};
}
if(process.argv[1]&&existsSync(process.argv[1])&&realpathSync(process.argv[1])===fileURLToPath(import.meta.url)){
 const [operation,root,directory,revision,...extra]=process.argv.slice(2);
 const result=operation==='capture'?captureReleaseProtocol(root,directory,revision,extra.length===1&&Object.hasOwn(RELEASE_PROTOCOL_FILES,extra[0])?RELEASE_PROTOCOL_FILES[extra[0]]:extra)
  :operation==='verify'?verifyReleaseProtocol(root,directory,revision,extra[0])
  :operation==='package'?verifySelectedPackage(root,revision,directory,extra[0]==='built'):assert.fail('capture, verify or package required');
 if(operation==='capture'){
  if(process.env.GITHUB_ENV)writeFileSync(process.env.GITHUB_ENV,'FORMAL_AI_RELEASE_PROTOCOL_DIR='+result.directory+'\n',{flag:'a'});
  if(process.env.GITHUB_OUTPUT)writeFileSync(process.env.GITHUB_OUTPUT,'protocol-sha256='+result.sha256+'\nupstream-commit='+(result.upstreamCommit??'')+'\n',{flag:'a'});
 }
 console.log(JSON.stringify(result));
}
