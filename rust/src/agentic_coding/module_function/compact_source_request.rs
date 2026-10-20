//! Closed compact source syntax; observing operands never certifies their semantics.
use crate::agentic_coding::shell_command_policy::sentences;
use crate::agentic_coding::write_request::{looks_like_file_path, safe_relative_path};
use crate::seed::parser::parse_lino;
use crate::source_fetch::sha256_hex;
use serde_json::{Value, json};
use std::collections::BTreeSet;

/// Bind operands only after every compact clause has a unique seeded form.
#[must_use]
pub fn compact_source_request(text: &str, seed: &str) -> Option<Value> {
    if crate::normal_markov::quote_fault(text).is_some()
        || !crate::normal_markov::quoted_segment_spans(text).is_empty()
    {
        return None;
    }
    let parsed = parse_lino(seed);
    let root = parsed
        .children
        .iter()
        .find(|node| node.name == "source-request-compact")?;
    let kinds = [
        "declaration-goal",
        "source-discovery",
        "reuse-imports",
        "verification",
        "protect-inputs",
    ];
    let forms: Vec<_> = root
        .children
        .iter()
        .filter(|node| node.name == "form")
        .collect();
    if forms.len() != kinds.len()
        || !kinds
            .iter()
            .all(|kind| forms.iter().filter(|form| form.id == *kind).count() == 1)
    {
        return None;
    }
    let mut clauses = Vec::new();
    for sentence in sentences(text) {
        let mut matches = Vec::new();
        for form in &forms {
            let pattern = regex::RegexBuilder::new(form.find_child_value("pattern"))
                .case_insensitive(true)
                .build()
                .ok()?;
            let Some(captures) = pattern.captures(sentence.text) else {
                continue;
            };
            if captures.get(0)?.as_str() != sentence.text {
                continue;
            }
            let mut values = serde_json::Map::new();
            for name in pattern.capture_names().flatten() {
                if let Some(value) = captures.name(name) {
                    values.insert(name.to_owned(), json!(value.as_str()));
                }
            }
            matches.push((form.id.as_str(), values));
        }
        if matches.len() != 1 {
            return None;
        }
        let (kind, captures) = matches.pop()?;
        let start = sentence.span.start + text[sentence.span.start..].find(sentence.text)?;
        let end = start + sentence.text.len();
        if end > sentence.span.end {
            return None;
        }
        clauses.push(
            json!({"kind":kind,"captures":captures,"sentence":sentence.text,
            "start":text[..start].encode_utf16().count(),"end":text[..end].encode_utf16().count()}),
        );
    }
    if !kinds.iter().all(|kind| {
        clauses
            .iter()
            .filter(|clause| clause["kind"] == *kind)
            .count()
            == 1
    }) {
        return None;
    }
    let captures = |kind: &str| {
        clauses
            .iter()
            .find(|clause| clause["kind"] == kind)
            .map(|clause| &clause["captures"])
    };
    let declaration = captures("declaration-goal")?;
    let signature = declaration["signature"].as_str()?;
    let stated = super::signature(signature)?;
    let destination = declaration["destination"].as_str()?;
    if stated.at != 0
        || !signature.ends_with(')')
        || !safe_relative_path(destination)
        || super::extension_language(destination).as_deref() != Some("javascript")
    {
        return None;
    }
    let discovery = captures("source-discovery")?;
    let sources: Vec<_> = discovery["sources"]
        .as_str()?
        .split(root.find_child_value("source-joiner"))
        .collect();
    let command = captures("verification")?["command"].as_str()?;
    let acceptance = super::paths_in(command);
    if sources.len() < 2
        || sources.iter().copied().collect::<BTreeSet<_>>().len() != sources.len()
        || sources
            .iter()
            .any(|path| !safe_relative_path(path) || !looks_like_file_path(path))
        || sources.contains(&destination)
        || acceptance.len() != 1
        || command
            != crate::agentic_coding::workspace_discovery::contract_text(
                "accepted-node-test-command",
                &[acceptance[0].as_str()],
            )
        || sources.contains(&acceptance[0].as_str())
        || destination == acceptance[0]
    {
        return None;
    }
    let mut aliases = Vec::new();
    if let Some(alias_text) = discovery["aliases"].as_str() {
        let expression = regex::Regex::new(
            r"^\s*([A-Za-z_$][A-Za-z0-9_$]*)\s+as\s+([A-Za-z_$][A-Za-z0-9_$]*)\s*$",
        )
        .ok()?;
        for pair in alias_text.split(',') {
            let matched = expression.captures(pair)?;
            aliases.push(
                json!({"exported":matched.get(1)?.as_str(),"local":matched.get(2)?.as_str()}),
            );
        }
    }
    for field in ["exported", "local"] {
        if aliases
            .iter()
            .map(|alias| alias[field].as_str())
            .collect::<BTreeSet<_>>()
            .len()
            != aliases.len()
        {
            return None;
        }
    }
    Some(
        json!({"name":stated.name,"parameters":stated.parameters,"destination":destination,
        "inputs":sources,"acceptance":acceptance,"command":command,"aliases":aliases,
        "goalClause":declaration["goal"],"sourceClauses":clauses}),
    )
}

fn byte_offset(text: &str, units: usize) -> Option<usize> {
    let mut cursor = 0;
    for (offset, character) in text.char_indices() {
        if cursor == units {
            return Some(offset);
        }
        cursor += character.len_utf16();
        if cursor > units {
            return None;
        }
    }
    (cursor == units).then_some(text.len())
}

pub(super) fn compact_source_frame(text: &str, seed: &str) -> Option<Value> {
    let request = compact_source_request(text, seed)?;
    let mut clauses = request["sourceClauses"].as_array()?.clone();
    for clause in &mut clauses {
        let start = byte_offset(text, usize::try_from(clause["start"].as_u64()?).ok()?)?;
        let end = byte_offset(text, usize::try_from(clause["end"].as_u64()?).ok()?)?;
        clause["evidence"] = super::source_contract::source_evidence(text, start, end);
        clause["status"] = json!("requires-contract");
    }
    let declaration = clauses
        .iter()
        .find(|clause| clause["kind"] == "declaration-goal")?;
    let end = byte_offset(text, usize::try_from(declaration["end"].as_u64()?).ok()?)?;
    let goal = request["goalClause"].as_str()?;
    let start = end.checked_sub(goal.len())?;
    clauses.push(json!({"kind":"goal","sentence":goal,
        "evidence":super::source_contract::source_evidence(text,start,end),"status":"requires-contract"}));
    Some(
        json!({"request":request,"requestIdentity":sha256_hex(text.as_bytes()),
        "seedIdentity":sha256_hex(seed.as_bytes()),"clauses":clauses,
        "syntaxCoverage":"complete-finite-seeded-forms","semanticAuthority":false,"writeAuthority":false,
        "inferredOperationPrerequisites":[
            {"kind":"read-prerequisites","authority":"candidate-operation-invariant"},
            {"kind":"repair-plan","authority":"source-qualified-composition-witness"},
            {"kind":"gap-before-write","authority":"candidate-transaction-refuses-unbound-needs"}]}),
    )
}
