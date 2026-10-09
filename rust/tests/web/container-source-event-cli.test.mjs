// Execute real source seal/import CLIs in small committed Git repositories; API replies are explicit fixtures.
import test from 'node:test';
import assert from 'node:assert/strict';
import {spawnSync,execFileSync} from 'node:child_process';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync,chmodSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const source=fileURLToPath(new URL('../../../scripts/native-release-source.mjs',import.meta.url));
const workflow=readFileSync(new URL('../../../.github/workflows/container-images.yml',import.meta.url),'utf8');
const git=(cwd,...args)=>execFileSync('git',args,{cwd,encoding:'utf8',stdio:'pipe'}).trim();
function fixture(t) {
 const cwd=mkdtempSync(join(tmpdir(),'container-source-cli-'));t.after(()=>rmSync(cwd,{recursive:true,force:true}));
 mkdirSync(join(cwd,'rust/src'),{recursive:true});mkdirSync(join(cwd,'bin'));mkdirSync(join(cwd,'selection'));
 writeFileSync(join(cwd,'rust/Cargo.toml'),'[package]\nname="formal-ai"\nversion="1.2.3"\n');
 writeFileSync(join(cwd,'rust/Cargo.lock'),'version = 4\n');writeFileSync(join(cwd,'rust/src/main.rs'),'fn main() {}\n');
 git(cwd,'init','--initial-branch=main');git(cwd,'config','user.name','Fixture');git(cwd,'config','user.email','fixture@example.invalid');
 git(cwd,'add','.');git(cwd,'commit','-m','real committed fixture');const head=git(cwd,'rev-parse','HEAD');git(cwd,'tag','v1.2.3');git(cwd,'remote','add','origin',cwd);
 writeFileSync(join(cwd,'metadata.json'),JSON.stringify({workspace_members:['fixture'],packages:[{id:'fixture',name:'formal-ai',version:'1.2.3',targets:[{name:'formal-ai',kind:['bin']}],features:{default:['server']}}]}));
 writeFileSync(join(cwd,'compiler.txt'),'release: 1.99.0\ncommit-hash: '+'a'.repeat(40)+'\n');
 const gh=join(cwd,'bin/gh');writeFileSync(gh,'#!/usr/bin/env node\nconst p=process.argv[3];const head=process.env.FIXTURE_HEAD;let result;if(p.endsWith("/releases/tags/v1.2.3"))result={tag_name:"v1.2.3",draft:false,prerelease:false,published_at:"2026-10-01T00:00:00Z"};else if(p.endsWith("/commits/main")||p.endsWith("/commits/v1.2.3"))result={sha:head};else if(p==="repos/fixture/repository/")result={default_branch:"main"};else throw Error("unexpected fixture API "+p);console.log(JSON.stringify(result));\n');chmodSync(gh,0o755);
 const environment={...process.env,PATH:join(cwd,'bin')+':'+process.env.PATH,FIXTURE_HEAD:head,
  NATIVE_SELECTED_HEAD:head,NATIVE_BASE_COMMIT:head,NATIVE_RELEASE_TAG:'',NATIVE_RELEASE_EVENT:'pull_request',NATIVE_RELEASE_PURPOSE:'validation',
  GITHUB_RUN_ID:'4242',NATIVE_REPOSITORY:'fixture/repository',NATIVE_AUTHORIZED_RELEASE_COMMIT:head,NATIVE_AUTHORIZED_DEFAULT_COMMIT:head,
  NATIVE_METADATA_FILE:join(cwd,'metadata.json'),NATIVE_COMPILER_FILE:join(cwd,'compiler.txt')};
 delete environment.GITHUB_OUTPUT;delete environment.SOURCE_PRODUCER_RUN;
 return {cwd,head,environment};
}
function cli(f,mode,change={}) {
 const env={...f.environment,...change};for(const key of Object.keys(env))if(env[key]===undefined)delete env[key];
 return spawnSync(process.execPath,[source,mode,join(f.cwd,'selection')],{cwd:f.cwd,env,encoding:'utf8'});
}
function roundtrip(f,change={}) {
 const seal=cli(f,'seal',change);assert.equal(seal.status,0,seal.stderr);
 const bytes=readFileSync(join(f.cwd,'selection/source-selection.json'));const record=JSON.parse(bytes);
 const imported=cli(f,'import',{...change,NATIVE_SELECTION_SHA256:createHash('sha256').update(bytes).digest('hex')});
 assert.equal(imported.status,0,imported.stderr);assert.deepEqual(JSON.parse(imported.stdout),record);return {record,bytes};
}
test('real PR source seal and import bind event, actual Git head, base and same producer run',t=>{
 const f=fixture(t);git(f.cwd,'checkout','-b','actual-pr');writeFileSync(join(f.cwd,'rust/src/main.rs'),'fn main() { /* PR */ }\n');git(f.cwd,'commit','-am','PR head');const head=git(f.cwd,'rev-parse','HEAD');
 git(f.cwd,'checkout','main');writeFileSync(join(f.cwd,'main-doc'),'current base');git(f.cwd,'add','main-doc');git(f.cwd,'commit','-m','actual pinned base');const base=git(f.cwd,'rev-parse','HEAD');git(f.cwd,'merge','--no-ff','actual-pr','-m','real selected merge');
 const {record,bytes}=roundtrip(f,{NATIVE_SELECTED_HEAD:head,NATIVE_BASE_COMMIT:base});assert.equal(record.release_tag,null);assert.equal(record.selected_head,head);assert.equal(record.pinned_base,base);assert.notEqual(record.source_commit,head);assert.ok(record.bundle);assert.equal(record.producer_run,'4242');
 const wrong=cli(f,'import',{NATIVE_SELECTED_HEAD:head,NATIVE_BASE_COMMIT:base,GITHUB_RUN_ID:'4243',NATIVE_SELECTION_SHA256:createHash('sha256').update(bytes).digest('hex')});assert.notEqual(wrong.status,0);assert.match(wrong.stderr,/4243/u);
});
test('actual explicit dispatch validates an unprivileged custom source without claiming PR authority',t=>{
 const f=fixture(t);git(f.cwd,'checkout','-b','custom');writeFileSync(join(f.cwd,'rust/src/main.rs'),'fn main() { /* custom */ }\n');git(f.cwd,'commit','-am','custom');
 const head=git(f.cwd,'rev-parse','HEAD');const {record}=roundtrip(f,{NATIVE_RELEASE_EVENT:'workflow_dispatch',NATIVE_SELECTED_HEAD:head,NATIVE_BASE_COMMIT:head});
 assert.equal(record.release_tag,null);assert.equal(record.selected_head,head);assert.equal(record.source_commit,head);
});
test('real stable release, successful workflow-run and authorized dispatch execute source CLI import authority',t=>{
 for(const event of ['release','workflow_run','workflow_dispatch']) {
  const f=fixture(t);const {record}=roundtrip(f,{NATIVE_RELEASE_EVENT:event,NATIVE_RELEASE_PURPOSE:'publication',NATIVE_RELEASE_TAG:'v1.2.3'});
  assert.equal(record.release_tag,'v1.2.3');assert.equal(record.bundle,null);assert.equal(record.source_commit,f.head);
 }
});
test('missing or wrong events, wrong purposes and false PR stable authority fail real CLI sealing',t=>{
 const f=fixture(t);
 for(const change of [{NATIVE_RELEASE_EVENT:undefined},{NATIVE_RELEASE_EVENT:'push'},{NATIVE_RELEASE_EVENT:'invented'},
  {NATIVE_RELEASE_PURPOSE:'invented'},{NATIVE_RELEASE_EVENT:'release',NATIVE_RELEASE_PURPOSE:'validation'},
  {NATIVE_RELEASE_EVENT:'workflow_dispatch',NATIVE_RELEASE_PURPOSE:'publication'},
  {NATIVE_RELEASE_EVENT:'pull_request',NATIVE_RELEASE_PURPOSE:'publication',NATIVE_RELEASE_TAG:'v1.2.3'},
  {NATIVE_RELEASE_EVENT:'workflow_dispatch',NATIVE_RELEASE_PURPOSE:'validation',NATIVE_RELEASE_TAG:'v1.2.3'}])assert.notEqual(cli(f,'seal',change).status,0,JSON.stringify(change));
});
test('actual source import refuses unbound event, wrong purpose and modified stable source identity',t=>{
 const f=fixture(t);const {bytes}=roundtrip(f,{NATIVE_RELEASE_EVENT:'release',NATIVE_RELEASE_PURPOSE:'publication',NATIVE_RELEASE_TAG:'v1.2.3'});
 const digest=createHash('sha256').update(bytes).digest('hex');
 for(const change of [{NATIVE_RELEASE_EVENT:undefined},{NATIVE_RELEASE_EVENT:'invented'},
  {NATIVE_RELEASE_EVENT:'release',NATIVE_RELEASE_PURPOSE:'validation'},
  {NATIVE_RELEASE_EVENT:'release',NATIVE_RELEASE_PURPOSE:'publication',NATIVE_SELECTED_HEAD:'f'.repeat(40)}])
  assert.notEqual(cli(f,'import',{NATIVE_SELECTION_SHA256:digest,...change}).status,0);
});
test('workflow binds actual events and only maps reusable push after independent stable authorization',()=>{
 assert.equal((workflow.match(/NATIVE_RELEASE_EVENT: \$\{\{ github\.event_name \}\}/gu)??[]).length,6);
 assert.equal((workflow.match(/NATIVE_RELEASE_PURPOSE: \$\{\{ needs\.resolve\.outputs\.publish == 'true' && 'publication' \|\| 'validation' \}\}/gu)??[]).length,6);
 assert.match(workflow,/NATIVE_PR_HEAD: \$\{\{ github\.event\.pull_request\.head\.sha \}\}/u);assert.match(workflow,/NATIVE_PR_BASE: \$\{\{ github\.event\.pull_request\.base\.sha \}\}/u);
 assert.match(workflow,/native-release-trust\.mjs"\n        case "\$NATIVE_RELEASE_EVENT" in\n          push\|workflow_call\)/u);
 assert.equal((workflow.match(/push\|workflow_call\) export NATIVE_RELEASE_EVENT=workflow_dispatch/gu)??[]).length,5);
 assert.doesNotMatch(workflow,/export NATIVE_RELEASE_EVENT=pull_request/u);
});
