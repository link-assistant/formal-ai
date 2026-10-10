#!/usr/bin/env node
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdirSync,existsSync,lstatSync} from 'node:fs';
import {execFileSync,spawnSync} from 'node:child_process';
import {resolve,join,relative} from 'node:path';
import {projectionRegistry,projectionInputs,verifyProjectionDirectory,digest,sessionSchema} from './lib/native-session-projections.mjs';
const cwd=process.cwd(), output=resolve(process.argv[2]??'');
const deadline=Date.now()+1200000;
assert.equal(process.env.GITHUB_ACTIONS,'true','native session producer runs only in hosted CI');
for(const name of ['GITHUB_RUN_ID','GITHUB_RUN_ATTEMPT']) assert.match(process.env[name]??'',/^[1-9]\d*$/u,'hosted producer identity missing');
assert.ok(process.env.GITHUB_JOB && process.env.GITHUB_REPOSITORY);
assert.ok(process.argv[2] && !existsSync(output),'fresh explicit output directory required');
assert.ok(!output.startsWith(cwd+'/') && output!==cwd,'generated data must stage outside source tree');
const git=args=>execFileSync('git',args,{cwd,encoding:'utf8'}).trim();
const commit=git(['rev-parse','HEAD']), tree=git(['rev-parse','HEAD^{tree}']);
assert.equal(commit,process.env.GITHUB_SHA,'hosted checkout differs from workflow source');
const {trackedTestInputIdentity}=await import('./lib/native-response-identity.mjs');
const {nativeInputIdentity}=await import('./native-release-source.mjs');
const nativeInputs=nativeInputIdentity(cwd);
const witness=trackedTestInputIdentity(cwd);
const producer='rust/examples/regenerate_agent_cli_sessions.rs';
const producerBytes=readFileSync(join(cwd,producer));
const driverBytes=readFileSync(join(cwd,'rust/src/agentic_coding/driver.rs'));
const schema=sessionSchema(driverBytes.toString('utf8'));
const paths=projectionRegistry(producerBytes.toString('utf8'));
const inputs=projectionInputs(cwd,paths);
mkdirSync(output);const generated=join(output,'generated');mkdirSync(generated);
function processReceipt(command,args,environment=process.env) {
  const budgetMs=deadline-Date.now();assert.ok(budgetMs>0,'shared native generation deadline exhausted');
  const result=spawnSync(command,args,{cwd,env:environment,encoding:null,maxBuffer:64*1024*1024,timeout:budgetMs,killSignal:'SIGTERM'});
  return {result,receipt:{command,args,budgetMs,exitCode:result.status,signal:result.signal,error:result.error?.message??null,
    stdoutSha256:digest(result.stdout??Buffer.alloc(0)),stderrSha256:digest(result.stderr??Buffer.alloc(0))}};
}
const build=processReceipt('cargo',['build','--manifest-path','rust/Cargo.toml','--locked','--release','--example','regenerate_agent_cli_sessions','--message-format=json']);
writeFileSync(join(output,'build.stdout.bin'),build.result.stdout??Buffer.alloc(0));
writeFileSync(join(output,'build.stderr.bin'),build.result.stderr??Buffer.alloc(0));
assert.equal(build.receipt.error,null);assert.equal(build.receipt.signal,null);assert.equal(build.receipt.exitCode,0,'actual native producer compilation failed');
const artifacts=build.result.stdout.toString('utf8').split(/\r?\n/u).filter(Boolean).map(line=>JSON.parse(line)).filter(row=>row.reason==='compiler-artifact' && row.target.name==='regenerate_agent_cli_sessions' && row.target.kind.includes('example') && row.executable);
assert.equal(artifacts.length,1,'compiled native producer executable must be unique');
const executable=resolve(artifacts[0].executable);assert.ok(lstatSync(executable).isFile());
const executableSha256=digest(readFileSync(executable));
const run=processReceipt(executable,[],{...process.env,FORMAL_AI_SESSION_PROJECTION_OUTPUT:generated});
writeFileSync(join(output,'producer.stdout.bin'),run.result.stdout??Buffer.alloc(0));
writeFileSync(join(output,'producer.stderr.bin'),run.result.stderr??Buffer.alloc(0));
assert.equal(run.receipt.error,null);assert.equal(run.receipt.signal,null);assert.equal(run.receipt.exitCode,0,'actual compiled session producer failed');
assert.deepEqual(nativeInputIdentity(cwd),nativeInputs,'native compilation inputs changed during generation');
assert.deepEqual(trackedTestInputIdentity(cwd),witness,'physical source changed during generation');
assert.equal(digest(readFileSync(executable)),executableSha256,'compiled executable changed during generation');
assert.deepEqual(projectionInputs(cwd,paths),inputs,'pinned task/tools input changed during generation');
const members=verifyProjectionDirectory(generated,inputs,schema);
const receipt={schema:'native-session-projection-producer/v1',authority:'not-granted',semanticPassCredit:false,
  identity:{commit,tree,nativeInputs,physicalInputs:witness,repository:process.env.GITHUB_REPOSITORY,run:process.env.GITHUB_RUN_ID,attempt:process.env.GITHUB_RUN_ATTEMPT,job:process.env.GITHUB_JOB,
    producer,producerSha256:digest(producerBytes),driverSourceSha256:digest(driverBytes),executable:relative(cwd,executable),executableSha256,
    compiler:execFileSync('rustc',['-vV'],{cwd,encoding:'utf8'}).trim(),profile:'release',features:'Cargo default',
    rustFlags:process.env.RUSTFLAGS??'',encodedRustFlags:process.env.CARGO_ENCODED_RUSTFLAGS??'',target:process.env.CARGO_BUILD_TARGET??'',
    cargoProfileOverrides:Object.fromEntries(Object.entries(process.env).filter(([key])=>/^CARGO_PROFILE_[A-Z0-9_]+$/u.test(key)).sort())},
  sessionSchemaSha256:digest(Buffer.from(JSON.stringify(schema))),registrySha256:digest(Buffer.from(paths.join('\n')+'\n')),inputs,members,build:build.receipt,producerExecution:run.receipt,
  originalAssertionsModifiedByProducer:false,expectedOutputFieldsUsedForGeneration:false,
  completeness:'Fresh compiled maintained driver returns; data projections require independent hosted provenance and unchanged native semantic checks.'};
writeFileSync(join(output,'producer-receipt.json'),JSON.stringify(receipt,null,2)+'\n');
console.log(JSON.stringify({generated:members.length,semanticPassCredit:false,receipt:join(output,'producer-receipt.json')}));
