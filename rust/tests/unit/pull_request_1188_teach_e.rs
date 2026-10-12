//! PR #1188 TEACH-E: edit-composer and line-operation gaps found by driving
//! Formal AI as a subagent (`experiments/formal_ai_subagent/gaps.md`, ledger
//! rows T150-T169). Twin of
//! `rust/tests/web/pull-request-1188-teach-e.test.mjs`.

use std::collections::BTreeMap;

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::normal_markov::quoted_segments;
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::source_fetch::sha256_hex;

const TOOLS: [&str; 7] = ["bash", "edit", "glob", "grep", "list", "read", "write"];

/// One driven session: the workspace afterwards, the bash commands and tools
/// it ran, and its final answer.
struct Run {
    files: BTreeMap<String, String>,
    tools: Vec<String>,
    commands: Vec<String>,
    answer: Option<String>,
}

fn path_of(arguments: &serde_json::Value) -> String {
    ["filePath", "file_path", "path"]
        .iter()
        .find_map(|key| arguments[key].as_str())
        .unwrap_or_default()
        .to_owned()
}

/// The Agent CLI's tools over an in-memory workspace; bash knows
/// `sha256sum --`, `cat`, `test [!] -e`, `mkdir -p` and `mv`.
fn execute(
    files: &mut BTreeMap<String, String>,
    tool: &str,
    arguments: &serde_json::Value,
) -> String {
    match tool {
        "read" => files
            .get(&path_of(arguments))
            .cloned()
            .unwrap_or_else(|| format!("Error: File not found: {}", path_of(arguments))),
        "write" => {
            let content = arguments["content"].as_str().unwrap_or_default().to_owned();
            files.insert(path_of(arguments), content);
            String::new()
        }
        "edit" => {
            let path = path_of(arguments);
            let text = files.get(&path).cloned().unwrap_or_default();
            let old = arguments["oldString"].as_str().unwrap_or_default();
            let new = arguments["newString"].as_str().unwrap_or_default();
            if text.matches(old).count() != 1 {
                return "Error: oldString not found in content".to_owned();
            }
            files.insert(path, text.replacen(old, new, 1));
            String::new()
        }
        "bash" => {
            let command = arguments["command"].as_str().unwrap_or_default();
            if let Some(path) = command.strip_prefix("sha256sum -- ") {
                let text = files.get(path).cloned().unwrap_or_default();
                return format!("{}  {path}\n", sha256_hex(text.as_bytes()));
            }
            if command.starts_with("mkdir -p -- ") {
                return String::new();
            }
            if let Some((from, to)) = command
                .strip_prefix("mv ")
                .and_then(|operands| operands.split_once(' '))
            {
                if let Some(text) = files.remove(from) {
                    files.insert(to.to_owned(), text);
                }
                return String::new();
            }
            if let Some(path) = command.strip_prefix("cat ") {
                return files.get(path).cloned().unwrap_or_default();
            }
            let (negated, path) = command.strip_prefix("test ! -e ").map_or_else(
                || (false, command.strip_prefix("test -e ")),
                |path| (true, Some(path)),
            );
            path.map_or_else(
                || format!("Error: {command} is not simulated"),
                |path| {
                    if files.contains_key(path) == negated {
                        "Output: \nError: \nExit Code: 1".to_owned()
                    } else {
                        String::new()
                    }
                },
            )
        }
        _ => format!("Error: {tool} is not simulated"),
    }
}

fn drive(prompt: &str, workspace: &[(&str, &str)]) -> Run {
    let mut run = Run {
        files: workspace
            .iter()
            .map(|(path, text)| ((*path).to_owned(), (*text).to_owned()))
            .collect(),
        tools: Vec::new(),
        commands: Vec::new(),
        answer: None,
    };
    let mut messages = vec![ChatMessage::user(prompt)];
    for index in 0..8 {
        let calls = match plan_chat_step(&messages, &TOOLS) {
            Some(AgenticPlan::ToolCalls(calls)) => calls,
            Some(AgenticPlan::Final(text)) => {
                run.answer = Some(text);
                break;
            }
            None => break,
        };
        let call = calls[0].clone();
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
        if call.tool == "bash" {
            run.commands
                .push(arguments["command"].as_str().unwrap_or_default().to_owned());
        }
        run.tools.push(call.tool.clone());
        let result = execute(&mut run.files, &call.tool, &arguments);
        let id = format!("call_{index}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(id, call.tool.clone(), result));
    }
    run
}

const NOTES: &str = "alpha\nx\nbox\nfox\n";

#[test]
fn g60_a_quoted_line_is_the_whole_line_that_equals_it() {
    let run = drive("Delete the line 'x' from f.md.", &[("f.md", NOTES)]);
    assert_eq!(run.tools, ["read", "edit", "bash"]);
    assert_eq!(run.files["f.md"], "alpha\nbox\nfox\n");
    assert_eq!(
        run.answer.as_deref(),
        Some("Removed `x` from `f.md` and observed the result.")
    );
    let ambiguous = drive(
        "Delete the line 'x' from f.md.",
        &[("f.md", "alpha\nbox\nfox\n")],
    );
    assert_eq!(ambiguous.files["f.md"], "alpha\nbox\nfox\n");
}

#[test]
fn g60_the_containment_cue_removes_every_line_containing_the_text() {
    for prompt in [
        "Delete the lines containing 'ox' from f.md.",
        "Delete the lines that contain 'ox' from f.md.",
    ] {
        let run = drive(prompt, &[("f.md", NOTES)]);
        assert_eq!(run.files["f.md"], "alpha\nx\n", "{prompt}");
        assert_eq!(
            run.answer.as_deref(),
            Some("Deleted 2 line(s) containing `ox` from `f.md` and observed the result.")
        );
    }
}

#[test]
fn g61_the_outer_quote_pair_is_the_payload_and_nothing_moves() {
    let prompt =
        "Append the line '- G31 \"Delete the line 'x' from f.md.\" removed lines' to g2.md.";
    assert_eq!(
        quoted_segments(prompt),
        ["- G31 \"Delete the line 'x' from f.md.\" removed lines"]
    );
    assert_eq!(quoted_segments("Replace 'a' with 'b' in f."), ["a", "b"]);
    assert_eq!(
        quoted_segments("Insert 'don't' after 'x' in f"),
        ["don't", "x"]
    );
    let run = drive(prompt, &[("g2.md", "# gaps\n"), ("f.md", "keep\n")]);
    assert!(
        !run.commands
            .iter()
            .any(|command| command.starts_with("mv ")),
        "{:?}",
        run.commands
    );
    assert_eq!(run.files["f.md"], "keep\n");
    assert_eq!(
        run.files["g2.md"],
        "# gaps\n- G31 \"Delete the line 'x' from f.md.\" removed lines\n"
    );
}

#[test]
fn g53_an_escape_inside_one_of_several_literals_is_content() {
    let run = drive(
        "Insert the lines '  text \"a\\nb\"' and '  row z' after the line '  row a' in f.lino.",
        &[("f.lino", "table t\n  row a\nend\n")],
    );
    assert_eq!(
        run.files["f.lino"],
        "table t\n  row a\n  text \"a\\nb\"\n  row z\nend\n"
    );
}

#[test]
fn g54_a_list_of_literals_is_appended_line_by_line() {
    let run = drive(
        "Append the lines '  row b', '  row c \"q\" d' to f.lino.",
        &[("f.lino", "table t\n  row a\n")],
    );
    assert!(!run.tools.iter().any(|tool| tool == "write"));
    assert_eq!(
        run.files["f.lino"],
        "table t\n  row a\n  row b\n  row c \"q\" d\n"
    );
    let joined = drive(
        "Append the lines 'b1' and 'b2' to f.txt.",
        &[("f.txt", "a\n")],
    );
    assert_eq!(joined.files["f.txt"], "a\nb1\nb2\n");
}

#[test]
fn g65_a_needle_quoting_an_escape_is_the_text_as_written() {
    let run = drive(
        "Replace «parts.join('\\n')» with «parts.join('\\n\\n')» in m.mjs.",
        &[("m.mjs", "const x = parts.join('\\n');\n")],
    );
    assert_eq!(run.files["m.mjs"], "const x = parts.join('\\n\\n');\n");
    let broken = drive("Replace 'a\\nb' with 'c' in f.txt.", &[("f.txt", "a\nb\n")]);
    assert_eq!(broken.files["f.txt"], "c\n");
}

#[test]
fn g50_another_files_contents_are_appended_without_cat() {
    let run = drive(
        "Append the contents of a.txt to the end of b.txt.",
        &[("a.txt", "a1\n  a2\n"), ("b.txt", "b1\n")],
    );
    assert_eq!(run.tools, ["read", "read", "edit", "bash"]);
    assert!(
        !run.commands
            .iter()
            .any(|command| command.starts_with("cat "))
    );
    assert_eq!(run.files["b.txt"], "b1\na1\n  a2\n");
    assert_eq!(run.files["a.txt"], "a1\n  a2\n");
    assert_eq!(
        run.answer.as_deref(),
        Some("Appended `a1\n  a2` to the end of `b.txt` and observed the result.")
    );
}

#[test]
fn g51_an_insert_places_the_source_after_the_anchor_its_context_names() {
    let run = drive(
        "Insert the contents of rows.txt after the line 'Y' that follows the line 'Z' in f.lino.",
        &[("rows.txt", "r1\nr2\n"), ("f.lino", "X\nY\nZ\nY\n")],
    );
    assert_eq!(run.files["f.lino"], "X\nY\nZ\nY\nr1\nr2\n");
    let block = drive(
        "Insert these lines after the line '  Y' that follows the line 'Z' in f.lino:\n```\n  r1\n```",
        &[("f.lino", "X\n  Y\nZ\n  Y\n")],
    );
    assert_eq!(block.files["f.lino"], "X\n  Y\nZ\n  Y\n  r1\n");
}

#[test]
fn g33_a_line_slice_answers_only_those_lines() {
    let source = "one\ntwo\nthree\nfour\n";
    for (prompt, expected) in [
        (
            "Show the last 2 lines of n.txt.",
            "Lines 3-4 of `n.txt`:\n\n```text\nthree\nfour\n```",
        ),
        (
            "Show the first 2 lines of n.txt.",
            "Lines 1-2 of `n.txt`:\n\n```text\none\ntwo\n```",
        ),
        (
            "Show lines 2-3 of n.txt.",
            "Lines 2-3 of `n.txt`:\n\n```text\ntwo\nthree\n```",
        ),
        (
            "Show the last line of n.txt.",
            "Lines 4-4 of `n.txt`:\n\n```text\nfour\n```",
        ),
        (
            "Show n.txt.",
            "Contents of `n.txt`:\n\n```text\none\ntwo\nthree\nfour\n```",
        ),
    ] {
        let run = drive(prompt, &[("n.txt", source)]);
        assert_eq!(run.tools, ["read"], "{prompt}");
        assert_eq!(run.answer.as_deref(), Some(expected), "{prompt}");
    }
}

const README: &str =
    "# Demo\n\nIntro.\n\n## Usage\n\nRun it.\n\n### Flags\n\n- a\n\n## License\n\nMIT\n";

#[test]
fn g35_an_insert_at_the_end_of_a_markdown_section() {
    let run = drive(
        "Insert the line 'See also docs.' at the end of the section '## Usage' in README.md.",
        &[("README.md", README)],
    );
    assert_eq!(run.tools, ["read", "edit", "bash"]);
    assert_eq!(
        run.files["README.md"],
        README.replace("- a\n", "- a\nSee also docs.\n")
    );
    assert_eq!(
        run.answer.as_deref(),
        Some("Inserted `See also docs.` after `- a` in `README.md` and observed the result.")
    );
    let start = drive(
        "Add the line 'First.' at the start of the section '## Usage' in README.md.",
        &[("README.md", README)],
    );
    assert_eq!(
        start.files["README.md"],
        README.replace("## Usage\n\n", "## Usage\n\nFirst.\n")
    );
    let fenced = drive(
        "Append the line 'x' to the end of the section '## A' in r.md.",
        &[("r.md", "## A\n\n```sh\n# comment\n```\n\n## B\n")],
    );
    assert_eq!(
        fenced.files["r.md"],
        "## A\n\n```sh\n# comment\n```\nx\n\n## B\n"
    );
}

#[test]
fn g37_a_rename_with_both_paths_quoted_moves_the_file() {
    for prompt in [
        "Rename the file 'a.txt' to 'b.txt'.",
        "Rename the file a.txt to b.txt.",
        "Rename the file \"a.txt\" to \"b.txt\".",
    ] {
        let run = drive(prompt, &[("a.txt", "x\n")]);
        assert!(
            run.commands
                .iter()
                .any(|command| command == "mv a.txt b.txt"),
            "{prompt}: {:?}",
            run.commands
        );
        assert_eq!(
            run.files.into_iter().collect::<Vec<_>>(),
            [("b.txt".to_owned(), "x\n".to_owned())]
        );
    }
    let inside = drive(
        "Rename the import './a.mjs' to './b.mjs' in m.mjs.",
        &[("m.mjs", "import x from './a.mjs';\n")],
    );
    assert_eq!(inside.files["m.mjs"], "import x from './b.mjs';\n");
}

#[test]
fn g62_a_backticked_compound_command_runs_as_written() {
    let run = drive(
        "Run `cat a.lino >> b.lino && cat c.lino >> d.lino`",
        &[
            ("a.lino", "a\n"),
            ("b.lino", "b\n"),
            ("c.lino", "c\n"),
            ("d.lino", "d\n"),
        ],
    );
    assert_eq!(run.tools.first().map(String::as_str), Some("bash"));
    assert_eq!(
        run.commands.first().map(String::as_str),
        Some("cat a.lino >> b.lino && cat c.lino >> d.lino")
    );
}

#[test]
fn g52_a_replace_inside_a_seven_thousand_character_line() {
    let long = format!(
        "    note \"{}Lowered from 37 to 36 on 2026-10-08 by the thirteenth {}\"",
        "word ".repeat(700),
        "tail ".repeat(700)
    );
    let source = format!("ratchet\n{long}\nend\n");
    for _ in 0..3 {
        let run = drive(
            "Replace 'Lowered from 37 to 36' with 'Lowered from 37 to 35' in d.lino.",
            &[("d.lino", source.as_str())],
        );
        assert_eq!(run.tools, ["read", "edit", "bash"]);
        assert_eq!(run.files["d.lino"], source.replace("37 to 36", "37 to 35"));
        assert_eq!(
            run.answer.as_deref(),
            Some(
                "Replaced `Lowered from 37 to 36` with `Lowered from 37 to 35` in `d.lino` and observed the result."
            )
        );
    }
}
