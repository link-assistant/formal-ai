// Read-only release controls. Canned gh/native evidence is fixture data, never CI/publication proof.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdirSync,mkdtempSync,renameSync,rmSync} from 'node:fs';
import {join,resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
const root=process.env.RELEASE_CONTROL_REPO_ROOT??resolve(import.meta.dirname,'../../..');
const script=readFileSync(join(root,'scripts/desktop-release-resolve.sh'),'utf8');
const originalAssets=[
 'formal-ai-desktop-macos-arm64-0.201.0.dmg','formal-ai-desktop-macos-arm64-0.201.0.zip',
 'formal-ai-desktop-macos-x64-0.201.0.dmg','formal-ai-desktop-macos-x64-0.201.0.zip',
 'formal-ai-desktop-windows-installer-x64-0.201.0.exe','formal-ai-desktop-windows-installer-arm64-0.201.0.exe',
 'formal-ai-desktop-windows-portable-x64-0.201.0.exe','formal-ai-desktop-windows-portable-arm64-0.201.0.exe',
 'formal-ai-desktop-linux-x64-0.201.0.AppImage','formal-ai-desktop-linux-arm64-0.201.0.AppImage',
 'formal-ai-desktop-linux-x64-0.201.0.deb','formal-ai-desktop-linux-arm64-0.201.0.deb',
 'formal-ai-desktop-linux-x64-0.201.0.tar.gz','formal-ai-desktop-linux-arm64-0.201.0.tar.gz',
 'latest.yml','latest-mac.yml','latest-linux.yml'
];
function functionBlock(name){
 const start=script.indexOf(name+'() {');assert.ok(start>=0);
 const end=script.indexOf('\n}\n',start);assert.ok(end>start);
 return script.slice(start,end+3);
}
function expectedAssets(){
 const source=['expected_desktop_assets','expected_cli_assets','expected_native_targets'].map(functionBlock).join('\n');
 const observed=spawnSync('/bin/bash',['-c',source+'\nexpected_desktop_assets 0.201.0'],{
  encoding:'utf8',env:{...process.env,DESKTOP_RELEASE_WORKFLOW:join(root,'.github/workflows/desktop-release.yml')}
 });assert.equal(observed.status,0,observed.stderr);return observed.stdout.trim().split('\n');
}
const assets=expectedAssets();
function resolveFixture(names,{unknownAssets=false,unknownEvidence=false}={}){
 const directory=mkdtempSync(join(tmpdir(),'release-operation-controls-')),bin=join(directory,'bin');mkdirSync(bin);
 try{
  // Reuse the original typed producer mock, preserving its command operands.
  const original=readFileSync(join(root,'rust/tests/unit/ci-cd/desktop_release_resolve.rs'),'utf8');
  const start=original.indexOf('let mock = r#"');assert.ok(start>=0);
  const bodyStart=start+'let mock = r#"'.length,end=original.indexOf('\n"#;',bodyStart);assert.ok(end>bodyStart);
  const mock=original.slice(bodyStart,end);
  writeFileSync(join(bin,'gh-original'),mock,{mode:0o755});
  writeFileSync(join(bin,'gh.stage'),`#!/usr/bin/env bash
if [ "\${UNKNOWN_ASSETS:-false}" = true ] && [[ "$*" == *"--json assets"* ]]; then exit 33; fi
if [ "\${UNKNOWN_EVIDENCE:-false}" = true ] && [ "\${1:-}" = attestation ]; then exit 34; fi
exec "$(dirname "$0")/gh-original" "$@"
`,{mode:0o755});renameSync(join(bin,'gh.stage'),join(bin,'gh'));
  const output=join(directory,'output');writeFileSync(output,'');
  const result=spawnSync('/bin/bash',[join(root,'scripts/desktop-release-resolve.sh')],{encoding:'utf8',timeout:30000,
   env:{...process.env,PATH:bin+':'+process.env.PATH,GITHUB_OUTPUT:output,REPO:'link-assistant/formal-ai',GH_TOKEN:'declared-fixture-token',
    EVENT:'workflow_run',
      WORKFLOW_RUN_HEAD_SHA:'a'.repeat(40),
      WORKFLOW_RUN_ID: '42',
       WORKFLOW_RUN_ATTEMPT: '1',
       WORKFLOW_RUN_WORKFLOW_ID: '7',
       WORKFLOW_RUN_BRANCH: 'main',
       WORKFLOW_RUN_HEAD_REPOSITORY: 'link-assistant/formal-ai',
       WORKFLOW_RUN_CONCLUSION: 'success',
      MOCK_TAGS_JQ_OUTPUT:'v0.201.0',
      MOCK_LATEST_TAG:'v0.201.0',

    MOCK_PARENT_SHA:'a'.repeat(40),MOCK_RELEASE_EXISTS:'1',MOCK_ASSET_NAMES:names.join('\n'),
    MOCK_EVIDENCE_CREATOR:join(root,'rust/tests/fixtures/native-release-evidence/observations.mjs'),
    UNKNOWN_ASSETS:String(unknownAssets),UNKNOWN_EVIDENCE:String(unknownEvidence)}});
  assert.equal(result.status,0,result.stderr);return {outputs:readFileSync(output,'utf8'),stdout:result.stdout};
 }finally{rmSync(directory,{recursive:true,force:true});}
}
test('the actual inventory retains all original seventeen obligations and new artifact families',()=>{
 assert.equal(originalAssets.length,17);assert.equal(new Set(assets).size,assets.length);
 for(const asset of originalAssets)assert.ok(assets.includes(asset),'original asset obligation '+asset);
 for(const name of ['formal-ai-vscode-','formal-ai-cli-','formal-ai-native-source-','formal-ai-native-protocol-','formal-ai-native-x86_64-','formal-ai-signing-macos-']){
  assert.ok(assets.some(asset=>asset.startsWith(name)),name);
 }
 assert.ok(assets.includes('SHA256SUMS.txt'));assert.ok(assets.includes('BUILD-PROVENANCE.txt'));assert.ok(assets.length>17);
});
test('only complete actual resolver fixture evidence permits the automatic skip',()=>{
 const result=resolveFixture(assets);assert.match(result.outputs,/^tag=v0\.201\.0$/mu);assert.match(result.outputs,/^should_build=false$/mu);
 assert.match(result.stdout,/complete byte-consistent workflow-authenticated evidence/u);
});
for(const omitted of originalAssets)test('the original missing asset forces a healing build: '+omitted,()=>{
 const result=resolveFixture(assets.filter(asset=>asset!==omitted));assert.match(result.outputs,/^tag=v0\.201\.0$/mu);
 assert.match(result.outputs,/^should_build=true$/mu);assert.ok(result.stdout.includes(omitted));
});
for(const options of [{unknownAssets:true},{unknownEvidence:true}])test('unknown asset API or authentication refuses the automatic skip '+JSON.stringify(options),()=>{
 const result=resolveFixture(assets,options);assert.match(result.outputs,/^should_build=true$/mu);
});

const logicalOperations=['Wait for Crate availability on Crates.io','Log in to GitHub Container Registry','Set up Docker Buildx',
 'Extract GHCR Docker metadata','Publish Docker image to GHCR','Configure Docker Hub publishing','Create GitHub Release'];
function operation(job,label){
 const actual=label==='Extract GHCR Docker metadata'?'Publish Docker image to GHCR':label;
 const selected=job.steps.filter(step=>step.name===actual);assert.equal(selected.length,1,'unique operation '+label);
 if(label==='Extract GHCR Docker metadata')assert.match(selected[0].run,/^node scripts\/release-image-factory\.mjs publish prepared-release "\$RELEASE_VERSION" "\$GHCR_IMAGE"$/u);
 return selected[0];
}
function gate(expression,values){
 const tokens=String(expression).match(/steps\.[a-z_-]+\.outputs\.[a-z_]+|'[^']*'|==|!=|&&|\|\||[()]|\S/gu)??[];
 let cursor=0;
 function atom(){const token=tokens[cursor++];if(token==='('){const value=or();assert.equal(tokens[cursor++],')');return value;}
  if(/^'[^']*'$/u.test(token??''))return token.slice(1,-1);
  assert.ok(Object.hasOwn(values,token),'unknown gate operand '+token);return values[token];}
 function compare(){const left=atom(),op=tokens[cursor];if(op!=='=='&&op!=='!=')return left;cursor++;const right=atom();return op==='=='?left===right:left!==right;}
 function and(){let value=compare();while(tokens[cursor]==='&&'){cursor++;const right=compare();value=Boolean(value&&right);}return value;}
 function or(){let value=and();while(tokens[cursor]==='||'){cursor++;const right=and();value=Boolean(value||right);}return value;}
 const value=or();assert.equal(cursor,tokens.length,'gate tail must be consumed');return value;
}
test('all seven original release operations retain publication gates and actual metadata binding',async()=>{
 const {createRequire}=await import('node:module');
 const {parse}=createRequire(join(root,'package.json'))('yaml');
 const workflow=parse(readFileSync(join(root,'.github/workflows/release.yml'),'utf8'));
 for(const name of ['auto-release','manual-release']){
  const job=workflow.jobs[name];
  for(const label of logicalOperations){
   const step=operation(job,label);
   for(const first of [false,true])for(const second of [false,true])for(const published of [false,true]){
    const values=name==='auto-release'?{'steps.check.outputs.should_release':String(first),'steps.prepared.outputs.crate_published':String(second),'steps.publish-crate.outputs.publish_result':published?'success':'failed'}
     :{'steps.version.outputs.version_committed':String(first),'steps.version.outputs.already_released':String(second),'steps.publish-crate.outputs.publish_result':published?'success':'failed'};
    assert.equal(gate(step.if,values),name==='auto-release'?first&&(second||published):(first||second)&&published,name+' '+label);
   }
  }
  const order=['Build release','Verify packaged crate archive','Publish to Crates.io','Create GitHub Release'].map(label=>job.steps.findIndex(step=>step.name===label));
  assert.ok(order.every(index=>index>=0));assert.ok(order.slice(1).every((index,i)=>index>order[i]));
  assert.match(operation(job,'Wait for Crate availability on Crates.io').run,/--release-version/u);
  assert.ok(job.steps.find(step=>step.name==='Verify packaged crate archive').run.includes('cargo package --manifest-path rust/Cargo.toml --locked -p formal-ai'));
  for(const mutation of [structuredClone(job),structuredClone(job)]){
   const metadata=mutation.steps.find(step=>step.name==='Publish Docker image to GHCR');metadata.run+=' --unproved';
   assert.throws(()=>operation(mutation,'Extract GHCR Docker metadata'));
  }
  const missing=structuredClone(job);missing.steps=missing.steps.filter(step=>step.name!=='Publish Docker image to GHCR');
  assert.throws(()=>operation(missing,'Extract GHCR Docker metadata'));
 }
 assert.throws(()=>gate("unknown()",{}));assert.throws(()=>gate("'true' == 'true'; effect()",{}));
});
