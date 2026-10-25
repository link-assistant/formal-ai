import test from 'node:test';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {execFileSync,spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {fileDigest,workflowRunIdentity,sourceBinaryIdentity,requireImageBundle,sealDeliveryEvidence,verifyDeliveryEvidence} from '../../../scripts/lib/hive-delivery-bundle.mjs';
const environment={...process.env,GITHUB_ACTIONS:'true',GITHUB_REPOSITORY:'owner/repo',GITHUB_RUN_ID:'123',GITHUB_RUN_ATTEMPT:'2',GITHUB_SHA:'a'.repeat(40),GITHUB_WORKFLOW_REF:'owner/repo/.github/workflows/hive-original-delivery.yml@refs/heads/fix'};
function fixture() {
 const root=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'hive-handoff-')));
 const directory=path.join(root,'evidence');fs.mkdirSync(directory);
 const expected={source:'b'.repeat(40),sourceTree:'c'.repeat(40),binarySha256:'d'.repeat(64),workflowRun:workflowRunIdentity(environment)};
 const inputs={hiveCommit:'e'.repeat(40),taskRuntimeImage:'runtime:fixture',clients:{agent:'Scala',claude:'Kotlin',codex:'Rust'}};
 const identity={...expected,hiveCommit:inputs.hiveCommit,runtimeImage:inputs.taskRuntimeImage,sameRunBinary:true,actualTaskImageBinarySha256:expected.binarySha256,operationalEvidence:{formalAiSource:{commit:expected.source,tree:expected.sourceTree}},authorshipCredit:0};
 const options={directory,tool:'agent',expected,inputs};
 const write=()=>fs.writeFileSync(path.join(directory,'source-binary-identity.json'),JSON.stringify(identity));write();
 fs.writeFileSync(path.join(directory,'result.json'),JSON.stringify({passed:false}));
 fs.mkdirSync(path.join(directory,'private'));fs.writeFileSync(path.join(directory,'private','held-out.log'),'actual evidence bytes\n');
 fs.writeFileSync(path.join(directory,'private','.auth'),'caller-owned hidden material must stay excluded');
 return {root,directory,expected,inputs,identity,options,write,cleanup:()=>fs.rmSync(root,{recursive:true,force:true})};
}
test('same-run evidence custody preserves failed harnesses and the original hidden-file exclusion without granting success',()=>{
 const f=fixture();try {
 const sealed=sealDeliveryEvidence(f.options);assert.equal(sealed.authorshipCredit,0);
 assert.ok(sealed.members.some(row=>row.path==='private/held-out.log'));
 assert.ok(sealed.members.every(row=>!row.path.split('/').some(part=>part.startsWith('.'))));
 assert.deepEqual(verifyDeliveryEvidence(f.options),sealed);
 assert.equal(JSON.parse(fs.readFileSync(path.join(f.directory,'result.json'))).passed,false,'original oracle is still required');
 assert.throws(()=>sealDeliveryEvidence(f.options),/EEXIST/);
 }finally{f.cleanup();}
});
for(const mutation of ['changed','added','removed','symlink','source','tree','binary','run','attempt','client','runtime','hive'])test('handoff rejects '+mutation,()=>{
 const f=fixture();try {
 sealDeliveryEvidence(f.options);
 if(mutation==='changed')fs.appendFileSync(path.join(f.directory,'result.json'),' ');
 if(mutation==='added')fs.writeFileSync(path.join(f.directory,'extra.log'),'unexpected');
 if(mutation==='removed')fs.unlinkSync(path.join(f.directory,'result.json'));
 if(mutation==='symlink')fs.symlinkSync(path.join(f.directory,'result.json'),path.join(f.directory,'linked'));
 if(mutation==='source')f.expected.source='f'.repeat(40);
 if(mutation==='tree')f.expected.sourceTree='f'.repeat(40);
 if(mutation==='binary')f.expected.binarySha256='f'.repeat(64);
 if(mutation==='run')f.expected.workflowRun={...f.expected.workflowRun,run:'124'};
 if(mutation==='attempt')f.expected.workflowRun={...f.expected.workflowRun,attempt:'3'};
 if(mutation==='client')f.options.tool='codex';
 if(mutation==='runtime')f.inputs.taskRuntimeImage='runtime:stale';
 if(mutation==='hive')f.inputs.hiveCommit='f'.repeat(40);
 assert.throws(()=>verifyDeliveryEvidence(f.options));
 }finally{f.cleanup();}
});
test('image manifest verifies exact archive bytes and current binary/source/run before Docker load',()=>{
 const f=fixture();try {
 fs.writeFileSync(path.join(f.directory,'candidate.tar.gz'),'fixture compressed image');
 const manifest={schema:'formal-ai-hive-image-bundle/v1',identity:f.identity,imageId:'sha256:'+'9'.repeat(64),archiveSha256:fileDigest(path.join(f.directory,'candidate.tar.gz'))};
 assert.equal(requireImageBundle(manifest,f.options),f.identity);
 for(const [key,value] of [['schema','other'],['imageId','candidate:mutable'],['archiveSha256','0'.repeat(64)]])assert.throws(()=>requireImageBundle({...manifest,[key]:value},f.options));
 fs.appendFileSync(path.join(f.directory,'candidate.tar.gz'),'tampered');assert.throws(()=>requireImageBundle(manifest,f.options),/archive digest/);
 }finally{f.cleanup();}
});
test('hosted workflow identity refuses missing, different workflow and zero run selectors',()=>{
 for(const patch of [{GITHUB_ACTIONS:'false'},{GITHUB_RUN_ID:'0'},{GITHUB_RUN_ATTEMPT:''},{GITHUB_WORKFLOW_REF:'owner/repo/.github/workflows/other.yml@refs/heads/fix'},{GITHUB_REPOSITORY:'other/repo'},{GITHUB_SHA:'head'}])assert.throws(()=>workflowRunIdentity({...environment,...patch}));
});

test('actual load entrypoint rechecks image bytes and adapters with mocked Docker and never rebuilds or solves',()=>{
 const f=fixture();try {
 const git=(cwd,args)=>execFileSync('git',args,{cwd,encoding:'utf8'}).trim();
 for(const dir of [f.root,path.join(f.root,'.hive-mind')]){fs.mkdirSync(dir,{recursive:true});git(dir,['init','-q']);git(dir,['-c','user.name=Fixture','-c','user.email=fixture@example.com','commit','--allow-empty','-qm','source']);}
 fs.mkdirSync(path.join(f.root,'rust/target/release'),{recursive:true});fs.writeFileSync(path.join(f.root,'rust/target/release/formal-ai'),'fixture bytes, never executed');
 const expected=sourceBinaryIdentity({root:f.root,environment});
 const hive=path.join(f.root,'.hive-mind');
 const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
 const files={'src/held-out.mjs':'export const retained=true;','src/owner.mjs':'import "./held-out.mjs";','scripts/e2e-hello-world.mjs':'fixture unchanged task entrypoint','scripts/formal-ai-hive-delivery-evidence.mjs':'fixture transport helper'};
 const hashes={};
 for(const [name,bytes] of Object.entries(files)){fs.mkdirSync(path.dirname(path.join(hive,name)),{recursive:true});fs.writeFileSync(path.join(hive,name),bytes);hashes['/opt/hive-e2e/'+name]=sha(bytes);}
 hashes['/usr/local/bin/formal-ai']=expected.binarySha256;
 const pin=git(hive,['rev-parse','HEAD']);
 const operational={upstreamCommit:pin,formalAiSource:{commit:expected.source,tree:expected.sourceTree},helperPath:'src/held-out.mjs',helperSha256:hashes['/opt/hive-e2e/src/held-out.mjs'],sources:[{path:'src/owner.mjs',adaptedSha256:hashes['/opt/hive-e2e/src/owner.mjs']}],authorshipCredit:0};
 const transport={upstreamCommit:pin,adaptedSha256:hashes['/opt/hive-e2e/scripts/e2e-hello-world.mjs'],helperSha256:hashes['/opt/hive-e2e/scripts/formal-ai-hive-delivery-evidence.mjs']};
 for(const [name,value] of [['operational',operational],['transport',transport]])fs.writeFileSync(path.join(hive,'.formal-ai-'+name+'-adapter.json'),JSON.stringify(value));
 const inputs={...f.inputs,hiveCommit:pin};fs.mkdirSync(path.join(f.root,'data/benchmarks'),{recursive:true});fs.writeFileSync(path.join(f.root,'data/benchmarks/hive-original-delivery-inputs.json'),JSON.stringify(inputs));
 const identity={...f.identity,...expected,hiveCommit:pin,actualTaskImageBinarySha256:expected.binarySha256,
  clientPrerequisites:{clientVersions:'passed',originalHiveMcpPreflight:'passed',containerCleanup:'passed'},
  transportAdapter:transport,operationalEvidence:{...operational,actualImageHelperSha256:operational.helperSha256,adaptedOwnerImports:'passed'}};
 fs.writeFileSync(path.join(f.directory,'candidate.tar.gz'),'fixture image archive');
 const imageId='sha256:'+'9'.repeat(64),manifest={schema:'formal-ai-hive-image-bundle/v1',identity,imageId,archiveSha256:fileDigest(path.join(f.directory,'candidate.tar.gz'))};
 fs.writeFileSync(path.join(f.directory,'image-bundle.json'),JSON.stringify(manifest));
 const bin=path.join(f.root,'bin'),calls=path.join(f.root,'calls.jsonl');fs.mkdirSync(bin);
 fs.writeFileSync(path.join(bin,'docker'),'#!/usr/bin/env node\nconst fs=require("node:fs"),args=process.argv.slice(2);fs.appendFileSync(process.env.FIXTURE_CALLS,JSON.stringify(args)+"\\n");if(args[0]==="image")console.log(JSON.stringify([{Id:process.env.FIXTURE_IMAGE_ID}]));else if(args.includes("sha256sum"))console.log(JSON.parse(process.env.FIXTURE_HASHES)[args.at(-1)]+"  file");else if(args[0]==="rm"&&args[1]==="--force"&&args[2].startsWith("formal-ai-hive-client-probe-")){}else if(!["load","run"].includes(args[0]))process.exitCode=95;\n',{mode:0o700});
 const script=fileURLToPath(new URL('../../../scripts/build-hive-original-delivery-image.mjs',import.meta.url));
 const env={...environment,PATH:bin+path.delimiter+environment.PATH,HIVE_IMAGE_BUNDLE_DIR:f.directory,E2E_LOG_DIR:path.join(f.root,'loaded'),FIXTURE_CALLS:calls,FIXTURE_HASHES:JSON.stringify(hashes),FIXTURE_IMAGE_ID:imageId};
 const run=()=>spawnSync(process.execPath,[script,'load'],{cwd:f.root,env,encoding:'utf8',timeout:15000});
 const result=run();assert.equal(result.status,0,result.stderr);
 assert.deepEqual(JSON.parse(fs.readFileSync(path.join(env.E2E_LOG_DIR,'source-binary-identity.json'))),identity);
 const observed=fs.readFileSync(calls,'utf8').trim().split('\n').map(JSON.parse);assert.equal(observed[0][0],'load');
 assert.ok(!observed.some(args=>args.includes('build')||args.includes('pull')));
 const wrongCases=[['FIXTURE_IMAGE_ID','sha256:'+'8'.repeat(64)],['FIXTURE_HASHES',JSON.stringify({...hashes,'/usr/local/bin/formal-ai':'0'.repeat(64)})],['FIXTURE_HASHES',JSON.stringify({...hashes,'/opt/hive-e2e/src/owner.mjs':'0'.repeat(64)})]];
 for(const [key,value] of wrongCases){const old=env[key];env[key]=value;assert.notEqual(run().status,0);env[key]=old;}
 }finally{f.cleanup();}
});
