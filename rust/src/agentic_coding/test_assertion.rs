//! Adding one assertion to a test file (PR #1188 G13).
//!
//! "Add an assertion that add(2, 2) equals 4 to m.test.mjs." A request that
//! adds the seeded `coding_assertion_kind` concept to one named test file, and
//! states a call, the seeded `coding_assertion_equality_cue` and a value, is
//! written in the file's own assertion form -- the form of its last assertion
//! line, from the language's `assertion_styles` in
//! `data/meta/function-test-contracts.lino` -- on the line after it at its
//! indentation, and the file is then run with its seeded runner. The Rust
//! original of `js/agentic/test_assertion.mjs`.

use serde_json::json;

use super::code_task::{render_seeded_change, render_seeded_outcome};
use super::final_result::{FinalDisposition, FinalResult, record};
use super::intent_router::edit_arguments;
use super::module_function::{extension_language, paths_in, read_source};
use super::planner::{AgenticPlan, Capability, plan_one, tool_for};
use super::tool_result::{failure_message, render};
use super::workspace_change::{
    changed_lines_edit, read_arguments, result_for_command, result_for_edit, result_for_path,
};
use super::write_request::{bare_surfaces, clean_cue_token, clean_path_token, tokens};
use crate::protocol::ChatMessage;
use crate::seed::{self, parser::LinoNode, parser::parse_lino};

const CONTRACTS: &str = include_str!("../../embedded/data/meta/function-test-contracts.lino");
const ACTUAL_SLOT: &str = concat!("{", "actual", "}");
const EXPECTED_SLOT: &str = concat!("{", "expected", "}");
const ASSERTION_SLOT: &str = concat!("{", "assertion", "}");
const COMMAND_SLOT: &str = concat!("{", "command", "}");
const STOP_ROLES: &[&str] = &[
    seed::ROLE_FILE_EDIT_NEW_LEAD_CUE,
    seed::ROLE_FILE_EDIT_TARGET_CUE,
    "file_write_destination_cue",
    "file_edit_joiner_cue",
];
const CUE_ROLES: &[&str] = &["coding_assertion_equality_cue", "coding_expectation_cue"];
const CLAUSE_ENDS: &[char] = &[',', ';', '，', '；'];
const STEM_SLOT: &str = concat!("{", "stem", "}");
const NAME_SLOT: &str = concat!("{", "name", "}");
const VALUE_EDGES: &[char] = &['.', ',', ';', ':', '!', '?', '。', '，', '！', '？'];
const WORKSPACE_TEST_COMMAND: &str = "formal-ai:workspace-test";

/// The last call written in `text` the way a request states one: `add(2, 2)`.
///
/// A name, then parentheses that hold no parentheses (mirrors `lastCall`).
fn last_call(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    let named = |byte: u8| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'.');
    text.match_indices('(')
        .filter_map(|(open, _)| {
            let close = open + 1 + text[open + 1..].find(')')?;
            if text[open + 1..close].contains('(') {
                return None;
            }
            let end = text[..open].trim_end().len();
            let mut start = end;
            while start > 0 && named(bytes[start - 1]) {
                start -= 1;
            }
            while start < end && (bytes[start].is_ascii_digit() || bytes[start] == b'.') {
                start += 1;
            }
            (start < end).then(|| &text[start..=close])
        })
        .next_back()
}

/// An assertion a request states: the test file, its language, the call and
/// the value the call should give.
struct AssertionRequest {
    path: String,
    language: String,
    actual: String,
    expected: String,
}

/// The seeded `assertion_styles` record of `language`.
fn styles_record(language: &str) -> Option<LinoNode> {
    parse_lino(CONTRACTS)
        .children
        .first()?
        .children
        .iter()
        .find(|node| node.name == "assertion_styles" && node.id == language)
        .cloned()
}

/// The seeded assertion forms of `language` (mirrors `assertionStyles`).
fn assertion_styles(language: &str) -> Vec<String> {
    styles_record(language)
        .map(|group| {
            group
                .children
                .iter()
                .filter(|node| node.name == "style")
                .map(|node| node.id.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// A new test file of `language` at `path` holding the one assertion.
///
/// From the seeded `new_file` template, importing `name` from the module the
/// seeded `test_name` pattern names (mirrors `newTestFile`).
fn new_test_file(language: &str, path: &str, name: &str, assertion: &str) -> Option<String> {
    let record = styles_record(language)?;
    let (prefix, suffix) = record.find_child_value("test_name").split_once(STEM_SLOT)?;
    let file = path.rsplit('/').next().unwrap_or(path);
    if file.len() <= prefix.len() + suffix.len() {
        return None;
    }
    let stem = file.strip_prefix(prefix)?.strip_suffix(suffix)?;
    let template = record.find_child_value("new_file");
    (!template.is_empty()).then(|| {
        template
            .replace(STEM_SLOT, stem)
            .replace(NAME_SLOT, name)
            .replace(ASSERTION_SLOT, assertion)
    })
}

/// The assertion a request adds to one named test file, if it states one.
///
/// The call before the seeded equality cue, and the value after it (mirrors
/// `assertionRequest`).
fn assertion_request(task: &str) -> Option<AssertionRequest> {
    let normalized = crate::engine::normalize_prompt(task).to_lowercase();
    let lexicon = seed::lexicon();
    // `Add an assertion that … to f`, or `Create a test in f that …` (G25).
    let adds = lexicon.mentions_role("coding_member_add_action", &normalized)
        && lexicon.mentions_role("coding_assertion_kind", &normalized);
    let writes = lexicon.mentions_role("coding_request_verb", &normalized)
        && lexicon.mentions_role("coding_test_artifact_kind", &normalized);
    if !adds && !writes {
        return None;
    }
    let [path]: [String; 1] = paths_in(task).try_into().ok()?;
    let language = extension_language(&path)?;
    if assertion_styles(&language).is_empty() {
        return None;
    }
    let lowered = task.to_lowercase();
    let lowered = if lowered.len() == task.len() {
        lowered
    } else {
        task.to_owned()
    };
    let cue = CUE_ROLES
        .iter()
        .flat_map(|role| lexicon.words_for_role(role))
        .filter_map(|surface| {
            let surface = surface.to_lowercase();
            lowered.find(&surface).map(|at| (at, at + surface.len()))
        })
        .min_by_key(|(at, _)| *at)?;
    let actual = last_call(task.get(..cue.0)?)?.to_owned();
    let stops: Vec<String> = STOP_ROLES
        .iter()
        .flat_map(|role| bare_surfaces(role))
        .collect();
    let mut value = Vec::new();
    for token in tokens(task)
        .into_iter()
        .filter(|token| token.start >= cue.1)
    {
        if clean_path_token(token.text) == path || stops.contains(&clean_cue_token(token.text)) {
            break;
        }
        let ends = token.text.ends_with(CLAUSE_ENDS);
        value.push(token);
        if ends {
            break;
        }
    }
    let (first, last) = (value.first()?, value.last()?);
    let expected = task
        .get(first.start..last.end)?
        .trim_matches(|character: char| {
            character.is_whitespace() || VALUE_EDGES.contains(&character)
        })
        .to_owned();
    (!expected.is_empty()).then_some(AssertionRequest {
        path,
        language,
        actual,
        expected,
    })
}

/// The file's last line written in one of `styles`.
///
/// Its index, the style (the longest lead when several fit) and its
/// indentation (mirrors `lastAssertion`).
fn last_assertion<'a>(source: &'a str, styles: &[String]) -> Option<(usize, String, &'a str)> {
    let count = source.split('\n').count();
    (0..count)
        .rev()
        .zip(source.split('\n').rev())
        .find_map(|(index, line)| {
            let line = line.strip_suffix('\r').unwrap_or(line);
            let text = line.trim();
            styles
                .iter()
                .filter_map(|style| {
                    let (head, rest) = style.split_once(ACTUAL_SLOT).unwrap_or((style, ""));
                    let (middle, tail) = rest.split_once(EXPECTED_SLOT).unwrap_or((rest, ""));
                    let fits = text.starts_with(head)
                        && text.ends_with(tail)
                        && text.len() >= head.len() + middle.len() + tail.len()
                        && text[head.len()..text.len() - tail.len()].contains(middle);
                    fits.then_some((head.len(), style))
                })
                .max_by_key(|(lead, _)| *lead)
                .map(|(_, style)| {
                    (
                        index,
                        style.clone(),
                        &line[..line.len() - line.trim_start().len()],
                    )
                })
        })
}

/// Read the test file, add the assertion in its own form, run the file.
///
/// The answer states both (mirrors `planTestAssertionStep`).
pub(super) fn plan_test_assertion_step(
    task: &str,
    messages: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let request = assertion_request(task)?;
    let current_turn = &messages[super::planner::evidence_window_start(messages)..];
    let Some(source) = read_source(current_turn, &request.path) else {
        let read = tool_for(tool_names, Capability::Read)?;
        return Some(plan_one(read, read_arguments(&request.path)));
    };
    if source.is_empty() {
        return plan_new_test_file(task, &request, current_turn, tool_names, result);
    }
    let (index, style, indentation) =
        last_assertion(&source, &assertion_styles(&request.language))?;
    let written = style
        .replace(ACTUAL_SLOT, &request.actual)
        .replace(EXPECTED_SLOT, &request.expected);
    let mut lines: Vec<String> = source.split('\n').map(str::to_owned).collect();
    lines.insert(index + 1, [indentation, written.as_str()].concat());
    let updated = lines.join("\n");
    let (old, new) = changed_lines_edit(&source, &updated)?;
    let edit_tool = tool_for(tool_names, Capability::Edit)?;
    let Some(edited) = result_for_edit(current_turn, &request.path, &old, &new) else {
        return Some(plan_one(
            edit_tool,
            edit_arguments(&request.path, &old, &new),
        ));
    };
    if failure_message(&edited, false, true).is_some() {
        return render_seeded_outcome("coding_workspace_verification_failed", task, &request.path)
            .map(AgenticPlan::Final);
    }
    plan_run_step(
        task,
        &request.path,
        current_turn,
        tool_names,
        ("test_assertion_added", &written),
        result,
    )
}

/// Write a missing test file whole, around the one assertion, then run it.
///
/// The language's new-file form (PR #1188 G25; mirrors `planNewTestFile`).
fn plan_new_test_file(
    task: &str,
    request: &AssertionRequest,
    current_turn: &[ChatMessage],
    tool_names: &[&str],
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let style = assertion_styles(&request.language).into_iter().next()?;
    let written = style
        .replace(ACTUAL_SLOT, &request.actual)
        .replace(EXPECTED_SLOT, &request.expected);
    let name = request
        .actual
        .split_once('(')
        .map_or(request.actual.as_str(), |(name, _)| name)
        .trim();
    let content = new_test_file(&request.language, &request.path, name, &written)?;
    let write_tool = tool_for(tool_names, Capability::Write)?;
    let Some(written_result) = result_for_path(
        current_turn,
        Capability::Write,
        &request.path,
        Some(content.as_str()),
    ) else {
        return Some(plan_one(
            write_tool,
            super::planner::write_arguments(&request.path, &content),
        ));
    };
    if failure_message(&written_result, false, true).is_some() {
        return render_seeded_outcome("coding_workspace_verification_failed", task, &request.path)
            .map(AgenticPlan::Final);
    }
    plan_run_step(
        task,
        &request.path,
        current_turn,
        tool_names,
        ("test_file_written", &written),
        result,
    )
}

/// Run the test file with its seeded runner, then state the change and the run.
///
/// `stated` is the change's intent and its assertion (mirrors `planRunStep`).
fn plan_run_step(
    task: &str,
    path: &str,
    current_turn: &[ChatMessage],
    tool_names: &[&str],
    stated: (&str, &str),
    result: &mut Option<FinalResult>,
) -> Option<AgenticPlan> {
    let run = super::test_file_runner::test_file_command(WORKSPACE_TEST_COMMAND, Some(path))?;
    let shell = tool_for(tool_names, Capability::Run)?;
    let Some(raw) = result_for_command(current_turn, &run) else {
        return Some(plan_one(shell, json!({ "command": run }).to_string()));
    };
    let change = render_seeded_change(
        stated.0,
        task,
        path,
        &[(ASSERTION_SLOT, stated.1), (COMMAND_SLOT, &run)],
    )?;
    let disposition =
        if super::tool_result::step_outcome(&raw) == super::tool_result::StepOutcome::Failed {
            FinalDisposition::Failure
        } else {
            FinalDisposition::Finding
        };
    Some(record(
        AgenticPlan::Final(format!("{change}\n\n{}", render(&run, &raw, task))),
        disposition,
        "test_assertion_observed",
        result,
    ))
}
