//! Derive a structured document from an inspected workspace record.
//!
//! A request such as “inspect INPUT; author OUTPUT as Links Notation with
//! ROOT and nested FIELD_A, FIELD_B” is neither a plain read nor a literal
//! write.  The bytes of OUTPUT do not exist in the prompt: they have to be
//! derived from INPUT.  This bounded state machine keeps the two obligations
//! together and makes the derivation inspectable:
//!
//! `read(input) -> derive named fields -> write(output) -> read(output)`.
//!
//! Schema identifiers are taken from the request, source values are selected
//! from the parsed Links tree by structural token overlap, and the output
//! records the exact input digest separately from the derived formalization.
//! No domain name or field is built into the planner.

use std::collections::BTreeSet;

use super::planner::{AgenticPlan, Capability, plan_one, tool_for, write_arguments};
use super::progress::Progress;
use super::write_request::{clean_path_token, looks_like_file_path, safe_relative_path, tokens};
use crate::links_format::push_lino_node;
use crate::protocol::ChatMessage;
use crate::seed::parser::LinoNode;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Specification {
    input: String,
    output: String,
    root: String,
    fields: Vec<String>,
    schema_clause: String,
}

/// Plan the next step of a source-backed structured-document transaction.
pub(super) fn plan_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let specification = recognise(task)?;
    let read_tool = tool_for(tool_names, Capability::Read)?;
    let write_tool = tool_for(tool_names, Capability::Write)?;
    let progress = Progress::scan(messages);

    if progress.attempted_write_for(&specification.output) {
        if !progress.successful_write_for(&specification.output) {
            return Some(AgenticPlan::Final(format!(
                "The structured document could not be written to `{}`.",
                specification.output
            )));
        }
        let Some(expected) = progress.successful_write_content_for(&specification.output) else {
            return Some(AgenticPlan::Final(format!(
                "The client reported writing `{}`, but returned no inspectable content.",
                specification.output
            )));
        };
        if let Some(observed) = progress.successful_read_output_for(&specification.output) {
            let observed = super::code_artifact::source_from_read_result(observed);
            return Some(AgenticPlan::Final(if observed.trim() == expected.trim() {
                format!(
                    "Derived `{}` from `{}` and verified the written Links Notation byte-for-byte.",
                    specification.output, specification.input
                )
            } else {
                format!(
                    "The read-back of `{}` did not match the derived Links Notation.",
                    specification.output
                )
            }));
        }
        return Some(plan_one(
            read_tool,
            read_arguments(&specification.output),
        ));
    }

    let Some(source) = progress.successful_read_output_for(&specification.input) else {
        return Some(plan_one(
            read_tool,
            read_arguments(&specification.input),
        ));
    };
    let source = super::code_artifact::source_from_read_result(source);
    let content = render(&specification, &source);
    Some(plan_one(
        write_tool,
        write_arguments(&specification.output, &content),
    ))
}

fn recognise(task: &str) -> Option<Specification> {
    let normalized = crate::engine::normalize_prompt(task);
    let lexicon = crate::seed::lexicon();
    if !lexicon.mentions_role(crate::seed::ROLE_WORKSPACE_INSPECTION_ACTION, &normalized)
        || !lexicon.mentions_role(crate::seed::ROLE_DOCUMENT_COMPOSITION_ACTION, &normalized)
        || !lexicon.mentions_role(crate::seed::ROLE_LINKS_NOTATION_FORMAT, &normalized)
    {
        return None;
    }

    let paths = tokens(task)
        .into_iter()
        .map(|token| clean_path_token(token.text))
        .filter(|path| looks_like_file_path(path) && safe_relative_path(path))
        .map(str::to_owned)
        .fold(Vec::<String>::new(), |mut paths, path| {
            if !paths.contains(&path) {
                paths.push(path);
            }
            paths
        });
    let [input, output, ..] = paths.as_slice() else {
        return None;
    };
    if input == output || !output.ends_with(".lino") {
        return None;
    }

    let identifiers = machine_identifiers(task);
    let (root, fields) = identifiers.split_first()?;
    let schema_clause = super::shell_command_policy::sentences(task)
        .into_iter()
        .find(|sentence| sentence.text.contains(root))?
        .text
        .to_owned();
    (!fields.is_empty()).then(|| Specification {
        input: input.clone(),
        output: output.clone(),
        root: root.clone(),
        fields: fields.to_vec(),
        schema_clause,
    })
}

/// Identifiers are schema supplied by the caller, not prose interpreted by the
/// planner. Requiring an underscore keeps ordinary words and file names out.
fn machine_identifiers(task: &str) -> Vec<String> {
    let mut seen = BTreeSet::new();
    task.split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|token| token.contains('_'))
        .filter(|token| {
            token
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphabetic())
        })
        .map(str::to_owned)
        .filter(|identifier| seen.insert(identifier.clone()))
        .collect()
}

fn render(specification: &Specification, source: &str) -> String {
    let tree = crate::seed::parser::parse_lino(source);
    let mut out = String::new();
    push_lino_node(&mut out, 0, &specification.root, None);
    push_lino_node(&mut out, 2, "source_observation", None);
    push_lino_node(&mut out, 4, "path", Some(&specification.input));
    push_lino_node(
        &mut out,
        4,
        "sha256",
        Some(&crate::source_fetch::sha256_hex(source.as_bytes())),
    );
    push_lino_node(&mut out, 4, "root", tree.children.first().map(|node| node.name.as_str()));
    push_lino_node(&mut out, 2, "derived_formalization", None);
    for field in requested_fields(specification, &tree) {
        let value = best_source_value(&tree, &field)
            .unwrap_or_else(|| "not established by the inspected source".to_owned());
        push_lino_node(&mut out, 4, &field, Some(&value));
    }
    out.trim_end().to_owned()
}

/// Recover plain schema identifiers (for example `validation`) by relating
/// request words to names actually present in the inspected tree. The caller's
/// compound identifiers remain authoritative; a plain prose word becomes a
/// field only when a source node independently supplies the same structural
/// token.
fn requested_fields(specification: &Specification, tree: &LinoNode) -> Vec<String> {
    let mut fields = specification.fields.clone();
    let mut seen = fields.iter().cloned().collect::<BTreeSet<_>>();
    let after_root = specification
        .schema_clause
        .split_once(&specification.root)
        .map_or("", |(_, after)| after);
    for candidate in after_root
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|candidate| candidate.len() > 2)
    {
        if !candidate.contains('_')
            && tree_contains_identifier_token(tree, candidate)
            && seen.insert(candidate.to_owned())
        {
            fields.push(candidate.to_owned());
        }
    }
    fields
}

fn tree_contains_identifier_token(node: &LinoNode, token: &str) -> bool {
    node.children.iter().any(|child| {
        child.name.split('_').any(|part| part == token)
            || tree_contains_identifier_token(child, token)
    })
}

fn best_source_value(tree: &LinoNode, field: &str) -> Option<String> {
    let wanted = identifier_tokens(field);
    let mut candidates = Vec::new();
    collect_candidates(tree, field, &wanted, &mut candidates);
    candidates
        .into_iter()
        .max_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| right.1.len().cmp(&left.1.len()))
        })
        .filter(|(score, _)| *score > 0)
        .map(|(_, value)| value)
}

fn collect_candidates(
    node: &LinoNode,
    field: &str,
    wanted: &BTreeSet<String>,
    out: &mut Vec<(usize, String)>,
) {
    for child in &node.children {
        let found = identifier_tokens(&child.name);
        let overlap = wanted.intersection(&found).count();
        if overlap > 0 {
            let mut leaves = Vec::new();
            collect_leaf_values(child, &mut leaves);
            if !child.id.is_empty() {
                leaves.insert(0, child.id.clone());
            }
            if !leaves.is_empty() {
                let exact = usize::from(child.name == field);
                out.push((overlap + exact * 100, leaves.join("; ")));
            }
        }
        collect_candidates(child, field, wanted, out);
    }
}

fn collect_leaf_values(node: &LinoNode, out: &mut Vec<String>) {
    for child in &node.children {
        if !child.id.is_empty() {
            out.push(child.id.clone());
        }
        collect_leaf_values(child, out);
    }
}

fn identifier_tokens(identifier: &str) -> BTreeSet<String> {
    identifier
        .split('_')
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect()
}

fn read_arguments(path: &str) -> String {
    serde_json::json!({
        "path": path,
        "filePath": path,
        "file_path": path,
    })
    .to_string()
}
