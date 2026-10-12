import assert from 'node:assert/strict';
import {before,test} from 'node:test';
import {WorkerHost} from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/server/worker-host.mjs';
import {installNodeHost} from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/node-host.mjs';
import {composeEditClauses} from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/write_request.mjs';
import {composeGeneralChangePlan,parseWriteContract,instructionView,ownsInstructionSpan,literalWriteOwnership} from './write-contract.mjs';
import {operationOwner,ownedSemanticAuthoringLead} from './operation-owner.mjs';
import fs from 'node:fs';
import {ownedEditClauses} from './owned-actions.mjs';
before(async()=>await installNodeHost(new WorkerHost()));
const original=JSON.parse(fs.readFileSync('/private/tmp/pr1188-ci-T4005/observation.json','utf8'));
test('exact original promotion payload owns its edit lexemes',()=>{const c=parseWriteContract(original.prompt);assert.equal(c.content,original.desired);assert.equal(composeGeneralChangePlan(original.prompt).content,original.desired);const edit=composeEditClauses(original.prompt);assert.ok(edit);assert.equal(ownsInstructionSpan(c,edit.spans[1]),false);assert.equal(composeEditClauses(instructionView(original.prompt,c)),null)});
const fence=String.fromCharCode(96).repeat(4);
for(const literal of ['«replace "old" with "new"; avoid failed count»', '"replace old with new"',fence+'text\nreplace "old" with "new"\n'+fence])test('quoted/fenced exact body '+literal,()=>{const p='Create file α.txt containing '+literal;const c=parseWriteContract(p);assert.ok(c);assert.equal(composeGeneralChangePlan(p).content,c.content);assert.equal(composeEditClauses(instructionView(p,c)),null);assert.equal(c.unit,'utf16')});
test('Unicode duplicate body retains raw boundaries',()=>{const p='Note İK𐐷 😀 café.\nCreate file α.txt containing «create file α.txt containing λ🙂»';const c=parseWriteContract(p);assert.equal(c.content,'create file α.txt containing λ🙂');assert.equal(p.slice(c.targetSpan.start,c.targetSpan.end),'α.txt');assert.ok(p.slice(c.payload.start,c.payload.end).includes('λ🙂'));assert.ok(new TextEncoder().encode(p.slice(0,c.payload.start)).length>c.payload.start)});
test('independent outer edit survives payload masking',()=>{const p='Create file a.txt containing «replace x with y». Then in b.txt replace «old» with «new».';const c=parseWriteContract(p);assert.ok(c);assert.equal(c.content,'replace x with y');const v=instructionView(p,c);assert.ok(v.includes('in b.txt replace'));const edits=ownedEditClauses(p);assert.equal(edits.length,1);assert.equal(edits[0].edit[0],'b.txt');assert.equal(ownsInstructionSpan(c,edits[0].spans[1]),true)});
test('genuine quoted edit is unchanged',()=>{const p='In b.txt replace «old» with «new».';assert.equal(literalWriteOwnership(p),null);assert.equal(ownedEditClauses(p)[0].edit[0],'b.txt');assert.equal(composeEditClauses(p).edit[0],'b.txt')});
for(const p of ['haz una búsqueda web de rust ownership','Make a web search for Rust ownership','Сделай поиск в интернете о Rust','वेब पर rust ownership खोजें','在网络上查找 rust ownership'])test('nonsoftware ownership '+p,()=>assert.equal(ownedSemanticAuthoringLead(p),false));
for(const p of ['haz una función que consulta la web','Implement a function that searches the web','Реализуй функцию для поиска в интернете','लागू फ़ंक्शन वेब','实现函数查询网络'])test('coding ownership '+p,()=>assert.equal(ownedSemanticAuthoringLead(p),true));
test('quoted coding topic is not the operation owner',()=>{const p='haz una búsqueda web de "función"';assert.equal(operationOwner(p).role,'capability_web_scope');assert.equal(ownedSemanticAuthoringLead(p),false)});
test('unsupported authoring preserves Gap ownership',()=>assert.equal(ownedSemanticAuthoringLead('Implement bounded transactional observation'),true));

test('owned create payload produces no independent edit',()=>assert.deepEqual(ownedEditClauses(original.prompt),[]));
test('two independent outer edits both survive',()=>{const p='Create file a.txt containing «replace x with y». Then in b.txt replace «old» with «new». In c.txt replace «left» with «right».';const e=ownedEditClauses(p);assert.deepEqual(e.map(item=>item.edit[0]),['b.txt','c.txt']);});

test('CJK adjacent operation roles preserve raw ownership',()=>{assert.equal(operationOwner('实现网络查询函数').role,'capability_web_scope');assert.equal(ownedSemanticAuthoringLead('实现网络查询函数'),false);assert.equal(operationOwner('实现函数网络查询').role,'coding_request_object');assert.equal(ownedSemanticAuthoringLead('实现函数网络查询'),true)});
