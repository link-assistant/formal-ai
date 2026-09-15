//! Derive a structured document from an inspected workspace record.
//!
//! A request such as “inspect INPUT; author OUTPUT as Links Notation with
//! `ROOT` and nested `FIELD_A`, `FIELD_B`” is neither a plain read nor a literal
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

use std::collections::{BTreeMap, BTreeSet};

use super::planner::{AgenticPlan, Capability, plan_one, tool_for, write_arguments};
use super::progress::Progress;
use super::shell_command_policy::sentences;
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
    // File shape alone does not bind either side of this transaction. Technical
    // prose routinely carries dotted identifiers (`module.Error`, package
    // names, symbols), and a later "read it back" cue used to turn the first
    // such token into the source file. The output must be a stated write target;
    // the input must occur in the sentence that states the inspection.
    let output = paths
        .iter()
        .find(|path| {
            is_lino_path(path) && super::write_request::is_stated_write_target(task, path)
        })
        .cloned()
        .or_else(|| composed_output_path(task))?;
    let input = inspected_source_path(task, &output)?;
    if input == output {
        return None;
    }

    let identifiers = machine_identifiers(task);
    let (root, fields) = identifiers.split_first()?;
    // A schema list may cross commas, colons, or semicolons. Keep the complete
    // declarative sentence from the caller-supplied root onward. Compound
    // identifiers are collected message-wide above, while plain field names
    // stay confined to this declaration so later provenance prose cannot add
    // an accidental generic `source` field.
    let schema_clause = task.split_once(root).map(|(_, after_root)| {
        let declaration = after_root.split_once('.').map_or(after_root, |(head, _)| head);
        format!("{root}{declaration}")
    })?;
    (!fields.is_empty()).then(|| Specification {
        input,
        output,
        root: root.clone(),
        fields: fields.to_vec(),
        schema_clause,
    })
}

fn is_lino_path(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("lino"))
}

/// The `LiNo` path named in the sentence that asks for document composition.
///
/// `author` is intentionally a document-composition cue rather than a generic
/// file-write cue. Keeping this second binding lets "Author OUTPUT" work while
/// still refusing file-shaped identifiers from unrelated diagnostic prose.
fn composed_output_path(task: &str) -> Option<String> {
    let lexicon = crate::seed::lexicon();
    sentences(task).into_iter().find_map(|sentence| {
        let normalized = crate::engine::normalize_prompt(sentence.text);
        if !lexicon.mentions_role(crate::seed::ROLE_DOCUMENT_COMPOSITION_ACTION, &normalized) {
            return None;
        }
        tokens(sentence.text)
            .into_iter()
            .map(|token| clean_path_token(token.text))
            .find(|path| is_lino_path(path) && safe_relative_path(path))
            .map(str::to_owned)
    })
}

/// The safe file-shaped token named by an inspection sentence.
///
/// This relation, rather than message-wide token order, is the evidence that a
/// dotted token is the source record. A sentence may also ask to write and then
/// read back the output; excluding its stated write target keeps that verifier
/// from turning the destination into its own input.
fn inspected_source_path(task: &str, output: &str) -> Option<String> {
    let lexicon = crate::seed::lexicon();
    sentences(task).into_iter().find_map(|sentence| {
        let normalized = crate::engine::normalize_prompt(sentence.text);
        if !lexicon.mentions_role(crate::seed::ROLE_WORKSPACE_INSPECTION_ACTION, &normalized) {
            return None;
        }
        tokens(sentence.text)
            .into_iter()
            .map(|token| clean_path_token(token.text))
            .filter(|path| looks_like_file_path(path) && safe_relative_path(path))
            .find(|path| {
                *path != output
                    && !super::write_request::is_stated_write_target(sentence.text, path)
            })
            .map(str::to_owned)
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
    let fields = requested_fields(specification, &tree);
    let scopes = collection_scopes(specification, &tree);
    for scope in scopes {
        push_lino_node(
            &mut out,
            2,
            "derived_formalization",
            (!scope.id.is_empty()).then_some(scope.id.as_str()),
        );
        for field in &fields {
            for value in best_source_values(scope, field) {
                push_lino_node(&mut out, 4, field, Some(&value));
            }
        }
    }
    out.trim_end().to_owned()
}

/// Return one source scope per record when the requested schema is an index or
/// carries the seed-defined universal quantifier ("each", "every", and their
/// registered translations). The record kind is discovered from repeated
/// direct children of the inspected root, preferring a kind named by the
/// caller. Otherwise the entire parsed tree is one derivation scope.
fn collection_scopes<'a>(
    specification: &Specification,
    tree: &'a LinoNode,
) -> Vec<&'a LinoNode> {
    let normalized = crate::engine::normalize_prompt(&specification.schema_clause);
    let asks_for_index = specification
        .root
        .split('_')
        .any(|token| token == "index");
    let asks_for_every = crate::seed::lexicon()
        .meaning("quantifier_all")
        .is_some_and(|meaning| meaning.evidenced_in(&normalized));
    if !asks_for_index && !asks_for_every {
        return vec![tree];
    }

    let Some(container) = tree.children.first() else {
        return vec![tree];
    };
    let mut counts = BTreeMap::<&str, usize>::new();
    for child in &container.children {
        *counts.entry(child.name.as_str()).or_default() += 1;
    }
    let task_tokens = normalized.split_whitespace().collect::<BTreeSet<_>>();
    let mut repeated = counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    repeated.sort_by_key(|name| (!task_tokens.contains(name), *name));
    let Some(record_name) = repeated.first() else {
        return vec![tree];
    };
    container
        .children
        .iter()
        .filter(|child| child.name == *record_name)
        .collect()
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
        child.name == token
            || child
                .name
                .strip_suffix(token)
                .is_some_and(|prefix| prefix.ends_with('_'))
            || (!child.children.is_empty()
                && child
                    .name
                    .strip_prefix(token)
                    .is_some_and(|suffix| suffix.starts_with('_')))
            || tree_contains_identifier_token(child, token)
    })
}

fn best_source_values(tree: &LinoNode, field: &str) -> Vec<String> {
    let wanted = identifier_tokens(field);
    let mut candidates = Vec::new();
    collect_candidates(tree, field, &wanted, &mut candidates);
    let exact = candidates
        .iter()
        .filter(|candidate| candidate.exact)
        .map(|candidate| candidate.value.clone())
        .collect::<BTreeSet<_>>();
    if !exact.is_empty() {
        return exact.into_iter().collect();
    }
    candidates
        .into_iter()
        .max_by(|left, right| {
            left.score
                .cmp(&right.score)
                .then_with(|| right.value.len().cmp(&left.value.len()))
        })
        .map(|candidate| vec![candidate.value])
        .unwrap_or_default()
}

struct SourceCandidate {
    score: usize,
    exact: bool,
    value: String,
}

fn collect_candidates(
    node: &LinoNode,
    field: &str,
    wanted: &BTreeSet<String>,
    out: &mut Vec<SourceCandidate>,
) {
    for child in &node.children {
        let found = identifier_tokens(&child.name);
        let overlap = wanted.intersection(&found).count();
        let exact = child.name == field;
        // A partial match is trustworthy only when one structural identifier
        // contains the other. This retains `transition` ->
        // `recursive_transition`, but rejects ambiguous one-token collisions
        // such as `source_function` -> `concept_source_url`.
        let structurally_related = overlap == wanted.len().min(found.len());
        if exact || (overlap > 0 && structurally_related) {
            let mut leaves = Vec::new();
            collect_leaf_values(child, &mut leaves);
            if !child.id.is_empty() {
                leaves.insert(0, child.id.clone());
            }
            if !leaves.is_empty() {
                out.push(SourceCandidate {
                    score: overlap + usize::from(exact) * 100,
                    exact,
                    value: leaves.join("; "),
                });
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
