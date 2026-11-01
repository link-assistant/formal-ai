// Externally specified bootstrap regression; no autonomous test synthesis claimed.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {isInconclusive} from '../../../js/agentic/crate/engine_answer.mjs';
import {drive} from '../../../experiments/js_dogfood/drive.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
const blocked=()=>{throw Error('LocalProductWasmForbidden');};
await installNodeHost(new WorkerHost({WebAssembly:{...WebAssembly,compile:blocked,instantiate:blocked,Module:blocked,Instance:blocked}}),{sourceUsageRustGrammar:false});
test('typed open and unverified reasoning cannot certify a conclusive finding',()=>{
  assert.deepEqual(['meta_reasoning_open','meta_reasoned_program_unverified'].map(intent=>isInconclusive({intent})),[true,true]);
});
test('verified program intents retain their prior classification',()=>{
  assert.deepEqual(['meta_reasoned_program','write_program'].map(intent=>isInconclusive({intent})),[false,false]);
});
test('the unchanged profile request never writes its referenced source file',{timeout:10000},async context=>{
  const workspace=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-open-reasoning-')));
  context.after(()=>fs.rmSync(workspace,{recursive:true,force:true}));
  const sourcePath='rust/tests/web/repository-documentation.test.mjs';
  const profilePath='data/benchmarks/fast-pull-request-javascript-tests.json';
  const inputs=new Map([[sourcePath,'// Controlled pre-existing source fixture.\nexport const retained = true;\n'],
    [profilePath,'{"schema":"fast-pr-javascript-tests/v1","files":["rust/tests/web/keep.test.mjs"]}\n']]);
  for(const [name,content] of inputs){const target=path.join(workspace,name);fs.mkdirSync(path.dirname(target),{recursive:true});fs.writeFileSync(target,content);}
  const prompt="Add rust/tests/web/repository-documentation.test.mjs to the files array in data/benchmarks/fast-pull-request-javascript-tests.json exactly once. Preserve every existing entry, keep the file valid JSON, and leave all other files unchanged. This is the JavaScript test profile; native tests and product builds must remain on main CI only.";
  const result=await drive(planChatStep,workspace,prompt,{steps:6,tools:['read','edit','write'],allowedCommands:[]});
  assert.ok(result.transcript.every(row=>row.tool!=='write'&&row.tool!=='edit'),'unresolved reasoning never grants a physical mutation');
  for(const [name,content] of inputs)assert.equal(fs.readFileSync(path.join(workspace,name),'utf8'),content,name);
});
