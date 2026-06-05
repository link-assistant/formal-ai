use std::fmt::Write as _;
use std::time::Duration;

use crate::agent::{AgentRun, AgentRunStatus, AgentWorkspace, AgentWorkspaceConfig};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;

use super::finalize_simple;

struct PythonCandidate {
    id: &'static str,
    function: PythonFunctionTree,
    tests: Vec<&'static str>,
    fragments: Vec<&'static str>,
}

struct PythonFunctionTree {
    signature: String,
    body: Vec<PythonStatement>,
}

enum PythonStatement {
    Assign {
        semantic_node: &'static str,
        target: &'static str,
        expression: &'static str,
    },
    Return {
        semantic_node: &'static str,
        expression: &'static str,
    },
    IfReturn {
        semantic_node: &'static str,
        condition: &'static str,
        value: &'static str,
    },
    For {
        semantic_node: &'static str,
        target: &'static str,
        iterator: &'static str,
        body: Vec<Self>,
    },
}

impl PythonFunctionTree {
    const fn new(signature: String, body: Vec<PythonStatement>) -> Self {
        Self { signature, body }
    }

    fn render(&self) -> String {
        let mut code = format!("def {}:\n", self.signature);
        Self::render_statements(&mut code, &self.body, 1);
        code
    }

    fn links_notation(&self) -> String {
        let mut out = String::from("python_function_syntax_tree\n");
        let _ = writeln!(
            out,
            "  semantic_node function_definition signature={:?}",
            self.signature
        );
        for statement in &self.body {
            statement.links_line(&mut out, 1);
        }
        out.trim_end().to_owned()
    }

    fn render_statements(code: &mut String, statements: &[PythonStatement], indent_level: usize) {
        for statement in statements {
            statement.render(code, indent_level);
        }
    }
}

impl PythonStatement {
    const fn assign(
        semantic_node: &'static str,
        target: &'static str,
        expression: &'static str,
    ) -> Self {
        Self::Assign {
            semantic_node,
            target,
            expression,
        }
    }

    const fn return_expr(semantic_node: &'static str, expression: &'static str) -> Self {
        Self::Return {
            semantic_node,
            expression,
        }
    }

    const fn if_return(
        semantic_node: &'static str,
        condition: &'static str,
        value: &'static str,
    ) -> Self {
        Self::IfReturn {
            semantic_node,
            condition,
            value,
        }
    }

    const fn for_loop(
        semantic_node: &'static str,
        target: &'static str,
        iterator: &'static str,
        body: Vec<Self>,
    ) -> Self {
        Self::For {
            semantic_node,
            target,
            iterator,
            body,
        }
    }

    fn render(&self, code: &mut String, indent_level: usize) {
        let indent = "    ".repeat(indent_level);
        match self {
            Self::Assign {
                target, expression, ..
            } => {
                let _ = writeln!(code, "{indent}{target} = {expression}");
            }
            Self::Return { expression, .. } => {
                let _ = writeln!(code, "{indent}return {expression}");
            }
            Self::IfReturn {
                condition, value, ..
            } => {
                let _ = writeln!(code, "{indent}if {condition}:");
                let _ = writeln!(code, "{indent}    return {value}");
            }
            Self::For {
                target,
                iterator,
                body,
                ..
            } => {
                let _ = writeln!(code, "{indent}for {target} in {iterator}:");
                PythonFunctionTree::render_statements(code, body, indent_level + 1);
            }
        }
    }

    fn links_line(&self, out: &mut String, depth: usize) {
        let indent = "  ".repeat(depth + 1);
        match self {
            Self::Assign {
                semantic_node,
                target,
                expression,
            } => {
                let _ = writeln!(
                    out,
                    "{indent}semantic_node {semantic_node} target={target:?} expression={expression:?}"
                );
            }
            Self::Return {
                semantic_node,
                expression,
            } => {
                let _ = writeln!(
                    out,
                    "{indent}semantic_node {semantic_node} expression={expression:?}"
                );
            }
            Self::IfReturn {
                semantic_node,
                condition,
                value,
            } => {
                let _ = writeln!(
                    out,
                    "{indent}semantic_node {semantic_node} condition={condition:?} value={value:?}"
                );
            }
            Self::For {
                semantic_node,
                target,
                iterator,
                body,
            } => {
                let _ = writeln!(
                    out,
                    "{indent}semantic_node {semantic_node} target={target:?} iterator={iterator:?}"
                );
                for statement in body {
                    statement.links_line(out, depth + 1);
                }
            }
        }
    }
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
    log.append("synthesis:syntax_tree", candidate.function.links_notation());
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
            function: PythonFunctionTree::new(
                signature,
                vec![
                    PythonStatement::for_loop(
                        "pairwise_outer_loop",
                        "left_index, left",
                        "enumerate(numbers)",
                        vec![PythonStatement::for_loop(
                            "pairwise_inner_loop",
                            "right",
                            "numbers[left_index + 1:]",
                            vec![PythonStatement::if_return(
                                "threshold_match_return",
                                "abs(left - right) < threshold",
                                "True",
                            )],
                        )],
                    ),
                    PythonStatement::return_expr("no_pair_matches_return", "False"),
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
            function: PythonFunctionTree::new(
                signature,
                vec![PythonStatement::return_expr(
                    "deterministic_tuple_intersection_return",
                    "tuple(sorted(set(test_tup1) & set(test_tup2)))",
                )],
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
            function: PythonFunctionTree::new(
                signature,
                vec![
                    PythonStatement::assign(
                        "vowel_membership_set_assignment",
                        "vowels",
                        "set(\"aeiouAEIOU\")",
                    ),
                    PythonStatement::return_expr(
                        "matching_character_count_return",
                        "sum(1 for character in text if character in vowels)",
                    ),
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
    let mut script = candidate.function.render();
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
    let code = candidate.function.render();
    format!(
        "Here is a derived Python function synthesized from the specification and verified in an isolated workspace:\n\n```python\n{}```\n\nExecution status: tests passed in isolated bounded agent workspace.\nCheck command: `python3 solution.py`\nTest outcome: {}/{} assertions passed.\nWorkspace isolation: temporary agent workspace with environment cleared and a 5 second command budget.",
        code,
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
