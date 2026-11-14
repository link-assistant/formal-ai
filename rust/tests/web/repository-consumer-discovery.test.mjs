// Externally specified controller regression. Local Rust syntax below is an
// explicit host mock: these checks qualify transport/custody, not Rust meaning.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {drive} from '../../../experiments/js_dogfood/drive.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {host,installHost} from '../../../js/agentic/host.mjs';
import {fileIoCommand,fileIoSearchCommand} from '../../../js/agentic/work_item_file_io.mjs';
const blocked=()=>{throw Error('LocalProductWasmForbidden');};
await installNodeHost(new WorkerHost({WebAssembly:{...WebAssembly,compile:blocked,instantiate:blocked,Module:blocked,Instance:blocked}}),{sourceUsageRustGrammar:process.env.CI==='true'});
const baselineHost=host();
const hash=value=>createHash('sha256').update(value).digest('hex');
const consumer='use ribbon_core::codec::decode;\npub fn read(value: usize) -> usize { decode(value) }\n';
const guide='# Runtime\n\nPreserve this original guide.\n';
const cargo='[package]\nname="example"\nversion="0.1.0"\n[lib]\npath="src/lib.rs"\n[dependencies]\nribbon-core={package="actual-core",version="2.7"}\n';
const entry='import {decode as readPacket} from "ribbon-core";\nexport function load(value){return readPacket(value);}\n';
const prompt='Document the ribbon-core dependency in docs/runtime.md based on native/Cargo.toml and web/client.mjs. Preserve the existing guide and include observed declarations and imported calls.';
function workspace(context,variant){
 const root=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-consumer-discovery-')));
 context.after(()=>{installHost(baselineHost);fs.rmSync(root,{recursive:true,force:true});});
 for(const [owner,source] of [['native/Cargo.toml',cargo],['web/client.mjs',entry],['docs/runtime.md',guide],
  ['native/src/lib.rs',consumer],['native/src/comment.rs','// ribbon_core::codec::decode appears only in a comment.\n']]){
  const target=path.join(root,owner);fs.mkdirSync(path.dirname(target),{recursive:true});fs.writeFileSync(target,source);
 }
 if(variant==='symlink'){
  fs.writeFileSync(path.join(root,'unowned.rs'),'pub fn outside() {}\n');
  fs.symlinkSync('../../unowned.rs',path.join(root,'native/src/escape.rs'));
 }
 // This adapter emits a predeclared protocol witness only. The CI-only real
 // Rust grammar suite independently qualifies concrete syntax observations.
 installHost({...baselineHost,observedSourceUsage:request=>request.language!=='Rust'
  ?baselineHost.observedSourceUsage(request)
  :{schema:'observed-source-usage/v1',owner:request.owner,sourceIdentity:hash(request.source),language:'Rust',
    imports:request.owner.endsWith('/lib.rs')?[{module:'ribbon_core::codec'}]:[],
    calls:request.owner.endsWith('/lib.rs')?[{module:'ribbon_core::codec',imported:'decode',source:'decode(value)'}]:[],
    references:[],gaps:[],complete:true,truncated:false,scope:'controller-test-host-mock; no Rust syntax acceptance'}});
 return root;
}
async function run(root,variant){
 const search=fileIoSearchCommand(root,'native/src',['ribbon_core']);
 let lastCall=null,searchCompleted=false,interference=false;
 const result=await drive(planChatStep,root,prompt,{steps:40,tools:['read','edit','bash'],
  allowedCommands:[fileIoCommand('root',null,null),search],onEvent:event=>{
   if(event.kind==='plan'){
    lastCall=event.plan?.calls?.[0]??null;
    if(variant==='changed-before-read'&&searchCompleted&&!interference&&lastCall?.tool==='read'
      &&JSON.parse(lastCall.arguments).filePath==='native/src/lib.rs'){
     fs.writeFileSync(path.join(root,'native/src/lib.rs'),consumer+'// Changed after the search.\n');interference=true;
    }
   }
   if(event.kind==='tool'){
    if(lastCall?.tool==='bash'&&JSON.parse(lastCall.arguments).command===search)searchCompleted=true;
    if(variant==='changed-after-edit'&&!interference&&lastCall?.tool==='edit'){
     fs.writeFileSync(path.join(root,'native/Cargo.toml'),cargo.replace('"2.7"','"2.8"'));interference=true;
    }
   }
  }});
 assert.equal(result.stop,'final');
 return {...result,state:JSON.parse(result.answer),search,interference};
}
test('real bounded search uses dependency alias and fresh reads before a mocked syntax witness',async context=>{
 const root=workspace(context),result=await run(root);
 assert.equal(result.state.reason,'UsefulCandidateReadbackVerified');
 assert.equal(result.transcript.filter(row=>row.tool==='edit').length,1);
 const discovery=result.state.facts.find(row=>row.kind==='observed-consumer-discovery');
 assert.deepEqual(discovery.queries[0].identifiers,['ribbon_core']);
 assert.deepEqual(discovery.confirmedOwners,['native/src/lib.rs']);
 assert.equal(discovery.candidates.find(row=>row.path==='native/src/comment.rs').syntaxConfirmed,false);
 assert.ok(fs.readFileSync(path.join(root,'docs/runtime.md'),'utf8').startsWith(guide));
 assert.equal(result.state.complete,false);assert.equal(result.state.verified,false);
});
test('changing a searched source before its read refuses custody without an edit',async context=>{
 const root=workspace(context),result=await run(root,'changed-before-read');
 assert.equal(result.interference,true);
 assert.equal(result.state.reason,'ConsumerSearchReadCustodyMismatch');
 assert.ok(result.transcript.every(row=>row.tool!=='edit'));
 assert.equal(fs.readFileSync(path.join(root,'docs/runtime.md'),'utf8'),guide);
});
test('manifest changes after the first edit cause no new search or write',async context=>{
 const root=workspace(context),result=await run(root,'changed-after-edit');
 assert.equal(result.interference,true);
 assert.equal(result.state.reason,'SourceFreshnessMismatch');
 const effect=result.transcript.findIndex(row=>row.tool==='edit');assert.ok(effect>=0);
 assert.ok(result.transcript.slice(effect+1).every(row=>row.tool!=='edit'&&row.tool!=='bash'));
});
test('incomplete symlink search stays pending and never reads a linked candidate',{skip:process.platform==='win32'},async context=>{
 const root=workspace(context,'symlink'),result=await run(root);
 assert.equal(result.state.reason,'UsefulCandidateReadbackVerified');
 const discovery=result.state.facts.find(row=>row.kind==='observed-consumer-discovery');
 assert.equal(discovery.state,'incomplete');assert.deepEqual(discovery.confirmedOwners,[]);
 assert.ok(discovery.queries[0].searchGaps.length>0);assert.equal(discovery.queries[0].error,null);
 assert.ok(discovery.gaps.some(reason=>reason.startsWith('consumer-search-incomplete:')));
 assert.ok(result.transcript.filter(row=>row.tool==='read').every(row=>!JSON.parse(row.arguments).filePath.endsWith('escape.rs')));
 assert.equal(result.state.complete,false);
});

test('CI real Rust grammar confirms the discovered consumer without granting semantic acceptance',{
 skip:process.env.CI!=='true',timeout:20000
},async context=>{
 const root=workspace(context);
 installHost(baselineHost);
 const result=await run(root);
 assert.equal(result.state.reason,'UsefulCandidateReadbackVerified');
 const discovery=result.state.facts.find(row=>row.kind==='observed-consumer-discovery');
 assert.deepEqual(discovery.confirmedOwners,['native/src/lib.rs']);
 assert.equal(discovery.candidates.find(row=>row.path==='native/src/comment.rs').syntaxConfirmed,false);
 assert.equal(result.state.complete,false);assert.equal(result.state.verified,false);
 assert.equal(result.transcript.filter(row=>row.tool==='edit').length,1);
});
