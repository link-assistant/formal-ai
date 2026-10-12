// Record the existing release build against independently selected tagged-main inputs.
import assert from 'node:assert/strict';
import {spawnSync,execFileSync} from 'node:child_process';
import {readFileSync,writeFileSync,mkdirSync,copyFileSync} from 'node:fs';
import {join,resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import {nativeInputIdentity,packageContract,compilerContract,sealSource} from './native-release-source.mjs';
import {expectedIdentity,writeExecutableReceipt,verifyExecutableReceipt} from './native-release-artifact.mjs';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const git=(cwd,args)=>execFileSync('git',args,{cwd,encoding:'utf8',timeout:30000}).trim();
const BUILD_ARGS=['build','--manifest-path','rust/Cargo.toml','--release'];
export function validatePreparedSelection(cwd,version,environment,{repositoryUrl=null}={}) {
 assert.match(version,/^(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)$/u);
 assert.equal(environment.GITHUB_REF,'refs/heads/main');
 assert.ok(['push','workflow_dispatch'].includes(environment.GITHUB_EVENT_NAME),'only guarded main release events');
 if(environment.GITHUB_EVENT_NAME==='workflow_dispatch')assert.equal(environment.RELEASE_MODE,'instant');
 assert.match(environment.GITHUB_REPOSITORY,/^[\w.-]+\/[\w.-]+$/u);
 const remote=git(cwd,['remote','get-url','origin']);
 const expected=repositoryUrl??'https://github.com/'+environment.GITHUB_REPOSITORY;
 assert.equal(remote.replace(/\.git$/u,''),expected.replace(/\.git$/u,''),'independent source repository differs');
 const tag='refs/tags/v'+version,head=git(cwd,['rev-parse','HEAD']);
 assert.equal(git(cwd,['rev-parse',tag+'^{commit}']),head,'selected release tag differs from actual source');
 const refs=git(cwd,['ls-remote','origin','refs/heads/main',tag,tag+'^{}']).split('\n').map(line=>line.split(/\s+/u));
 const selected=name=>{const values=refs.filter(([,ref])=>ref===name).map(([value])=>value);assert.equal(values.length,1,'remote source ref missing or ambiguous: '+name);assert.match(values[0],/^[a-f0-9]{40}$/u);return values[0];};
 const main=selected('refs/heads/main'),peeled=refs.some(([,ref])=>ref===tag+'^{}')?selected(tag+'^{}'):selected(tag);
 assert.equal(peeled,head,'independent remote release tag differs');
 assert.equal(main,head,'prepared release must be the independently current main source');
 execFileSync('git',['fetch','--no-tags','origin',main],{cwd,stdio:'pipe',timeout:30000});
 execFileSync('git',['merge-base','--is-ancestor',head,main],{cwd,stdio:'pipe'});
 const inputs=nativeInputIdentity(cwd);
 return {head,tree:git(cwd,['rev-parse','HEAD^{tree}']),main,tag:'v'+version,version,nativeInputs:inputs};
}
function completed(result,label) {
 if(result.error)throw result.error;assert.equal(result.signal,null,label+' interrupted');
 assert.equal(result.status,0,label+' failed: '+String(result.stderr??''));return result;
}
export function prepareReleaseBinary({cwd,version,directory,environment=process.env,run=spawnSync,repositoryUrl=null}) {
 assert.equal(environment.RUSTFLAGS,'-Dwarnings');assert.ok(!environment.CARGO_ENCODED_RUSTFLAGS);
 for(const key of ['CARGO_BUILD_TARGET','CARGO_TARGET_DIR','RUSTC','RUSTC_WORKSPACE_WRAPPER'])assert.ok(!environment[key],key+' would alter the original build contract');
 assert.equal(Object.keys(environment).filter(key=>/^CARGO_TARGET_.*_RUSTFLAGS$/u.test(key)).length,0);
 assert.equal(Object.keys(environment).filter(key=>key.startsWith('CARGO_PROFILE_')).length,0);
 const before=validatePreparedSelection(cwd,version,environment,{repositoryUrl});
 const invoke=(command,args,stdio='pipe')=>completed(run(command,args,{cwd,env:environment,encoding:'utf8',stdio,timeout:1500000,maxBuffer:32*1024*1024}),command);
 const metadata=JSON.parse(invoke('cargo',['metadata','--manifest-path','rust/Cargo.toml','--no-deps','--locked','--format-version','1']).stdout);
 assert.equal(packageContract(metadata).version,version,'selected Cargo version differs');
 const compiler=invoke('rustc',['-vV']).stdout;compilerContract(compiler);
 const target=/^host: (\S+)$/mu.exec(compiler)?.[1];assert.ok(target,'actual compiler host missing');
 const build=invoke('cargo',BUILD_ARGS,'inherit');
 const after=validatePreparedSelection(cwd,version,environment,{repositoryUrl});assert.deepEqual(after,before,'selected source changed during build');
 assert.equal(invoke('rustc',['-vV']).stdout,compiler,'actual compiler changed during build');
 const binary=join(cwd,'rust/target/release',process.platform==='win32'?'formal-ai.exe':'formal-ai');
 assert.equal(invoke(binary,['--version']).stdout.trim(),'formal-ai '+version,'built executable version differs');
 mkdirSync(join(directory,'source'),{recursive:true});mkdirSync(join(directory,'binary'),{recursive:true});
 const source=sealSource(cwd,join(directory,'source'),{head:before.head,base:before.head,tag:before.tag,run:environment.GITHUB_RUN_ID,metadata,compiler});
 const identity=expectedIdentity(source.record,target,source.sha256,environment.GITHUB_RUN_ID);
 const recordedEnvironment=Object.fromEntries(Object.entries(environment).filter(([key])=>key==='RUSTFLAGS'||key==='RUSTC_WRAPPER'||key==='CARGO_ENCODED_RUSTFLAGS'||key==='CARGO_INCREMENTAL'||key.startsWith('CARGO_PROFILE_')||key.startsWith('CARGO_TARGET_')||key.startsWith('CC_')||key.startsWith('CXX_')));
 const selected=join(directory,'binary',process.platform==='win32'?'formal-ai.exe':'formal-ai');copyFileSync(binary,selected);
 const receipt=writeExecutableReceipt(join(directory,'binary'),identity,selected,{compiler,environment:recordedEnvironment,job:environment.GITHUB_JOB,attempt:environment.GITHUB_RUN_ATTEMPT});
 verifyExecutableReceipt(join(directory,'binary'),identity);
 const record={schema:'prepared-release-build/v1',authority:'guarded-main-prepared-release',selection:before,sourceSelectionSha256:source.sha256,
  nativeReceiptSha256:sha(readFileSync(join(directory,'binary/native-executable-receipt.json'))),build:{command:'cargo',arguments:BUILD_ARGS,status:build.status,signal:build.signal,complete:true},identity,executable:receipt.executable};
 writeFileSync(join(directory,'prepared-build.json'),JSON.stringify(record,null,2)+'\n');return record;
}
if(import.meta.url===pathToFileURL(process.argv[1]??'').href) {
 const [version,directory]=process.argv.slice(2);assert.ok(version&&directory);
 console.log(JSON.stringify(prepareReleaseBinary({cwd:process.cwd(),version,directory:resolve(directory)})));
}
