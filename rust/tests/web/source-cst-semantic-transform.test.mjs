import {before,after,test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {sourceCstFrontends} from '../../../scripts/lib/source-cst-frontends.mjs';
import {sourceToLinks,sourceFromLinks,ownedSourceLinks,writeSourceLinks,readSourceLinks,verifySourceLinks} from '../../../js/meta/readable-source.js';
import {createSourceCstTransformRule,readSourceCstTransformRule,writeSourceCstTransformRule,applySourceCstTransform,verifySourceCstTransform} from '../../../scripts/lib/source-cst-transform.mjs';
import {runSourceModel} from '../../../scripts/source-model.mjs';
let runtime;
before(async()=>{runtime=await sourceCstFrontends();});after(()=>runtime?.close());
const root=fileURLToPath(new URL('../../../',import.meta.url));
function fixture(source,language='JavaScript',owner='held-out.mjs'){
 const frontend=runtime.forSource(owner,language),document=ownedSourceLinks(owner,sourceToLinks(source,language,frontend));
 return {source,language,owner,frontend,document,rule:createSourceCstTransformRule(document,{declaration:'combine',binding:'count',target:'quantity'})};
}
for(const [language,owner,source,expected] of [
 ['JavaScript','held-out.mjs','\uFEFF// count docs 😀\r\nexport function combine(count, offset) { const adjusted = count + offset; return {count: count, adjusted}; }\r\nconst note = "count";\r\n','\uFEFF// count docs 😀\r\nexport function combine(quantity, offset) { const adjusted = quantity + offset; return {count: quantity, adjusted}; }\r\nconst note = "count";\r\n'],
 ['TypeScript','held-out.ts','// हिन्दी count\nexport function combine(count: number, offset: number): number { return count * offset; }\n','// हिन्दी count\nexport function combine(quantity: number, offset: number): number { return quantity * offset; }\n'],
 ['Rust','held-out.rs','// count Москва\nfn combine(count: usize, offset: usize) -> usize { let adjusted = count * offset; adjusted + count }\nconst NOTE: &str = "count";\n','// count Москва\nfn combine(quantity: usize, offset: usize) -> usize { let adjusted = quantity * offset; adjusted + quantity }\nconst NOTE: &str = "count";\n'],
])test(language+' declared alpha rule retains full concrete syntax, types, effect order and exact inverse',()=>{
 const item=fixture(source,language,owner),notation=writeSourceCstTransformRule(item.rule);
 assert.match(notation,/source-transform-contract[\s\S]*preconditions[\s\S]*complete-preimage/);
 assert.match(notation,/preimage[\s\S]*owned-source[\s\S]*source[\s\S]*function/);
 assert.ok(!notation.includes(JSON.stringify(source)),'full source is represented recursively rather than one quoted source blob');
 const persisted=readSourceLinks(writeSourceLinks(item.document));
 const result=applySourceCstTransform(persisted,readSourceCstTransformRule(notation).network,item.frontend);
 assert.equal(sourceFromLinks(result.document),expected);
 assert.equal(verifySourceCstTransform(persisted,result.document,item.rule,item.frontend),true);
 assert.equal(verifySourceLinks(result.document,item.frontend),true);
 const reversed=applySourceCstTransform(result.document,readSourceCstTransformRule(writeSourceCstTransformRule(result.inverse)).network,item.frontend);
 assert.deepEqual(reversed.document,persisted);assert.equal(sourceFromLinks(reversed.document),source);
 assert.equal(result.verification.arbitrarySemanticEquivalence,false);assert.equal(result.verification.sourceReflection,'not-certified');assert.equal(result.verification.authorshipCredit,0);
});
for(const [name,language,owner,source,reason] of [
 ['JS capture','JavaScript','x.mjs','function combine(count) { return count + quantity; }',/Capture/],
 ['TS capture','TypeScript','x.ts','function combine(count: number): number { return count + quantity; }',/Capture/],
 ['Rust capture','Rust','x.rs','fn combine(count: usize) -> usize { count + quantity }',/Capture/],
 ['JS shadow','JavaScript','x.mjs','function combine(count) { { let count=1; return count; } }',/Shadow/],
 ['Rust shadow','Rust','x.rs','fn combine(count: usize) -> usize { let count=1; count }',/Shadow/],
 ['JS closure','JavaScript','x.mjs','function combine(count) { return () => count; }',/ScopeOrEffect/],
 ['Rust closure','Rust','x.rs','fn combine(count: usize) { let next = || count; }',/ScopeOrEffect/],
 ['JS shorthand','JavaScript','x.mjs','function combine(count) { return {count}; }',/AmbiguousTransformReference/],
 ['Rust shorthand','Rust','x.rs','fn combine(count: usize) { Box { count } }',/ScopeOrEffect/],
 ['JS dynamic lookup','JavaScript','x.mjs','function combine(count) { return eval("count"); }',/Dynamic/],
 ['Rust macro','Rust','x.rs','fn combine(count: usize) { stringify!(count); }',/ScopeOrEffect/],
 ['JS destructuring','JavaScript','x.mjs','function combine({count}) { return count; }',/ParameterPattern/],
 ['Rust destructuring','Rust','x.rs','fn combine((count, other): (usize,usize)) -> usize { count }',/ParameterPattern/],
 ['TS type query','TypeScript','x.ts','function combine(count: number) { type Local = typeof count; return count; }',/TypeOrReference/],
 ['ambiguous declaration','Rust','x.rs','fn combine(count: usize) -> usize { count } fn combine(count: u32) -> u32 { count }',/AmbiguousOrMissingTransformDeclaration/],
])test(name+' is an explicit whole-transform refusal',()=>{
 const item=fixture(source,language,owner);const before=structuredClone(item.document);
 assert.throws(()=>applySourceCstTransform(item.document,item.rule,item.frontend),reason);assert.deepEqual(item.document,before);
});
test('computed member references bind while static properties keep their independent meaning',()=>{
 const item=fixture('function combine(count, object) { return object[count] + object.count + object["count"]; }');
 assert.equal(sourceFromLinks(applySourceCstTransform(item.document,item.rule,item.frontend).document),'function combine(quantity, object) { return object[quantity] + object.count + object["count"]; }');
});
test('wrong full preimage, weakened contract, overlapping definitions and unresolved meanings refuse',()=>{
 const item=fixture('function combine(count) { return count + 1; }');
 const drift=fixture('// different comment\n'+item.source);
 assert.throws(()=>applySourceCstTransform(drift.document,item.rule,item.frontend),/CompletePreimageMismatch/);
 const weak=structuredClone(item.rule);weak[1]=weak[1].filter(node=>!Array.isArray(node)||node[0]!=='preconditions');
 assert.throws(()=>readSourceCstTransformRule(weak),/UnsupportedSourceTransformContract/);
 const overlap=structuredClone(item.rule);overlap.push(structuredClone(overlap[2]));assert.throws(()=>readSourceCstTransformRule(overlap),/SourceTransformRuleRequired/);
 const annotated=structuredClone(item.document);
 const annotate=node=>{if(Array.isArray(node)&&node[0]==='function')node.push(['semantic-gap',['quoted','unknown']]);else if(Array.isArray(node))for(const child of node)annotate(child);};annotate(annotated);
 const rule=createSourceCstTransformRule(annotated,{declaration:'combine',binding:'count',target:'quantity'});
 assert.throws(()=>applySourceCstTransform(annotated,rule,item.frontend),/UnresolvedTransformMeaning/);
});
for(const [language,owner,source,changed] of [
 ['JavaScript','x.mjs','function combine(count) { return count + 1; }','function combine(quantity) { sideEffect(); return quantity + 1; }'],
 ['TypeScript','x.ts','function combine(count: number): number { return count; }','function combine(quantity: string): number { return quantity; }'],
 ['Rust','x.rs','fn combine(count: usize) -> usize { count }','fn combine(quantity: u64) -> usize { quantity }'],
 ['JavaScript','x.mjs','// retained\nfunction combine(count) { return count + 1; }','// changed\nfunction combine(quantity) { return quantity + 1; }'],
])test(language+' independently grammar-valid type/effect/comment deformation is refused',()=>{
 const item=fixture(source,language,owner),candidate=ownedSourceLinks(owner,sourceToLinks(changed,language,item.frontend));
 assert.equal(verifySourceLinks(candidate,item.frontend),true);
 assert.throws(()=>verifySourceCstTransform(item.document,candidate,item.rule,item.frontend),/TypeOrEffectResidualMismatch/);
});
test('a target that changes concrete grammar is refused rather than normalized',()=>{
 const item=fixture('function combine(count) { return count; }');
 const rule=createSourceCstTransformRule(item.document,{declaration:'combine',binding:'count',target:'return'});
 assert.throws(()=>applySourceCstTransform(item.document,rule,item.frontend));
});
for(const [owner,language,declaration,binding,target] of [
 ['js/agentic/crate/protocol_responses.mjs','JavaScript','isCustomResponseTool','tools','advertisedTools'],
 ['ts/server/structured-log.mts','TypeScript','diagnosticBody','body','receivedBody'],
 ['rust/src/structured_log.rs','Rust','rank','level','severity'],
])test('held-out actual repository '+language+' function produces a complete reversible self-edit candidate',()=>{
 const original=fs.readFileSync(path.join(root,owner),'utf8'),frontend=runtime.forSource(owner,language);
 const document=ownedSourceLinks(owner,sourceToLinks(original,language,frontend));
 const rule=createSourceCstTransformRule(document,{declaration,binding,target});
 const result=applySourceCstTransform(readSourceLinks(writeSourceLinks(document)),readSourceCstTransformRule(writeSourceCstTransformRule(rule)).network,frontend);
 assert.notEqual(sourceFromLinks(result.document),original);assert.equal(verifySourceCstTransform(document,result.document,rule,frontend),true);
 const scratch=fs.mkdtempSync(path.join(os.tmpdir(),'cst-self-edit-'));
 try{const candidate=path.join(scratch,path.basename(owner));fs.writeFileSync(candidate,sourceFromLinks(result.document));assert.equal(verifySourceLinks(ownedSourceLinks(owner,sourceToLinks(fs.readFileSync(candidate,'utf8'),language,frontend)),frontend),true);}
 finally{fs.rmSync(scratch,{recursive:true,force:true});}
 assert.equal(sourceFromLinks(applySourceCstTransform(result.document,result.inverse,frontend).document),original);
 assert.equal(fs.readFileSync(path.join(root,owner),'utf8'),original,'candidate evidence does not authorize modifying its physical owner');
});
test('production CLI transforms a persisted model after original source deletion and persists its usable exact inverse',async()=>{
 const scratch=fs.mkdtempSync(path.join(os.tmpdir(),'cst-semantic-cli-'));
 try{
  const source='// preserved 😀\r\nfunction combine(count) { return count + 7; }\r\n';
  const original=path.join(scratch,'original.mjs'),input=path.join(scratch,'original.lino'),rulePath=path.join(scratch,'rule.lino'),output=path.join(scratch,'changed.lino'),inverse=path.join(scratch,'inverse.lino');
  fs.writeFileSync(original,source);const modelText=await runSourceModel(['--frontend','complete','--from','js','--input',original]);
  fs.writeFileSync(input,modelText);const document=readSourceLinks(modelText);
  fs.writeFileSync(rulePath,writeSourceCstTransformRule(createSourceCstTransformRule(document,{declaration:'combine',binding:'count',target:'quantity'})));fs.unlinkSync(original);
  const args=['--frontend','complete','--from','meta','--input',input,'--transform',rulePath];
  await assert.rejects(runSourceModel(args),/PersistedInverse/);
  const report=JSON.parse(await runSourceModel([...args,'--check']));assert.equal(report.transformation.exactInverse,true);assert.equal(report.semanticEquivalenceClaim,false);
  assert.equal(await runSourceModel([...args,'--output',output,'--inverse',inverse]),'');
  assert.equal(await runSourceModel(['--frontend','complete','--from','meta','--input',output]),source.replaceAll('count','quantity'));
  const restored=path.join(scratch,'restored.lino');await runSourceModel(['--frontend','complete','--from','meta','--input',output,'--transform',inverse,'--output',restored,'--inverse',path.join(scratch,'reverse-again.lino')]);
  assert.equal(await runSourceModel(['--frontend','complete','--from','meta','--input',restored]),source);
  const refusedInverse=path.join(scratch,'must-not-write.lino');await assert.rejects(runSourceModel([...args,'--output',output,'--inverse',refusedInverse]),/FreshSourceTransformOutputRequired/);assert.equal(fs.existsSync(refusedInverse),false);
 }finally{fs.rmSync(scratch,{recursive:true,force:true});}
});


test('Rust procedural attributes cannot obtain alpha-equivalence credit from syntax fidelity',()=>{
 const item=fixture('#[inspect_parameters]\n// retained doc\nfn combine(count: usize) -> usize { count }','Rust','attributes.rs');
 assert.throws(()=>applySourceCstTransform(item.document,item.rule,item.frontend),/AttributeRewriter/);
});
test('Rust label namespaces do not become parameter references by equal spelling',()=>{
 const item=fixture("fn combine(count: usize) -> usize { 'count: loop { break 'count count; } }",'Rust','labels.rs');
 assert.throws(()=>applySourceCstTransform(item.document,item.rule,item.frontend),/ScopeOrEffect/);
});
test('forged CST binding labels are rejected by independent grammar before matching a declaration',()=>{
 const item=fixture('function combine(count) { return count; }');
 const forged=structuredClone(item.document);
 const alter=node=>{if(!Array.isArray(node))return;if(node[0]==='role'&&node[1]?.quoted==='params')node[1].quoted='name';else for(const child of node)alter(child);};alter(forged);
 const rule=createSourceCstTransformRule(forged,{declaration:'combine',binding:'count',target:'quantity'});
 assert.throws(()=>applySourceCstTransform(forged,rule,item.frontend),/SourceStructureMismatch/);
});


test('unchanged prose mentioning a renamed binding becomes a source-linked unresolved need, never a meaning certificate',()=>{
 const item=fixture('// count must remain bounded\nfunction combine(count) { return count; }');
 const result=applySourceCstTransform(item.document,item.rule,item.frontend);
 assert.equal(result.verification.commentMeaning,'not-certified');
 assert.equal(result.verification.compilerValidity,'not-checked');
 assert.equal(result.verification.documentationSynchronization,'unresolved-comment-references');
 assert.equal(result.verification.commentNeeds.length,1);
 assert.equal(result.verification.commentNeeds[0].owner,item.owner);
 assert.equal(result.verification.commentNeeds[0].status,'unresolved');
 assert.ok(Array.isArray(result.verification.commentNeeds[0].cstPath));
 assert.equal(result.verification.commentNeeds[0].text,'// count must remain bounded');
 assert.ok(sourceFromLinks(result.document).startsWith('// count must remain bounded\n'));
});
