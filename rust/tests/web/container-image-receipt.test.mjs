// Typed fixtures test contracts; no Docker build, native execution or registry publication is asserted here.
import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {expectedIdentity} from '../../../scripts/native-release-artifact.mjs';
import {CONTAINER_TARGETS,validateContainerReceipt,manifestSources,validatePublishedIndex,publicationTags,observeContainer} from '../../../scripts/container-image-receipt.mjs';
import {verifyAnonymousImageManifest} from '../../../scripts/verify-anonymous-image-manifest.mjs';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const encode=value=>Buffer.from(JSON.stringify(value)+'\n');
const repository='ghcr.io/example/fixture-image';
function fixture(arch='amd64',variant='slim',published=true) {
 const source={version:1,source_commit:'a'.repeat(40),source_tree:'b'.repeat(40),producer_run:'81',
  package_version:'1.2.3',profile:'release',requested_features:'default',root_default_features:['portable'],rust_flags:'-Dwarnings',
  cargo_lock_sha256:'c'.repeat(64),native_inputs_sha256:'d'.repeat(64),compiler_release:'1.91.0',compiler_commit:'e'.repeat(40)};
 const sourceBytes=encode(source),sourceSha256=hash(sourceBytes),identity=expectedIdentity(source,CONTAINER_TARGETS[arch],sourceSha256,'81');
 const native={version:1,identity,executable:{name:'formal-ai',bytes:71,sha256:'f'.repeat(64)},producer:{job:'native',attempt:'1',
  compiler:'release: 1.91.0\ncommit-hash: '+'e'.repeat(40)+'\nhost: '+identity.host+'\n',environment:{RUSTFLAGS:'-Dwarnings'}}};
 const nativeBytes=encode(native),inputs={sourceBytes,nativeBytes,sourceCommit:source.source_commit,sourceTree:source.source_tree,
  sourceSha256,version:'1.2.3',run:'81',protocolSha256:'1'.repeat(64)};
 const manifest={digest:'sha256:'+(arch==='amd64'?'2':'3').repeat(64),mediaType:'application/vnd.oci.image.manifest.v1+json',size:713};
 const record={version:1,architecture:arch,variant,os:'linux',target:CONTAINER_TARGETS[arch],source_commit:source.source_commit,
  source_tree:source.source_tree,package_version:'1.2.3',producer_run:'81',source_selection_sha256:sourceSha256,
  protocol_sha256:inputs.protocolSha256,native_receipt_sha256:hash(nativeBytes),native_identity:identity,
  executable_sha256:native.executable.sha256,local_config_digest:'sha256:'+'4'.repeat(64),size_bytes:1024,
  published,repository,manifest:published?manifest:null,registry_reference:published?repository+'@'+manifest.digest:null,
  observations:{version_output:'formal-ai 1.2.3\n',binary_sha256:native.executable.sha256,runtime:{exit_code:0,stdout:'fixture runtime checked\n'}}};
 return {inputs,record};
}

test('both actual native GNU targets and variants carry independent complete executable identities',()=>{
 for(const arch of ['amd64','arm64'])for(const variant of ['full','slim']) {
  const {inputs,record}=fixture(arch,variant);assert.deepEqual(validateContainerReceipt(record,inputs,{arch,variant,published:true}),record);
 }
});
test('source run profile flags target and observation mismatches cannot pass',()=>{
 const {inputs,record}=fixture();
 for(const [field,value] of [['source_commit','0'.repeat(40)],['source_tree','0'.repeat(40)],['producer_run','80'],
  ['target',CONTAINER_TARGETS.arm64],['source_selection_sha256','0'.repeat(64)],['protocol_sha256','0'.repeat(64)],
  ['native_receipt_sha256','0'.repeat(64)],['executable_sha256','0'.repeat(64)]]) {
  assert.throws(()=>validateContainerReceipt({...record,[field]:value},inputs,{arch:'amd64',variant:'slim',published:true}),field);
 }
 for(const change of [r=>r.native_identity.rust_flags='',r=>r.native_identity.profile='debug',
  r=>r.observations.version_output='formal-ai 1.2.4',r=>r.observations.binary_sha256='0'.repeat(64),
  r=>r.observations.runtime.exit_code=1,r=>r.observations.runtime.stdout='',r=>r.size_bytes=2147483649]) {
  const changed=structuredClone(record);change(changed);assert.throws(()=>validateContainerReceipt(changed,inputs,{arch:'amd64',variant:'slim',published:true}));
 }
 assert.throws(()=>validateContainerReceipt(record,{...inputs,sourceSha256:'0'.repeat(64)},{arch:'amd64',variant:'slim',published:true}));
});
test('dry validation reports local identities without inventing registry manifests',()=>{
 const {inputs,record}=fixture('arm64','slim',false);
 assert.equal(validateContainerReceipt(record,inputs,{arch:'arm64',variant:'slim',published:false}).registry_reference,null);
 assert.throws(()=>validateContainerReceipt(record,inputs,{arch:'arm64',variant:'slim',published:true}));
 assert.throws(()=>validateContainerReceipt({...record,manifest:{digest:record.local_config_digest}},inputs,{arch:'arm64',variant:'slim',published:false}));
});
test('merge requires all four genuine architecture variant records with no omissions or duplicates',()=>{
 const all=['full','slim'].flatMap(v=>['amd64','arm64'].map(a=>fixture(a,v).record));
 const inputs=Object.fromEntries(['amd64','arm64'].map(a=>[a,fixture(a).inputs]));
 const selected=manifestSources(all,inputs,'full');assert.deepEqual(selected.map(r=>r.architecture),['amd64','arm64']);
 assert(selected.every(r=>r.registry_reference.startsWith(repository+'@sha256:')));
 assert.throws(()=>manifestSources(all.slice(1),inputs,'full'));
 assert.throws(()=>manifestSources([all[0],all[0],all[2],all[3]],inputs,'full'));
 const changed=structuredClone(all);changed[0].published=false;assert.throws(()=>manifestSources(changed,inputs,'slim'));
});
test('final immutable index must contain both exact source manifest descriptors once',()=>{
 const receipts=['amd64','arm64'].map(a=>fixture(a).record);
 const index={schemaVersion:2,mediaType:'application/vnd.oci.image.index.v1+json',manifests:receipts.map(r=>({...r.manifest,platform:{os:'linux',architecture:r.architecture}}))};
 assert.deepEqual(validatePublishedIndex(index,receipts),index);
 for(const change of [i=>i.manifests.pop(),i=>i.manifests[1].platform.architecture='amd64',
  i=>i.manifests[0].digest='sha256:'+'0'.repeat(64),i=>i.manifests[0].size++,i=>i.mediaType='not-an-index']) {
  const bad=structuredClone(index);change(bad);assert.throws(()=>validatePublishedIndex(bad,receipts));
 }
});
function observerFixture(arch,variant,published) {
 const f=fixture(arch,variant,published),calls=[];
 const inspection={Os:'linux',Architecture:arch,Id:f.record.local_config_digest,Size:f.record.size_bytes,
  RepoDigests:published?[f.record.registry_reference]:[],Config:{Env:['FORMAL_AI_IMAGE_VARIANT='+(variant==='slim'?'slim':'dind')],Labels:{
   'org.opencontainers.image.revision':f.record.source_commit,'org.opencontainers.image.version':'1.2.3',
   'io.link-assistant.formal-ai.executable.sha256':f.record.executable_sha256}}};
 const outputs=[JSON.stringify([inspection]),f.record.observations.version_output,
  f.record.executable_sha256+'  /usr/local/bin/formal-ai\n',f.record.observations.runtime.stdout,JSON.stringify(f.record.manifest)];
 const run=(command,args)=>{assert.equal(command,'docker');calls.push(args);return outputs.shift();};
 return {...f,inspection,calls,outputs,run};
}
test('actual command path uses explicit slim entrypoint and full privileged DinD checks',()=>{
 for(const variant of ['slim','full']) {
  const f=observerFixture('arm64',variant,true),result=observeContainer(f.inputs,{image:'fixture-image',arch:'arm64',variant,published:true,repository},f.run);
  assert.equal(result.registry_reference,f.record.registry_reference);assert.equal(f.calls.length,5);
  assert.deepEqual(f.calls[1],['run','--rm','--entrypoint','/usr/local/bin/formal-ai','fixture-image','--version']);
  assert.deepEqual(f.calls[3],variant==='slim'?['run','--rm','--entrypoint','verify-formal-ai-slim','fixture-image']
   :['run','--rm','--privileged','fixture-image','verify-formal-ai-dind']);
  assert.deepEqual(f.calls[4],['buildx','imagetools','inspect',f.record.registry_reference,'--format','{{json .Manifest}}']);
 }
});
test('actual absent registry identity, wrong image architecture and failed execution refuse receipts',()=>{
 for(const change of [i=>i.RepoDigests=[],i=>i.Architecture='amd64',i=>i.Config.Env=['FORMAL_AI_IMAGE_VARIANT=dind'],i=>i.Config.Labels['org.opencontainers.image.revision']='0'.repeat(40)]) {
  const f=observerFixture('arm64','slim',true);change(f.inspection);f.outputs[0]=JSON.stringify([f.inspection]);
  assert.throws(()=>observeContainer(f.inputs,{image:'fixture-image',arch:'arm64',variant:'slim',published:true,repository},f.run));
 }
 const f=fixture();assert.throws(()=>observeContainer(f.inputs,{image:'fixture-image',arch:'amd64',variant:'slim',published:true,repository},()=>{throw Error('actual runtime failed');}),/actual runtime failed/u);
});

test('an older stable release can publish its immutable version without replacing moving tags',()=>{
 assert.deepEqual(publicationTags(repository,'1.2.3','full',false),[repository+':1.2.3']);
 assert.deepEqual(publicationTags(repository,'1.2.3','slim',false),[repository+':1.2.3-slim']);
 assert.deepEqual(publicationTags(repository,'1.2.3','full',true),[repository+':1.2.3',repository+':latest']);
 assert.deepEqual(publicationTags(repository,'1.2.3','slim',true),[repository+':1.2.3-slim',repository+':slim']);
});

test('both exact native platform descriptors are asserted after genuine anonymous byte verification',async()=>{
 const receipts=['amd64','arm64'].map(a=>fixture(a).record);
 const index={schemaVersion:2,mediaType:'application/vnd.oci.image.index.v1+json',
  manifests:receipts.map(r=>({...r.manifest,platform:{os:'linux',architecture:r.architecture}}))};
 const verify=async manifest=>{
  const bytes=encode(manifest),digest='sha256:'+hash(bytes),image=repository+'@'+digest;
  let calls=0,observed=false;
  const fetcher=async()=>{calls++;return calls===1?new Response(JSON.stringify({token:'canned-anonymous-token'}),{status:200})
   :new Response(bytes,{status:200,headers:{'Docker-Content-Digest':digest}});};
  const result=await verifyAnonymousImageManifest(image,fetcher,actual=>{observed=true;validatePublishedIndex(actual,receipts);});
  assert.equal(calls,2);assert.equal(observed,true);assert.equal(result.manifest_sha256,digest);
 };
 await verify(index);
 const missing=structuredClone(index);missing.manifests.pop();await assert.rejects(()=>verify(missing));
 const wrong=structuredClone(index);wrong.manifests[1].platform.architecture='amd64';await assert.rejects(()=>verify(wrong));
});
