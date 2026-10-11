import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,existsSync,mkdirSync} from 'node:fs';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {gzipSync} from 'node:zlib';
const root='/Users/konard/Code/Archive/link-assistant/formal-ai',draft='/private/tmp/spec-recurrence-1188/release-control-repair';
const plan=JSON.parse(readFileSync(join(draft,'replacements.json'))),sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const packet='experiments/formal_ai_subagent/evidence/specification-delivery-1188/release-control-migration';
const test='rust/tests/web/release-workflow-operation-controls.test.mjs',fragment='changelog.d/1188-release-workflow-operation-controls.md';
for(const row of plan.replacements)assert.equal(sha(readFileSync(join(root,row.path))),row.sha256,row.path+' preimage drift');
for(const path of [packet,test,fragment])assert.equal(existsSync(join(root,path)),false,path+' already exists');
const originals=plan.replacements.map(row=>({path:row.path,bytes:readFileSync(join(root,row.path))}));
const source=[];
for(const row of plan.replacements){
 let body=row.content;
 if(row.path.endsWith('.rs')){
  const path=join(draft,row.path.split('/').at(-1));
  const formatted=execFileSync('rustfmt',['--edition','2024','--emit','stdout',path],{encoding:'utf8'});
  const prefix=path+':\n\n';assert.ok(formatted.startsWith(prefix));body=formatted.slice(prefix.length);
 }
 source.push({path:row.path,body});
}
source.push({path:test,body:readFileSync(join(draft,'release-workflow-operation-controls.test.mjs'),'utf8')});
source.push({path:fragment,body:'Release controls now bind the metadata operation to the prepared image factory and retain every original desktop and updater asset obligation. Independent fixture checks cover version/latest tag ordering, complete release evidence, each missing original asset and unknown API/authentication responses. Production release gates and budgets are unchanged; U9 remains Open.\n'});
mkdirSync(join(root,packet,'original'),{recursive:true});mkdirSync(join(root,packet,'raw'),{recursive:true});
for(const item of originals)writeFileSync(join(root,packet,'original',item.path.split('/').at(-1)+'.txt'),item.bytes);
const failedLogs=['new-failure-114030300740.log','new-failure-114030300797.log'];
const originalObservations=failedLogs.map(name=>{const bytes=readFileSync('/private/tmp/pr1188-0f701-official/'+name);writeFileSync(join(root,packet,'original',name+'.gz'),gzipSync(bytes));return{name,bytes:bytes.length,sha256:sha(bytes)};});
for(const row of source)writeFileSync(join(root,row.path),row.body);
for(const task of ['T4216'])writeFileSync(join(root,packet,'raw',task+'.json'),readFileSync('/private/tmp/spec-recurrence-1188/'+task+'.json'));
const result={kind:'OriginalReleaseOperationControlMigration',preimages:plan.replacements.map(row=>({path:row.path,sha256:row.sha256})),originalObservations,originalLogicalOperations:7,originalRequiredAssets:plan.originalAssets,originalMinimumAssetCount:17,installed:source.map(row=>({path:row.path,bytes:Buffer.byteLength(row.body),sha256:sha(row.body)})),autonomousCredit:0,nativeCompilation:false,livePublication:false,U9:'Open'};
writeFileSync(join(root,packet,'manifest.json'),JSON.stringify(result,null,2)+'\n');
writeFileSync(join(draft,'installed-paths.json'),JSON.stringify([...source.map(row=>row.path),packet],null,2)+'\n');
console.log(JSON.stringify(result));console.log('T4217_RELEASE_CONTROLS_INSTALLED');
