// Typed observation fixtures exercise validation; they do not claim actual CI signing or execution.
import assert from 'node:assert/strict';
import {test} from 'node:test';
import {mkdtempSync,readFileSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {expectedIdentity,NATIVE_TARGETS} from '../../../scripts/native-release-artifact.mjs';
import {collectReleaseEvidence,prepareReleaseEvidence,validateSigningEvidence,verifyPublishedEvidence,PROTOCOL_HELPERS,sha256} from '../../../scripts/native-release-evidence.mjs';
import {materializePublishedFixture} from '../fixtures/native-release-evidence/observations.mjs';
const encode=value=>Buffer.from(JSON.stringify(value,null,2)+'\n');
function fixture() {
 const source={version:1,source_commit:'a'.repeat(40),source_tree:'b'.repeat(40),compiler_commit:'c'.repeat(40),
  compiler_release:'1.91.0',cargo_lock_sha256:'d'.repeat(64),native_inputs_sha256:'e'.repeat(64),
  package_version:'2.7.11',producer_run:'91',profile:'release',requested_features:'default',
  root_default_features:['portable'],rust_flags:'-Dwarnings'};
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
 return {source,protocol,sourceBytes,protocolBytes,helperBytes,bindings,receipts,signing};
}
function changed(entry,change){const value=JSON.parse(entry.bytes);change(value);return {...entry,bytes:encode(value)};}

test('complete eight target observations preserve every original attachment byte and deterministic digest',context=>{
 const f=fixture(),result=collectReleaseEvidence(f);
 assert.equal(result.complete,true);assert.deepEqual(result.failures,[]);assert.equal(result.assets.length,12);
 assert.deepEqual(result.signingModes,[{label:'macos-arm64',mode:'adhoc'},{label:'macos-x64',mode:'adhoc'}]);
 const root=mkdtempSync(join(tmpdir(),'native-durable-evidence-'));context.after(()=>rmSync(root,{recursive:true,force:true}));
 prepareReleaseEvidence(root,result);
 for(const asset of result.assets)assert.deepEqual(readFileSync(join(root,asset.name)),asset.bytes);
 assert.deepEqual(result.assets.find(a=>a.name.startsWith('formal-ai-native-source-')).bytes,f.sourceBytes);
 assert.deepEqual(result.assets.find(a=>a.name.startsWith('formal-ai-native-protocol-')).bytes,f.protocolBytes);
 for(const entry of f.receipts)assert.deepEqual(result.assets.find(a=>a.name==='formal-ai-native-'+entry.target+'-2.7.11.json').bytes,entry.bytes);
 assert.equal(readFileSync(join(root,'SHA256SUMS-evidence.partial'),'utf8'),result.assets.map(a=>sha256(a.bytes)+'  '+a.name+'\n').join(''));
});

test('source and protocol digests are independent and source bindings refuse each mismatch',()=>{
 const f=fixture();
 for(const key of ['sourceSha256','sourceCommit','sourceTree','version','run','compilerRelease','compilerCommit','protocolSha256','protocolCommit','protocolTree']){
  assert.throws(()=>collectReleaseEvidence({...f,bindings:{...f.bindings,[key]:'mismatch'}}),key);
 }
 assert.throws(()=>collectReleaseEvidence({...f,sourceBytes:Buffer.concat([f.sourceBytes,Buffer.from(' ')] )}));
 assert.throws(()=>collectReleaseEvidence({...f,protocolBytes:Buffer.concat([f.protocolBytes,Buffer.from(' ')] )}));
});

test('protocol helper payload corruption, traversal and duplicate helper names refuse',()=>{
 const f=fixture();assert.throws(()=>collectReleaseEvidence({...f,helperBytes:{...f.helperBytes,[PROTOCOL_HELPERS[0]]:Buffer.from('wrong')}}));
 for(const change of [p=>p.files[0].name='../outside.mjs',p=>p.files[1].name=p.files[0].name,p=>p.files.pop()]){
  const protocol=structuredClone(f.protocol);change(protocol);const bytes=encode(protocol);
  assert.throws(()=>collectReleaseEvidence({...f,protocolBytes:bytes,bindings:{...f.bindings,protocolSha256:sha256(bytes)}}));
 }
});

test('missing and duplicate target observations remain failures with no substituted receipt asset',()=>{
 const f=fixture(),target=f.receipts[0].target;
 for(const receipts of [f.receipts.slice(1),[...f.receipts,f.receipts[0]]]){
  const result=collectReleaseEvidence({...f,receipts});assert.equal(result.complete,false);
  assert.deepEqual(result.failures.map(x=>[x.kind,x.target]),[['native',target]]);
  assert.equal(result.assets.some(a=>a.name==='formal-ai-native-'+target+'-2.7.11.json'),false);
 }
});

test('all independent compiler target flags source and executable record defects are refused',()=>{
 const f=fixture(),first=f.receipts[0];
 const changes=[r=>r.identity.source_commit='0'.repeat(40),r=>r.identity.source_tree='0'.repeat(40),
  r=>r.identity.target=f.receipts[1].target,r=>r.identity.compiler_commit='0'.repeat(40),r=>r.identity.producer_run='90',
  r=>r.producer.compiler=r.producer.compiler.replace('host: ','host: wrong-'),r=>r.producer.environment.RUSTFLAGS='',
  r=>r.producer.environment.CARGO_ENCODED_RUSTFLAGS='override',r=>r.producer.environment.CARGO_PROFILE_RELEASE_LTO='true',
  r=>r.executable.name='substitute',r=>r.executable.bytes=0,r=>r.executable.sha256='not-a-sha'];
 for(const change of changes){
  const result=collectReleaseEvidence({...f,receipts:[changed(first,change),...f.receipts.slice(1)]});
  assert.equal(result.complete,false);assert.equal(result.failures[0].target,first.target);
 }
});

test('unknown target and label are honest extra-input failures',()=>{
 const f=fixture(),result=collectReleaseEvidence({...f,receipts:[...f.receipts,{target:'unknown',bytes:Buffer.from('{}')}],
  signing:[...f.signing,{label:'windows-x64',bytes:Buffer.from('{}')}]});
 assert.equal(result.complete,false);assert.deepEqual(result.failures.map(x=>x.target),['unknown','windows-x64']);
});

test('mixed signed and adhoc modes follow captured public observations independently',()=>{
 const f=fixture(),entry=changed(f.signing[0],r=>{
  r.mode='signed';r.observations.display.stderr='Authority=Developer ID Application: Example\nTeamIdentifier=ABC123XYZ\n';
  r.observations.assess={status:0,stdout:'',stderr:'accepted\n'};r.observations.stapler={status:0,stdout:'validated\n',stderr:''};
 });
 const result=collectReleaseEvidence({...f,signing:[entry,f.signing[1]]});assert.equal(result.complete,true);
 assert.deepEqual(result.signingModes,[{label:'macos-arm64',mode:'signed'},{label:'macos-x64',mode:'adhoc'}]);
});

test('missing failed contradictory or misbound signing observations never default to adhoc',()=>{
 const f=fixture();
 const changes=[r=>r.mode='signed',r=>r.source_selection_sha256='0'.repeat(64),r=>r.source_commit='0'.repeat(40),
  r=>r.package_version='3.0.0',r=>r.observations.verify.status=1,r=>r.observations.display.status=1,
  r=>r.observations.display.stderr='Signature=adhoc\nAuthority=Developer ID Application: Example\n'];
 for(const change of changes){const result=collectReleaseEvidence({...f,signing:[changed(f.signing[0],change),f.signing[1]]});
  assert.equal(result.complete,false);assert.deepEqual(result.signingModes,[{label:'macos-x64',mode:'adhoc'}]);}
 const missing=collectReleaseEvidence({...f,signing:[f.signing[1]]});assert.equal(missing.complete,false);
 assert.equal(missing.failures[0].target,'macos-arm64');
 const duplicated=collectReleaseEvidence({...f,signing:[...f.signing,f.signing[0]]});assert.equal(duplicated.complete,false);
});

test('signed observations require successful assessment stapling and a public team identity',()=>{
 const f=fixture(),record=JSON.parse(f.signing[0].bytes);record.mode='signed';
 record.observations.display.stderr='Authority=Developer ID Application: Example\nTeamIdentifier=ABC123XYZ\n';
 record.observations.assess={status:0};record.observations.stapler={status:0};
 assert.equal(validateSigningEvidence(record,f.source,record.label,f.bindings.sourceSha256),'signed');
 for(const key of ['assess','stapler']){const failed=structuredClone(record);failed.observations[key].status=1;
  assert.throws(()=>validateSigningEvidence(failed,f.source,failed.label,f.bindings.sourceSha256));}
 record.observations.display.stderr='Authority=Developer ID Application: Example\n';
 assert.throws(()=>validateSigningEvidence(record,f.source,record.label,f.bindings.sourceSha256));
});

// The successful attestation callback is an explicit fixture, not a cryptographic observation.
test('published DMG bytes must match the independently captured signing package digest',context=>{
 const root=mkdtempSync(join(tmpdir(),'native-published-dmg-'));context.after(()=>rmSync(root,{recursive:true,force:true}));
 const expectedAssets=['formal-ai-desktop-macos-arm64-2.7.11.dmg','formal-ai-desktop-macos-x64-2.7.11.dmg'];
 const observed=materializePublishedFixture(root,{version:'2.7.11',expectedAssets});
 const bindings={version:'2.7.11',sourceCommit:observed.fixture.source.source_commit,
  sourceTree:observed.fixture.source.source_tree,expectedAssets,assetMetadata:observed.metadata,verifyAttestation:()=>true};
 assert.equal(verifyPublishedEvidence(root,bindings).complete,true);
 const name=expectedAssets[0],replacement=Buffer.from('different published DMG bytes\n'),digest=sha256(replacement);
 const original=sha256(observed.payloads.get(name));
 writeFileSync(join(root,name),replacement);
 writeFileSync(join(root,'SHA256SUMS.txt'),readFileSync(join(root,'SHA256SUMS.txt'),'utf8').replace(original+'  '+name,digest+'  '+name));
 observed.metadata.assets.find(asset=>asset.name===name).digest='sha256:'+digest;
 assert.throws(()=>verifyPublishedEvidence(root,bindings),/published DMG differs from signing observation/u);
});
