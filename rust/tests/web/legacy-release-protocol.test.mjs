import assert from 'node:assert/strict';
import test from 'node:test';
import {mkdtempSync,mkdirSync,readFileSync,writeFileSync,rmSync,existsSync,symlinkSync,statSync,chmodSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {tmpdir} from 'node:os';
import {join,dirname,resolve} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {captureReleaseProtocol,verifyReleaseProtocol,verifySelectedPackage,protocolUpstreamCommit,RELEASE_PROTOCOL_FILES} from '../../../scripts/release-protocol.mjs';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'../../..');
const paths=['scripts/release-protocol.mjs','scripts/publish-browser-engine.mjs','scripts/check-source-networks.mjs','scripts/verify-source-network-distribution.mjs','scripts/translate-js-rust.mjs','scripts/lib/source-network-packets.mjs','scripts/lib/source-network-workers.mjs','scripts/lib/source-network-worker.mjs','scripts/lib/translation-blockers.mjs','scripts/lib/translation-lowering.mjs','scripts/lib/translation-workarounds.mjs','scripts/run-with-budget-warning.sh','scripts/check-sccache-write-health.sh','.github/workflows/layered-ci.yml'];
const git=(cwd,...args)=>execFileSync('git',['-C',cwd,...args],{encoding:'utf8',stdio:['ignore','pipe','pipe']}).trim();
function put(cwd,path,body){mkdirSync(dirname(join(cwd,path)),{recursive:true});writeFileSync(join(cwd,path),body);}
function fixture(t){
 const base=mkdtempSync(join(tmpdir(),'legacy protocol ')),source=join(base,'selected source'),protocol=join(base,'captured protocol');mkdirSync(source);
 t.after(()=>rmSync(base,{recursive:true,force:true}));git(source,'init','-q');git(source,'config','user.email','fixture@example.invalid');git(source,'config','user.name','Fixture');
 put(source,'rust/Cargo.toml','[package]\nname = "example"\nversion = "1.2.3"\n');
 put(source,'rust/src/lib.rs','pub fn value() -> u32 { 17 }\r\n');put(source,'js/meaning.mjs','export const meaning = "真实来源😀";\r\n');put(source,'ts/meaning.mts','export const meaning: number = 17;\n');
 put(source,'packages/formal-ai-engine/package.json',JSON.stringify({name:'@example/engine',version:'0.8.0',type:'module',files:['assets']},null,2)+'\n');
 git(source,'add','.');git(source,'commit','-qm','historical source without protocol tools');const selected=git(source,'rev-parse','HEAD');git(source,'tag','v1.2.3');
 for(const path of paths){put(source,path,readFileSync(join(root,path)));chmodSync(join(source,path),statSync(join(root,path)).mode&0o777);}
 git(source,'add','.');git(source,'commit','-qm','trusted workflow tools');const workflow=git(source,'rev-parse','HEAD');
 const record=captureReleaseProtocol(source,protocol,workflow,paths);git(source,'checkout','-q','--detach','v1.2.3');
 assert.equal(existsSync(join(source,'scripts')),false);assert.equal(existsSync(join(source,'.github')),false);
 return {base,source,protocol,selected,workflow,record};
}
test('real historical checkout executes an independently captured CLI and keeps original source bytes',t=>{
 const f=fixture(t),before=readFileSync(join(f.source,'rust/src/lib.rs'));
 const stdout=execFileSync(process.execPath,[join(f.protocol,'scripts/release-protocol.mjs'),'verify',f.source,f.protocol,f.workflow,f.record.sha256],{encoding:'utf8'});
 const actual=JSON.parse(stdout);assert.equal(actual.commit,f.workflow);assert.equal(actual.tree,f.record.tree);assert.equal(actual.sha256,f.record.sha256);
 assert.equal(git(f.source,'rev-parse','HEAD'),f.selected);assert.ok(readFileSync(join(f.source,'rust/src/lib.rs')).equals(before));
 assert.notEqual(actual.commit,f.selected);
});
test('both actual workflow role recipes capture and verify their complete CLI dependency closures',t=>{
 const f=fixture(t);git(f.source,'checkout','-q','--detach',f.workflow);
 for(const role of ['engine','source-networks']){
  const directory=join(f.base,'role-'+role);
  const environment=join(f.base,role+'.env'),outputs=join(f.base,role+'.outputs');
  const observed=JSON.parse(execFileSync(process.execPath,[join(f.source,'scripts/release-protocol.mjs'),'capture',f.source,directory,f.workflow,role],{encoding:'utf8',env:{...process.env,GITHUB_ENV:environment,GITHUB_OUTPUT:outputs}}));
  assert.equal(readFileSync(environment,'utf8'),'FORMAL_AI_RELEASE_PROTOCOL_DIR='+directory+'\n');
  assert.ok(readFileSync(outputs,'utf8').includes('protocol-sha256='+observed.sha256+'\n'));
  assert.deepEqual(observed.files.map(file=>file.path),[...RELEASE_PROTOCOL_FILES[role]].sort());
  git(f.source,'checkout','-q','--detach',f.selected);
  const verified=JSON.parse(execFileSync(process.execPath,[join(directory,'scripts/release-protocol.mjs'),'verify',f.source,directory,f.workflow,observed.sha256],{encoding:'utf8'}));
  assert.equal(verified.sha256,observed.sha256);assert.equal(git(f.source,'rev-parse','HEAD'),f.selected);
  git(f.source,'checkout','-q','--detach',f.workflow);
 }
});
test('captured imports enumerate selected roots even when the selected tag has no layered workflow',async t=>{
 const f=fixture(t),producer=await import(pathToFileURL(join(f.protocol,'scripts/check-source-networks.mjs')).href);
 assert.deepEqual(producer.ownedProductionSources(f.source),['js/meaning.mjs','rust/src/lib.rs','ts/meaning.mts']);
 assert.equal(verifyReleaseProtocol(f.source,f.protocol,f.workflow).upstreamCommit,f.record.upstreamCommit);
});
test('protocol tampering is rejected by the immutable Git blob even without a supplied manifest digest',t=>{
 const f=fixture(t),path=join(f.protocol,'scripts/publish-browser-engine.mjs');writeFileSync(path,readFileSync(path,'utf8')+'\n// altered\n');
 assert.throws(()=>verifyReleaseProtocol(f.source,f.protocol,f.workflow),/differs from workflow source/);
});
test('a forged manifest cannot bless changed helper bytes',t=>{
 const f=fixture(t),file=join(f.protocol,'protocol-selection.json'),record=JSON.parse(readFileSync(file));record.files[0].sha256='0'.repeat(64);writeFileSync(file,JSON.stringify(record)+'\n');
 assert.throws(()=>verifyReleaseProtocol(f.source,f.protocol,f.workflow),/Expected values to be strictly equal/);
 assert.throws(()=>verifyReleaseProtocol(f.source,f.protocol,f.workflow,f.record.sha256),/protocol digest differs/);
});
test('wrong, short and unavailable workflow revisions fail closed',t=>{
 const f=fixture(t);
 for(const revision of [f.selected,'abc','f'.repeat(40)])assert.throws(()=>verifyReleaseProtocol(f.source,f.protocol,revision));
});
test('new capture refuses a dirty trusted source, duplicate inputs, escapes and symlinks',t=>{
 const f=fixture(t);git(f.source,'checkout','-q','--detach',f.workflow);
 const target=join(f.source,'scripts/publish-browser-engine.mjs'),old=readFileSync(target);writeFileSync(target,Buffer.concat([old,Buffer.from('\n// local\n')]));
 assert.throws(()=>captureReleaseProtocol(f.source,join(f.base,'dirty'),f.workflow,paths),/differs from workflow commit/);writeFileSync(target,old);
 assert.throws(()=>captureReleaseProtocol(f.source,join(f.base,'duplicate'),f.workflow,[paths[0],paths[0]]),/duplicate/);
 assert.throws(()=>captureReleaseProtocol(f.source,join(f.base,'escape'),f.workflow,['../escape']),/unsafe/);
 rmSync(target);symlinkSync(join(f.source,'scripts/check-source-networks.mjs'),target);
 assert.throws(()=>captureReleaseProtocol(f.source,join(f.base,'symlink'),f.workflow,paths));
});
test('captured symlink and missing import companion are not certified',t=>{
 const f=fixture(t),path=join(f.protocol,'scripts/lib/translation-blockers.mjs');rmSync(path);assert.throws(()=>verifyReleaseProtocol(f.source,f.protocol,f.workflow));
 symlinkSync(join(f.source,'js/meaning.mjs'),path);assert.throws(()=>verifyReleaseProtocol(f.source,f.protocol,f.workflow));
});
test('selected package metadata retains its source identity and permits only the source Cargo version projection',t=>{
 const f=fixture(t),initial=verifySelectedPackage(f.source,f.selected);assert.equal(initial.originalVersion,'0.8.0');assert.equal(initial.derivedVersion,'1.2.3');assert.equal(initial.observedVersion,'0.8.0');
 const path=join(f.source,'packages/formal-ai-engine/package.json'),original=JSON.parse(readFileSync(path));writeFileSync(path,JSON.stringify({...original,version:'1.2.3'},null,2)+'\n');
 const built=verifySelectedPackage(f.source,f.selected,undefined,true);assert.equal(built.observedVersion,'1.2.3');assert.equal(built.originalSha256,initial.originalSha256);
 assert.throws(()=>verifySelectedPackage(f.source,f.selected),/differs before build/);
 writeFileSync(path,JSON.stringify({...original,version:'1.2.3',name:'@foreign/package'})+'\n');assert.throws(()=>verifySelectedPackage(f.source,f.selected,undefined,true),/differs beyond/);
});
test('missing selected package and changed selected Cargo bytes never acquire current protocol package data',t=>{
 const f=fixture(t);assert.throws(()=>verifySelectedPackage(f.source,f.selected,'packages/absent/package.json'));
 writeFileSync(join(f.source,'rust/Cargo.toml'),'[package]\nname="foreign"\nversion="9.9.9"\n');assert.throws(()=>verifySelectedPackage(f.source,f.selected),/Cargo metadata changed/);
});
test('actual selected package version and tarball integrity reach the captured publication adapter',async t=>{
 const f=fixture(t),directory=join(f.source,'packages/formal-ai-engine'),adapter=await import(pathToFileURL(join(f.protocol,'scripts/publish-browser-engine.mjs')).href);
 put(f.source,'packages/formal-ai-engine/engine.tgz',Buffer.from('genuine fixture tarball bytes'));const calls=[];
 const result=adapter.publishBrowserEngine(directory,args=>{calls.push(args);if(args[0]==='view')throw Object.assign(new Error('missing'),{stdout:JSON.stringify({error:{code:'E404'}})});return 'fixture npm adapter; no publication\n';});
 assert.deepEqual(result,{publish:true});assert.equal(calls[0][1],'@example/engine@0.8.0');assert.equal(calls[1][1],join(directory,'engine.tgz'));
});
test('missing, disagreeing and malformed serializer pins refuse while repeated identical pins bind',()=>{
 const first='a'.repeat(40),second='b'.repeat(40),entry=pin=>'repository: link-foundation/meta-language\n  ref: '+pin+'\n';
 assert.equal(protocolUpstreamCommit(entry(first)+entry(first)),first);
 for(const text of ['',entry(first)+entry(second),entry('abc')])assert.throws(()=>protocolUpstreamCommit(text),/one exact/);
});
test('actual pinned upstream serializes selected historical sources and joins all eight unchanged source shards',
 {skip:!process.env.FORMAL_AI_LEGACY_PROTOCOL_UPSTREAM},t=>{
 const f=fixture(t),upstream=process.env.FORMAL_AI_LEGACY_PROTOCOL_UPSTREAM,distribution=join(f.base,'packets');
 for(let index=0;index<8;index++){put(f.source,'js/more-'+index+'.mjs','export const value = '+index+';\n');}
 // These are genuine source fixture inputs, committed before observing the new selected source identity.
 git(f.source,'add','js');git(f.source,'commit','-qm','more independently selected source inputs');const selected=git(f.source,'rev-parse','HEAD');
 for(let index=0;index<8;index++)execFileSync(process.execPath,[join(f.protocol,'scripts/check-source-networks.mjs'),'--source-root',f.source,'--upstream-commit',f.record.upstreamCommit,'--meta-language',upstream,'--shard-count','8','--shard-index',String(index),'--write','--output',join(distribution,'shard-'+index)],{cwd:f.source,stdio:['ignore','pipe','pipe'],timeout:60000});
 const joined=JSON.parse(execFileSync(process.execPath,[join(f.protocol,'scripts/verify-source-network-distribution.mjs'),distribution,'--source-root',f.source,'--upstream-commit',f.record.upstreamCommit],{cwd:f.source,encoding:'utf8'}));
 assert.equal(joined.sourceHead,selected);assert.equal(joined.checkedSources,11);assert.equal(joined.shardCount,8);assert.equal(joined.upstreamCommit,f.record.upstreamCommit);
 const outside=JSON.parse(execFileSync(process.execPath,[join(f.protocol,'scripts/verify-source-network-distribution.mjs'),distribution,'--source-root',f.source,'--upstream-commit',f.record.upstreamCommit],{cwd:f.base,encoding:'utf8'}));
 assert.deepEqual(outside,joined,'explicit selected source must not require ambient working-directory Git');
 assert.throws(()=>execFileSync(process.execPath,[join(f.protocol,'scripts/check-source-networks.mjs'),'--source-root',f.source,'--upstream-commit','a'.repeat(40),'--meta-language',upstream,'--write','--output',join(f.base,'wrong-upstream')],{cwd:f.source,stdio:['ignore','pipe','pipe']}));
 assert.equal(existsSync(join(f.base,'wrong-upstream/source-network-receipts.json')),false);
 assert.throws(()=>execFileSync(process.execPath,[join(f.protocol,'scripts/verify-source-network-distribution.mjs'),distribution,'--source-root',f.source,'--upstream-commit','a'.repeat(40)],{cwd:f.source,stdio:['ignore','pipe','pipe']}));
 put(f.source,'js/meaning.mjs','export const changed = true;\n');
 assert.throws(()=>execFileSync(process.execPath,[join(f.protocol,'scripts/verify-source-network-distribution.mjs'),distribution,'--source-root',f.source,'--upstream-commit',f.record.upstreamCommit],{cwd:f.source,stdio:['ignore','pipe','pipe']}));
});
