//! Seed-backed lowering of bounded coding tasks into executable Rust source.
//!
//! The coding ladder exposed a dangerous false-green: the literal-file parser
//! treated a description of source code as the bytes to write. This module is
//! the semantic route that runs before literal writes. It recognises source
//! concepts through the multilingual seed, renders Rust syntax, and completes
//! only after the client reads the written bytes back through an allowlisted
//! command.

use serde_json::json;
mod identifier_domain;
mod source_contract;
mod target_guard;
use super::final_result::FinalResult;

use super::code_artifact::{latest_result, result_for_command};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for, write_arguments};
use crate::normal_markov::unwrap_transport_quotes;
use crate::protocol::ChatMessage;
use crate::seed::{self, Slot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RustItemKind {
    Function,
    Constant,
    Test,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GeneratedSource {
    pub(super) path: String,
    pub(super) content: String,
}

pub(super) fn plan_generated_source_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let task = unwrap_transport_quotes(task);
    // Issue #1096: "In the file src/x.rs, replace "A" with "B" ... keep it valid
    // Rust" is an edit of a tracked file, not a request to generate one. The
    // program-request cue ("valid Rust") made this path claim it, synthesise
    // its own artifact, and then verify the agent's correct edit against that
    // artifact -- seven ladder leaves failed on the resulting byte mismatch.
    // An edit request is recognised by the same reader the edit route uses, so
    // the two routes cannot disagree about what counts as one; this step
    // simply declines, and the edit step, later in the router, claims it.
    if super::general_planner::compose_edit_request(task).is_some() {
        return None;
    }
    let artifact = rust_source_for_task(task)?;
    let write_tool = tool_for(tool_names, Capability::Write)?;
    let latest_user = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))?;
    let current_turn = &messages[latest_user + 1..];

    if let Some(observed) = result_for_command(current_turn, &format!("cat {}", artifact.path)) {
        let outcome = if super::tool_result::observed_bytes_match(&observed, &artifact.content) {
            "coding_workspace_effect_observed"
        } else {
            "coding_workspace_verification_failed"
        };
        return Some(AgenticPlan::Final(render_seeded_outcome(
            outcome,
            task,
            &artifact.path,
        )?));
    }
    if latest_result(current_turn, Capability::Write).is_some() {
        return Some(match tool_for(tool_names, Capability::Run) {
            Some(run_tool) => plan_one(
                run_tool,
                json!({"command": format!("cat {}", artifact.path)}).to_string(),
            ),
            None => AgenticPlan::Final(render_seeded_outcome(
                "coding_workspace_written_unverified",
                task,
                &artifact.path,
            )?),
        });
    }
    target_guard::guarded_source_step(task, &artifact, current_turn, tool_names, result).or_else(
        || {
            Some(plan_one(
                write_tool,
                write_arguments(&artifact.path, &artifact.content),
            ))
        },
    )
}

// These are seed-template placeholders, not Rust formatting arguments.
#[allow(clippy::literal_string_with_formatting_args)]
pub(super) fn rust_source_for_task(task: &str) -> Option<GeneratedSource> {
    let normalized = task.to_lowercase();
    let lexicon = seed::lexicon();
    if !lexicon.mentions_role(seed::ROLE_FILE_WRITE_ACTION_CUE, &normalized)
        && !lexicon.mentions_role(seed::ROLE_PROGRAM_REQUEST, &normalized)
    {
        return None;
    }
    let path = rust_path(task)?;
    let extra_kind = lexicon.first_role_match(seed::ROLE_CODING_ARTIFACT_KIND, &normalized);
    let kind = if extra_kind.is_some_and(|meaning| meaning.slug == "coding_constant") {
        RustItemKind::Constant
    } else if extra_kind.is_some_and(|meaning| meaning.slug == "coding_test") {
        RustItemKind::Test
    } else if lexicon
        .first_role_match(seed::ROLE_PROGRAM_KIND, &normalized)
        .is_some_and(|meaning| meaning.slug == "function")
    {
        RustItemKind::Function
    } else {
        return None;
    };
    let name = requested_identifier(task, &path, kind)?;
    let task_without_path = task.replacen(&path, "", 1);
    let numbers = numeric_literals(&task_without_path);
    let public = lexicon.mentions_role(seed::ROLE_CODING_VISIBILITY, &normalized);

    let content = match kind {
        RustItemKind::Function => {
            if normalized.contains("f64")
                && lexicon.mentions_role(seed::ROLE_CODING_DIVISION_ACTION, &normalized)
            {
                let divisor = numbers.last()?;
                render_rust_template(
                    "coding_source_function_division",
                    &[
                        ("{visibility}", if public { "pub " } else { "" }),
                        ("{name}", &name),
                        ("{divisor}", divisor),
                    ],
                )?
            } else if lexicon.mentions_role(seed::ROLE_CODING_RETURN_ACTION, &normalized) {
                let value = numbers.last()?;
                render_rust_template(
                    "coding_source_function_return",
                    &[
                        ("{visibility}", if public { "pub " } else { "" }),
                        ("{name}", &name),
                        ("{value}", value),
                    ],
                )?
            } else {
                return None;
            }
        }
        RustItemKind::Constant => {
            if !normalized.contains("&str") {
                return None;
            }
            let value = slot_identifier(&normalized, seed::ROLE_CODING_VALUE_SLOT)?;
            render_rust_template(
                "coding_source_string_constant",
                &[
                    ("{visibility}", if public { "pub " } else { "" }),
                    ("{name}", &name),
                    ("{value}", &value),
                ],
            )?
        }
        RustItemKind::Test => {
            let left = numbers.first()?;
            let right = numbers.get(1)?;
            if let (Some(expected), Some(operator)) = (
                numbers.get(2),
                lexicon
                    .first_role_match(seed::ROLE_ARITHMETIC_OPERATOR_WORD, &normalized)
                    .and_then(|meaning| {
                        meaning
                            .words()
                            .find(|surface| !surface.chars().any(char::is_alphabetic))
                    }),
            ) {
                render_rust_template(
                    "coding_source_binary_operation_test",
                    &[
                        ("{name}", &name),
                        ("{left}", left),
                        ("{operator}", operator),
                        ("{right}", right),
                        ("{expected}", expected),
                    ],
                )?
            } else {
                render_rust_template(
                    "coding_source_equality_test",
                    &[("{name}", &name), ("{left}", left), ("{right}", right)],
                )?
            }
        }
    };
    Some(GeneratedSource { path, content })
}

pub(super) fn render_rust_template(intent: &str, substitutions: &[(&str, &str)]) -> Option<String> {
    Some(render_template(
        &seed::response_for(intent, "rust")?,
        substitutions,
    ))
}

// `{path}` is replaced in seed-owned text rather than interpolated by Rust.
#[allow(clippy::literal_string_with_formatting_args)]
pub(super) fn render_seeded_outcome(intent: &str, task: &str, path: &str) -> Option<String> {
    let language = super::tool_result::response_language(task);
    Some(render_template(
        &seed::localized_response(intent, language)?,
        &[("{path}", path)],
    ))
}

/// Report a change by naming it, not merely by naming the file it touched.
///
/// [`render_seeded_outcome`] can only say *that* a path was written, because
/// `{path}` is all it substitutes. A caller who asked for the change and for a
/// record of it -- the issue #1028 ladder does, and its `verify-node.sh` demands
/// "at least four words that state the change you made" -- gets a status line
/// where the statement should be, and the record it verifies never mentions the
/// edit. A route that derived the new bytes already knows what it altered, so
/// the seed sentence takes that as further slots: which members went into a
/// list, or which name became which.
// `{path}` is replaced in seed-owned text rather than interpolated by Rust; so
// are the slots the caller names.
#[allow(clippy::literal_string_with_formatting_args)]
pub(super) fn render_seeded_change(
    intent: &str,
    task: &str,
    path: &str,
    slots: &[(&str, &str)],
) -> Option<String> {
    let language = super::tool_result::response_language(task);
    let mut substitutions = vec![("{path}", path)];
    substitutions.extend_from_slice(slots);
    Some(render_template(
        &seed::localized_response(intent, language)?,
        &substitutions,
    ))
}

/// Render an explicitly typed list slot; mirrors `renderSeededListChange`.
pub(super) fn render_seeded_list_change(
    intent: &str,
    task: &str,
    path: &str,
    slot: &str,
    values: &[String],
) -> Option<String> {
    render_seeded_change_with_lists(intent, task, path, &[], &[(slot, values)])
}

/// Render scalar slots and explicitly typed lists without interpreting scalar text.
#[allow(clippy::literal_string_with_formatting_args)]
pub(super) fn render_seeded_change_with_lists(
    intent: &str,
    task: &str,
    path: &str,
    slots: &[(&str, &str)],
    lists: &[(&str, &[String])],
) -> Option<String> {
    let language = super::tool_result::response_language(task);
    let template = seed::localized_response(intent, language)?;
    let mut substitutions = vec![("{path}", path)];
    substitutions.extend_from_slice(slots);
    let mut values: Vec<(&str, String, String)> = substitutions
        .into_iter()
        .filter(|(slot, _)| !lists.iter().any(|(listed, _)| listed == slot))
        .map(|(slot, value)| (slot, value.to_owned(), code_span(value)))
        .collect();
    for (slot, items) in lists {
        let rendered = items
            .iter()
            .map(|value| code_span_item(value))
            .collect::<Vec<_>>()
            .join(", ");
        values.push((slot, rendered.clone(), rendered));
    }
    Some(render_template_values(&template, &values))
}

/// A placeholder the seed sentence wraps in backticks becomes a `CommonMark`
/// code span: a value holding a backtick run gets a longer fence, padded when
/// it starts or ends with a backtick, so the inserted text reads back verbatim.
fn code_span_item(value: &str) -> String {
    let longest = value
        .split(|character| character != '`')
        .map(str::len)
        .max()
        .unwrap_or(0);
    if longest == 0 {
        return format!("`{value}`");
    }
    let fence = "`".repeat(longest + 1);
    let pad = if value.starts_with('`') || value.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{fence}{pad}{value}{pad}{fence}")
}

fn code_span(value: &str) -> String {
    code_span_item(value)
}

fn render_template(template: &str, substitutions: &[(&str, &str)]) -> String {
    let values: Vec<(&str, String, String)> = substitutions
        .iter()
        .map(|(slot, value)| (*slot, (*value).to_owned(), code_span(value)))
        .collect();
    render_template_values(template, &values)
}

/// Substitute only seed-owned slots; inserted literal values are never templates.
fn render_template_values(mut template: &str, values: &[(&str, String, String)]) -> String {
    let mut rendered = String::new();
    while !template.is_empty() {
        let found = values.iter().find_map(|(slot, plain, quoted)| {
            let fenced = format!("`{slot}`");
            if template.starts_with(&fenced) {
                return Some((fenced.len(), quoted.as_str()));
            }
            template
                .starts_with(*slot)
                .then_some((slot.len(), plain.as_str()))
        });
        if let Some((length, value)) = found {
            rendered.push_str(value);
            template = &template[length..];
        } else {
            let character = template.chars().next().expect("nonempty template");
            rendered.push(character);
            template = &template[character.len_utf8()..];
        }
    }
    rendered
}

fn rust_path(task: &str) -> Option<String> {
    let end = task.find(".rs")?.checked_add(3)?;
    let start = task[..end]
        .char_indices()
        .rev()
        .take_while(|(_, character)| is_path_character(*character))
        .last()
        .map_or(0, |(index, _)| index);
    let path = task.get(start..end)?;
    (!path.is_empty()
        && !path.starts_with('/')
        && !path.split('/').any(|component| component == ".."))
    .then(|| path.to_owned())
}

const fn is_path_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | '/')
}

fn requested_identifier(task: &str, path: &str, kind: RustItemKind) -> Option<String> {
    let normalized = task.replacen(path, " ", 1).to_lowercase();
    if kind != RustItemKind::Constant
        && let Some(name) = slot_identifier(&normalized, seed::ROLE_CODING_NAME_SLOT)
        && valid_identifier(&name)
    {
        return Some(name);
    }
    if kind == RustItemKind::Function
        && let Some(meaning) = seed::lexicon().first_role_match("program_kind", &normalized)
    {
        for surface in meaning.words() {
            let pattern = format!(
                r"(?:^|[^A-Za-z_0-9]){}[ \u0009-\u000D]+([A-Za-z_][A-Za-z_0-9]*)",
                regex::escape(surface)
            );
            let expression = regex::RegexBuilder::new(&pattern)
                .case_insensitive(true)
                .build()
                .ok()?;
            if let Some(captured) = expression.captures(task)
                && valid_identifier(&captured[1])
            {
                return Some(captured[1].to_owned());
            }
        }
    }
    let without_path = task.replacen(path, "", 1);
    let mut candidates = identifier_tokens(&without_path)
        .filter(|candidate| valid_identifier(candidate))
        .collect::<Vec<_>>();
    if kind == RustItemKind::Constant {
        candidates.retain(|candidate| {
            candidate.chars().any(|character| character == '_')
                && candidate
                    .chars()
                    .all(|character| character.is_ascii_uppercase() || character == '_')
        });
    } else {
        candidates.retain(|candidate| candidate.contains('_'));
    }
    candidates.into_iter().next().map(str::to_owned)
}

fn slot_identifier(normalized: &str, role: &str) -> Option<String> {
    seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .find_map(|form| match form.slot() {
            Slot::Prefix => {
                let prefix = form.before_slot().trim().to_lowercase();
                let start = normalized.find(&prefix)?.checked_add(prefix.len())?;
                identifier_tokens(normalized.get(start..)?)
                    .next()
                    .map(str::to_owned)
            }
            Slot::Suffix => {
                let suffix = form.after_slot().trim().to_lowercase();
                let end = normalized.find(&suffix)?;
                identifier_tokens(normalized.get(..end)?)
                    .last()
                    .map(str::to_owned)
            }
            Slot::Circumfix => {
                let before = form.before_slot().trim().to_lowercase();
                let after = form.after_slot().trim().to_lowercase();
                let start = normalized.find(&before)?.checked_add(before.len())?;
                let remainder = normalized.get(start..)?;
                let end = remainder.find(&after)?;
                identifier_tokens(remainder.get(..end)?)
                    .next()
                    .map(str::to_owned)
            }
            Slot::Bare => None,
        })
}

fn identifier_tokens(text: &str) -> impl Iterator<Item = &str> {
    text.split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|token| !token.is_empty())
}

fn valid_identifier(identifier: &str) -> bool {
    identifier_domain::rust_identifier_is_valid(identifier)
}

fn numeric_literals(text: &str) -> Vec<String> {
    let chars = text.char_indices().collect::<Vec<_>>();
    let mut values = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        let (start, character) = chars[index];
        if !character.is_ascii_digit()
            || index.checked_sub(1).is_some_and(|prior| {
                chars[prior].1.is_ascii_alphanumeric() || chars[prior].1 == '_'
            })
        {
            index += 1;
            continue;
        }
        let mut end_index = index + 1;
        while end_index < chars.len()
            && (chars[end_index].1.is_ascii_digit() || chars[end_index].1 == '.')
        {
            end_index += 1;
        }
        if end_index < chars.len()
            && (chars[end_index].1.is_ascii_alphanumeric() || chars[end_index].1 == '_')
        {
            index = end_index + 1;
            continue;
        }
        let end = chars
            .get(end_index)
            .map_or(text.len(), |(offset, _)| *offset);
        let signed_start = if index > 0
            && chars[index - 1].1 == '-'
            && (index < 2
                || matches!(
                    chars[index - 2].1,
                    ' ' | '\t' | '\n' | '\r' | '\u{000b}' | '\u{000c}'
                )) {
            chars[index - 1].0
        } else {
            start
        };
        let value = text[signed_start..end].trim_end_matches('.');
        if !value.is_empty() {
            values.push(value.to_owned());
        }
        index = end_index;
    }
    values
}

pub(super) fn plan_verified_generated_source_step(
    raw_task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let task = unwrap_transport_quotes(raw_task);
    if super::write_request::compose_edit_request(task).is_some() {
        return None;
    }
    if super::general_planner::compose_general_change_plan(task).is_some_and(|plan| {
        plan.mode == super::general_planner::GeneralPlanMode::LiteralFile
            && super::general_planner::owns_literal_body(task, &plan.content)
    }) {
        return None;
    }
    if let Some(compound) =
        source_contract::source_registration_contract(task, rust_source_for_task)
    {
        return if compound["wholeRequestConsumed"].as_bool() == Some(true) {
            None
        } else {
            Some(super::final_result::record(
                AgenticPlan::Final(render_seeded_outcome(
                    "coding-source-authoring-contract-missing",
                    task,
                    "",
                )?),
                super::final_result::FinalDisposition::Gap,
                "source-registration-goal-coverage-unbound",
                result,
            ))
        };
    }
    let Some(artifact) = rust_source_for_task(task) else {
        let literal = super::general_planner::compose_general_change_plan(task);
        let semantic = literal.as_ref().is_some_and(|plan| {
            let body = plan.content.to_lowercase();
            plan.mode == super::general_planner::GeneralPlanMode::LiteralFile
                && crate::seed::lexicon()
                    .first_role_match("program_language_alias", &body)
                    .is_some_and(|meaning| meaning.slug == "program_language_rust")
                && crate::seed::lexicon()
                    .first_role_match("program_kind", &body)
                    .is_some_and(|meaning| meaning.slug == "function")
        });
        if !semantic {
            return None;
        }
        return Some(super::final_result::record(
            AgenticPlan::Final(render_seeded_outcome(
                "coding-source-authoring-contract-missing",
                task,
                "",
            )?),
            super::final_result::FinalDisposition::Gap,
            "source-description-goal-coverage-unbound",
            result,
        ));
    };
    if !source_contract::source_whitespace_supported(raw_task)
        || source_contract::source_description_contract(task, &artifact).is_none()
    {
        return Some(super::final_result::record(
            AgenticPlan::Final(render_seeded_outcome(
                "coding-source-authoring-contract-missing",
                task,
                "",
            )?),
            super::final_result::FinalDisposition::Gap,
            "source-description-goal-coverage-unbound",
            result,
        ));
    }
    plan_generated_source_step(task, messages, tool_names, result)
}

pub(super) fn verified_source_description(raw_task: &str) -> Option<serde_json::Value> {
    let task = unwrap_transport_quotes(raw_task);
    if super::write_request::compose_edit_request(task).is_some() {
        return None;
    }
    if super::general_planner::compose_general_change_plan(task).is_some_and(|plan| {
        plan.mode == super::general_planner::GeneralPlanMode::LiteralFile
            && super::general_planner::owns_literal_body(task, &plan.content)
    }) {
        return None;
    }
    if let Some(compound) =
        source_contract::source_registration_contract(task, rust_source_for_task)
    {
        return Some(compound);
    }
    if !source_contract::source_whitespace_supported(raw_task) {
        return None;
    }
    let artifact = rust_source_for_task(task)?;
    source_contract::source_description_contract(task, &artifact)
}
