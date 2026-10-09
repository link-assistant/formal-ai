// Preserve producer observations as exact-byte release evidence; JSON validation never executes a binary.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {readFileSync,readdirSync,mkdirSync,writeFileSync,existsSync} from 'node:fs';
import {basename,join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {expectedIdentity,validateExecutableReceipt,NATIVE_TARGETS,RECEIPT_FILE} from './native-release-artifact.mjs';
import {SOURCE_FILE} from './native-release-source.mjs';

export const PROTOCOL_FILE='protocol-selection.json';
export const PROTOCOL_HELPERS=['native-release-artifact.mjs','native-release-evidence.mjs','native-release-source.mjs','native-release-trust.mjs'];
export const SIGNING_LABELS=['macos-arm64','macos-x64'];
export const sha256=bytes=>createHash('sha256').update(bytes).digest('hex');
const decode=bytes=>JSON.parse(Buffer.from(bytes).toString('utf8'));
const commit=value=>assert.match(value,/^[a-f0-9]{40}$/u);
const digest=value=>assert.match(value,/^[a-f0-9]{64}$/u);
const semver=value=>assert.match(value,/^[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?$/u);

export function validateSourceEvidence(bytes,bindings) {
 assert.equal(sha256(bytes),bindings.sourceSha256);
 const source=decode(bytes);assert.equal(source.version,1);
 commit(source.source_commit);commit(source.source_tree);commit(source.compiler_commit);
 digest(source.cargo_lock_sha256);digest(source.native_inputs_sha256);semver(source.package_version);
 assert.equal(source.source_commit,bindings.sourceCommit);assert.equal(source.source_tree,bindings.sourceTree);
 assert.equal(source.package_version,bindings.version);assert.equal(source.producer_run,String(bindings.run));
 assert.equal(source.compiler_release,bindings.compilerRelease);assert.equal(source.compiler_commit,bindings.compilerCommit);
 expectedIdentity(source,Object.keys(NATIVE_TARGETS)[0],bindings.sourceSha256,bindings.run);
 return source;
}

export function validateProtocolEvidence(bytes,bindings,helperBytes=null) {
 assert.equal(sha256(bytes),bindings.protocolSha256);
 const protocol=decode(bytes);assert.equal(protocol.version,1);commit(protocol.commit);commit(protocol.tree);
 assert.equal(protocol.commit,bindings.protocolCommit);
 if(bindings.protocolTree!==undefined)assert.equal(protocol.tree,bindings.protocolTree);
 assert.deepEqual(protocol.files.map(file=>file.name).sort(),PROTOCOL_HELPERS);
 for(const file of protocol.files) {
  assert.ok(Number.isSafeInteger(file.bytes)&&file.bytes>0);digest(file.sha256);
  if(helperBytes!==null) {
   const actual=helperBytes[file.name];assert.ok(actual,'missing independently captured protocol helper');
   assert.equal(actual.length,file.bytes);assert.equal(sha256(actual),file.sha256);
  }
 }
 return protocol;
}

/** Signing mode follows captured command observations, never secret presence. */
export function validateSigningEvidence(record,source,label,sourceSha256) {
 assert.equal(record.version,1);assert.equal(record.label,label);assert.ok(SIGNING_LABELS.includes(label));
 assert.equal(record.source_selection_sha256,sourceSha256);assert.equal(record.package_version,source.package_version);
 assert.equal(record.source_commit,source.source_commit);assert.equal(record.producer_run,source.producer_run);
 assert.equal(record.observations.verify.status,0);assert.equal(record.observations.display.status,0);
 const display=record.observations.display.stdout+'\n'+record.observations.display.stderr;
 const adhoc=/^Signature=adhoc$/mu.test(display),developer=/^Authority=Developer ID Application:/mu.test(display);
 assert.notEqual(adhoc,developer,'ambiguous or unsupported actual signing mode');
 const mode=adhoc?'adhoc':'signed';assert.equal(record.mode,mode);
 if(mode==='signed') {
  assert.match(display,/^TeamIdentifier=[A-Z0-9]+$/mu);
  assert.equal(record.observations.assess.status,0);assert.equal(record.observations.stapler.status,0);
 } else {
  assert.equal(record.observations.assess,null);assert.equal(record.observations.stapler,null);
 }
 assert.equal(record.package.name,'formal-ai-desktop-'+label+'-'+source.package_version+'.dmg');semver(record.package_version);
 assert.ok(record.package.bytes>0);digest(record.package.sha256);
 assert.ok(record.native_component.bytes>0);digest(record.native_component.sha256);
 return mode;
}

/** Execute real macOS checks and keep public command output; any failed check refuses a record. */
export function observeMacSigning(application,packageFile,{source,sourceSha256,label,runCommand=spawnSync}) {
 assert.equal(process.platform,'darwin');
 const observed=(command,args)=>{
  const result=runCommand(command,args,{encoding:'utf8',timeout:30000});
  if(result.error)throw result.error;assert.equal(result.signal,null);assert.equal(result.status,0,result.stderr);
  return {status:result.status,stdout:result.stdout,stderr:result.stderr};
 };
 const verify=observed('codesign',['--verify','--deep','--strict','--verbose=2',application]);
 const display=observed('codesign',['--display','--verbose=2',application]);
 const text=display.stdout+'\n'+display.stderr,mode=/^Signature=adhoc$/mu.test(text)?'adhoc':'signed';
 const assess=mode==='signed'?observed('spctl',['--assess','--type','execute','--verbose=4',application]):null;
 const stapler=mode==='signed'?observed('xcrun',['stapler','validate',application]):null;
 const packageBytes=readFileSync(packageFile),nativeBytes=readFileSync(join(application,'Contents/Resources/bin/formal-ai'));
 const record={version:1,label,source_selection_sha256:sourceSha256,source_commit:source.source_commit,
  package_version:source.package_version,producer_run:source.producer_run,mode,
  package:{name:basename(packageFile),bytes:packageBytes.length,sha256:sha256(packageBytes)},
  native_component:{bytes:nativeBytes.length,sha256:sha256(nativeBytes)},observations:{verify,display,assess,stapler}};
 validateSigningEvidence(record,source,label,sourceSha256);return record;
}

/** Bind every actual target record; missing/corrupt inputs remain explicit and cannot become defaults. */
export function collectReleaseEvidence({sourceBytes,protocolBytes,receipts=[],signing=[],bindings,helperBytes=null}) {
 const source=validateSourceEvidence(sourceBytes,bindings);validateProtocolEvidence(protocolBytes,bindings,helperBytes);
 const assets=[],failures=[],signingModes=[];
 const add=(name,bytes)=>assets.push({name,bytes:Buffer.from(bytes),sha256:sha256(bytes)});
 add('formal-ai-native-source-'+source.package_version+'.json',sourceBytes);
 add('formal-ai-native-protocol-'+source.package_version+'.json',protocolBytes);
 const targets=Object.keys(NATIVE_TARGETS).sort(),labels=SIGNING_LABELS;
 for(const target of targets) {
  const matches=receipts.filter(entry=>entry.target===target);
  try {
   assert.equal(matches.length,1,'missing or duplicate target receipt');
   const receipt=validateExecutableReceipt(decode(matches[0].bytes),expectedIdentity(source,target,bindings.sourceSha256,bindings.run));
   add('formal-ai-native-'+target+'-'+source.package_version+'.json',matches[0].bytes);
   assert.ok(receipt.executable.sha256);
  } catch(error) {failures.push({kind:'native',target,reason:error.message});}
 }
 for(const entry of receipts)if(!targets.includes(entry.target))failures.push({kind:'native',target:entry.target,reason:'unknown target'});
 for(const label of labels) {
  const matches=signing.filter(entry=>entry.label===label);
  try {
   assert.equal(matches.length,1,'missing or duplicate signing observation');
   const mode=validateSigningEvidence(decode(matches[0].bytes),source,label,bindings.sourceSha256);
   signingModes.push({label,mode});add('formal-ai-signing-'+label+'-'+source.package_version+'.json',matches[0].bytes);
  } catch(error) {failures.push({kind:'signing',target:label,reason:error.message});}
 }
 for(const entry of signing)if(!labels.includes(entry.label))failures.push({kind:'signing',target:entry.label,reason:'unknown signing label'});
 assets.sort((left,right)=>left.name.localeCompare(right.name,'en'));
 return {assets,failures,signingModes,complete:failures.length===0};
}

export function prepareReleaseEvidence(directory,result) {
 mkdirSync(directory,{recursive:true});
 for(const asset of result.assets)writeFileSync(join(directory,asset.name),asset.bytes);
 writeFileSync(join(directory,'SHA256SUMS-evidence.partial'),result.assets.map(a=>a.sha256+'  '+a.name+'\n').join(''));
 writeFileSync(join(directory,'signing-summary.txt'),result.signingModes.map(a=>a.label+': '+a.mode+'\n').join(''));
 writeFileSync(join(directory,'collection-result.json'),JSON.stringify({complete:result.complete,failures:result.failures},null,2)+'\n');
 return result;
}

/** Require real public asset digests, manifest coverage and workflow-authenticated observation bytes. */
export function verifyPublishedEvidence(directory,{version,sourceCommit,sourceTree,expectedAssets,assetMetadata,verifyAttestation}) {
 const sourceBytes=readFileSync(join(directory,'formal-ai-native-source-'+version+'.json'));
 const protocolBytes=readFileSync(join(directory,'formal-ai-native-protocol-'+version+'.json'));
 const source=decode(sourceBytes),protocol=decode(protocolBytes);
 const bindings={sourceSha256:sha256(sourceBytes),sourceCommit,sourceTree,version,run:source.producer_run,
  compilerRelease:source.compiler_release,compilerCommit:source.compiler_commit,
  protocolSha256:sha256(protocolBytes),protocolCommit:protocol.commit};
 const receipts=Object.keys(NATIVE_TARGETS).map(target=>({target,bytes:readFileSync(join(directory,'formal-ai-native-'+target+'-'+version+'.json'))}));
 const signing=SIGNING_LABELS.map(label=>({label,bytes:readFileSync(join(directory,'formal-ai-signing-'+label+'-'+version+'.json'))}));
 const result=collectReleaseEvidence({sourceBytes,protocolBytes,receipts,signing,bindings});
 assert.equal(result.complete,true,JSON.stringify(result.failures));
 const checksums=new Map();
 for(const line of readFileSync(join(directory,'SHA256SUMS.txt'),'utf8').trim().split(/\r?\n/u)) {
  const match=/^([a-f0-9]{64})  ([^/\\\r\n]+)$/u.exec(line);assert.ok(match,'invalid public checksum record');
  assert.ok(!checksums.has(match[2]),'duplicate public checksum record');checksums.set(match[2],match[1]);
 }
 const names=new Set(assetMetadata.assets.map(asset=>asset.name));
 assert.equal(names.size,assetMetadata.assets.length,'duplicate public release asset name');
 for(const name of expectedAssets.filter(name=>!['SHA256SUMS.txt','BUILD-PROVENANCE.txt'].includes(name))) {
  assert.ok(checksums.has(name),'missing public asset checksum: '+name);
  const asset=assetMetadata.assets.find(entry=>entry.name===name);assert.ok(asset,'missing public release asset: '+name);
  assert.equal(asset.digest,'sha256:'+checksums.get(name),'public API byte digest differs: '+name);
 }
 for(const entry of signing) {
  const record=decode(entry.bytes);
  assert.equal(checksums.get(record.package.name),record.package.sha256,'published DMG differs from signing observation: '+record.package.name);
 }
 for(const asset of result.assets) {
  assert.equal(checksums.get(asset.name),asset.sha256,'observation bytes differ from published checksum');
  const verified=verifyAttestation(join(directory,asset.name),protocol.commit);
  assert.equal(verified,true,'workflow attestation did not verify: '+asset.name);
 }
 return result;
}

function entries(directory,prefix,key,filename) {
 if(!existsSync(directory))return [];
 return readdirSync(directory).sort().filter(name=>name.startsWith(prefix)).map(name=>{
  try{return {[key]:name.slice(prefix.length),bytes:readFileSync(join(directory,name,filename))};}
  catch{return {[key]:name.slice(prefix.length),bytes:Buffer.from('{}')};}
 });
}

if(import.meta.url===pathToFileURL(process.argv[1]??'').href) {
 const [mode,sourceDirectory,receiptDirectory,signingDirectory,outputDirectory]=process.argv.slice(2);
 if(mode==='published') {
  const directory=sourceDirectory,version=receiptDirectory;
  const metadata=decode(readFileSync(join(directory,'release-assets.json')));
  const expectedAssets=readFileSync(join(directory,'expected-assets.txt'),'utf8').trim().split(/\r?\n/u);
  const verifyAttestation=(file,protocolCommit)=>{
   const result=spawnSync('gh',['attestation','verify',file,'--repo',process.env.REPO,
    '--signer-workflow',process.env.REPO+'/.github/workflows/desktop-release.yml','--signer-digest',protocolCommit],
    {encoding:'utf8',timeout:30000});
   if(result.error)throw result.error;assert.equal(result.signal,null);assert.equal(result.status,0,result.stderr);return true;
  };
  verifyPublishedEvidence(directory,{version,sourceCommit:signingDirectory,sourceTree:outputDirectory,
   expectedAssets,assetMetadata:metadata,verifyAttestation});
  console.log('Verified public release asset digests and complete durable observations.');
 } else {
 const sourceBytes=readFileSync(join(sourceDirectory,SOURCE_FILE)),source=decode(sourceBytes);
 if(mode==='signing') {
  assert.equal(sha256(sourceBytes),process.env.NATIVE_SELECTION_SHA256);
  assert.equal(source.producer_run,process.env.GITHUB_RUN_ID);
  const record=observeMacSigning(receiptDirectory,signingDirectory,{source,sourceSha256:sha256(sourceBytes),label:process.env.NATIVE_SIGNING_LABEL});
  writeFileSync(outputDirectory,JSON.stringify(record,null,2)+'\n');
 } else if(mode==='collect') {
  const protocolDirectory=process.env.FORMAL_AI_NATIVE_PROTOCOL_DIR;
  const bindings={sourceSha256:process.env.NATIVE_SELECTION_SHA256,sourceCommit:process.env.NATIVE_SOURCE_COMMIT,
   sourceTree:process.env.NATIVE_SOURCE_TREE,version:process.env.NATIVE_PACKAGE_VERSION,run:process.env.GITHUB_RUN_ID,
   compilerRelease:process.env.NATIVE_COMPILER_RELEASE,compilerCommit:process.env.NATIVE_COMPILER_COMMIT,
   protocolSha256:process.env.NATIVE_PROTOCOL_SHA256,protocolCommit:process.env.NATIVE_PROTOCOL_COMMIT};
  const helperBytes=Object.fromEntries(PROTOCOL_HELPERS.map(name=>[name,readFileSync(join(protocolDirectory,name))]));
  const result=collectReleaseEvidence({sourceBytes,protocolBytes:readFileSync(join(sourceDirectory,PROTOCOL_FILE)),
   receipts:entries(receiptDirectory,'native-receipt-','target',RECEIPT_FILE),
   signing:entries(signingDirectory,'signing-','label','signing-observation.json'),bindings,helperBytes});
  prepareReleaseEvidence(outputDirectory,result);
  console.log(JSON.stringify({complete:result.complete,failures:result.failures}));
 } else assert.fail('unknown release evidence operation');
 }
}
