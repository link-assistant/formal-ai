// Standalone default-feature executable receipts; no test-harness contract is weakened.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync,spawnSync} from 'node:child_process';
import {chmodSync,readFileSync,readdirSync,statSync,writeFileSync} from 'node:fs';
import {basename,join,resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {compilerContract,nativeInputIdentity,SOURCE_FILE} from './native-release-source.mjs';

export const RECEIPT_FILE='native-executable-receipt.json';
export const NATIVE_TARGETS=Object.freeze({
 'x86_64-unknown-linux-gnu':{platform:'linux',architecture:'x64',host:'x86_64-unknown-linux-gnu'},
 'aarch64-unknown-linux-gnu':{platform:'linux',architecture:'arm64',host:'aarch64-unknown-linux-gnu'},
 'x86_64-unknown-linux-musl':{platform:'linux',architecture:'x64',host:'x86_64-unknown-linux-gnu'},
 'aarch64-unknown-linux-musl':{platform:'linux',architecture:'arm64',host:'aarch64-unknown-linux-gnu'},
 'x86_64-apple-darwin':{platform:'darwin',architecture:'x64',host:'x86_64-apple-darwin'},
 'aarch64-apple-darwin':{platform:'darwin',architecture:'arm64',host:'aarch64-apple-darwin'},
 'x86_64-pc-windows-msvc':{platform:'win32',architecture:'x64',host:'x86_64-pc-windows-msvc'},
 'aarch64-pc-windows-msvc':{platform:'win32',architecture:'arm64',host:'aarch64-pc-windows-msvc'},
});
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');

export function expectedIdentity(source,target,selectionSha,run) {
 assert.ok(Object.hasOwn(NATIVE_TARGETS,target),'unknown native target');
 assert.equal(source.version,1);assert.equal(source.producer_run,String(run));
 assert.equal(source.profile,'release');assert.equal(source.requested_features,'default');
 assert.equal(source.rust_flags,'-Dwarnings');assert.match(selectionSha,/^[a-f0-9]{64}$/u);
 return {source_commit:source.source_commit,source_tree:source.source_tree,
  source_selection_sha256:selectionSha,cargo_lock_sha256:source.cargo_lock_sha256,
  native_inputs_sha256:source.native_inputs_sha256,target,profile:source.profile,
  requested_features:source.requested_features,root_default_features:source.root_default_features,
  rust_flags:source.rust_flags,compiler_release:source.compiler_release,compiler_commit:source.compiler_commit,
  producer_run:String(run),...NATIVE_TARGETS[target]};
}

export function writeExecutableReceipt(directory,identity,binary,{compiler,environment={},job,attempt}) {
 const actual=compilerContract(compiler);assert.equal(actual.release,identity.compiler_release);
 assert.equal(actual.commit,identity.compiler_commit);
 assert.equal(/^host: (\S+)$/mu.exec(compiler)?.[1],identity.host);
 assert.equal(process.platform,identity.platform);assert.equal(process.arch,identity.architecture);
 assert.equal(environment.RUSTFLAGS,identity.rust_flags);
 assert.ok(!environment.CARGO_ENCODED_RUSTFLAGS,'encoded flags must not override requested flags');
 assert.equal(Object.keys(environment).filter(k=>k.startsWith('CARGO_PROFILE_')).length,0);
 const name=identity.platform==='win32'?'formal-ai.exe':'formal-ai',bytes=readFileSync(binary);assert.ok(bytes.length>0);
 const receipt={version:1,identity,producer:{job:String(job),attempt:String(attempt),compiler,environment},
  executable:{name,bytes:bytes.length,sha256:hash(bytes)}};
 writeFileSync(join(directory,RECEIPT_FILE),JSON.stringify(receipt,null,2)+'\n');return receipt;
}

/** Validate the original producer contract independently of a filesystem or execution claim. */
export function validateExecutableReceipt(receipt,identity) {
 assert.equal(receipt.version,1);assert.deepEqual(receipt.identity,identity);
 const name=identity.platform==='win32'?'formal-ai.exe':'formal-ai';
 assert.equal(receipt.executable.name,name);
 assert.ok(Number.isSafeInteger(receipt.executable.bytes)&&receipt.executable.bytes>0);
 assert.match(receipt.executable.sha256,/^[a-f0-9]{64}$/u);
 assert.match(receipt.producer.job,/^[A-Za-z_][A-Za-z0-9_-]*$/u);assert.match(receipt.producer.attempt,/^[1-9][0-9]*$/u);
 const compiler=compilerContract(receipt.producer.compiler);
 assert.equal(compiler.release,identity.compiler_release);assert.equal(compiler.commit,identity.compiler_commit);
 assert.equal(/^host: (\S+)$/mu.exec(receipt.producer.compiler)?.[1],identity.host);
 assert.equal(receipt.producer.environment.RUSTFLAGS,identity.rust_flags);
 assert.ok(!receipt.producer.environment.CARGO_ENCODED_RUSTFLAGS);
 assert.equal(Object.keys(receipt.producer.environment).filter(k=>k.startsWith('CARGO_PROFILE_')).length,0);
 return receipt;
}

/** Recompute executable and independent source contract before execution or packaging. */
export function verifyExecutableReceipt(directory,identity) {
 const receipt=validateExecutableReceipt(JSON.parse(readFileSync(join(directory,RECEIPT_FILE),'utf8')),identity);
 const name=receipt.executable.name;
 assert.deepEqual(readdirSync(directory).sort(),[name,RECEIPT_FILE].sort());
 const binary=join(directory,name),bytes=readFileSync(binary);
 assert.ok(bytes.length>0);assert.equal(bytes.length,receipt.executable.bytes);assert.equal(hash(bytes),receipt.executable.sha256);
 chmodSync(binary,0o755);return {receipt,binary:resolve(binary)};
}


/** The archive must contain the exact verified native executable bytes. */
export function verifyArchivedExecutable(binary,receipt) {
 const bytes=readFileSync(binary);
 assert.equal(basename(binary),receipt.executable.name);
 assert.equal(bytes.length,receipt.executable.bytes);
 assert.equal(hash(bytes),receipt.executable.sha256);
 chmodSync(binary,0o755);
 return resolve(binary);
}

export function smokeExecutable(binary,version,cwd) {
 const output=execFileSync(binary,['--version'],{cwd,encoding:'utf8',timeout:30000}).trim();
 assert.equal(output,'formal-ai '+version);
 const result=spawnSync(binary,['--silent','chat','--prompt','2 + 2','--format','chat'],{
  cwd,encoding:'utf8',timeout:30000,env:{...process.env,FORMAL_AI_DEFINITION_FUSION:'offline',FORMAL_AI_NETWORK_MODE:'offline'},
 });
 if(result.error)throw result.error;assert.equal(result.signal,null);assert.equal(result.status,0,result.stderr);
 const answer=JSON.parse(result.stdout);assert.match(String(answer.choices?.[0]?.message?.content??''),/(?:^|=\s*)4(?:\s|[.]|$)/u,'actual arithmetic smoke missing result');
 return {version_output:output,arithmetic_stdout:result.stdout};
}

if(import.meta.url===pathToFileURL(process.argv[1]??'').href) {
 const [mode,directory,sourceDirectory,target,binary]=process.argv.slice(2);
 const selectionBytes=readFileSync(join(sourceDirectory,SOURCE_FILE)),source=JSON.parse(selectionBytes);
 assert.equal(hash(selectionBytes),process.env.NATIVE_SELECTION_SHA256);
 const identity=expectedIdentity(source,target,hash(selectionBytes),process.env.GITHUB_RUN_ID);
 assert.equal(execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim(),identity.source_commit);
 assert.equal(execFileSync('git',['rev-parse','HEAD^{tree}'],{encoding:'utf8'}).trim(),identity.source_tree);
 assert.equal(nativeInputIdentity(process.cwd()).sha256,identity.native_inputs_sha256);
 if(mode==='write') {
  const compiler=execFileSync('rustc',['-vV'],{encoding:'utf8'});
  const environment=Object.fromEntries(Object.entries(process.env).filter(([k])=>
   k==='RUSTFLAGS'||k==='RUSTC_WRAPPER'||k==='CARGO_ENCODED_RUSTFLAGS'||k==='CARGO_INCREMENTAL'
    ||k.startsWith('CARGO_PROFILE_')||k.startsWith('CARGO_TARGET_')||k.startsWith('CC_')||k.startsWith('CXX_')));
  writeExecutableReceipt(directory,identity,binary,{compiler,environment,job:process.env.GITHUB_JOB,attempt:process.env.GITHUB_RUN_ATTEMPT});
 } else if(mode==='verify'||mode==='verify-extracted') {
  const verified=verifyExecutableReceipt(directory,identity);
  const selected=mode==='verify-extracted'?verifyArchivedExecutable(binary,verified.receipt):verified.binary;
  console.log(JSON.stringify(smokeExecutable(selected,source.package_version,process.cwd())));
  if(mode==='verify'&&process.env.GITHUB_ENV)writeFileSync(process.env.GITHUB_ENV,'FORMAL_AI_VERIFIED_NATIVE_BINARY='+verified.binary.replaceAll('\\','/')+'\n',{flag:'a'});
 } else assert.fail('unknown native executable mode');
}
