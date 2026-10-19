// Resolve native provenance dependencies on hosted CI; never build or install locally.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
export const nativeProvenanceDependencies=Object.freeze({
 blake3:'{ version = "1.8.7", features = ["rayon"] }',
 ring:'"0.17"',
 tracing:'"0.1.44"',
 wgpu:'{ version = "30.0.1", optional = true }',
 pollster:'{ version = "0.4.0", optional = true }'
});
export function nativeProvenanceManifest(source){
 const section='[target.\'cfg(not(target_arch = "wasm32"))\'.dependencies]\n';
 const features='[features]\n';
 assert.equal(source.split(section).length,2,'unique native dependency section required');
 assert.equal(source.split(features).length,2,'unique feature section required');
 const start=source.indexOf(section)+section.length,end=source.indexOf('\n[',start),nativeSection=source.slice(start,end<0?undefined:end);
 const missing=[];
 for(const [name,value]of Object.entries(nativeProvenanceDependencies)){
  const pattern=new RegExp('^'+name+'\\s*=.*$', 'gmu'),existing=[...source.matchAll(pattern)];
  if(existing.length){assert.equal(existing.length,1);assert.equal(existing[0][0],name+' = '+value,'existing dependency differs');assert.ok(nativeSection.includes(existing[0][0]),'dependency belongs to another target');}
  else missing.push(name+' = '+value+'\n');
 }
 const feature='provenance-gpu = ["dep:wgpu", "dep:pollster"]';
 const current=[...source.matchAll(/^provenance-gpu\s*=.*$/gmu)];
 if(current.length){assert.equal(current.length,1);assert.equal(current[0][0],feature,'existing feature differs');}
 return source.replace(section,section+missing.join('')).replace(features,features+(current.length?'':feature+'\n'));
}
export function resolveNativeProvenanceDependencies(root,output){
 assert.equal(process.env.GITHUB_ACTIONS,'true','native dependency resolution is hosted-CI-only');
 assert.ok(process.env.GITHUB_RUN_ID&&process.env.GITHUB_RUN_ATTEMPT&&process.env.GITHUB_SHA,'hosted producer identity required');
 const runner=fs.realpathSync(process.env.RUNNER_TEMP),destination=path.resolve(output),repository=fs.realpathSync(root);
 assert.ok(destination.startsWith(runner+path.sep)&&!destination.startsWith(repository+path.sep),'explicit external runner destination required');
 fs.mkdirSync(destination,{recursive:false,mode:0o700});
 const manifest=path.join(repository,'rust/Cargo.toml'),lock=path.join(repository,'rust/Cargo.lock');
 const original=fs.readFileSync(manifest),originalLock=fs.readFileSync(lock),candidate=nativeProvenanceManifest(original.toString('utf8'));
 fs.writeFileSync(manifest,candidate);
 for(const args of [['update','--workspace','--manifest-path',manifest],['metadata','--locked','--no-deps','--format-version','1','--manifest-path',manifest]]){
  const result=spawnSync('cargo',args,{cwd:repository,encoding:'utf8',timeout:300000,maxBuffer:4*1024*1024});
  if(result.error||result.status!==0)throw Error('hosted_native_dependency_resolution_failed: '+String(result.stderr).slice(-12000));
 }
 const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
 fs.copyFileSync(manifest,path.join(destination,'Cargo.toml'));fs.copyFileSync(lock,path.join(destination,'Cargo.lock'));
 const checkout=spawnSync('git',['rev-parse','HEAD','HEAD^{tree}'],{cwd:repository,encoding:'utf8',timeout:15000,maxBuffer:65536});
 assert.equal(checkout.status,0);const [checkoutCommit,checkoutTree]=checkout.stdout.trim().split('\n');
 assert.match(checkoutCommit,/^[a-f0-9]{40}$/u);assert.match(checkoutTree,/^[a-f0-9]{40}$/u);
 const receipt={schema:'formal-ai-hosted-native-dependencies-v1',repository:process.env.GITHUB_REPOSITORY,checkoutCommit,checkoutTree,
  sourceCommit:process.env.GITHUB_SHA,runId:process.env.GITHUB_RUN_ID,runAttempt:process.env.GITHUB_RUN_ATTEMPT,
  originalManifestSHA256:sha(original),originalLockSHA256:sha(originalLock),
  manifestSHA256:sha(Buffer.from(candidate)),lockSHA256:sha(fs.readFileSync(lock)),
  dependencies:nativeProvenanceDependencies,nativeCompilation:false,publication:false};
 fs.writeFileSync(path.join(destination,'producer-receipt.json'),JSON.stringify(receipt,null,2)+'\n');
 return receipt;
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 const root=path.resolve(fileURLToPath(new URL('..',import.meta.url)));
 console.log(JSON.stringify(resolveNativeProvenanceDependencies(root,process.argv[2])));
}
