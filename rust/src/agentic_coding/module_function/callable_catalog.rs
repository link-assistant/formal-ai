//! Complete token-tree declaration observations and conservative contract graphs.
use super::conditional_schema::derive_conditional_schema_graph;
use super::source_contract::{guarded_call_graph, infer_return_contract, source_evidence, text};
use crate::es_tokenizer::{Delimiter, TemplatePart, TokenKind, Tree};
use serde_json::{Value, json};

fn tree_value(tree: &Tree<'_>) -> Value {
    let span = tree.span();
    let range = json!({"start":span.start,"end":span.end});
    match tree {
        Tree::Leaf(token) => {
            let kind = match token.kind {
                TokenKind::Identifier => "identifier",
                TokenKind::Numeric => "numeric",
                TokenKind::String => "string",
                TokenKind::TemplateChunk => "chunk",
                TokenKind::RegExp => "regexp",
                TokenKind::Punctuator => "punctuator",
            };
            json!({"$":"leaf","kind":kind,"text":token.text,"span":range})
        }
        Tree::Group { delim, trees, .. } => {
            let delim = match delim {
                Delimiter::Paren => "paren",
                Delimiter::Bracket => "bracket",
                Delimiter::Brace => "brace",
            };
            json!({"$":"group","delim":delim,"trees":trees.iter().map(tree_value).collect::<Vec<_>>(),"span":range})
        }
        Tree::Template { parts, .. } => {
            let parts: Vec<_> = parts.iter().map(|part| match part {
                TemplatePart::Chunk(token) => json!({"$":"chunk","text":token.text,"span":{"start":token.span.start,"end":token.span.end}}),
                TemplatePart::Interpolation { span, trees } => json!({"$":"interp","trees":trees.iter().map(tree_value).collect::<Vec<_>>(),"span":{"start":span.start,"end":span.end}}),
            }).collect();
            json!({"$":"template","parts":parts,"span":range})
        }
    }
}
fn group(tree: &Value, delim: &str) -> bool {
    tree["$"] == "group" && tree["delim"] == delim
}
fn identifier(tree: &Value) -> bool {
    tree["kind"] == "identifier"
        && !crate::es_tokenizer::is_keyword(text(tree))
        && !matches!(
            text(tree),
            "yield"
                | "implements"
                | "interface"
                | "package"
                | "private"
                | "protected"
                | "public"
                | "eval"
                | "arguments"
        )
}
fn safe_binding(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$'))
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
}
fn names(trees: &[Value]) -> Option<Vec<String>> {
    let mut result = Vec::new();
    for at in (0..trees.len()).step_by(2) {
        if !identifier(&trees[at])
            || !safe_binding(text(&trees[at]))
            || (at + 1 < trees.len() && text(&trees[at + 1]) != ",")
        {
            return None;
        }
        result.push(text(&trees[at]).to_owned());
    }
    let unique: std::collections::BTreeSet<_> = result.iter().collect();
    (unique.len() == result.len()).then_some(result)
}
fn aliases(trees: &[Value]) -> Option<Vec<Value>> {
    let mut result = Vec::new();
    let mut at = 0;
    while at < trees.len() {
        if !identifier(&trees[at]) {
            return None;
        }
        let local = text(&trees[at]);
        at += 1;
        let mut exposed = local;
        if trees.get(at).is_some_and(|tree| text(tree) == "as") {
            if !trees.get(at + 1).is_some_and(identifier) {
                return None;
            }
            exposed = text(&trees[at + 1]);
            at += 2;
        }
        result.push(json!({"local":local,"exposed":exposed}));
        if at < trees.len() {
            if text(&trees[at]) != "," {
                return None;
            }
            at += 1;
        }
    }
    Some(result)
}
fn dependency(path: &str, specifier: &str) -> Option<String> {
    if !specifier.starts_with('.') || specifier.contains('\\') {
        return None;
    }
    let mut parts: Vec<_> = path.split('/').collect();
    parts.pop();
    for part in specifier.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            _ => parts.push(part),
        }
    }
    Some(parts.join("/"))
}
fn unknown(reason: &str) -> Value {
    json!({"inputs":null,"result":null,"callEffects":"unknown","preconditions":[],"status":"unknown","gap":{"reason":reason}})
}
fn span_start(tree: &Value) -> usize {
    usize::try_from(tree["span"]["start"].as_u64().expect("token span")).expect("host span")
}
fn span_end(tree: &Value) -> usize {
    usize::try_from(tree["span"]["end"].as_u64().expect("token span")).expect("host span")
}

/// Mirrors observeSourceCallables: full source-derived declarations, including exact UTF-8 spans.
#[must_use]
pub fn observe_source_callables(source: &str, path: &str) -> Value {
    let content_identifier = crate::source_fetch::sha256_hex(source.as_bytes());
    let raw = match crate::es_tokenizer::tokenize(source) {
        Ok(trees) => trees,
        Err(error) => {
            return json!({"path":path,"contentId":content_identifier,"bytes":source.len(),"declarations":[],"exports":[],"imports":[],"initialization":[],
            "gaps":[{"reason":"LexicalFailure","start":error.span.start}],"moduleEffects":"unknown","moduleSyntax":"unknown"});
        }
    };
    let trees: Vec<_> = raw.iter().map(tree_value).collect();
    let mut declarations = Vec::new();
    let mut exports = Vec::new();
    let mut imports: Vec<Value> = Vec::new();
    let mut initialization = Vec::new();
    let mut gaps = Vec::new();
    let mut module_effects = "none";
    let mut module_syntax = "supported";
    let mut at = 0;
    while at < trees.len() {
        let start = at;
        let mut cursor = at;
        let exported = text(&trees[cursor]) == "export";
        if exported {
            cursor += 1;
        }
        let default_export = trees
            .get(cursor)
            .is_some_and(|tree| text(tree) == "default");
        if default_export {
            cursor += 1;
        }
        let asynchronous = trees.get(cursor).is_some_and(|tree| text(tree) == "async");
        if asynchronous {
            cursor += 1;
        }
        if trees
            .get(cursor)
            .is_some_and(|tree| text(tree) == "function")
            && trees.get(cursor + 1).is_some_and(identifier)
            && trees
                .get(cursor + 2)
                .is_some_and(|tree| group(tree, "paren"))
            && trees
                .get(cursor + 3)
                .is_some_and(|tree| group(tree, "brace"))
        {
            let name = text(&trees[cursor + 1]);
            let parameters = names(
                trees[cursor + 2]["trees"]
                    .as_array()
                    .expect("parameter group"),
            );
            let byte_start = span_start(&trees[start]);
            let byte_end = span_end(&trees[cursor + 3]);
            let span = json!({"byteStart":byte_start,"byteEnd":byte_end,"start":source[..byte_start].encode_utf16().count(),"end":source[..byte_end].encode_utf16().count()});
            let body = &source[byte_start..byte_end];
            let contract = if safe_binding(name) {
                parameters.as_deref().map_or_else(
                    || unknown("UnsupportedParameters"),
                    |parameters| {
                        if asynchronous {
                            unknown("AsyncResult")
                        } else {
                            infer_return_contract(
                                parameters,
                                trees[cursor + 3]["trees"].as_array().expect("body group"),
                                source,
                            )
                        }
                    },
                )
            } else {
                unknown("UnsupportedBinding")
            };
            if !safe_binding(name) || parameters.is_none() {
                module_effects = "unknown";
            }
            if contract["status"] != "supported" {
                module_syntax = "unknown";
            }
            // Declaration does not execute its deferred body.
            declarations.push(json!({"name":name,"parameters":parameters,"source":body,"span":span,"contract":contract,
                "identity":{"path":path,"moduleContentId":content_identifier,"declarationContentId":crate::source_fetch::sha256_hex(body.as_bytes()),"span":span}}));
            if exported {
                exports.push(
                    json!({"local":name,"exposed":if default_export {"default"} else {name}}),
                );
            }
            at = cursor + 4;
            if trees.get(at).is_some_and(|tree| text(tree) == ";") {
                at += 1;
            }
            continue;
        }
        if exported
            && trees.get(cursor).is_some_and(|tree| group(tree, "brace"))
            && trees
                .get(cursor + 1)
                .is_none_or(|tree| text(tree) != "from")
        {
            if let Some(named) = aliases(trees[cursor]["trees"].as_array().expect("export group")) {
                exports.extend(named);
            } else {
                gaps.push(json!({"reason":"UnsupportedExport","start":span_start(&trees[start])}));
            }
            at = cursor + 1;
            if trees.get(at).is_some_and(|tree| text(tree) == ";") {
                at += 1;
            }
            continue;
        }
        if text(&trees[start]) == "import" {
            let mut end = start + 1;
            while end < trees.len() && text(&trees[end]) != ";" {
                end += 1;
            }
            let terms = &trees[start + 1..end];
            let mut bindings = None;
            let mut literal = None;
            if let [item] = terms
                && item["kind"] == "string"
            {
                literal = Some(text(item));
                bindings = Some(Vec::new());
            }
            if let [named, from, item] = terms
                && group(named, "brace")
                && text(from) == "from"
                && item["kind"] == "string"
                && let Some(named) = aliases(named["trees"].as_array().expect("import group"))
            {
                bindings = Some(
                    named
                        .into_iter()
                        .map(|entry| json!({"imported":entry["local"],"local":entry["exposed"]}))
                        .collect(),
                );
                literal = Some(text(item));
            }
            let specifier = literal
                .filter(|literal| !literal.contains('\\'))
                .map(|literal| &literal[1..literal.len() - 1]);
            if let (Some(bindings), Some(specifier)) = (bindings, specifier) {
                imports.push(json!({"specifier":specifier,"path":dependency(path,specifier),"bindings":bindings}));
            } else {
                gaps.push(json!({"reason":"UnsupportedImport","start":span_start(&trees[start])}));
            }
            let mut observation = source_evidence(
                source,
                span_start(&trees[start]),
                span_end(&trees[end.min(trees.len() - 1)]),
            );
            observation["kind"] = json!("import");
            observation["effects"] = json!("unknown");
            initialization.push(observation);
            module_effects = "unknown";
            at = end + 1;
            continue;
        }
        module_effects = "unknown";
        gaps.push(json!({"reason":"UnprovedModuleStatement","start":span_start(&trees[start])}));
        at += 1;
        while at < trees.len() && text(&trees[at]) != ";" {
            at += 1;
        }
        let mut observation = source_evidence(
            source,
            span_start(&trees[start]),
            span_end(&trees[at.min(trees.len() - 1)]),
        );
        observation["kind"] = json!("unclassified");
        observation["effects"] = json!("unknown");
        initialization.push(observation);
        at += 1;
    }
    let mut locals: Vec<_> = declarations
        .iter()
        .map(|entry| entry["name"].clone())
        .collect();
    for imported in &imports {
        locals.extend(
            imported["bindings"]
                .as_array()
                .expect("import bindings")
                .iter()
                .map(|entry| entry["local"].clone()),
        );
    }
    let unique: std::collections::BTreeSet<_> = locals.iter().map(Value::to_string).collect();
    if unique.len() != locals.len() {
        gaps.push(json!({"reason":"DuplicateBinding","start":null}));
    }
    let unique: std::collections::BTreeSet<_> = exports
        .iter()
        .map(|entry| entry["exposed"].to_string())
        .collect();
    if unique.len() != exports.len() {
        gaps.push(json!({"reason":"DuplicateExport","start":null}));
    }
    for exported in &exports {
        if !locals.iter().any(|local| local == &exported["local"]) {
            gaps.push(json!({"reason":"ExportDeclarationUnobserved","start":null}));
        }
    }
    if !gaps.is_empty() {
        module_effects = "unknown";
        module_syntax = "unknown";
    }
    json!({"path":path,"contentId":content_identifier,"bytes":source.len(),"declarations":declarations,"exports":exports,"imports":imports,"initialization":initialization,"gaps":gaps,"moduleEffects":module_effects,"moduleSyntax":module_syntax})
}
fn resolve_entry(
    catalogs: &[Value],
    binding: &Value,
    seen: &mut std::collections::BTreeSet<String>,
) -> Value {
    let key = format!("{}:{}", binding["path"], binding["exported"]);
    if !seen.insert(key) {
        return json!({"kind":"gap","reason":"BindingCycle"});
    }
    let Some(catalog) = catalogs
        .iter()
        .find(|entry| entry["path"] == binding["path"])
    else {
        return json!({"kind":"gap","reason":"ModuleUnobserved"});
    };
    if binding
        .get("contentId")
        .is_some_and(|id| id != &catalog["contentId"])
    {
        return json!({"kind":"gap","reason":"SourceChanged"});
    }
    if !catalog["gaps"].as_array().expect("catalog gaps").is_empty() {
        return json!({"kind":"gap","reason":"ModuleContractGap"});
    }
    if catalog["moduleSyntax"] != "supported" {
        return json!({"kind":"gap","reason":"ModuleSyntaxUnknown"});
    }
    let names: Vec<_> = catalog["exports"]
        .as_array()
        .expect("catalog exports")
        .iter()
        .filter(|entry| entry["exposed"] == binding["exported"])
        .collect();
    if names.len() != 1 {
        return json!({"kind":"gap","reason":"ExportUnresolved"});
    }
    let import_binding = json!({"path":binding["path"],"exported":binding["exported"]});
    if let Some(declaration) = catalog["declarations"]
        .as_array()
        .expect("catalog declarations")
        .iter()
        .find(|entry| entry["name"] == names[0]["local"])
    {
        let mut declaration = declaration.clone();
        declaration["moduleEffects"] = catalog["moduleEffects"].clone();
        declaration["binding"] = import_binding;
        return declaration;
    }
    for imported in catalog["imports"].as_array().expect("catalog imports") {
        if let Some(item) = imported["bindings"]
            .as_array()
            .expect("import bindings")
            .iter()
            .find(|entry| entry["local"] == names[0]["local"])
        {
            if imported["path"].is_null() {
                return json!({"kind":"gap","reason":"DeclarationUnobserved"});
            }
            let next = json!({"path":imported["path"],"exported":item["imported"]});
            let mut target = resolve_entry(catalogs, &next, seen);
            if target["kind"] == "gap" {
                return target;
            }
            target["moduleEffects"] = json!("unknown");
            target["binding"] = import_binding;
            return target;
        }
    }
    json!({"kind":"gap","reason":"DeclarationUnobserved"})
}

/// Mirrors observedGuardedGraph: rebuild identities/contracts from the actual source observations.
#[must_use]
pub fn observed_guarded_graph(observations: &[Value], first: &Value, second: &Value) -> Value {
    let catalogs: Vec<_> = observations
        .iter()
        .map(|item| {
            observe_source_callables(
                item["content"].as_str().expect("source content"),
                item["path"].as_str().expect("source path"),
            )
        })
        .collect();
    let producer = resolve_entry(&catalogs, first, &mut std::collections::BTreeSet::new());
    let consumer = resolve_entry(&catalogs, second, &mut std::collections::BTreeSet::new());
    if producer["kind"] == "gap" {
        return producer;
    }
    if consumer["kind"] == "gap" {
        return consumer;
    }
    guarded_call_graph(&producer, &consumer)
}

/// Mirrors observedCallableGraphs: a supported call graph is not a request goal proof.
#[must_use]
pub fn observed_callable_graphs(observations: &[Value]) -> Vec<Value> {
    let mut bindings = Vec::new();
    for observation in observations {
        let catalog = observe_source_callables(
            observation["content"].as_str().expect("source content"),
            observation["path"].as_str().expect("source path"),
        );
        if catalog["moduleEffects"] != "none"
            || catalog["moduleSyntax"] != "supported"
            || !catalog["gaps"].as_array().expect("catalog gaps").is_empty()
        {
            continue;
        }
        for exported in catalog["exports"].as_array().expect("catalog exports") {
            let declaration = catalog["declarations"]
                .as_array()
                .expect("catalog declarations")
                .iter()
                .find(|entry| entry["name"] == exported["local"]);
            let Some(declaration) = declaration else {
                continue;
            };
            if declaration["contract"]["status"] != "supported"
                || declaration["parameters"].as_array().map_or(0, Vec::len) != 1
            {
                continue;
            }
            bindings.push(json!({"path":catalog["path"],"exported":exported["exposed"],"contentId":catalog["contentId"],"optional":declaration["contract"]["result"]["kind"] == "optional"}));
        }
    }
    let mut graphs = Vec::new();
    for first in bindings
        .iter()
        .filter(|binding| binding["optional"] == true)
    {
        for second in &bindings {
            let graph = observed_guarded_graph(observations, first, second);
            if graph["kind"] == "guarded-call-graph" {
                graphs.push(graph);
            }
        }
    }
    graphs
}

/// Mirrors observedConditionalGraphs; unknown module/init effects never become purity.
#[must_use]
pub fn observed_conditional_graphs(observations: &[Value]) -> Vec<Value> {
    let mut entries = Vec::new();
    for observation in observations {
        let catalog = observe_source_callables(
            observation["content"].as_str().unwrap_or(""),
            observation["path"].as_str().unwrap_or(""),
        );
        if catalog["gaps"]
            .as_array()
            .expect("catalog gaps")
            .iter()
            .any(|gap| {
                matches!(
                    gap["reason"].as_str(),
                    Some(
                        "LexicalFailure"
                            | "DuplicateBinding"
                            | "DuplicateExport"
                            | "ExportDeclarationUnobserved"
                    )
                )
            })
        {
            continue;
        }
        for exported in catalog["exports"].as_array().expect("exports") {
            let Some(declaration) = catalog["declarations"]
                .as_array()
                .expect("declarations")
                .iter()
                .find(|entry| entry["name"] == exported["local"])
            else {
                continue;
            };
            if declaration["contract"]["conditionalIR"]["status"] != "parsed" {
                continue;
            }
            let mut entry = declaration.clone();
            entry["moduleEffects"] = catalog["moduleEffects"].clone();
            entry["moduleSyntax"] = catalog["moduleSyntax"].clone();
            entry["binding"] = json!({"path":catalog["path"],"exported":exported["exposed"]});
            entries.push(entry);
        }
    }
    let mut graphs = Vec::new();
    for first in &entries {
        for second in &entries {
            if first["identity"]["declarationContentId"]
                == second["identity"]["declarationContentId"]
                && first["identity"]["path"] == second["identity"]["path"]
            {
                continue;
            }
            let mut graph = derive_conditional_schema_graph(first, second);
            if graph["kind"] == "conditional-schema-graph" {
                graph["moduleSyntax"] = json!([first["moduleSyntax"], second["moduleSyntax"]]);
                graphs.push(graph);
            }
        }
    }
    graphs
}
