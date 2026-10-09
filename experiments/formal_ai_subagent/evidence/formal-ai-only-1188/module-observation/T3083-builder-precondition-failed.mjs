import fs from 'node:fs';import {execFileSync} from 'node:child_process';
const root='/Users/konard/Code/Archive/link-assistant/formal-ai',scratch='/private/tmp/formal-ai-only-1188',id='T3083',changes=[];
function replace(source,old,next){if(!source.includes(old))throw Error('missing '+old.slice(0,100));return source.replace(old,next);}
function modify(path,edit){const before=fs.readFileSync(root+'/'+path,'utf8');let expected=edit(before);if(path.endsWith('.rs')){const filename=scratch+'/'+id+'-'+path.split('/').at(-1);fs.writeFileSync(filename,expected);expected=execFileSync('rustfmt',['--edition','2024','--config','skip_children=true','--emit','stdout',filename],{encoding:'utf8'});const prefix=filename+':\n\n';if(expected.startsWith(prefix))expected=expected.slice(prefix.length);}changes.push({path,before,expected});}
modify('js/agentic/module_function/callable_catalog.mjs',source=>{
source=replace(source,"gaps: [], moduleEffects: 'none'","gaps: [], moduleEffects: 'none', moduleSyntax: 'supported'");
source=replace(source,"return { ...catalog, moduleEffects: 'unknown', gaps:","return { ...catalog, moduleEffects: 'unknown', moduleSyntax: 'unknown', gaps:");
source=replace(source,"      // A deferred body has its own effect contract; declaring it does not execute it.","      // Unsupported bodies lack a syntax proof even though they do not execute here.\n      if (contract.status !== 'supported') catalog.moduleSyntax = 'unknown';\n      // A deferred body has its own effect contract; declaring it does not execute it.");
source=replace(source,"  if (catalog.gaps.length > 0) catalog.moduleEffects = 'unknown';","  if (catalog.gaps.length > 0) { catalog.moduleEffects = 'unknown'; catalog.moduleSyntax = 'unknown'; }");
source=replace(source,"    if (catalog.gaps.length > 0) return { kind: 'gap', reason: 'ModuleContractGap' };","    if (catalog.gaps.length > 0) return { kind: 'gap', reason: 'ModuleContractGap' };\n    if (catalog.moduleSyntax !== 'supported') return { kind: 'gap', reason: 'ModuleSyntaxUnknown' };");
return replace(source,"    if (catalog.moduleEffects !== 'none' || catalog.gaps.length !== 0) continue;","    if (catalog.moduleEffects !== 'none' || catalog.moduleSyntax !== 'supported' || catalog.gaps.length !== 0) continue;");
});
modify('rust/src/agentic_coding/module_function/callable_catalog.rs',source=>{
source=replace(source,'"moduleEffects":"unknown"});','"moduleEffects":"unknown","moduleSyntax":"unknown"});');
source=replace(source,'    let mut module_effects = "none";','    let mut module_effects = "none";\n    let mut module_syntax = "supported";');
source=replace(source,'            // Declaration does not execute its deferred body.','            if contract["status"] != "supported" {\n                module_syntax = "unknown";\n            }\n            // Declaration does not execute its deferred body.');
source=replace(source,'    if !gaps.is_empty() {\n        module_effects = "unknown";\n    }','    if !gaps.is_empty() {\n        module_effects = "unknown";\n        module_syntax = "unknown";\n    }');
source=replace(source,'"moduleEffects":module_effects})','"moduleEffects":module_effects,"moduleSyntax":module_syntax})');
const old='    if catalog["gaps"].as_array().is_some_and(|gaps| !gaps.is_empty()) {\n        return json!({"kind":"gap","reason":"ModuleContractGap"});\n    }';
source=replace(source,old,old+'\n    if catalog["moduleSyntax"] != "supported" {\n        return json!({"kind":"gap","reason":"ModuleSyntaxUnknown"});\n    }');
source=replace(source,'        if catalog["moduleEffects"] != "none"','        if catalog["moduleEffects"] != "none"\n            || catalog["moduleSyntax"] != "supported"');
return source;
});
modify('rust/tests/web/observed-callable-contracts.test.mjs',source=>{
source=replace(source,"assert.equal(observedGuardedGraph(data, first, { path: 'consumer.mjs', exported: 'f' }).reason, 'MissingContract');","assert.equal(observedGuardedGraph(data, first, { path: 'consumer.mjs', exported: 'f' }).reason, 'ModuleSyntaxUnknown');");
source=replace(source,"  assert.equal(observedGuardedGraph(data, { ...first, contentId: undefined }, second).kind, 'guarded-call-graph');","  assert.equal(catalog.moduleSyntax, 'unknown');\n  assert.equal(observedGuardedGraph(data, { ...first, contentId: undefined }, second).reason, 'ModuleSyntaxUnknown');");
return source+String.raw`

test('an unvalidated balanced body cannot certify an otherwise supported module import', () => {
  const source = firstSource + ' function broken(value) { return value++++; }';
  const catalog = observe(source);
  assert.equal(catalog.moduleEffects, 'none');
  assert.equal(catalog.moduleSyntax, 'unknown');
  const data = observations(); data[0].content = source;
  assert.equal(observedGuardedGraph(data, { ...first, contentId: undefined }, second).reason, 'ModuleSyntaxUnknown');
});
`;
});
modify('rust/tests/fixtures/observed-callable-contracts.rs',source=>{
// Only the new deferred-body assertion changes from an unsafe success to a syntax obligation.
source=replace(source,'        "guarded-call-graph"\n    );\n}\n\n#[test]\nfn initialization_regions_keep_exact_unicode_byte_witnesses', '        "gap"\n    );\n    assert_eq!(catalog["moduleSyntax"], "unknown");\n}\n\n#[test]\nfn initialization_regions_keep_exact_unicode_byte_witnesses');
// Existing unknown-body refusal remains a refusal, with a more precise prerequisite reason.
source=source.replaceAll('"MissingContract"\n        );','"ModuleSyntaxUnknown"\n        );');
return source+String.raw`

#[test]
fn balanced_unvalidated_body_cannot_certify_module_import() {
    let source = format!("{FIRST} function broken(value) {{ return value++++; }}");
    let catalog = observe_source_callables(&source,"producer.mjs");
    assert_eq!(catalog["moduleEffects"],"none");
    assert_eq!(catalog["moduleSyntax"],"unknown");
    let data = vec![json!({"path":"producer.mjs","content":source}),json!({"path":"consumer.mjs","content":SECOND})];
    assert_eq!(observed_guarded_graph(&data,&binding("producer.mjs","publicChoice",&source),&binding("consumer.mjs","render",SECOND))["reason"],"ModuleSyntaxUnknown");
}
`;
});
const file=scratch+'/'+id+'-request.json';const code="const fs=require('fs');const changes=JSON.parse(fs.readFileSync('"+file+"','utf8')).changes;for(const change of changes){if(fs.readFileSync(change.path,'utf8')!==change.before)throw Error('preimage changed '+change.path);}for(const change of changes)fs.writeFileSync(change.path,change.expected);console.log(JSON.stringify({writes:changes.map(change=>change.path)}));";const prompt='Run node -e '+"'"+code.replaceAll("'","'\\''")+"'";fs.writeFileSync(file,JSON.stringify({id,dir:root,prompt,changes},null,2));
const {drive}=await import(root+'/experiments/js_dogfood/drive.mjs');const {WorkerHost}=await import(root+'/js/server/worker-host.mjs');const {installNodeHost}=await import(root+'/js/agentic/node-host.mjs');await installNodeHost(new WorkerHost());const {planChatStep}=await import(root+'/js/agentic/planner.mjs');const out=await drive(planChatStep,root,prompt,{steps:4});const raw=JSON.stringify(out,null,2);fs.writeFileSync(scratch+'/'+id+'-transcript.json',raw);fs.writeFileSync(scratch+'/'+id+'-transcript.log',raw);console.log(JSON.stringify({id,tools:out.transcript.map(row=>row.tool),exact:changes.every(change=>fs.readFileSync(root+'/'+change.path,'utf8')===change.expected)}));
