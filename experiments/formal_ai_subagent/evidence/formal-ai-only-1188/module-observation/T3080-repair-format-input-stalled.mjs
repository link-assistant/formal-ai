import fs from 'node:fs';import {execFileSync} from 'node:child_process';
const root='/Users/konard/Code/Archive/link-assistant/formal-ai',scratch='/private/tmp/formal-ai-only-1188',id='T3080',changes=[];
function modify(path,edit){const before=fs.readFileSync(root+'/'+path,'utf8');let expected=edit(before);if(path.endsWith('.rs'))expected=execFileSync('rustfmt',['--edition','2024','--config','skip_children=true','--emit','stdout'],{input:expected,encoding:'utf8'});if(expected===before)throw Error('no change '+path);changes.push({path,before,expected});}
function replace(source,old,next){if(!source.includes(old))throw Error('missing '+old.slice(0,100));return source.replace(old,next);}
modify('js/agentic/module_function/source_contract.mjs',source=>{
source=replace(source,'const text =','import { sha256Hex } from \'../crate/source_fetch.mjs\';\nconst text =');
const helper=String.raw`
/** Mirrors fn source_evidence: bytes, coordinates and identity of an observed region. */
export function sourceEvidence(source, start, end) {
  const bytes = new TextEncoder().encode(source), decode = (value) => new TextDecoder('utf-8', { ignoreBOM: true }).decode(value);
  const content = decode(bytes.subarray(start, end));
  return { source: content, contentId: sha256Hex(content), span: { byteStart: start, byteEnd: end,
    start: decode(bytes.subarray(0, start)).length, end: decode(bytes.subarray(0, end)).length } };
}
function unsafeAccessScope(trees) {
  return trees.some((tree) => ['=', '+=', '-=', '*=', '/=', '%=', '**=', '&&=', '||=', '??=', '&=', '|=', '^=', '<<=', '>>=', '>>>=', '++', '--', '=>', 'function', 'class', 'delete', 'yield', 'await'].includes(text(tree))
    || tree.$ === 'group' && (tree.delim === 'brace' || unsafeAccessScope(tree.trees)));
}
// These are unmet schema requirements, never proof that a property/getter is pure.
function structuralRequirements(parameters, body, source) {
  const result = [];
  if (text(body[0]) !== 'return' || unsafeAccessScope(body)) return result;
  const visit = (trees) => {
    for (let at = 0; at < trees.length; at += 1) {
      const tree = trees[at];
      if (parameters.includes(text(tree))) {
        const selectors = []; let cursor = at + 1;
        while (cursor < trees.length) {
          if (['.', '?.'].includes(text(trees[cursor])) && trees[cursor + 1]?.kind === 'identifier') {
            selectors.push({ property: text(trees[cursor + 1]) }); cursor += 2;
          } else if (trees[cursor].$ === 'group' && trees[cursor].delim === 'bracket') {
            const index = trees[cursor];
            selectors.push({ index: sourceEvidence(source, index.span.start + 1, index.span.end - 1).source }); cursor += 1;
          } else break;
        }
        if (selectors.length > 0) {
          result.push({ root: text(tree), selectors, status: 'unproved', ...sourceEvidence(source, tree.span.start, trees[cursor - 1].span.end) });
          for (const term of trees.slice(at + 1, cursor)) if (term.$ === 'group') visit(term.trees);
          at = cursor - 1; continue;
        }
      }
      if (tree.$ === 'group') visit(tree.trees);
    }
  };
  visit(body);
  return result;
}
`;
source=replace(source,'/** Mirrors `fn infer_return_contract`',helper+'\n/** Mirrors `fn infer_return_contract`');
return replace(source,"callEffects: 'unknown', preconditions: [], status: 'unknown', gap: null };","callEffects: 'unknown', preconditions: [], status: 'unknown', gap: null,\n    structuralRequirements: structuralRequirements(parameters, body, source) };");
});
modify('js/agentic/module_function/callable_catalog.mjs',source=>{
source=replace(source,"inferReturnContract, guardedCallGraph }","inferReturnContract, guardedCallGraph, sourceEvidence }");
source=replace(source,"declarations: [], exports: [], imports: [], gaps: [], moduleEffects: 'none'","declarations: [], exports: [], imports: [], initialization: [], gaps: [], moduleEffects: 'none'");
source=replace(source,"      // Only the grammar whose complete bodies were checked establishes a module effect boundary.\n      if (contract.status !== 'supported') catalog.moduleEffects = 'unknown';\n", "      // A deferred body has its own effect contract; declaring it does not execute it.\n");
source=replace(source,"      catalog.moduleEffects = 'unknown'; at = end + 1; continue;","      catalog.initialization.push({ kind: 'import', effects: 'unknown',\n        ...sourceEvidence(source, trees[start].span.start, trees[Math.min(end, trees.length - 1)].span.end) });\n      catalog.moduleEffects = 'unknown'; at = end + 1; continue;");
source=replace(source,"    at += 1; while (at < trees.length && text(trees[at]) !== ';') at += 1; at += 1;","    at += 1; while (at < trees.length && text(trees[at]) !== ';') at += 1;\n    catalog.initialization.push({ kind: 'unclassified', effects: 'unknown',\n      ...sourceEvidence(source, trees[start].span.start, trees[Math.min(at, trees.length - 1)].span.end) });\n    at += 1;");
return source;
});
modify('rust/src/agentic_coding/module_function/source_contract.rs',source=>{
const helper=String.raw`
/// Mirrors sourceEvidence: observed bytes and coordinate systems, not semantics.
pub(super) fn source_evidence(source: &str, start: usize, end: usize) -> Value {
    let content = &source[start..end];
    json!({"source":content,"contentId":crate::source_fetch::sha256_hex(content.as_bytes()),
        "span":{"byteStart":start,"byteEnd":end,"start":source[..start].encode_utf16().count(),"end":source[..end].encode_utf16().count()}})
}
fn unsafe_access_scope(trees: &[Value]) -> bool {
    trees.iter().any(|tree| {
        matches!(text(tree), "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "**=" | "&&=" | "||=" | "??=" | "&=" | "|=" | "^=" | "<<=" | ">>=" | ">>>=" | "++" | "--" | "=>" | "function" | "class" | "delete" | "yield" | "await")
            || (tree["$"] == "group" && (tree["delim"] == "brace" || unsafe_access_scope(tree["trees"].as_array().expect("token group"))))
    })
}
fn structural_requirements(parameters: &[String], body: &[Value], source: &str) -> Vec<Value> {
    fn visit(parameters: &[String], trees: &[Value], source: &str, result: &mut Vec<Value>) {
        let span = |tree: &Value, key: &str| usize::try_from(tree["span"][key].as_u64().expect("token span")).expect("host span");
        let mut at = 0;
        while at < trees.len() {
            let tree = &trees[at];
            if parameters.iter().any(|name| name == text(tree)) {
                let mut selectors = Vec::new();
                let mut cursor = at + 1;
                while cursor < trees.len() {
                    if matches!(text(&trees[cursor]), "." | "?.") && trees.get(cursor + 1).is_some_and(|tree| tree["kind"] == "identifier") {
                        selectors.push(json!({"property":text(&trees[cursor + 1])}));
                        cursor += 2;
                    } else if trees[cursor]["$"] == "group" && trees[cursor]["delim"] == "bracket" {
                        let index = &trees[cursor];
                        selectors.push(json!({"index":source_evidence(source,span(index,"start") + 1,span(index,"end") - 1)["source"]}));
                        cursor += 1;
                    } else { break; }
                }
                if !selectors.is_empty() {
                    let mut requirement = source_evidence(source,span(tree,"start"),span(&trees[cursor - 1],"end"));
                    requirement["root"] = json!(text(tree));
                    requirement["selectors"] = json!(selectors);
                    requirement["status"] = json!("unproved");
                    result.push(requirement);
                    for term in &trees[at + 1..cursor] {
                        if term["$"] == "group" { visit(parameters,term["trees"].as_array().expect("token group"),source,result); }
                    }
                    at = cursor; continue;
                }
            }
            if tree["$"] == "group" { visit(parameters,tree["trees"].as_array().expect("token group"),source,result); }
            at += 1;
        }
    }
    let mut result = Vec::new();
    if body.first().map_or("",text) == "return" && !unsafe_access_scope(body) { visit(parameters,body,source,&mut result); }
    result
}
`;
source=replace(source,'/// Mirrors inferReturnContract:',helper+'\n/// Mirrors inferReturnContract:');
return replace(source,'"result":null,"callEffects":"unknown","preconditions":[],"status":"unknown","gap":null});','"result":null,"callEffects":"unknown","preconditions":[],"status":"unknown","gap":null,\n        "structuralRequirements":structural_requirements(parameters,body,source)});');
});
modify('rust/src/agentic_coding/module_function/callable_catalog.rs',source=>{
source=replace(source,'guarded_call_graph, infer_return_contract, text','guarded_call_graph, infer_return_contract, source_evidence, text');
source=replace(source,'"declarations":[],"exports":[],"imports":[],\n            "gaps"','"declarations":[],"exports":[],"imports":[],"initialization":[],\n            "gaps"');
source=replace(source,'    let mut imports: Vec<Value> = Vec::new();','    let mut imports: Vec<Value> = Vec::new();\n    let mut initialization = Vec::new();');
source=replace(source,'            if contract["status"] != "supported" {\n                module_effects = "unknown";\n            }\n','            // Declaration does not execute its deferred body.\n');
source=replace(source,'            module_effects = "unknown";\n            at = end + 1;','            let mut observation = source_evidence(source, span_start(&trees[start]), span_end(&trees[end.min(trees.len() - 1)]));\n            observation["kind"] = json!("import");\n            observation["effects"] = json!("unknown");\n            initialization.push(observation);\n            module_effects = "unknown";\n            at = end + 1;');
source=replace(source,'        while at < trees.len() && text(&trees[at]) != ";" {\n            at += 1;\n        }\n        at += 1;','        while at < trees.len() && text(&trees[at]) != ";" {\n            at += 1;\n        }\n        let mut observation = source_evidence(source, span_start(&trees[start]), span_end(&trees[at.min(trees.len() - 1)]));\n        observation["kind"] = json!("unclassified");\n        observation["effects"] = json!("unknown");\n        initialization.push(observation);\n        at += 1;');
return replace(source,'"imports":imports,"gaps":gaps,"moduleEffects":module_effects','"imports":imports,"initialization":initialization,"gaps":gaps,"moduleEffects":module_effects');
});
const file=scratch+'/'+id+'-request.json';const code="const fs=require('fs');const changes=JSON.parse(fs.readFileSync('"+file+"','utf8')).changes;for(const change of changes){if(fs.readFileSync(change.path,'utf8')!==change.before)throw Error('preimage changed '+change.path);}for(const change of changes)fs.writeFileSync(change.path,change.expected);console.log(JSON.stringify({writes:changes.map(change=>change.path)}));";const prompt='Run node -e '+"'"+code.replaceAll("'","'\\''")+"'";fs.writeFileSync(file,JSON.stringify({id,dir:root,prompt,changes},null,2));
const {drive}=await import(root+'/experiments/js_dogfood/drive.mjs');const {WorkerHost}=await import(root+'/js/server/worker-host.mjs');const {installNodeHost}=await import(root+'/js/agentic/node-host.mjs');await installNodeHost(new WorkerHost());const {planChatStep}=await import(root+'/js/agentic/planner.mjs');const out=await drive(planChatStep,root,prompt,{steps:4});const raw=JSON.stringify(out,null,2);fs.writeFileSync(scratch+'/'+id+'-transcript.json',raw);fs.writeFileSync(scratch+'/'+id+'-transcript.log',raw);console.log(JSON.stringify({id,tools:out.transcript.map(row=>row.tool),exact:changes.every(change=>fs.readFileSync(root+'/'+change.path,'utf8')===change.expected)}));
