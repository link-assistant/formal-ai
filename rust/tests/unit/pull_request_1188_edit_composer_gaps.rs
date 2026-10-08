//! PR #1188 TEACH-D: the edit-composer gaps Formal AI showed as a subagent
//! (`experiments/formal_ai_subagent/gaps.md` G16, G19, G20, G21, G22, G24,
//! G28, G31). Before: lines under an insert request lost their indentation
//! (G16); an ordinal line was only read (G19); a Spanish line operation
//! answered in English (G20); a quoted path-shaped anchor was taken for the
//! file (G21); `with these three lines in f:` replaced the line by the words
//! (G22); a file rename rewrote the word `the` (G24); an empty line beside
//! an anchor only read the file (G31). Twin of
//! `rust/tests/web/pull-request-1188-edit-composer-gaps.test.mjs`.

use formal_ai::agentic_coding::general_planner::compose_edit_request;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::source_fetch::sha256_hex;

const TOOLS: [&str; 4] = ["read", "edit", "bash", "write"];
const THREE: &str = "one\ntwo\nthree\n";
const LINO: &str = "meanings\n  table a\n    row x\n  b\nc\n";

/// The calls made (a shell call by its command), the file afterwards, and the
/// final answer.
struct Run {
    calls: Vec<String>,
    file: String,
    answer: Option<String>,
}

/// Run `prompt` over one file `name` holding `source`; the tools act on it the
/// way the Agent CLI's do.
fn drive(prompt: &str, source: &str, name: &str) -> Run {
    let mut file = source.to_owned();
    let mut messages = vec![ChatMessage::user(prompt)];
    let mut calls = Vec::new();
    let mut answer = None;
    for index in 0..6 {
        let planned = match plan_chat_step(&messages, &TOOLS) {
            Some(AgenticPlan::ToolCalls(planned)) => planned,
            Some(AgenticPlan::Final(text)) => {
                answer = Some(text);
                break;
            }
            None => break,
        };
        let call = planned[0].clone();
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
        let result = match call.tool.as_str() {
            "read" => file.clone(),
            "edit" => {
                let old = arguments["oldString"].as_str().unwrap_or_default();
                let new = arguments["newString"].as_str().unwrap_or_default();
                file = file.replacen(old, new, 1);
                String::new()
            }
            "write" => {
                file = arguments["content"].as_str().unwrap_or_default().to_owned();
                String::new()
            }
            "bash" => format!("{}  {name}\n", sha256_hex(file.as_bytes())),
            _ => String::new(),
        };
        calls.push(if call.tool == "bash" {
            arguments["command"].as_str().unwrap_or_default().to_owned()
        } else {
            call.tool.clone()
        });
        let id = format!("call_{index}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(id, call.tool.clone(), result));
    }
    Run {
        calls,
        file,
        answer,
    }
}

#[test]
fn lines_under_a_request_keep_their_structure() {
    let rebased = drive(
        "Insert the following lines after the line 'b' in f.lino:\ntable q\n  row 1",
        LINO,
        "f.lino",
    );
    assert_eq!(
        rebased.file,
        "meanings\n  table a\n    row x\n  b\n  table q\n    row 1\nc\n"
    );
    let fenced = drive(
        "Insert these lines after the line 'b' in f.lino:\n```\ntable q\n```",
        LINO,
        "f.lino",
    );
    assert_eq!(
        fenced.file,
        "meanings\n  table a\n    row x\n  b\ntable q\nc\n"
    );
    let appended = drive(
        "Append these lines to f.lino:\n  table z\n    row 9",
        LINO,
        "f.lino",
    );
    assert_eq!(appended.file, [LINO, "  table z\n    row 9\n"].concat());
}

#[test]
fn a_line_named_by_its_ordinal() {
    for (prompt, expected) in [
        ("Remove the first line of f.txt.", "two\nthree\n"),
        ("Delete the last line of f.txt.", "one\ntwo\n"),
        ("Удали вторую строку из f.txt.", "one\nthree\n"),
        ("f.txt की तीसरी पंक्ति हटाओ।", "one\ntwo\n"),
        ("删除 f.txt 的最后一行。", "one\ntwo\n"),
        ("Elimina la primera línea de f.txt.", "two\nthree\n"),
    ] {
        assert_eq!(drive(prompt, THREE, "f.txt").file, expected, "{prompt}");
    }
    let last = drive("Delete the last line of f.txt.", THREE, "f.txt");
    assert_eq!(
        last.answer.as_deref(),
        Some("Deleted line(s) 3 of `f.txt` and observed the result.")
    );
    let inserted = drive(
        "Insert the line 'zero' before the first line of f.txt.",
        THREE,
        "f.txt",
    );
    assert_eq!(inserted.file, "zero\none\ntwo\nthree\n");
    assert_eq!(
        inserted.answer.as_deref(),
        Some("Inserted `zero` before line 1 of `f.txt` and observed the result.")
    );
}

#[test]
fn the_answer_is_in_the_language_of_the_request() {
    let removed = drive("Elimina las líneas 2 a 3 de f.txt.", THREE, "f.txt");
    assert_eq!(
        removed.answer.as_deref(),
        Some("Se eliminaron las líneas 2-3 de `f.txt` y se verificó el resultado.")
    );
    let swapped = drive(
        "Intercambia las líneas 'one' y 'two' en f.txt.",
        THREE,
        "f.txt",
    );
    assert_eq!(swapped.file, "two\none\nthree\n");
    assert_eq!(
        swapped.answer.as_deref(),
        Some("Se intercambiaron `one` y `two` en `f.txt` y se verificó el resultado.")
    );
    let english = drive("Delete lines 2-3 from f.txt.", THREE, "f.txt");
    assert_eq!(
        english.answer.as_deref(),
        Some("Deleted line(s) 2-3 of `f.txt` and observed the result.")
    );
}

#[test]
fn a_quoted_path_shaped_anchor_is_not_the_file() {
    let run = drive(
        "Insert the line 'foo' after the line 'js/ocr.bundle.js' in paths.txt.",
        "x\njs/ocr.bundle.js\ny\n",
        "paths.txt",
    );
    assert_eq!(run.calls, ["read", "edit", "sha256sum -- paths.txt"]);
    assert_eq!(run.file, "x\njs/ocr.bundle.js\nfoo\ny\n");
}

#[test]
fn a_new_clause_describing_lines_takes_the_lines_under_the_request() {
    let run = drive(
        "Replace the line 'const X = 2;' with these three lines in f.mjs:\nconst X = 2;\nconst Y = 3;\nconst Z = 4;",
        "function f() {\n  const X = 2;\n  return X;\n}\n",
        "f.mjs",
    );
    assert_eq!(
        run.file,
        "function f() {\n  const X = 2;\n  const Y = 3;\n  const Z = 4;\n  return X;\n}\n"
    );
}

#[test]
fn renames_of_files_and_functions() {
    assert_eq!(
        compose_edit_request("Rename the file m.py to math_utils.py."),
        None
    );
    let messages = [ChatMessage::user("Rename the file m.py to math_utils.py.")];
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("the move is planned");
    };
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("tool arguments are JSON");
    assert_eq!(arguments["command"], "test -e m.py");
    let source = "export function add(a, b) {\n  return a + b;\n}\nexport function mul(a, b) {\n  return a * b;\n}\n";
    let run = drive(
        "Rename the function mul to multiply in src/m.mjs.",
        source,
        "src/m.mjs",
    );
    assert_eq!(
        run.file,
        source.replace("function mul(", "function multiply(")
    );
    assert_eq!(
        run.answer.as_deref(),
        Some("Renamed `mul` to `multiply` in `src/m.mjs` and observed the result.")
    );
}

#[test]
fn an_empty_line_beside_an_anchor_line() {
    let readme = "# T\ntext\n## Probe\nx\n";
    let before = drive(
        "Insert an empty line before the line '## Probe' in README.md.",
        readme,
        "README.md",
    );
    assert_eq!(before.file, "# T\ntext\n\n## Probe\nx\n");
    assert_eq!(
        before.answer.as_deref(),
        Some("Added an empty line to `README.md` and observed the result.")
    );
    let after = drive(
        "Add a blank line after the line 'text' in README.md.",
        readme,
        "README.md",
    );
    assert_eq!(after.file, "# T\ntext\n\n## Probe\nx\n");
}
