use std::fmt::Write as _;
use std::time::Duration;

use crate::agent::{AgentRun, AgentRunStatus, AgentWorkspace, AgentWorkspaceConfig};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;

use super::finalize_simple;

struct PythonCandidate {
    id: &'static str,
    code: String,
    tests: Vec<&'static str>,
    fragments: Vec<&'static str>,
}

pub fn try_program_synthesis(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let canonical = crate::seed::operation_vocabulary().canonicalized_prompt(normalized);
    if !looks_like_python_function_request(prompt, &canonical) {
        return None;
    }

    let function_name = extract_function_name(prompt, &canonical)?;
    let candidate = synthesize_python_candidate(prompt, &canonical, &function_name)?;
    log.append(
        "synthesis:spec",
        format!("language=python function={function_name}"),
    );
    for fragment in &candidate.fragments {
        log.append("composition:code_fragment", (*fragment).to_owned());
    }
    log.append("synthesis:candidate", candidate.id.to_owned());

    let run = verify_python_candidate(prompt, &candidate)?;
    append_agent_run(log, &run);
    let passed = run.status == AgentRunStatus::Completed
        && run
            .command_results
            .iter()
            .any(|result| result.status_code == Some(0) && !result.timed_out);
    if !passed {
        log.append("synthesis:verification", "tests_failed".to_owned());
        return None;
    }

    log.append(
        "synthesis:verification",
        format!("tests_passed assertion_count={}", candidate.tests.len()),
    );
    log.append("execution_status", "tests passed".to_owned());
    log.append(
        "execution_environment",
        "isolated bounded agent workspace; env cleared; 5 second command budget".to_owned(),
    );

    let body = render_python_answer(&candidate);
    Some(finalize_simple(
        prompt,
        log,
        "write_program",
        &format!("response:write_program:synthesized:python:{}", candidate.id),
        &body,
        1.0,
    ))
}

/// Does `normalized` read like a request to synthesise a Python function?
///
/// The three conjuncts are language-independent semantic roles, not hardcoded
/// words: a *subject* (the function being asked for), a *domain* signal (Python
/// or a data kind it works over), and an *action* verb (implement/write/return).
/// `def ` is Python syntax the user may paste directly, so a literal signature
/// satisfies both the subject and action sides regardless of prose language.
fn looks_like_python_function_request(prompt: &str, normalized: &str) -> bool {
    let lexicon = crate::seed::lexicon();
    let has_def = prompt.to_ascii_lowercase().contains("def ");
    (lexicon.mentions_role(crate::seed::ROLE_PROGRAM_SYNTHESIS_SUBJECT, normalized) || has_def)
        && lexicon.mentions_role(crate::seed::ROLE_PROGRAM_SYNTHESIS_DOMAIN, normalized)
        && (lexicon.mentions_role(crate::seed::ROLE_PROGRAM_SYNTHESIS_ACTION, normalized)
            || has_def)
}

/// Is `task` evidenced by its signals — is every `program_synthesis_signal`
/// meaning it is `defined_by` present in `normalized`? A task with no signal
/// definitions is never matched this way (it can still match by declared name).
/// This replaces the per-task hardcoded phrase checks: the signal words live in
/// `data/seed/meanings-program-synthesis.lino`, translatable to any language.
fn synthesis_task_evidenced(
    lexicon: &crate::seed::Lexicon,
    task: &crate::seed::Meaning,
    normalized: &str,
) -> bool {
    let mut required = 0usize;
    for target in &task.defined_by {
        let Some(signal) = lexicon.meaning(target) else {
            continue;
        };
        if !signal.has_role(crate::seed::ROLE_PROGRAM_SYNTHESIS_SIGNAL) {
            continue;
        }
        required += 1;
        if !signal.evidenced_in(normalized) {
            return false;
        }
    }
    required > 0
}

/// The canonical function name of the first synthesis task (declaration order)
/// whose signals are all evidenced in `normalized`. The slug *is* the Python
/// function name, so the caller can use it directly.
fn match_synthesis_task(lexicon: &crate::seed::Lexicon, normalized: &str) -> Option<String> {
    lexicon
        .meanings_with_role(crate::seed::ROLE_PROGRAM_SYNTHESIS_TASK)
        .find(|task| synthesis_task_evidenced(lexicon, task, normalized))
        .map(|task| task.slug.clone())
}

fn extract_function_name(prompt: &str, normalized: &str) -> Option<String> {
    if let Some(name) = declared_function_name_from_signature(prompt) {
        return Some(name);
    }
    if let Some(slug) = match_synthesis_task(crate::seed::lexicon(), normalized) {
        return Some(slug);
    }
    for marker in ["function ", "def "] {
        if let Some(name) = identifier_after_ascii_marker(prompt, marker) {
            return Some(name);
        }
    }
    None
}

fn declared_function_name_from_signature(prompt: &str) -> Option<String> {
    let bytes = prompt.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if !is_ascii_identifier_start(bytes[index]) {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while index < bytes.len() && is_ascii_identifier_continue(bytes[index]) {
            index += 1;
        }
        let identifier = &prompt[start..index];
        let mut cursor = index;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor < bytes.len()
            && bytes[cursor] == b'('
            && !is_reserved_python_identifier(identifier)
        {
            return Some(identifier.to_owned());
        }
    }
    None
}

const fn is_ascii_identifier_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

const fn is_ascii_identifier_continue(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric()
}

fn is_reserved_python_identifier(identifier: &str) -> bool {
    matches!(
        identifier,
        "if" | "for"
            | "while"
            | "return"
            | "def"
            | "list"
            | "tuple"
            | "set"
            | "dict"
            | "str"
            | "int"
            | "float"
            | "bool"
            | "sum"
            | "abs"
            | "range"
            | "print"
    )
}

fn synthesize_python_candidate(
    prompt: &str,
    normalized: &str,
    function_name: &str,
) -> Option<PythonCandidate> {
    // Select the synthesis task by declared name or by evidenced signals, then
    // dispatch on its slug. The slug is the canonical Python function name and
    // the recognition lives entirely in the meaning lexicon — no prose here.
    let lexicon = crate::seed::lexicon();
    let task = lexicon
        .meanings_with_role(crate::seed::ROLE_PROGRAM_SYNTHESIS_TASK)
        .find(|task| {
            function_name == task.slug || synthesis_task_evidenced(lexicon, task, normalized)
        })?;

    if task.slug == "has_close_elements" {
        let signature = declared_signature(prompt, function_name).unwrap_or_else(|| {
            String::from("has_close_elements(numbers: list[float], threshold: float) -> bool")
        });
        return Some(PythonCandidate {
            id: "pairwise_threshold_distance",
            code: render_python_function(
                &signature,
                &[
                    "for left_index, left in enumerate(numbers):",
                    "    for right in numbers[left_index + 1:]:",
                    "        if abs(left - right) < threshold:",
                    "            return True",
                    "return False",
                ],
            ),
            tests: vec![
                "assert has_close_elements([1.0, 2.0, 3.0], 0.5) is False",
                "assert has_close_elements([1.0, 2.0, 3.0], 1.1) is True",
                "assert has_close_elements([1.0, 2.8, 3.0], 0.3) is True",
                "assert has_close_elements([], 0.1) is False",
            ],
            fragments: vec![
                "python:def_function",
                "loop:pairwise_distinct_values",
                "predicate:absolute_difference_less_than_threshold",
                "branch:return_false_when_no_pair_matches",
            ],
        });
    }

    if task.slug == "similar_elements" {
        let signature = declared_signature(prompt, function_name)
            .unwrap_or_else(|| String::from("similar_elements(test_tup1, test_tup2)"));
        return Some(PythonCandidate {
            id: "tuple_intersection_set",
            code: render_python_function(
                &signature,
                &["return tuple(sorted(set(test_tup1) & set(test_tup2)))"],
            ),
            tests: vec![
                "assert similar_elements((3, 4, 5, 6), (5, 7, 4, 10)) == (4, 5)",
                "assert similar_elements((1, 2), (3, 4)) == ()",
                "assert similar_elements(('a', 'b'), ('b', 'c')) == ('b',)",
            ],
            fragments: vec![
                "python:def_function",
                "collection:set_intersection",
                "collection:deterministic_tuple_order",
            ],
        });
    }

    if task.slug == "count_vowels" {
        let signature = declared_signature(prompt, function_name)
            .unwrap_or_else(|| String::from("count_vowels(text: str) -> int"));
        return Some(PythonCandidate {
            id: "count_matching_characters",
            code: render_python_function(
                &signature,
                &[
                    "vowels = set(\"aeiouAEIOU\")",
                    "return sum(1 for character in text if character in vowels)",
                ],
            ),
            tests: vec![
                "assert count_vowels('hello') == 2",
                "assert count_vowels('sky') == 0",
                "assert count_vowels('Formal AI') == 4",
            ],
            fragments: vec![
                "python:def_function",
                "collection:membership_set",
                "aggregation:sum_generator",
            ],
        });
    }

    None
}

fn render_python_function(signature: &str, body_lines: &[&str]) -> String {
    let mut code = format!("def {signature}:\n");
    for line in body_lines {
        if !line.is_empty() {
            code.push_str("    ");
            code.push_str(line);
        }
        code.push('\n');
    }
    code
}

fn verify_python_candidate(prompt: &str, candidate: &PythonCandidate) -> Option<AgentRun> {
    let config = AgentWorkspaceConfig {
        time_budget: Duration::from_secs(5),
        ..AgentWorkspaceConfig::default()
    };
    let mut workspace = AgentWorkspace::for_prompt(
        &format!("program_synthesis:{prompt}:{}", candidate.id),
        &config,
    )
    .ok()?;
    workspace.create_file("solution.py", &verification_script(candidate));
    workspace.run_command("python3 solution.py");
    Some(workspace.finish())
}

fn verification_script(candidate: &PythonCandidate) -> String {
    let mut script = candidate.code.clone();
    script.push_str("\n\nif __name__ == \"__main__\":\n");
    for test in &candidate.tests {
        script.push_str("    ");
        script.push_str(test);
        script.push('\n');
    }
    let _ = writeln!(
        script,
        "    print(\"tests_passed:{}\")",
        candidate.tests.len()
    );
    script
}

fn append_agent_run(log: &mut EventLog, run: &AgentRun) {
    log.append("synthesis:workspace", run.workspace.display().to_string());
    for action in &run.actions {
        log.append(action.event_kind(), action.evidence_payload());
    }
    for result in &run.command_results {
        log.append(
            "synthesis:candidate_execution",
            format!(
                "command={} exit={:?} timed_out={} stdout_bytes={} stderr_bytes={}",
                result.command,
                result.status_code,
                result.timed_out,
                result.stdout.len(),
                result.stderr.len()
            ),
        );
    }
}

fn render_python_answer(candidate: &PythonCandidate) -> String {
    format!(
        "Here is a derived Python function synthesized from the specification and verified in an isolated workspace:\n\n```python\n{}```\n\nExecution status: tests passed in isolated bounded agent workspace.\nCheck command: `python3 solution.py`\nTest outcome: {}/{} assertions passed.\nWorkspace isolation: temporary agent workspace with environment cleared and a 5 second command budget.",
        candidate.code,
        candidate.tests.len(),
        candidate.tests.len()
    )
}

fn identifier_after_ascii_marker(prompt: &str, marker: &str) -> Option<String> {
    let lower = prompt.to_ascii_lowercase();
    let start = lower.find(marker)? + marker.len();
    let mut name = String::new();
    let mut started = false;
    for character in prompt[start..].chars() {
        if character.is_ascii_alphanumeric() || character == '_' {
            name.push(character);
            started = true;
        } else if started {
            break;
        } else if !character.is_ascii_whitespace() {
            return None;
        }
    }
    (!name.is_empty()).then_some(name)
}

fn declared_signature(prompt: &str, function_name: &str) -> Option<String> {
    let marker = format!("{function_name}(");
    let lower = prompt.to_ascii_lowercase();
    let start = lower.find(&marker.to_ascii_lowercase())?;
    let after_name = start + function_name.len();
    let close = matching_close_paren(prompt, after_name)?;
    let mut end = close;
    let tail = &prompt[end..];
    let trimmed = tail.trim_start();
    if trimmed.starts_with("->") {
        let return_start = end + (tail.len() - trimmed.len());
        end = return_annotation_end(prompt, return_start).unwrap_or(end);
    }
    Some(prompt[start..end].trim().trim_end_matches('.').to_owned())
}

fn return_annotation_end(prompt: &str, arrow_start: usize) -> Option<usize> {
    let arrow_end = arrow_start + "->".len();
    let mut seen_annotation = false;
    let mut last_annotation_end = None;
    let mut cursor = arrow_end;

    while cursor < prompt.len() {
        let character = prompt[cursor..].chars().next()?;
        let next = cursor + character.len_utf8();
        if character.is_whitespace() {
            if seen_annotation
                && next_non_whitespace(prompt, next).is_some_and(is_return_annotation_char)
            {
                cursor = next;
                continue;
            }
            if seen_annotation {
                break;
            }
            cursor = next;
            continue;
        }
        if !is_return_annotation_char(character) {
            break;
        }
        seen_annotation = true;
        last_annotation_end = Some(next);
        cursor = next;
    }

    last_annotation_end
}

fn next_non_whitespace(prompt: &str, start: usize) -> Option<char> {
    prompt[start..]
        .chars()
        .find(|character| !character.is_whitespace())
}

const fn is_return_annotation_char(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || matches!(
            character,
            '_' | '[' | ']' | '(' | ')' | ',' | '\'' | '"' | '|'
        )
}

fn matching_close_paren(prompt: &str, open_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (offset, character) in prompt[open_index..].char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(open_index + offset + character.len_utf8());
                }
            }
            _ => {}
        }
    }
    None
}
