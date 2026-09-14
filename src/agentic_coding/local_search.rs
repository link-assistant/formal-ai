//! Meanings-driven, stateful local path discovery (issue #840).
//!
//! An explicit local scope wins over generic web-search verbs. The planner
//! performs one observable shell action at a time, reads the result from the
//! transcript, widens only after an empty result, and reports the searched
//! scope plus any name discrepancy.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::json;

use super::planner::{plan_one, tool_for, AgenticPlan, Capability};
use crate::protocol::ChatMessage;
use crate::seed;

const DESKTOP_ROOT: &str = "${FORMAL_AI_DESKTOP_DIR:-$HOME/Desktop}";
const HOME_ROOT: &str = "${FORMAL_AI_HOME_DIR:-$HOME}";
const CURRENT_ROOT: &str = ".";
const PATH_SLOT: &str = concat!("{", "path", "}");
const REQUESTED_SLOT: &str = concat!("{", "requested", "}");
const ACTUAL_SLOT: &str = concat!("{", "actual", "}");
const SCOPE_SLOT: &str = concat!("{", "scope", "}");
const STAGE_SLOT: &str = concat!("{", "stage", "}");
const ITEMS_SLOT: &str = concat!("{", "items", "}");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchMode {
    Find,
    ListScope,
    ListContents,
    Type,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PathKind {
    Directory,
    File,
}

#[derive(Clone, Debug)]
pub(super) struct LocalSearchRequest {
    scope: &'static str,
    root: &'static str,
    mode: SearchMode,
    kind: Option<PathKind>,
    subject: String,
    tokens: Vec<String>,
}

pub(super) struct LocalSearchNarration {
    pub subject: String,
    pub intent: String,
}

/// Recognise and plan a local grounded-action loop.
pub(super) fn plan_local_search_step(
    messages: &[ChatMessage],
    tool_names: &[&str],
) -> Option<AgenticPlan> {
    let task = messages
        .iter()
        .rev()
        .find(|message| message.role.eq_ignore_ascii_case("user"))?
        .content
        .user_request_text();
    let request = request_for(&task)?;
    let language = crate::language::detect(&task).slug();
    if seed::lexicon().mentions_role(
        seed::ROLE_LOCAL_PATH_ROUTE_QUESTION,
        &crate::engine::normalize_prompt(&task),
    ) {
        return Some(AgenticPlan::Final(localized(
            "local_search_route_local",
            language,
        )));
    }

    let results = match run_results(messages, &task) {
        Ok(results) => results,
        Err(error) => return Some(AgenticPlan::Final(error)),
    };
    match request.mode {
        SearchMode::ListScope => {
            if let Some(output) = results.first() {
                return Some(AgenticPlan::Final(render_listing(
                    "local_search_scope_listing",
                    language,
                    request.root,
                    output,
                )));
            }
            run(tool_names, &scope_listing_command(&request))
        }
        SearchMode::ListContents => plan_contents(tool_names, &request, &results, language),
        SearchMode::Find | SearchMode::Type => plan_find(tool_names, &request, &results, language),
    }
}

pub(super) fn narration_for(prompt: &str) -> Option<LocalSearchNarration> {
    let request = request_for(prompt)?;
    Some(LocalSearchNarration {
        subject: if request.subject.is_empty() {
            String::new()
        } else {
            request.subject
        },
        intent: format!(
            "agentic_action_{}_{}",
            if request.mode == SearchMode::ListScope {
                "list"
            } else {
                "find"
            },
            request.scope
        ),
    })
}

/// Recognition is location-led: a local scope plus a local action is enough,
/// regardless of whether the verb also belongs to open-web search vocabulary.
pub(super) fn request_for(prompt: &str) -> Option<LocalSearchRequest> {
    let normalized = crate::engine::normalize_prompt(prompt);
    let file_literal = explicit_file_literal(prompt);
    let target_literal = quoted_literal(prompt).or_else(|| file_literal.clone());
    let lexicon = seed::lexicon();
    let (scope, root, scope_role) = [
        ("desktop", DESKTOP_ROOT, seed::ROLE_LOCAL_PATH_SCOPE_DESKTOP),
        ("home", HOME_ROOT, seed::ROLE_LOCAL_PATH_SCOPE_HOME),
        ("current", CURRENT_ROOT, seed::ROLE_LOCAL_PATH_SCOPE_CURRENT),
    ]
    .into_iter()
    .find(|(_, _, role)| lexicon.mentions_role(role, &normalized))?;

    let contents = role_matches(seed::ROLE_LOCAL_PATH_CONTENTS_REQUEST, &normalized);
    let type_request = role_matches(seed::ROLE_LOCAL_PATH_TYPE_REQUEST, &normalized);
    let list = role_matches(seed::ROLE_LOCAL_PATH_LIST_ACTION, &normalized);
    let find = role_matches(seed::ROLE_LOCAL_PATH_SEARCH_ACTION, &normalized);
    let route_question = role_matches(seed::ROLE_LOCAL_PATH_ROUTE_QUESTION, &normalized);
    if !contents && !type_request && !list && !find && !route_question {
        return None;
    }

    let mode = if contents {
        SearchMode::ListContents
    } else if type_request {
        SearchMode::Type
    } else if list && !find {
        SearchMode::ListScope
    } else {
        SearchMode::Find
    };
    // Generic current-directory listings already have mature `ls` and typed
    // list-directory routes. This recipe owns explicitly scoped Desktop/Home
    // listings and named current-directory discovery, not that fallback.
    if scope == "current" && mode == SearchMode::ListScope {
        return None;
    }
    // Scope phrases such as "in the current folder" describe where to search,
    // not what kind of path to search for.
    let mut kind_context = normalized.clone();
    strip_role(&mut kind_context, scope_role);
    let directory = longest_role_surface(seed::ROLE_LOCAL_PATH_DIRECTORY_KIND, &kind_context);
    let file = longest_role_surface(seed::ROLE_LOCAL_PATH_FILE_KIND, &kind_context);
    let kind = if mode == SearchMode::Type {
        None
    } else if mode == SearchMode::ListContents {
        // Listing a path's children is only a grounded operation for a
        // directory. Carry that constraint into discovery so a similarly
        // named file cannot be mistaken for the requested container.
        Some(PathKind::Directory)
    } else {
        match (directory, file) {
            (Some(directory), Some(file)) => Some(if directory >= file {
                PathKind::Directory
            } else {
                PathKind::File
            }),
            (Some(_), None) => Some(PathKind::Directory),
            (None, Some(_)) => Some(PathKind::File),
            (None, None) if file_literal.is_some() => Some(PathKind::File),
            (None, None) => None,
        }
    };

    let mut subject = normalized;
    for role in [
        seed::ROLE_LOCAL_PATH_ROUTE_QUESTION,
        seed::ROLE_LOCAL_PATH_CONTENTS_REQUEST,
        seed::ROLE_LOCAL_PATH_TYPE_REQUEST,
        seed::ROLE_LOCAL_PATH_SEARCH_ACTION,
        seed::ROLE_LOCAL_PATH_LIST_ACTION,
        scope_role,
        seed::ROLE_LOCAL_PATH_DIRECTORY_KIND,
        seed::ROLE_LOCAL_PATH_FILE_KIND,
        seed::ROLE_LOCAL_PATH_QUERY_NOISE,
    ] {
        strip_role(&mut subject, role);
    }
    let mut tokens = subject
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let subject = if let Some(target_literal) = target_literal {
        tokens = target_literal
            .split(|character: char| !character.is_alphanumeric())
            .filter(|token| !token.is_empty())
            .map(str::to_owned)
            .collect();
        target_literal
    } else {
        tokens.join("-")
    };
    if matches!(
        mode,
        SearchMode::Find | SearchMode::ListContents | SearchMode::Type
    ) && tokens.is_empty()
        && !route_question
    {
        return None;
    }

    Some(LocalSearchRequest {
        scope,
        root,
        mode,
        kind,
        subject,
        tokens,
    })
}

fn role_matches(role: &str, text: &str) -> bool {
    let lexicon = seed::lexicon();
    lexicon.mentions_role(role, text)
        || lexicon.role_word_forms(role).into_iter().any(|form| {
            let before = crate::engine::normalize_prompt(form.before_slot());
            let after = crate::engine::normalize_prompt(form.after_slot());
            match (before.is_empty(), after.is_empty()) {
                (false, false) => surface_position(text, &before).is_some_and(|position| {
                    surface_position(&text[position + before.len()..], &after).is_some()
                }),
                (false, true) => surface_present(text, &before),
                (true, false) => surface_present(text, &after),
                (true, true) => false,
            }
        })
}

fn surface_position(text: &str, surface: &str) -> Option<usize> {
    text.match_indices(surface)
        .find_map(|(position, _)| {
            let end = position + surface.len();
            let left = position == 0
                || text[..position]
                    .chars()
                    .next_back()
                    .is_some_and(|character| !character.is_alphanumeric());
            let right = end == text.len()
                || text[end..]
                    .chars()
                    .next()
                    .is_some_and(|character| !character.is_alphanumeric());
            (left && right).then_some(position)
        })
        .or_else(|| contains_cjk(surface).then(|| text.find(surface)).flatten())
}

fn explicit_file_literal(prompt: &str) -> Option<String> {
    prompt
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|character: char| {
                matches!(
                    character,
                    '"' | '\'' | '`' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | ':'
                )
            })
        })
        .find_map(|word| {
            let path = Path::new(word);
            let extension = path.extension()?.to_str()?;
            (!extension.is_empty()
                && extension.chars().all(char::is_alphanumeric)
                && extension.chars().count() <= 10)
                .then(|| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or(word)
                        .to_lowercase()
                })
        })
}

fn quoted_literal(prompt: &str) -> Option<String> {
    ['\'', '"', '`'].into_iter().find_map(|quote| {
        let positions = prompt
            .char_indices()
            .filter_map(|(index, character)| (character == quote).then_some(index))
            .collect::<Vec<_>>();
        positions.iter().enumerate().find_map(|(offset, opening)| {
            let left_boundary = *opening == 0
                || prompt[..*opening]
                    .chars()
                    .next_back()
                    .is_some_and(|character| !character.is_alphanumeric());
            if !left_boundary {
                return None;
            }
            positions[offset + 1..].iter().find_map(|closing| {
                let after = closing + quote.len_utf8();
                let right_boundary = after == prompt.len()
                    || prompt[after..]
                        .chars()
                        .next()
                        .is_some_and(|character| !character.is_alphanumeric());
                let transport_wrapper =
                    prompt[..*opening].trim().is_empty() && prompt[after..].trim().is_empty();
                let literal = prompt[*opening + quote.len_utf8()..*closing].trim();
                (right_boundary
                    && !transport_wrapper
                    && !literal.is_empty()
                    && literal.chars().count() <= 128
                    && literal.chars().any(char::is_alphanumeric))
                .then(|| literal.to_lowercase())
            })
        })
    })
}

fn strip_role(text: &mut String, role: &str) {
    let mut literals = seed::lexicon()
        .role_word_forms(role)
        .into_iter()
        .flat_map(|form| [form.before_slot(), form.after_slot()])
        .map(crate::engine::normalize_prompt)
        .filter(|literal| !literal.is_empty())
        .collect::<Vec<_>>();
    literals.sort_by_key(|literal| std::cmp::Reverse(literal.chars().count()));
    literals.dedup();
    for literal in literals {
        remove_surface(text, &literal);
    }
}

fn longest_role_surface(role: &str, text: &str) -> Option<usize> {
    seed::lexicon()
        .words_for_role(role)
        .into_iter()
        .filter(|surface| surface_present(text, surface))
        .map(|surface| surface.chars().count())
        .max()
}

fn remove_surface(text: &mut String, surface: &str) {
    if contains_cjk(surface) {
        *text = text.replace(surface, " ");
        return;
    }
    let mut search_from = 0;
    while let Some(relative) = text[search_from..].find(surface) {
        let start = search_from + relative;
        let end = start + surface.len();
        let left = start == 0 || text[..start].ends_with(char::is_whitespace);
        let right =
            end == text.len() || text[end..].chars().next().is_some_and(char::is_whitespace);
        if left && right {
            text.replace_range(start..end, " ");
            search_from = start + 1;
        } else {
            search_from = end;
        }
    }
}

fn surface_present(text: &str, surface: &str) -> bool {
    if contains_cjk(surface) {
        return text.contains(surface);
    }
    text == surface
        || text.starts_with(&format!("{surface} "))
        || text.ends_with(&format!(" {surface}"))
        || text.contains(&format!(" {surface} "))
}

fn contains_cjk(text: &str) -> bool {
    text.chars()
        .any(|character| matches!(character as u32, 0x3400..=0x9fff))
}

fn plan_find(
    tool_names: &[&str],
    request: &LocalSearchRequest,
    results: &[String],
    language: &str,
) -> Option<AgenticPlan> {
    if results.is_empty() {
        return run(tool_names, &exact_command(request));
    }
    if !results[0].trim().is_empty() {
        return answer_for_candidates(
            tool_names,
            request,
            &results[0],
            None,
            language,
            results.get(1),
        );
    }
    if results.len() == 1 && results[0].trim().is_empty() {
        return run(tool_names, &substring_command(request));
    }
    if !results[1].trim().is_empty() {
        return answer_for_candidates(
            tool_names,
            request,
            &results[1],
            Some("widened"),
            language,
            results.get(2),
        );
    }
    if results.len() == 2 {
        return run(tool_names, &inventory_command(request, false));
    }
    answer_for_candidates(
        tool_names,
        request,
        &results[2],
        Some("inventory"),
        language,
        results.get(3),
    )
}

fn answer_for_candidates(
    tool_names: &[&str],
    request: &LocalSearchRequest,
    output: &str,
    widened: Option<&str>,
    language: &str,
    metadata: Option<&String>,
) -> Option<AgenticPlan> {
    let Some(candidate) = best_candidate(output, request) else {
        return Some(AgenticPlan::Final(render_not_found(request, language)));
    };
    if request.mode == SearchMode::Type {
        if let Some(metadata) = metadata {
            return Some(AgenticPlan::Final(render_type(
                request, &candidate, metadata, language,
            )));
        }
        return run(tool_names, &metadata_command(&candidate));
    }
    if widened == Some("inventory")
        && let Some(expected) = request.kind {
            let Some(metadata) = metadata else {
                return run(tool_names, &metadata_command(&candidate));
            };
            let kind_matches = match expected {
                PathKind::Directory => metadata.trim_start().starts_with('d'),
                PathKind::File => metadata.trim_start().starts_with('-'),
            };
            if !kind_matches {
                let intent = match expected {
                    PathKind::Directory => "local_search_mismatch_requested_directory",
                    PathKind::File => "local_search_mismatch_requested_file",
                };
                let actual = basename(&candidate);
                return Some(AgenticPlan::Final(
                    localized(intent, language)
                        .replace(PATH_SLOT, &candidate)
                        .replace(REQUESTED_SLOT, &request.subject)
                        .replace(ACTUAL_SLOT, &actual)
                        .replace(SCOPE_SLOT, request.root),
                ));
            }
        }

    let actual = basename(&candidate);
    let exact = actual.eq_ignore_ascii_case(&request.subject);
    let intent = if exact {
        match request.kind {
            Some(PathKind::Directory) => "local_search_found_directory",
            Some(PathKind::File) => "local_search_found_file",
            None => "local_search_found_path",
        }
    } else {
        "local_search_found_closest"
    };
    let answer = localized(intent, language)
        .replace(PATH_SLOT, &candidate)
        .replace(REQUESTED_SLOT, &request.subject)
        .replace(ACTUAL_SLOT, &actual)
        .replace(SCOPE_SLOT, request.root)
        .replace(STAGE_SLOT, widened.unwrap_or("exact"));
    Some(AgenticPlan::Final(answer))
}

fn plan_contents(
    tool_names: &[&str],
    request: &LocalSearchRequest,
    results: &[String],
    language: &str,
) -> Option<AgenticPlan> {
    if results.is_empty() {
        return run(tool_names, &exact_command(request));
    }
    if results.len() == 1 && results[0].trim().is_empty() {
        return run(tool_names, &substring_command(request));
    }
    if results[0].trim().is_empty() && results[1].trim().is_empty() && results.len() == 2 {
        return run(tool_names, &inventory_command(request, true));
    }

    let (candidate_result, contents_index) = if !results[0].trim().is_empty() {
        (&results[0], 1)
    } else if !results[1].trim().is_empty() {
        (&results[1], 2)
    } else {
        (&results[2], 3)
    };
    let Some(candidate) = best_candidate(candidate_result, request) else {
        return Some(AgenticPlan::Final(render_not_found(request, language)));
    };
    if let Some(contents) = results.get(contents_index) {
        return Some(AgenticPlan::Final(render_listing(
            "local_search_contents_listing",
            language,
            &candidate,
            contents,
        )));
    }
    run(tool_names, &contents_command(&candidate))
}

fn run(tool_names: &[&str], command: &str) -> Option<AgenticPlan> {
    let tool = tool_for(tool_names, Capability::Run)?;
    Some(plan_one(tool, json!({ "command": command }).to_string()))
}

fn exact_command(request: &LocalSearchRequest) -> String {
    find_command(request, &request.subject, false)
}

fn substring_command(request: &LocalSearchRequest) -> String {
    let token = request
        .tokens
        .iter()
        .find(|token| token.chars().count() >= 4)
        .or_else(|| request.tokens.first())
        .map_or("", String::as_str);
    find_command(request, token, true)
}

fn find_command(request: &LocalSearchRequest, name: &str, substring: bool) -> String {
    let pattern = if substring {
        format!("*{name}*")
    } else {
        name.to_owned()
    };
    let predicate = kind_predicate(request.kind);
    format!(
        concat!("find \"{}\"", "{} -iname {} -print"),
        request.root,
        predicate,
        shell_quote(&pattern)
    )
}

fn inventory_command(request: &LocalSearchRequest, honor_kind: bool) -> String {
    let predicate = if honor_kind {
        kind_predicate(request.kind)
    } else {
        ""
    };
    format!(
        concat!("find \"{}\"", " -mindepth 1 -maxdepth 3{} -print"),
        request.root, predicate
    )
}

fn scope_listing_command(request: &LocalSearchRequest) -> String {
    format!(
        concat!("find \"{}\"", " -mindepth 1 -maxdepth 1{} -print"),
        request.root,
        kind_predicate(request.kind)
    )
}

fn contents_command(path: &str) -> String {
    format!(
        concat!("find {}", " -mindepth 1 -maxdepth 1 -print"),
        shell_quote(path)
    )
}

fn metadata_command(path: &str) -> String {
    format!("ls -ld -- {}", shell_quote(path))
}

const fn kind_predicate(kind: Option<PathKind>) -> &'static str {
    match kind {
        Some(PathKind::Directory) => " -type d",
        Some(PathKind::File) => " -type f",
        None => "",
    }
}

fn run_results(messages: &[ChatMessage], task: &str) -> Result<Vec<String>, String> {
    let current_turn = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .map_or(0, |index| index + 1);
    let mut results = Vec::new();
    for (index, message) in messages
        .iter()
        .enumerate()
        .skip(current_turn)
        .filter(|(_, message)| message.role.eq_ignore_ascii_case("tool"))
    {
        if super::progress::result_capability(messages, index) != Some(Capability::Run) {
            continue;
        }
        let raw = message.content.plain_text();
        let Some(payload) = super::tool_result::normalized_payload(&raw) else {
            return Err(super::tool_result::render("shell", &raw, task));
        };
        results.push(payload);
    }
    Ok(results)
}

fn best_candidate(output: &str, request: &LocalSearchRequest) -> Option<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|path| (similarity(path, &request.tokens), path))
        .filter(|(score, _)| *score > 0)
        .max_by(|left, right| left.0.cmp(&right.0).then_with(|| right.1.cmp(left.1)))
        .map(|(_, path)| path.to_owned())
}

fn similarity(path: &str, requested: &[String]) -> isize {
    let actual = basename(path);
    let actual = actual
        .split(|character: char| !character.is_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
        .collect::<BTreeSet<_>>();
    let requested = requested
        .iter()
        .map(|token| token.to_lowercase())
        .collect::<BTreeSet<_>>();
    let shared = requested.intersection(&actual).count().cast_signed();
    let different = requested
        .symmetric_difference(&actual)
        .count()
        .cast_signed();
    shared * 4 - different
}

fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_owned()
}

fn render_listing(intent: &str, language: &str, scope: &str, output: &str) -> String {
    localized(intent, language)
        .replace(SCOPE_SLOT, scope)
        .replace(ITEMS_SLOT, output.trim())
}

fn render_not_found(request: &LocalSearchRequest, language: &str) -> String {
    localized("local_search_not_found", language)
        .replace(REQUESTED_SLOT, &request.subject)
        .replace(SCOPE_SLOT, request.root)
}

fn render_type(
    request: &LocalSearchRequest,
    candidate: &str,
    metadata: &str,
    language: &str,
) -> String {
    let directory = metadata.trim_start().starts_with('d');
    let intent = if directory {
        "local_search_type_directory"
    } else {
        "local_search_type_file"
    };
    localized(intent, language)
        .replace(PATH_SLOT, candidate)
        .replace(REQUESTED_SLOT, &request.subject)
        .replace(SCOPE_SLOT, request.root)
}

fn localized(intent: &str, language: &str) -> String {
    seed::localized_response(intent, language).unwrap_or_default()
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
