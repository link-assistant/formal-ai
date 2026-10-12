//! Seeded lowering of a source graph, never copied source implementations.
use super::callable_catalog::observe_source_callables;
use super::complete_source_preflight::source_goal_scope;
use super::seeded_composition_hook::qualify_seeded_composition;
use super::source_composition_effects::{checked_composition_source, source_null_absence};
use super::source_module_identity_port::SourceModuleIdentityHost;
use crate::seed::parser::parse_lino;
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};

fn safe(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || matches!(character, '_' | '$'))
        && characters
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '$'))
        && !crate::es_tokenizer::is_keyword(name)
        && ![
            "eval",
            "arguments",
            "yield",
            "implements",
            "interface",
            "package",
            "private",
            "protected",
            "public",
            "static",
        ]
        .contains(&name)
}
fn fill(text: &str, slots: &[(&str, String)]) -> Result<String, &'static str> {
    let names = regex::Regex::new(r"\{([a-z]+(?:-[a-z]+)*)\}").map_err(|_| "InvalidSlotGrammar")?;
    if names
        .captures_iter(text)
        .any(|capture| !slots.iter().any(|(name, _)| *name == &capture[1]))
    {
        return Err("UnknownTemplateSlot");
    }
    Ok(slots.iter().fold(text.to_owned(), |text, (name, value)| {
        text.replace(&format!("{{{name}}}"), value)
    }))
}
fn observed_declaration(observations: &[Value], binding: &Value) -> Result<Value, &'static str> {
    let source = observations
        .iter()
        .find(|source| source["path"] == binding["path"])
        .ok_or("UnknownSourceModule")?;
    let catalog = observe_source_callables(
        source["content"].as_str().ok_or("MissingSourceBytes")?,
        source["path"].as_str().ok_or("MissingSourcePath")?,
    );
    let exported = catalog["exports"]
        .as_array()
        .ok_or("MissingSourceExports")?
        .iter()
        .find(|value| value["exposed"] == binding["exported"])
        .ok_or("UnknownSourceExport")?;
    catalog["declarations"]
        .as_array()
        .ok_or("MissingSourceDeclarations")?
        .iter()
        .find(|value| value["name"] == exported["local"])
        .cloned()
        .ok_or("UnobservedDeclaration")
}
pub fn derive_seeded_source_plan<Host: SourceModuleIdentityHost>(
    host: &Host,
    request: &Value,
    observations: &[Value],
    acceptance: &Value,
    operation: &Value,
    seed: &str,
) -> Result<Value, &'static str> {
    let name = request["name"].as_str().ok_or("UnsupportedDeclaration")?;
    let parameters = request["parameters"]
        .as_array()
        .filter(|values| values.len() == 1)
        .ok_or("UnsupportedDeclaration")?;
    let parameter = parameters[0].as_str().ok_or("UnsupportedDeclaration")?;
    if !safe(name) || !safe(parameter) {
        return Err("UnsupportedDeclaration");
    }
    let mut candidate =
        qualify_seeded_composition(host, request, observations, acceptance, operation, seed)?;
    let parsed = parse_lino(seed);
    let root = parsed
        .children
        .iter()
        .find(|node| node.name == "source-callable-composition")
        .ok_or("MissingCompositionSeed")?;
    let goal = root
        .children
        .iter()
        .find(|node| node.name == "goal" && candidate["goal"] == node.id)
        .ok_or("MissingSeededGoal")?;
    let bindings = candidate["graph"]["bindings"]
        .as_array()
        .filter(|values| values.len() == 2)
        .ok_or("UnsupportedGraphBindings")?;
    let consumer = observed_declaration(observations, &bindings[1])?;
    let consumer_parameters = consumer["parameters"]
        .as_array()
        .filter(|values| values.len() == 1)
        .ok_or("UnknownConsumerArity")?;
    let consumer_parameter = consumer_parameters[0]
        .as_str()
        .ok_or("UnsupportedConsumerParameter")?;
    if !safe(consumer_parameter) {
        return Err("UnsupportedConsumerParameter");
    }
    let lexemes: Vec<_> = goal
        .children
        .iter()
        .filter(|node| node.name == "lexeme" && node.id == "en")
        .flat_map(|node| {
            node.children
                .iter()
                .filter(|surface| surface.name == "surface")
        })
        .flat_map(|surface| surface.children.iter().filter(|value| value.name == "text"))
        .map(|value| regex::escape(&value.id))
        .collect();
    let styles: Vec<_> = goal
        .children
        .iter()
        .filter(|node| node.name == "style")
        .map(|node| regex::escape(node.find_child_value("surface")))
        .collect();
    let slots = [
        ("style", format!("(?:{})?", styles.join("|"))),
        ("goal", format!("(?:{})", lexemes.join("|"))),
        ("consumer-parameter", regex::escape(consumer_parameter)),
        ("wrapper-parameter", regex::escape(parameter)),
    ];
    let scope = source_goal_scope(request["text"].as_str().ok_or("MissingRequestBytes")?)
        .ok_or("UnprovedQuotationScope")?;
    let sentences = crate::agentic_coding::shell_command_policy::sentences(&scope);
    let mut matches = Vec::new();
    for form in goal.children.iter().filter(|node| node.name == "form") {
        let mut qualified_pattern = form.find_child_value("pattern").to_owned();
        let mut technical_captures = Vec::new();
        loop {
            let next = [
                ("consumer-parameter", consumer_parameter),
                ("wrapper-parameter", parameter),
            ]
            .into_iter()
            .filter_map(|(role, expected)| {
                let marker = format!("{{{role}}}");
                qualified_pattern
                    .find(&marker)
                    .map(|start| (start, marker, expected))
            })
            .min_by_key(|(start, _, _)| *start);
            let Some((start, marker, expected)) = next else {
                break;
            };
            let capture_name = format!("sourceParameter{}", technical_captures.len());
            qualified_pattern.replace_range(
                start..start + marker.len(),
                &format!("(?<{capture_name}>{})", regex::escape(expected)),
            );
            technical_captures.push((capture_name, expected));
        }
        let pattern = fill(&qualified_pattern, &slots)?;
        let expression = regex::RegexBuilder::new(&pattern)
            .case_insensitive(true)
            .build()
            .map_err(|_| "InvalidReturnForm")?;
        for sentence in &sentences {
            if expression.captures(sentence.text).is_some_and(|captures| {
                technical_captures.iter().all(|(name, expected)| {
                    captures
                        .name(name)
                        .is_some_and(|matched| matched.as_str() == *expected)
                })
            }) {
                matches.push(json!({"form":form.id,"sentence":sentence.text}));
            }
        }
    }
    if matches.len() != 1 {
        return Err("UnprovedCompleteReturnForm");
    }
    let mut locals = std::collections::BTreeSet::from([name.to_owned(), parameter.to_owned()]);
    let mut fresh = |base: &str| {
        let mut name = base.to_owned();
        let mut index = 0;
        while locals.contains(&name) {
            index += 1;
            name = format!("{base}{index}");
        }
        locals.insert(name.clone());
        name
    };
    let producer_local = fresh("_sourceProjection");
    let consumer_local = fresh("_sourceRendering");
    let value_local = fresh("_sourceValue");
    let templates: Vec<_> = goal
        .children
        .iter()
        .filter(|node| node.name == "template")
        .collect();
    if templates.len() != 1 || templates[0].find_child_value("text").is_empty() {
        return Err("AmbiguousSourceTemplate");
    }
    let imports = candidate["imports"]
        .as_array()
        .filter(|values| values.len() == 2)
        .ok_or("UnsupportedImportBindings")?;
    if !imports
        .iter()
        .all(|value| value["exported"].as_str().is_some_and(safe))
    {
        return Err("UnsupportedImportBinding");
    }
    let producer = observed_declaration(observations, &bindings[0])?;
    if !source_null_absence(&producer["contract"]["conditionalIR"]) {
        return Err("UndefinedAbsenceIsNotNull");
    }
    let source = fill(
        templates[0].find_child_value("text"),
        &[
            (
                "producer-export",
                imports[0]["exported"]
                    .as_str()
                    .ok_or("MissingExport")?
                    .to_owned(),
            ),
            ("producer-local", producer_local),
            (
                "producer-url",
                serde_json::to_string(&imports[0]["moduleURL"])
                    .map_err(|_| "UnencodableModuleURL")?,
            ),
            (
                "consumer-export",
                imports[1]["exported"]
                    .as_str()
                    .ok_or("MissingExport")?
                    .to_owned(),
            ),
            ("consumer-local", consumer_local),
            (
                "consumer-url",
                serde_json::to_string(&imports[1]["moduleURL"])
                    .map_err(|_| "UnencodableModuleURL")?,
            ),
            ("name", name.to_owned()),
            ("parameter", parameter.to_owned()),
            ("value-local", value_local),
        ],
    )?;
    let effects = checked_composition_source(&source, request, imports)?;
    let fields = candidate.as_object_mut().ok_or("MissingCandidateObject")?;
    let observed_request = super::discovery::observed_callable_request(
        request["text"].as_str().ok_or("MissingRequestBytes")?,
    )
    .ok_or("UnboundCallableRequest")?;
    let user_message =
        crate::protocol::ChatMessage::user(request["text"].as_str().ok_or("MissingRequestBytes")?);
    fields.insert(
        "goalLedger".to_owned(),
        super::discovery::observed_callable_goal_ledger(&observed_request, &[user_message]),
    );
    fields.insert("effectPlan".to_owned(), effects);
    fields.insert("kind".to_owned(), json!("seeded-source-composition-plan"));
    fields.insert(
        "sourceIdentity".to_owned(),
        json!(sha256_hex(source.as_bytes())),
    );
    fields.insert("source".to_owned(), json!(source));
    fields.insert(
        "seedIdentity".to_owned(),
        json!(sha256_hex(seed.as_bytes())),
    );
    fields.insert(
        "goalClauseCoverage".to_owned(),
        json!("complete-seeded-return-form"),
    );
    fields.insert(
        "returnForm".to_owned(),
        matches.pop().ok_or("MissingReturnForm")?,
    );
    fields.insert(
        "effectAuthorization".to_owned(),
        json!("unproved-complete-Needs"),
    );
    fields.insert("verification".to_owned(), json!("not-executed"));
    fields.insert(
        "profile".to_owned(),
        json!("conditional JavaScript source-owned plain data; caller effects inherited, not pure"),
    );
    Ok(candidate)
}
