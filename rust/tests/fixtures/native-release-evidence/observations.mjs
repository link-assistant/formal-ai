// Canned typed producer observations for release resolver tests; no CI execution is claimed.
import {mkdirSync,writeFileSync,readFileSync} from 'node:fs';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {expectedIdentity,NATIVE_TARGETS} from '../../../../scripts/native-release-artifact.mjs';
import {bindMacPackageExecutable,MAC_PACKAGE_TARGETS,collectReleaseEvidence,PROTOCOL_HELPERS,sha256} from '../../../../scripts/native-release-evidence.mjs';
const encode=value=>Buffer.from(JSON.stringify(value,null,2)+'\n');
export function createEvidenceFixture(options={}) {
 const source={version:1,source_commit:'a'.repeat(40),source_tree:'b'.repeat(40),compiler_commit:'c'.repeat(40),
  compiler_release:'1.91.0',cargo_lock_sha256:'d'.repeat(64),native_inputs_sha256:'e'.repeat(64),
  package_version:'2.7.11',producer_run:'91',profile:'release',requested_features:'default',
  root_default_features:['portable'],rust_flags:'-Dwarnings'};
 Object.assign(source,{package_version:options.version??source.package_version,source_commit:options.sourceCommit??source.source_commit,source_tree:options.sourceTree??source.source_tree});
 const sourceBytes=encode(source),helperBytes=Object.fromEntries(PROTOCOL_HELPERS.map(name=>[name,Buffer.from('actual selected fixture helper bytes '+name+'\n')]));
 const protocol={version:1,commit:'f'.repeat(40),tree:'1'.repeat(40),files:PROTOCOL_HELPERS.map(name=>({name,bytes:helperBytes[name].length,sha256:sha256(helperBytes[name])}))};
 const protocolBytes=encode(protocol),bindings={sourceSha256:sha256(sourceBytes),sourceCommit:source.source_commit,
  sourceTree:source.source_tree,version:source.package_version,run:source.producer_run,
  compilerRelease:source.compiler_release,compilerCommit:source.compiler_commit,
  protocolSha256:sha256(protocolBytes),protocolCommit:protocol.commit,protocolTree:protocol.tree};
 const receipts=Object.keys(NATIVE_TARGETS).map(target=>{
  const identity=expectedIdentity(source,target,bindings.sourceSha256,bindings.run);
  return {target,bytes:encode({version:1,identity,producer:{job:'native',attempt:'1',
   compiler:'rustc 1.91.0\nrelease: 1.91.0\ncommit-hash: '+source.compiler_commit+'\nhost: '+identity.host+'\n',
   environment:{RUSTFLAGS:'-Dwarnings'}},executable:{name:identity.platform==='win32'?'formal-ai.exe':'formal-ai',bytes:59,sha256:'2'.repeat(64)}})};
 });
 const observation=(stderr='')=>({status:0,stdout:'',stderr});
 const signing=['macos-arm64','macos-x64'].map(label=>({label,bytes:encode({version:1,label,
  source_selection_sha256:bindings.sourceSha256,source_commit:source.source_commit,package_version:source.package_version,
  producer_run:source.producer_run,mode:'adhoc',package:{name:'formal-ai-desktop-'+label+'-'+source.package_version+'.dmg',bytes:123,sha256:'3'.repeat(64)},
  native_component:{bytes:99,sha256:'4'.repeat(64)},observations:{verify:observation(),display:observation('Signature=adhoc\n'),assess:null,stapler:null}})}));
 for(const entry of signing) {
  const record=JSON.parse(entry.bytes),parent=receipts.find(item=>item.target===MAC_PACKAGE_TARGETS[entry.label]);
  record.package_executable_receipt=bindMacPackageExecutable(parent.bytes,{source,sourceSha256:bindings.sourceSha256,label:entry.label,
   packageObservation:record.package,executable:record.native_component,
   startup:{arguments:['--version'],status:0,signal:null,complete:true,stdout:'formal-ai '+source.package_version+'\n',stderr:''},signingObservations:record.observations});
  record.package_executable_sha256=sha256(Buffer.from(JSON.stringify(record.package_executable_receipt)+'\n'));entry.bytes=encode(record);
 }
 return {source,protocol,sourceBytes,protocolBytes,helperBytes,bindings,receipts,signing};
}
export function materializePublishedFixture(directory,{version,sourceCommit='b'.repeat(40),sourceTree='d'.repeat(40),expectedAssets}) {
 const fixture=createEvidenceFixture({version,sourceCommit,sourceTree});
 for(const entry of fixture.signing) {
  const record=JSON.parse(entry.bytes),bytes=Buffer.from('canned release artifact '+record.package.name+'\n');
  record.package.bytes=bytes.length;record.package.sha256=sha256(bytes);
  record.package_executable_receipt.package=record.package;
  record.package_executable_sha256=sha256(Buffer.from(JSON.stringify(record.package_executable_receipt)+'\n'));entry.bytes=encode(record);
 }
 const result=collectReleaseEvidence(fixture);if(!result.complete)throw new Error('incomplete typed fixture');
 const payloads=new Map(result.assets.map(asset=>[asset.name,asset.bytes]));
 for(const name of expectedAssets)if(!payloads.has(name))payloads.set(name,Buffer.from('canned release artifact '+name+'\n'));
 const checksums=[...payloads].filter(([name])=>!['SHA256SUMS.txt','BUILD-PROVENANCE.txt'].includes(name))
  .sort(([left],[right])=>left.localeCompare(right,'en')).map(([name,bytes])=>sha256(bytes)+'  '+name+'\n').join('');
 payloads.set('SHA256SUMS.txt',Buffer.from(checksums));payloads.set('BUILD-PROVENANCE.txt',Buffer.from('Canned typed test evidence; no native run asserted.\n'));
 mkdirSync(directory,{recursive:true});for(const [name,bytes] of payloads)writeFileSync(join(directory,name),bytes);
 const metadata={assets:[...payloads].map(([name,bytes])=>({name,digest:'sha256:'+sha256(bytes)}))};
 writeFileSync(join(directory,'release-assets.json'),JSON.stringify(metadata)+'\n');
 return {fixture,payloads,metadata};
}
if(import.meta.url===pathToFileURL(process.argv[1]??'').href){
 const [directory,version,namesFile,mode]=process.argv.slice(2);
 const result=materializePublishedFixture(directory,{version,expectedAssets:readFileSync(namesFile,'utf8').trim().split(/\r?\n/u)});
 if(mode==='metadata')console.log(JSON.stringify(result.metadata));
}
