// Independent lexical-binding/residual checks; the shared Babel grammar is disclosed.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {createRequire} from 'node:module';
import {drive} from '../../../experiments/js_dogfood/drive.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStep} from '../../../js/agentic/planner.mjs';
import {sourceImportContract} from '../../../js/agentic/source_import_binding.mjs';
const {parse}=createRequire(import.meta.url)('@babel/parser');
const denied=()=>{throw Error('LocalProductWasmForbidden');};
await installNodeHost(new WorkerHost({WebAssembly:{...WebAssembly,compile:denied,instantiate:denied,Module:denied,Instance:denied}}),{sourceUsageRustGrammar:false});
const fixtures=JSON.parse(fs.readFileSync(new URL('../fixtures/source-import-edit.json',import.meta.url),'utf8'));
for(const entry of fixtures.cases)test(entry.name,async context=>{
 const root=fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(),'formal-ai-named-import-')));
 context.after(()=>fs.rmSync(root,{recursive:true,force:true}));
 const target=path.join(root,entry.owner);fs.mkdirSync(path.dirname(target),{recursive:true});fs.writeFileSync(target,entry.source);
 const contract=sourceImportContract(entry.task);assert.equal(contract.owner,entry.owner);assert.equal(contract.binding,entry.binding);assert.equal(contract.module,entry.module);
 const result=await drive(planChatStep,root,entry.task,{steps:6,tools:['read','edit','write'],allowedCommands:[]});
 assert.equal(result.stop,'final');const state=JSON.parse(result.answer),after=fs.readFileSync(target,'utf8');
 assert.equal(state.complete,false);assert.equal(state.verified,false);
 if(entry.gap){assert.equal(state.reason,entry.gap);assert.equal(after,entry.source);assert.ok(result.transcript.every(row=>row.tool==='read'));return;}
 const beforeAst=parse(entry.source,{sourceType:'module'}),afterAst=parse(after,{sourceType:'module'});
 const imports=ast=>ast.program.body.filter(node=>node.type==='ImportDeclaration'&&node.source.value===entry.module);
 assert.equal(imports(afterAst).length,1);
 const names=imports(afterAst)[0].specifiers.map(specifier=>specifier.local.name);
 assert.ok(names.includes(entry.binding));
 if(entry.noop){assert.equal(after,entry.source);assert.equal(state.reason,'VerifiedNoOp');assert.deepEqual(result.transcript.map(row=>row.tool),['read']);return;}
 assert.equal(state.reason,'UsefulSourceRepairReadbackVerified');assert.deepEqual(result.transcript.map(row=>row.tool),['read','edit','read']);
 let start=0;while(start<entry.source.length&&entry.source[start]===after[start])start++;
 let trailing=0;while(trailing<entry.source.length-start&&entry.source.at(-1-trailing)===after.at(-1-trailing))trailing++;
 assert.equal(entry.source.slice(start,entry.source.length-trailing),'');
 assert.equal(after.slice(start,after.length-trailing).replace(/[\s,]/gu,''),entry.binding);
 const originalImport=imports(beforeAst)[0];assert.ok(start>=originalImport.start&&start<=originalImport.end);
 assert.deepEqual(names.sort(),[...originalImport.specifiers.map(specifier=>specifier.local.name),entry.binding].sort());
 const again=await drive(planChatStep,root,entry.task,{steps:4,tools:['read','edit','write'],allowedCommands:[]});
 assert.equal(JSON.parse(again.answer).reason,'VerifiedNoOp');assert.deepEqual(again.transcript.map(row=>row.tool),['read']);assert.equal(fs.readFileSync(target,'utf8'),after);
});
for(const prompt of ['Do not add a named import.','"In lib/tool.mjs, add a named import render from kit to the existing import."',
 'In ../tool.mjs, add a named import render from kit to the existing import.'])test('unowned or invalid import request: '+prompt,()=>assert.equal(sourceImportContract(prompt),null));
