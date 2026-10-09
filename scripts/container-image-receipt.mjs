// Native registry receipts derive from actual Docker observations and independent executable receipts.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {readFileSync,writeFileSync,readdirSync,appendFileSync} from 'node:fs';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {expectedIdentity,validateExecutableReceipt,RECEIPT_FILE} from './native-release-artifact.mjs';
import {SOURCE_FILE} from './native-release-source.mjs';
import {verifyAnonymousImageManifest} from './verify-anonymous-image-manifest.mjs';
export const CONTAINER_TARGETS=Object.freeze({amd64:'x86_64-unknown-linux-gnu',arm64:'aarch64-unknown-linux-gnu'});
export const VARIANTS=['full','slim'];
const imageTypes=['application/vnd.oci.image.manifest.v1+json','application/vnd.docker.distribution.manifest.v2+json'];
const indexTypes=['application/vnd.oci.image.index.v1+json','application/vnd.docker.distribution.manifest.list.v2+json'];
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const digest=value=>assert.match(value,/^sha256:[a-f0-9]{64}$/u);
const decode=bytes=>JSON.parse(Buffer.from(bytes).toString('utf8'));

export function validateImageInputs(inputs,arch) {
 assert.ok(Object.hasOwn(CONTAINER_TARGETS,arch),'unknown native container architecture');
 const {sourceBytes,nativeBytes,sourceCommit,sourceTree,sourceSha256,version,run,protocolSha256}=inputs;
 assert.equal(hash(sourceBytes),sourceSha256);assert.match(protocolSha256,/^[a-f0-9]{64}$/u);
 const source=decode(sourceBytes);assert.equal(source.source_commit,sourceCommit);assert.equal(source.source_tree,sourceTree);
 assert.match(sourceCommit,/^[a-f0-9]{40}$/u);assert.match(sourceTree,/^[a-f0-9]{40}$/u);
 assert.equal(source.package_version,version);assert.equal(source.producer_run,String(run));
 const native=validateExecutableReceipt(decode(nativeBytes),expectedIdentity(source,CONTAINER_TARGETS[arch],sourceSha256,run));
 return {source,native,nativeSha256:hash(nativeBytes)};
}

export function validateContainerReceipt(record,inputs,{arch,variant,published}) {
 assert.ok(VARIANTS.includes(variant),'unknown container variant');
 const {source,native,nativeSha256}=validateImageInputs(inputs,arch);
 assert.equal(record.version,1);assert.equal(record.architecture,arch);assert.equal(record.variant,variant);
 assert.equal(record.os,'linux');assert.equal(record.target,CONTAINER_TARGETS[arch]);
 assert.equal(record.source_commit,source.source_commit);assert.equal(record.source_tree,source.source_tree);
 assert.equal(record.package_version,source.package_version);assert.equal(record.producer_run,String(inputs.run));
 assert.equal(record.source_selection_sha256,inputs.sourceSha256);assert.equal(record.protocol_sha256,inputs.protocolSha256);
 assert.equal(record.native_receipt_sha256,nativeSha256);assert.deepEqual(record.native_identity,native.identity);
 assert.equal(record.executable_sha256,native.executable.sha256);
 assert.equal(record.observations.version_output.trim(),'formal-ai '+source.package_version);
 assert.equal(record.observations.binary_sha256,native.executable.sha256);
 assert.equal(record.observations.runtime.exit_code,0);assert.ok(record.observations.runtime.stdout.trim());
 digest(record.local_config_digest);assert.ok(Number.isSafeInteger(record.size_bytes)&&record.size_bytes>0);
 if(variant==='slim')assert.ok(record.size_bytes<=2147483648,'slim image exceeds the 2048MiB budget');
 if(published) {
  assert.equal(record.published,true);assert.match(record.repository,/^[a-z0-9.-]+\/[a-z0-9_./-]+$/u);
  digest(record.manifest.digest);assert.ok(imageTypes.includes(record.manifest.mediaType),'a per-architecture image manifest is required');
  assert.ok(Number.isSafeInteger(record.manifest.size)&&record.manifest.size>0);
  assert.equal(record.registry_reference,record.repository+'@'+record.manifest.digest);
 } else {
  assert.equal(record.published,false);assert.equal(record.manifest,null);assert.equal(record.registry_reference,null);
 }
 return record;
}

export function manifestSources(records,inputByArch,variant) {
 assert.equal(records.length,4,'all four actual architecture/variant receipts are required');
 const selected=[];
 for(const currentVariant of VARIANTS)for(const arch of Object.keys(CONTAINER_TARGETS)) {
  const matching=records.filter(record=>record.architecture===arch&&record.variant===currentVariant);
  assert.equal(matching.length,1,'missing or duplicate architecture/variant receipt');
  const receipt=validateContainerReceipt(matching[0],inputByArch[arch],{arch,variant:currentVariant,published:true});
  if(currentVariant===variant)selected.push(receipt);
 }
 assert.ok(VARIANTS.includes(variant));assert.equal(new Set(selected.map(record=>record.repository)).size,1);
 return selected;
}

export function publicationTags(repository,version,variant,isLatest) {
 assert.ok(VARIANTS.includes(variant));assert.match(version,/^[0-9]+\.[0-9]+\.[0-9]+$/u);
 const tags=[repository+':'+version+(variant==='slim'?'-slim':'')];
 if(isLatest)tags.push(repository+':'+(variant==='slim'?'slim':'latest'));
 return tags;
}

export function validatePublishedIndex(index,receipts) {
 assert.equal(index.schemaVersion,2);assert.ok(indexTypes.includes(index.mediaType));assert.equal(index.manifests.length,2);
 for(const receipt of receipts) {
  const matching=index.manifests.filter(item=>item.platform?.os==='linux'&&item.platform.architecture===receipt.architecture);
  assert.equal(matching.length,1,'index must contain each native architecture exactly once');
  assert.equal(matching[0].digest,receipt.manifest.digest);assert.equal(matching[0].mediaType,receipt.manifest.mediaType);
  assert.equal(matching[0].size,receipt.manifest.size);
 }
 return index;
}

/** Execute each Docker command; failed startup/runtime or unavailable registry refuses a receipt. */
export function observeContainer(inputs,{image,arch,variant,published,repository},run=execFileSync) {
 const {source,native,nativeSha256}=validateImageInputs(inputs,arch);
 const invoke=args=>run('docker',args,{encoding:'utf8',timeout:300000,maxBuffer:8*1024*1024});
 const inspections=JSON.parse(invoke(['image','inspect',image]));assert.equal(inspections.length,1);
 const inspection=inspections[0];assert.equal(inspection.Os,'linux');assert.equal(inspection.Architecture,arch);
 const environments=inspection.Config?.Env??[];
 const imageVariant=environments.filter(value=>value.startsWith('FORMAL_AI_IMAGE_VARIANT=')).at(-1);
 assert.equal(imageVariant,'FORMAL_AI_IMAGE_VARIANT='+(variant==='slim'?'slim':'dind'),'actual image variant differs');
 const labels=inspection.Config?.Labels??{};
 assert.equal(labels['org.opencontainers.image.revision'],source.source_commit);
 assert.equal(labels['org.opencontainers.image.version'],source.package_version);
 assert.equal(labels['io.link-assistant.formal-ai.executable.sha256'],native.executable.sha256);
 const versionOutput=invoke(['run','--rm','--entrypoint','/usr/local/bin/formal-ai',image,'--version']);
 const output=invoke(['run','--rm','--entrypoint','sha256sum',image,'/usr/local/bin/formal-ai']);
 const binary=/^([a-f0-9]{64})[ \t]+\*?\/usr\/local\/bin\/formal-ai\n?$/u.exec(output);assert.ok(binary);
 const runtimeArgs=variant==='slim'?['run','--rm','--entrypoint','verify-formal-ai-slim',image]
  :['run','--rm','--privileged',image,'verify-formal-ai-dind'];
 const runtimeOutput=invoke(runtimeArgs);
 let manifest=null,reference=null;
 if(published) {
  const references=(inspection.RepoDigests??[]).filter(value=>value.startsWith(repository+'@sha256:'));
  assert.equal(new Set(references).size,1,'one actual pushed repository digest required');reference=references[0];
  manifest=JSON.parse(invoke(['buildx','imagetools','inspect',reference,'--format','{{json .Manifest}}']));
  assert.equal(reference,repository+'@'+manifest.digest);
 }
 const record={version:1,architecture:arch,variant,os:'linux',target:CONTAINER_TARGETS[arch],source_commit:source.source_commit,
  source_tree:source.source_tree,package_version:source.package_version,producer_run:String(inputs.run),
  source_selection_sha256:inputs.sourceSha256,protocol_sha256:inputs.protocolSha256,native_receipt_sha256:nativeSha256,
  native_identity:native.identity,executable_sha256:native.executable.sha256,local_config_digest:inspection.Id,
  size_bytes:inspection.Size,published,repository,manifest,registry_reference:reference,
  observations:{version_output:versionOutput,binary_sha256:binary[1],runtime:{exit_code:0,stdout:runtimeOutput}}};
 return validateContainerReceipt(record,inputs,{arch,variant,published});
}

function inputsFor(sourceDirectory,nativeDirectory,environment=process.env) {
 return {sourceBytes:readFileSync(join(sourceDirectory,SOURCE_FILE)),nativeBytes:readFileSync(join(nativeDirectory,RECEIPT_FILE)),
  sourceCommit:environment.CONTAINER_SOURCE_COMMIT,sourceTree:environment.CONTAINER_SOURCE_TREE,
  sourceSha256:environment.NATIVE_SELECTION_SHA256,version:environment.CONTAINER_VERSION,run:environment.GITHUB_RUN_ID,
  protocolSha256:environment.CONTAINER_PROTOCOL_SHA256};
}
if(import.meta.url===pathToFileURL(process.argv[1]??'').href) {
 const [mode,sourceDirectory,nativeDirectory,image,arch,variant,output]=process.argv.slice(2);
 if(mode==='observe') {
  const record=observeContainer(inputsFor(sourceDirectory,nativeDirectory),{image,arch,variant,
   published:process.env.CONTAINER_PUBLISH==='true',repository:process.env.CONTAINER_REPOSITORY});
  writeFileSync(output,JSON.stringify(record,null,2)+'\n');console.log(JSON.stringify(record));
 } else if(mode==='merge') {
  const receiptRoot=image,repository=arch;
  const inputByArch=Object.fromEntries(Object.keys(CONTAINER_TARGETS).map(key=>[key,inputsFor(sourceDirectory,join(nativeDirectory,key))]));
  const files=readdirSync(receiptRoot).filter(name=>name.endsWith('.json'));
  const records=files.map(name=>decode(readFileSync(join(receiptRoot,name))));
  const latest=JSON.parse(execFileSync('gh',['api','repos/'+process.env.NATIVE_REPOSITORY+'/releases/latest'],
   {encoding:'utf8',timeout:30000}));
  assert.equal(latest.draft,false);assert.equal(latest.prerelease,false);
  const isLatest=latest.tag_name==='v'+inputByArch.amd64.version;
  for(const currentVariant of VARIANTS) {
   const receipts=manifestSources(records,inputByArch,currentVariant);
   assert.ok(receipts.every(record=>record.repository===repository));
   const versionTag=inputByArch.amd64.version+(currentVariant==='slim'?'-slim':'');
   if(process.env.GITHUB_OUTPUT)appendFileSync(process.env.GITHUB_OUTPUT,currentVariant+'-size='+
    receipts.map(record=>record.architecture+':'+Math.ceil(record.size_bytes/1048576)+'MiB').join(',')+'\n');
   const references=receipts.map(record=>record.registry_reference);
   const invoke=args=>execFileSync('docker',args,{encoding:'utf8',timeout:300000,maxBuffer:8*1024*1024});
   const tags=publicationTags(repository,inputByArch.amd64.version,currentVariant,isLatest);
   invoke(['buildx','imagetools','create',...tags.flatMap(tag=>['--tag',tag]),...references]);
   const descriptor=JSON.parse(invoke(['buildx','imagetools','inspect',repository+':'+versionTag,'--format','{{json .Manifest}}']));
   digest(descriptor.digest);const immutable=repository+'@'+descriptor.digest;
   const index=JSON.parse(invoke(['buildx','imagetools','inspect',immutable,'--raw']));validatePublishedIndex(index,receipts);
   const anonymous=await verifyAnonymousImageManifest(immutable,fetch,manifest=>validatePublishedIndex(manifest,receipts));
   writeFileSync(join(receiptRoot,'published-'+currentVariant+'-index.json'),JSON.stringify({version:1,image:immutable,index,anonymous},null,2)+'\n');
  }
 } else assert.fail('unknown container receipt operation');
}
