// Publish current guarded-main prepared binaries without compiling them again inside Docker.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {join,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {validatePreparedSelection} from './prepared-release-binary.mjs';
import {expectedIdentity,verifyExecutableReceipt} from './native-release-artifact.mjs';
import {verifyPublishedNativeImage} from './verify-published-native-image.mjs';
import {verifyAnonymousImageManifest} from './verify-anonymous-image-manifest.mjs';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const repository=value=>{assert.match(value,/^[a-z0-9.-]+\/[a-z0-9_./-]+$/u);return value;};
export function releaseImageCacheSettings(environment=process.env) {
 const required={'cache-from':'type=gha,scope=docker-image','cache-to':'type=gha,mode=max,scope=docker-image'};
 if(environment.BUILD_CACHE_SETTINGS===undefined)return required;
 assert.equal(typeof environment.BUILD_CACHE_SETTINGS,'string','BuildKit cache settings must be text');
 const lines=environment.BUILD_CACHE_SETTINGS.trimEnd().split(/\r?\n/u);assert.equal(lines.length,2,'exactly two cache settings required');
 const settings={};for(const line of lines){const match=/^(cache-from|cache-to): (.+)$/u.exec(line);assert.ok(match,'unknown or malformed BuildKit cache setting');assert.ok(!Object.hasOwn(settings,match[1]),'duplicate cache setting');settings[match[1]]=match[2];}
 assert.deepEqual(settings,required,'cache kind, mode and scope must preserve the source publish contract');return settings;
}
function completed(result,label) {if(result.error)throw result.error;assert.equal(result.signal,null,label+' interrupted');assert.equal(result.status,0,label+' failed: '+String(result.stderr??''));return result.stdout;}
export function validatePreparedBinary({cwd,directory,version,environment=process.env,repositoryUrl=null}) {
 const record=JSON.parse(readFileSync(join(directory,'prepared-build.json'),'utf8'));
 assert.equal(record.schema,'prepared-release-build/v1');assert.equal(record.authority,'guarded-main-prepared-release');
 assert.deepEqual(record.selection,validatePreparedSelection(cwd,version,environment,{repositoryUrl}));
 assert.deepEqual(record.build,{command:'cargo',arguments:['build','--manifest-path','rust/Cargo.toml','--release'],status:0,signal:null,complete:true});
 const sourceBytes=readFileSync(join(directory,'source/source-selection.json')),source=JSON.parse(sourceBytes);
 assert.equal(sha(sourceBytes),record.sourceSelectionSha256);assert.equal(source.source_commit,record.selection.head);
 assert.equal(source.source_tree,record.selection.tree);assert.equal(source.release_tag,'v'+version);assert.equal(source.package_version,version);
 assert.equal(source.native_inputs_sha256,record.selection.nativeInputs.sha256);assert.equal(source.native_input_files,record.selection.nativeInputs.files);
 const identity=expectedIdentity(source,'x86_64-unknown-linux-gnu',record.sourceSelectionSha256,environment.GITHUB_RUN_ID);
 assert.deepEqual(record.identity,identity);
 const receiptBytes=readFileSync(join(directory,'binary/native-executable-receipt.json'));assert.equal(sha(receiptBytes),record.nativeReceiptSha256);
 const verified=verifyExecutableReceipt(join(directory,'binary'),identity);assert.deepEqual(verified.receipt.executable,record.executable);
 assert.ok(readFileSync(join(cwd,'rust/target/release/formal-ai')).equals(readFileSync(verified.binary)),'Docker context binary differs from actual source-bound producer');
 return {record,source,identity,verified};
}
function dockerRunner(run,cwd,environment) {return args=>completed(run('docker',args,{cwd,env:environment,encoding:'utf8',timeout:1500000,maxBuffer:16*1024*1024}),'docker '+args[0]);}
async function anonymousManifest(reference,configDigest,fetcher) {
 const verify=manifest=>{assert.equal(manifest.schemaVersion,2);assert.equal(manifest.config?.digest,configDigest,'registry manifest is not the observed image config');};
 if(reference.startsWith('ghcr.io/'))return verifyAnonymousImageManifest(reference,fetcher,verify);
 const match=/^(?:docker\.io\/)?([a-z0-9][a-z0-9._/-]*)@(sha256:[a-f0-9]{64})$/u.exec(reference);assert.ok(match,'exact immutable Docker Hub image required');
 const [,name,digest]=match;assert.equal(name.split('/').length,2);assert.ok(name.split('/').every(part=>part&&!['.','..'].includes(part)));
 const query=new URLSearchParams({service:'registry.docker.io',scope:'repository:'+name+':pull'});
 const tokenResponse=await fetcher('https://auth.docker.io/token?'+query,{redirect:'error',signal:AbortSignal.timeout(15000)});assert.equal(tokenResponse.status,200,'anonymous destination token denied');
 const token=(await tokenResponse.json()).token;assert.ok(typeof token==='string'&&token.length>0&&!/\s/u.test(token));
 const types=['application/vnd.oci.image.manifest.v1+json','application/vnd.docker.distribution.manifest.v2+json'];
 const response=await fetcher('https://registry-1.docker.io/v2/'+name+'/manifests/'+digest,{redirect:'error',signal:AbortSignal.timeout(15000),headers:{Authorization:'Bearer '+token,Accept:types.join(', ')}});
 assert.equal(response.status,200,'anonymous destination manifest denied');const bytes=Buffer.from(await response.arrayBuffer());assert.ok(bytes.length>0&&bytes.length<=16*1024*1024);
 assert.equal('sha256:'+sha(bytes),digest,'destination manifest bytes differ');const reported=response.headers.get('Docker-Content-Digest');if(reported!==null)assert.equal(reported,digest);
 const manifest=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(bytes));assert.ok(types.includes(manifest.mediaType));verify(manifest);
 for(const descriptor of [manifest.config,...manifest.layers]){assert.match(descriptor.digest,/^sha256:[a-f0-9]{64}$/u);assert.ok(Number.isSafeInteger(descriptor.size)&&descriptor.size>0);}
 return {version:1,image:reference,manifest_sha256:digest,media_type:manifest.mediaType,observation:'AnonymousImmutableManifestBytesVerified'};
}
async function observePublished(reference,expected,invoke,fetcher) {
 const inspections=JSON.parse(invoke(['image','inspect',reference]));assert.equal(inspections.length,1);
 const inspection=inspections[0];assert.equal(inspection.Os,'linux');assert.equal(inspection.Architecture,'amd64');
 assert.ok(inspection.Config?.Env?.includes('FORMAL_AI_IMAGE_VARIANT=dind'),'actual full image variant differs');
 const versionOutput=invoke(['run','--rm','--entrypoint','/usr/local/bin/formal-ai',reference,'--version']);
 const binaryDigestOutput=invoke(['run','--rm','--entrypoint','sha256sum',reference,'/usr/local/bin/formal-ai']);
 const receipt=verifyPublishedNativeImage({...expected,image:reference},{inspection,versionOutput,binaryDigestOutput});
 const runtime=invoke(['run','--rm','--privileged',reference,'verify-formal-ai-dind']);assert.ok(runtime.trim(),'empty actual full runtime observation');
 const anonymous=await anonymousManifest(reference,inspection.Id,fetcher);
 return {receipt,runtime:{status:0,signal:null,complete:true,stdout:runtime},anonymous,configDigest:inspection.Id};
}
function immutableReference(image,invoke) {
 const inspections=JSON.parse(invoke(['image','inspect',image]));assert.equal(inspections.length,1);
 const expected=image.slice(0,image.lastIndexOf(':')).replace(/^docker\.io\//u,'');
 const references=[...new Set((inspections[0].RepoDigests??[]).filter(value=>value.replace(/^docker\.io\//u,'').startsWith(expected+'@sha256:')))];
 assert.equal(references.length,1,'one actual immutable repository digest required');assert.match(references[0],/@sha256:[a-f0-9]{64}$/u);return references[0];
}
export async function publishPreparedFullImage({cwd,directory,version,image,environment=process.env,run=spawnSync,fetcher=fetch,repositoryUrl=null}) {
 repository(image);assert.equal(image,'ghcr.io/'+environment.GITHUB_REPOSITORY.toLowerCase(),'full release factory requires the actual source repository');
 const {record}=validatePreparedBinary({cwd,directory,version,environment,repositoryUrl});
 assert.ok(typeof environment.ACTIONS_RUNTIME_TOKEN==='string'&&environment.ACTIONS_RUNTIME_TOKEN.length>0,'GHA cache runtime token missing');
 assert.ok(typeof environment.ACTIONS_RESULTS_URL==='string'&&/^https:\/\/[^\s,]+$/u.test(environment.ACTIONS_RESULTS_URL),'GHA cache endpoint missing or invalid');
 const cache=releaseImageCacheSettings(environment);
 const invoke=dockerRunner(run,cwd,environment),local='formal-ai:prepared-full-'+environment.GITHUB_RUN_ID;
 invoke(['buildx','build','--load','--file','Dockerfile','--build-arg','BINARY_SOURCE=prebuilt','--platform','linux/amd64','--tag',local,
  '--label','org.opencontainers.image.revision='+record.selection.head,'--label','org.opencontainers.image.version='+version,
  '--label','io.link-assistant.formal-ai.executable.sha256='+record.executable.sha256,
  '--cache-from',cache['cache-from'],'--cache-to',cache['cache-to'],'.']);
 for(const suffix of [version,'latest']){invoke(['tag',local,image+':'+suffix]);invoke(['push',image+':'+suffix]);}
 invoke(['pull',image+':'+version]);const reference=immutableReference(image+':'+version,invoke);
 const expected={revision:record.selection.head,version,binarySha256:record.executable.sha256};
 const observation=await observePublished(reference,expected,invoke,fetcher);
 validatePreparedBinary({cwd,directory,version,environment,repositoryUrl});
 const result={schema:'prepared-release-image/v1',authority:record.authority,preparedBuildSha256:sha(readFileSync(join(directory,'prepared-build.json'))),sourceSelectionSha256:record.sourceSelectionSha256,expected,image:reference,...observation};
 writeFileSync(join(directory,'full-image.json'),JSON.stringify(result,null,2)+'\n');return result;
}
export async function mirrorPreparedFullImage({cwd,directory,version,image,environment=process.env,run=spawnSync,fetcher=fetch,repositoryUrl=null}) {
 repository(image);assert.match(image,/^(?:docker\.io\/)?[a-z0-9][a-z0-9._-]*\/[a-z0-9][a-z0-9._-]*$/u,'mirror destination must be the configured Docker Hub repository');
 assert.equal(image,environment.DOCKERHUB_IMAGE,'mirror destination differs from configured opt-in target');
 const {record}=validatePreparedBinary({cwd,directory,version,environment,repositoryUrl});
 const source=JSON.parse(readFileSync(join(directory,'full-image.json'),'utf8'));
 assert.equal(source.schema,'prepared-release-image/v1');assert.equal(source.authority,record.authority);assert.equal(source.sourceSelectionSha256,record.sourceSelectionSha256);
 assert.ok(source.image.startsWith('ghcr.io/'+environment.GITHUB_REPOSITORY.toLowerCase()+'@'),'mirror source repository differs');
 assert.equal(source.preparedBuildSha256,sha(readFileSync(join(directory,'prepared-build.json'))));
 const expected={revision:record.selection.head,version,binarySha256:record.executable.sha256};assert.deepEqual(source.expected,expected);
 assert.match(source.image,/^ghcr\.io\/[^\s]+@sha256:[a-f0-9]{64}$/u);
 const invoke=dockerRunner(run,cwd,environment);invoke(['pull',source.image]);
 const original=await observePublished(source.image,expected,invoke,fetcher);
 for(const suffix of [version,'latest']){invoke(['tag',source.image,image+':'+suffix]);invoke(['push',image+':'+suffix]);}
 invoke(['pull',image+':'+version]);const reference=immutableReference(image+':'+version,invoke);
 const destination=await observePublished(reference,expected,invoke,fetcher);assert.equal(destination.configDigest,original.configDigest,'mirror changed actual image config');
 validatePreparedBinary({cwd,directory,version,environment,repositoryUrl});
 const result={schema:'prepared-release-image-mirror/v1',sourceImage:source.image,destinationImage:reference,source:original,destination,expected};
 writeFileSync(join(directory,'dockerhub-image.json'),JSON.stringify(result,null,2)+'\n');return result;
}
if(import.meta.url===pathToFileURL(process.argv[1]??'').href) {
 const [mode,directory,version,image]=process.argv.slice(2);assert.ok(directory&&version&&image);
 const options={cwd:process.cwd(),directory:resolve(directory),version,image};
 const record=mode==='publish'?await publishPreparedFullImage(options):mode==='mirror'?await mirrorPreparedFullImage(options):assert.fail('unknown publication mode');
 if(mode==='publish'&&process.env.GITHUB_OUTPUT)writeFileSync(process.env.GITHUB_OUTPUT,'digest='+record.image.split('@')[1]+'\n',{flag:'a'});
 console.log(JSON.stringify(record));
}
