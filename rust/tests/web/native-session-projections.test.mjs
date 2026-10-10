import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,rmSync,readFileSync,symlinkSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {projectionRegistry,projectionInputs,verifyProjectionDirectory,projectionDataProfile,verifyProjectionDataEntries,digest,sessionSchema} from '../../../scripts/lib/native-session-projections.mjs';
const source='let sessions: [(&str, &str); 1] = [(TASK, "docs/case-studies/issue-42/session.json")];';
const path=projectionRegistry(source)[0];
const driverBytes=readFileSync(new URL('../../../rust/src/agentic_coding/driver.rs',import.meta.url));
const schema=sessionSchema(driverBytes.toString('utf8'));
const value={driver:schema.root.driver.value,server:schema.root.server.value,turns:1,task:'inspect finite data',tools_advertised:['read_file'],hit_turn_cap:false,steps:[],final_answer:'observed'};
const bytes=Buffer.from(JSON.stringify(value)+'\n');
const input={path,sha256:digest(bytes),task:value.task,tools:value.tools_advertised};
function scratch(run) {const directory=mkdtempSync(join(tmpdir(),'native-projection-controls-'));try{mkdirSync(join(directory,'docs/case-studies/issue-42'),{recursive:true});writeFileSync(join(directory,path),bytes);return run(directory);}finally{rmSync(directory,{recursive:true,force:true});}}
test('maintained producer inventory is finite and complete',()=>{
  const producer=readFileSync(new URL('../../../rust/examples/regenerate_agent_cli_sessions.rs',import.meta.url));
  assert.equal(projectionRegistry(producer.toString()).length,13);
});
test('registry rejects incomplete unsafe and duplicated source selection',()=>{
  assert.throws(()=>projectionRegistry(source.replace('; 1]','; 2]')));
  assert.throws(()=>projectionRegistry(source.replace('issue-42','..')));
  assert.throws(()=>projectionRegistry(source.replace('; 1]','; 2]').replace('];',', (TASK, "'+path+'")];')));
});
test('actual byte membership and pinned task/tools stay coupled',()=>scratch(directory=>{
  assert.deepEqual(projectionInputs(directory,[path]),[input]);
  assert.deepEqual(verifyProjectionDirectory(directory,[input],schema),[{path,bytes:bytes.length,sha256:digest(bytes)}]);
}));
for(const [name,change] of [
  ['turn-cap',directory=>writeFileSync(join(directory,path),JSON.stringify({...value,hit_turn_cap:true})+'\n')],
  ['task-forgery',directory=>writeFileSync(join(directory,path),JSON.stringify({...value,task:'foreign'})+'\n')],
  ['tool-widening',directory=>writeFileSync(join(directory,path),JSON.stringify({...value,tools_advertised:['read_file','write_file']})+'\n')],
  ['extra-member',directory=>writeFileSync(join(directory,'foreign.json'),'{}')],
  ['symlink',directory=>{rmSync(join(directory,path));symlinkSync('/etc/hosts',join(directory,path));}],
]) test('refuses '+name,()=>scratch(directory=>{change(directory);assert.throws(()=>verifyProjectionDirectory(directory,[input],schema));}));
function packet() {
  const producerBytes=Buffer.from(source);
  const identity={driverSourceSha256:digest(driverBytes),producerSha256:digest(producerBytes),commit:'a'.repeat(40),tree:'b'.repeat(40),executableSha256:'c'.repeat(64),physicalInputs:{sha256:'d'.repeat(64)},producer:'rust/examples/regenerate_agent_cli_sessions.rs',run:'1',attempt:'1',profile:'release',features:'Cargo default'};
  const profile=projectionDataProfile(producerBytes,[input],identity,driverBytes);
  const empty=Buffer.alloc(0),process={budgetMs:1200000,exitCode:0,signal:null,error:null,stdoutSha256:digest(empty),stderrSha256:digest(empty)};
  const receipt={schema:'native-session-projection-producer/v1',
    authority:'not-granted',
    semanticPassCredit:false,
    originalAssertionsModifiedByProducer:false,
    expectedOutputFieldsUsedForGeneration:false,
    identity,
    inputs:[input],
    sessionSchemaSha256:digest(Buffer.from(JSON.stringify(schema))),
    registrySha256:digest(Buffer.from(path+'\n')),
    build:process,
    producerExecution:process,
    members:[{path,
    bytes:bytes.length,
    sha256:digest(bytes)}]};

  const entries=[{name:'generated/'+path,bytes,directory:false},{name:'producer-receipt.json',bytes:Buffer.from(JSON.stringify(receipt)),directory:false},...['build.stdout.bin','build.stderr.bin','producer.stdout.bin','producer.stderr.bin'].map(name=>({name,bytes:empty,directory:false}))];
  return {profile,entries,receipt};
}
test('pure profile validates data consistency without native or authority credit',()=>{const p=packet();const result=verifyProjectionDataEntries(p.profile,p.entries);assert.equal(result.authority,'not-granted');assert.equal(result.semanticPassCredit,false);});
test('profile refuses caller-forged copied profile',()=>{const p=packet();assert.throws(()=>verifyProjectionDataEntries({...p.profile},p.entries));});
for(const [name,change] of [
 ['source-identity',r=>r.identity.commit='f'.repeat(40)],['failed-producer',r=>r.producerExecution.exitCode=1],['semantic-credit',r=>r.semanticPassCredit=true],['generated-hash',r=>r.members[0].sha256='e'.repeat(64)],['pinned-input',r=>r.inputs[0].task='foreign'],
]) test('archive refuses '+name,()=>{const p=packet();change(p.receipt);p.entries.find(e=>e.name==='producer-receipt.json').bytes=Buffer.from(JSON.stringify(p.receipt));assert.throws(()=>verifyProjectionDataEntries(p.profile,p.entries));});
test('archive refuses missing duplicate and foreign members',
  ()=>{for(const mutate of [entries=>entries.pop(),
  entries=>entries.push(entries[0]),
  entries=>entries.push({name:'../foreign',
  bytes:Buffer.alloc(0),
  directory:false})]){const p=packet();
  mutate(p.entries);
  assert.throws(()=>verifyProjectionDataEntries(p.profile,
  p.entries));
  }});

test('CI producer refuses local invocation before any cargo or rustc execution',()=>{
 const cwd=fileURLToPath(new URL('../../../',import.meta.url));
 const result=spawnSync(process.execPath,['scripts/generate-native-session-projections.mjs','/tmp/unused-native-data'],{cwd,env:{...process.env,GITHUB_ACTIONS:'false'},encoding:'utf8'});
 assert.notEqual(result.status,0);assert.match(result.stderr,/runs only in hosted CI/u);
});

test('source-derived complete shape refuses missing driver server turns and malformed steps',()=>scratch(directory=>{
 for(const field of ['driver','server','turns']) {const changed={...value};delete changed[field];writeFileSync(join(directory,path),JSON.stringify(changed)+'\n');assert.throws(()=>verifyProjectionDirectory(directory,[input],schema));}
 for(const changed of [{...value,
   driver:'foreign'},
   {...value,
   server:'foreign'},
   {...value,
   turns:'1'},
   {...value,
   turns:-1},
   {...value,
   steps:[{tool:'read_file',
   arguments:{path:'x'}}]},
   {...value,
   steps:[{tool:'read_file',
   arguments:{path:'x'},
   result:42}]}]) {writeFileSync(join(directory,
   path),
   JSON.stringify(changed)+'\n');
   assert.throws(()=>verifyProjectionDirectory(directory,
   [input],
   schema));
   }
}));
test('schema derives types and literal values from source and refuses unsupported source',()=>{
 const changed=sessionSchema(driverBytes.toString().replace('formal-ai in-repo agentic CLI','source-owned alternate driver'));
 assert.equal(changed.root.driver.value,'source-owned alternate driver');
 assert.throws(()=>sessionSchema(driverBytes.toString().replace('"turns": self.turns','"turns": unknown_runtime()')));
});
test('producer identity refuses zero run and attempt IDs',()=>{
 const p=packet();for(const field of ['run','attempt']) {const identity={...p.receipt.identity,[field]:'0'};assert.throws(()=>projectionDataProfile(Buffer.from(source),[input],identity,driverBytes));}
});
test('archive refuses missing or excessive process budgets',()=>{
 for(const budgetMs of [undefined,
   0,
   1200001]) {const p=packet();
   p.receipt.producerExecution={...p.receipt.producerExecution,
   budgetMs};
   p.entries.find(e=>e.name==='producer-receipt.json').bytes=Buffer.from(JSON.stringify(p.receipt));
   assert.throws(()=>verifyProjectionDataEntries(p.profile,
   p.entries));
   }
});
