//! Checking a stated expectation before fixing (PR #1188 T93, gap G14).
//!
//! "Fix the bug in m.mjs: add should return the sum." on a correct module
//! fell to the unknown answer. A bug report that names a module and states
//! what one of its functions should return (the seeded
//! `coding_bug_fix_request` and `coding_expectation_cue` meanings, every
//! registered language) is checked first: the module is read, the expectation
//! is computed from the statement itself at the contract's sample arguments
//! (the calculator, as for an added function), the function is run at the
//! same arguments through the contract's seeded `probe` command, and the
//! observed value is compared. A function that meets the expectation is
//! reported as no defect found, from a seeded template; one that does not is
//! reported as confirmed, the file unchanged. Twin of
//! `js/agentic/function_expectation.mjs`.

use super::final_result::{FinalDisposition, FinalResult, record};
use serde_json::json;

use super::module_function::{
    contract, extension_language, paths_in, read_source, signature, stated_value,
};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use crate::protocol::ChatMessage;
use crate::seed;

const ROLE_BUG_FIX: &str = "coding_bug_fix_request";
const ROLE_EXPECTATION: &str = "coding_expectation_cue";
const NAME_SLOT: &str = concat!("{", "name", "}");
const SPEC_EDGES: &[char] = &[
    '.', ',', ';', ':', '!', '?', '。', '，', '！', '？', '：', '।',
];

/// A bug report with a stated expectation about a function of `module`.
struct ExpectationRequest {
    module: String,
    language: String,
    cue: (usize, usize),
}

/// The checked call of the function the request names.
struct CheckedCall {
    call: String,
    command: String,
    expected: String,
}

const fn is_identifier_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '$'
}

fn fill(template: &str, slots: &[(&str, &str)]) -> String {
    slots
        .iter()
        .fold(template.to_owned(), |text, (slot, value)| {
            text.replace(&["{", slot, "}"].concat(), value)
        })
}

/// The first char offset of `needle` in `haystack`.
fn find_chars(haystack: &[char], needle: &[char]) -> Option<usize> {
    (0..haystack.len())
        .take_while(|at| at + needle.len() <= haystack.len())
        .find(|&at| haystack[at..at + needle.len()] == *needle)
}

/// `chars[start..end]` without surrounding whitespace and sentence marks.
fn trimmed_span(chars: &[char], start: usize, end: usize) -> String {
    let edge = |character: char| character.is_whitespace() || SPEC_EDGES.contains(&character);
    let mut from = start;
    let mut to = end;
    while from < to && edge(chars[from]) {
        from += 1;
    }
    while to > from && edge(chars[to - 1]) {
        to -= 1;
    }
    chars[from..to].iter().collect()
}

/// The last identifier-bounded offset of `name` in `chars[..end]`.
fn last_name_offset(chars: &[char], name: &str, end: usize) -> Option<usize> {
    let needle: Vec<char> = name.chars().collect();
    let last = end.checked_sub(needle.len())?;
    (0..=last).rev().find(|&at| {
        chars[at..at + needle.len()] == *needle
            && (at == 0 || !is_identifier_char(chars[at - 1]))
            && chars
                .get(at + needle.len())
                .is_none_or(|next| !is_identifier_char(*next))
    })
}

/// A bug report that names a module with a probe contract and states an
/// expectation.
fn expectation_request(task: &str) -> Option<ExpectationRequest> {
    let lexicon = seed::lexicon();
    if !lexicon.mentions_role(ROLE_BUG_FIX, &crate::engine::normalize_prompt(task)) {
        return None;
    }
    let lower = super::workspace_search::lower_chars(&task.chars().collect::<Vec<_>>());
    let mut cue: Option<(usize, usize)> = None;
    for surface in lexicon.words_for_role(ROLE_EXPECTATION) {
        let needle: Vec<char> = surface.to_lowercase().chars().collect();
        if let Some(at) = find_chars(&lower, &needle)
            && cue.is_none_or(|(start, _)| at < start)
        {
            cue = Some((at, at + needle.len()));
        }
    }
    let cue = cue?;
    paths_in(task).into_iter().find_map(|module| {
        let language = extension_language(&module)?;
        let terms = contract(&language)?;
        (!terms.find_child_value("probe").is_empty()).then_some(ExpectationRequest {
            module,
            language,
            cue,
        })
    })
}

/// The checked call of the source's function the request names.
fn checked_call(task: &str, request: &ExpectationRequest, source: &str) -> Option<CheckedCall> {
    let terms = contract(&request.language)?;
    let definition = terms.find_child_value("definition");
    let prefix = definition.split(NAME_SLOT).next().unwrap_or_default();
    let chars: Vec<char> = task.chars().collect();
    let mut best: Option<(super::module_function::Signature, usize)> = None;
    for line in source.split('\n') {
        if !line.starts_with(prefix) {
            continue;
        }
        let Some(stated) = signature(line) else {
            continue;
        };
        let Some(at) = last_name_offset(&chars, &stated.name, request.cue.0) else {
            continue;
        };
        if best.as_ref().is_none_or(|(_, nearest)| at > *nearest) {
            best = Some((stated, at));
        }
    }
    let (stated, at) = best?;
    let after = trimmed_span(&chars, request.cue.1, chars.len());
    let spec = if after.is_empty() {
        trimmed_span(&chars, at + stated.name.chars().count(), request.cue.0)
    } else {
        after
    };
    let samples: Vec<String> = terms
        .find_child_value("samples")
        .split_whitespace()
        .take(stated.parameters.len())
        .map(str::to_owned)
        .collect();
    if samples.len() != stated.parameters.len() {
        return None;
    }
    let words: Vec<&str> = spec.split_whitespace().collect();
    let expected = stated_value(&words, &spec, &stated.parameters, &samples)?;
    let arguments = samples.join(", ");
    Some(CheckedCall {
        call: format!("{}({arguments})", stated.name),
        command: fill(
            terms.find_child_value("probe"),
            &[
                ("name", stated.name.as_str()),
                ("module", request.module.as_str()),
                ("arguments", arguments.as_str()),
            ],
        ),
        expected,
    })
}

/// The latest shell result of the current turn.
fn latest_run(messages: &[ChatMessage]) -> Option<String> {
    let current_turn = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .map_or(0, |index| index + 1);
    messages
        .iter()
        .enumerate()
        .skip(current_turn)
        .filter(|(index, message)| {
            message.role.eq_ignore_ascii_case("tool")
                && super::progress::result_capability(messages, *index) == Some(Capability::Run)
        })
        .map(|(_, message)| message.content.plain_text())
        .next_back()
}

/// The arm: read the module, run the named function at the contract's
/// samples, then answer whether it meets the stated expectation.
pub(super) fn plan_function_expectation_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let request = expectation_request(task)?;
    let current_turn = &messages[super::planner::evidence_window_start(messages)..];
    let Some(source) = read_source(current_turn, &request.module) else {
        let read = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(
            read,
            super::workspace_change::read_arguments(&request.module),
        ));
    };
    let checked = checked_call(task, &request, &source)?;
    let Some(raw) = latest_run(messages) else {
        let shell = tool_for(tool_names, Capability::Run)?;
        return Some(plan_one(
            shell,
            json!({ "command": checked.command }).to_string(),
        ));
    };
    let Some(payload) = super::tool_result::normalized_payload(&raw) else {
        return Some(record(
            AgenticPlan::Final(super::tool_result::render(&checked.command, &raw, task)),
            FinalDisposition::Failure,
            "function_probe_failed",
            result,
        ));
    };
    let observed = payload.trim();
    let intent = if observed == checked.expected {
        "function_expectation_holds"
    } else {
        "function_expectation_fails"
    };
    let values = [
        ("call", checked.call.as_str()),
        ("path", request.module.as_str()),
        ("observed", observed),
        ("expected", checked.expected.as_str()),
    ];
    let language = crate::language::detect(task).slug();
    seed::render_response(intent, language, &values)
        .or_else(|| seed::render_response(intent, "en", &values))
        .map(|text| {
            record(
                AgenticPlan::Final(text),
                FinalDisposition::Finding,
                intent,
                result,
            )
        })
}

/// The seeded question a test request earns when it states no expected result.
///
/// A request to write a test into a named file with no quoted literal, no
/// seeded expectation cue and no value names nothing the test could check, so
/// nothing is written and no interpreter is guessed (PR #1188 G25).
pub(super) fn test_expectation_question(
    task: &str,
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    if !crate::normal_markov::quoted_segment_spans(task).is_empty() {
        return None;
    }
    for sentence in super::shell_command_policy::sentences(task) {
        let paths = paths_in(sentence.text);
        let Some(path) = paths.first() else { continue };
        // Trailing acceptance goals do not own the already named source artifact.
        let Some(target_at) = sentence.text.find(path.as_str()) else {
            continue;
        };
        let prefix = crate::engine::normalize_prompt(&sentence.text[..target_at]);
        if !seed::lexicon().mentions_role("coding_test_artifact_kind", &prefix) {
            continue;
        }
        let prose = crate::engine::normalize_prompt(
            &paths.iter().fold(sentence.text.to_owned(), |text, named| {
                text.replace(named.as_str(), " ")
            }),
        );
        let lexicon = seed::lexicon();
        if prose.chars().any(|character| character.is_ascii_digit())
            || !lexicon.mentions_role("coding_request_verb", &prose)
            || !lexicon.mentions_role("coding_test_artifact_kind", &prose)
            || lexicon.mentions_role(ROLE_EXPECTATION, &prose)
        {
            continue;
        }
        return super::code_task::render_seeded_change("test_expectation_missing", task, path, &[])
            .map(|text| {
                record(
                    AgenticPlan::Final(text),
                    FinalDisposition::Clarification,
                    "test_expectation_missing",
                    result,
                )
            });
    }
    None
}

#[cfg(test)]
#[path = "../../tests/fixtures/test-target-ownership.rs"]
mod target_ownership_tests;
