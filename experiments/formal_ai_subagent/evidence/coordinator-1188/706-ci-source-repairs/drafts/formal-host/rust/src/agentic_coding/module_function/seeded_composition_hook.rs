//! Source-qualified composition; no output literal or unconditional effect authority.
use super::callable_catalog::{observe_source_callables, observed_conditional_graphs};
use super::complete_source_preflight::source_goal_scope;
use super::source_module_identity_port::{SourceModuleIdentityHost, source_operation_module_url};
use crate::seed::parser::{LinoNode, parse_lino};
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};

fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str, &'static str> {
    value[key].as_str().ok_or("MissingSourceContractField")
}
fn list<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, &'static str> {
    value[key].as_array().ok_or("MissingSourceContractList")
}
fn require(condition: bool, error: &'static str) -> Result<(), &'static str> {
    if condition { Ok(()) } else { Err(error) }
}
pub fn bind_accepted_import<Host: SourceModuleIdentityHost>(
    host: &Host,
    acceptance: &Value,
    source: &Value,
    exported: &str,
    operation: &Value,
) -> Result<Value, &'static str> {
    require(
        string(operation, "command")?
            == format!("node --test {}", string(operation, "acceptanceOperand")?),
        "DifferentOperation",
    )?;
    require(
        string(operation, "acceptancePath")? == string(acceptance, "path")?,
        "ForeignAcceptance",
    )?;
    let acceptance_identity = sha256_hex(string(acceptance, "content")?.as_bytes());
    require(
        string(operation, "acceptanceIdentity")? == acceptance_identity,
        "AcceptanceDrift",
    )?;
    let source_identity = sha256_hex(string(source, "content")?.as_bytes());
    let catalog =
        observe_source_callables(string(acceptance, "content")?, string(acceptance, "path")?);
    require(
        !list(&catalog, "gaps")?.iter().any(|gap| {
            matches!(
                gap["reason"].as_str(),
                Some("LexicalFailure" | "DuplicateBinding" | "DuplicateExport")
            )
        }),
        "AmbiguousAcceptance",
    )?;
    let source_catalog =
        observe_source_callables(string(source, "content")?, string(source, "path")?);
    let exported_items: Vec<_> = list(&source_catalog, "exports")?
        .iter()
        .filter(|value| value["exposed"] == exported)
        .collect();
    require(exported_items.len() == 1, "UnprovedExport")?;
    require(
        list(&source_catalog, "declarations")?
            .iter()
            .any(|entry| entry["name"] == exported_items[0]["local"]),
        "UnobservedDeclaration",
    )?;
    let module_url = source_operation_module_url(
        Some(host),
        string(source, "path")?,
        string(acceptance, "path")?,
    )?;
    let matching: Vec<_> = list(&catalog, "imports")?
        .iter()
        .filter(|entry| {
            let Some(specifier) = entry["specifier"].as_str() else {
                return false;
            };
            source_operation_module_url(
                Some(host),
                specifier,
                string(acceptance, "path").unwrap_or(""),
            )
            .is_ok_and(|url| url == module_url)
                && entry["bindings"].as_array().is_some_and(|bindings| {
                    bindings
                        .iter()
                        .any(|binding| binding["imported"] == exported)
                })
        })
        .collect();
    require(matching.len() == 1, "NotAlreadyAcceptedImport")?;
    Ok(
        json!({"kind":"accepted-operation-import-reuse","moduleURL":module_url,
        "exported":exported,"sourceIdentity":source_identity,"acceptanceIdentity":catalog["contentId"],
        "operation":operation["command"],"additionalModuleInitialization":false,
        "moduleEffects":source_catalog["moduleEffects"],"callEffects":"conditional",
        "scope":"same-operation-same-module-URL","rustEquivalence":"unknown"}),
    )
}
fn matches_goal(goal: &LinoNode, returned: &[String]) -> bool {
    let lexicon = crate::seed::lexicon();
    goal.children
        .iter()
        .filter(|node| node.name == "lexeme" && node.id == "en")
        .any(|node| {
            node.children
                .iter()
                .filter(|surface| surface.name == "surface")
                .any(|surface| {
                    surface
                        .children
                        .iter()
                        .filter(|text| text.name == "text" && !text.id.is_empty())
                        .any(|text| {
                            returned.iter().any(|clause| {
                                clause.find(&text.id.to_lowercase()).is_some_and(|at| {
                                    !lexicon.mentions_role("statement_negation_cue", &clause[..at])
                                })
                            })
                        })
                })
        })
}
pub fn qualify_seeded_composition<Host: SourceModuleIdentityHost>(
    host: &Host,
    request: &Value,
    observations: &[Value],
    acceptance: &Value,
    operation: &Value,
    seed: &str,
) -> Result<Value, &'static str> {
    let text = string(request, "text")?;
    require(
        string(request, "identity")? == sha256_hex(text.as_bytes()),
        "RequestDrift",
    )?;
    let scope = source_goal_scope(text).ok_or("UnprovedQuotationScope")?;
    let parsed = parse_lino(seed);
    let root = parsed
        .children
        .iter()
        .find(|node| node.name == "source-callable-composition")
        .ok_or("MissingSeededCompositionContract")?;
    let lexicon = crate::seed::lexicon();
    let returned: Vec<_> = crate::obligation_ledger::clauses_with_spans(&scope)
        .into_iter()
        .map(|(text, _)| text.to_lowercase())
        .filter(|text| lexicon.mentions_role("coding_return_action", text))
        .collect();
    let goals: Vec<_> = root
        .children
        .iter()
        .filter(|node| node.name == "goal" && matches_goal(node, &returned))
        .collect();
    require(goals.len() == 1, "UnboundOrAmbiguousSeededGoal")?;
    let goal = goals[0];
    require(
        goal.find_child_value("producer-result") == "optional-record"
            && goal.find_child_value("consumer-result") == "text"
            && goal.find_child_value("absent-result") == "null",
        "UnsupportedGoalShape",
    )?;
    let mut authorized = Vec::new();
    for source in observations {
        let catalog = observe_source_callables(string(source, "content")?, string(source, "path")?);
        for exported in list(&catalog, "exports")? {
            let exposed = string(exported, "exposed")?;
            if let Ok(receipt) = bind_accepted_import(host, acceptance, source, exposed, operation)
            {
                authorized.push((
                    string(source, "path")?.to_owned(),
                    exposed.to_owned(),
                    receipt,
                ));
            }
        }
    }
    let mut candidates = Vec::new();
    for graph in observed_conditional_graphs(observations) {
        if graph["kind"] != "conditional-schema-graph"
            || graph["schemas"]["produced"]["kind"] != "optional"
            || graph["schemas"]["produced"]["value"]["kind"] != "record"
            || graph["schemas"]["result"]["kind"] != "text"
        {
            continue;
        }
        let mut pair = Vec::new();
        for binding in list(&graph, "bindings")? {
            if let Some((_, _, receipt)) = authorized.iter().find(|(path, exported, _)| {
                binding["path"] == *path && binding["exported"] == *exported
            }) {
                pair.push(receipt.clone());
            }
        }
        if pair.len() == list(&graph, "bindings")?.len() {
            candidates.push((graph, pair));
        }
    }
    require(
        candidates.len() == 1,
        "UnprovedOrAmbiguousSourceComposition",
    )?;
    let (graph, imports) = candidates.pop().ok_or("MissingSourceComposition")?;
    Ok(
        json!({"kind":"source-qualified-seeded-composition-candidate","goal":goal.id,
        "graph":graph,"imports":imports,"requestIdentity":request["identity"],
        "goalClauseCoverage":"partial","writeAuthority":false,"authored":false,"verified":false,
        "callEffects":"conditional","moduleEffects":"unknown","rustEquivalence":"unknown"}),
    )
}
