//! PR #1188 TEACH-F: the open gaps of `experiments/formal_ai_subagent/gaps.md`
//! taught to Formal AI (ledger rows T210-T229, T290-T299). Twin of
//! `rust/tests/web/pull-request-1188-teach-f.test.mjs`.

use std::collections::BTreeMap;

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::normal_markov::quote_fault;
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
/// `sha256sum --`, `cat` and a passing `node --test`.
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
            // The file operations a copy recipe runs (G82).
            if let Some(path) = command.strip_prefix("test -e ") {
                return if files.contains_key(path) {
                    String::new()
                } else {
                    "Output: \nError: \nExit Code: 1".to_owned()
                };
            }
            if let Some(path) = command.strip_prefix("test ! -e ") {
                return if files.contains_key(path) {
                    "Output: \nError: \nExit Code: 1".to_owned()
                } else {
                    String::new()
                };
            }
            if command == "mkdir -p -- ." {
                return String::new();
            }
            if let Some((from, to)) = command
                .strip_prefix("cp ")
                .and_then(|rest| rest.split_once(' '))
            {
                let text = files.get(from).cloned().unwrap_or_default();
                files.insert(to.to_owned(), text);
                return String::new();
            }
            // A suite slower than the driver's timeout comes back killed (G77).
            if command == "node --test slow.test.mjs" {
                return "Output: TAP version 13\nError: \nSignal: SIGTERM\nTimeout: 60000 ms"
                    .to_owned();
            }
            if command.starts_with("node --test ") {
                return "# tests 1\n# pass 1\n# fail 0\n".to_owned();
            }
            command.strip_prefix("cat ").map_or_else(
                || format!("Error: {command} is not simulated"),
                |path| files.get(path).cloned().unwrap_or_default(),
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
    for index in 0..16 {
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

const LEDGER: &str = "| T208 | a | b |\n| T209 | c | d |\n";

#[test]
fn g71_an_escaped_quote_inside_a_quoted_line_is_declined() {
    let prompt = "Insert the line '| T210 | asked \\'set the contents of note.txt to hello\\' | Pass |' after the line containing 'T209' in ledger.md.";
    let run = drive(prompt, &[("ledger.md", LEDGER)]);
    assert!(run.tools.is_empty(), "{:?}", run.tools);
    assert_eq!(run.files.keys().collect::<Vec<_>>(), ["ledger.md"]);
    assert_eq!(run.files["ledger.md"], LEDGER);
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "A backslash before a quote in this request (`\\'set the contents of note.txt t`) leaves its quoted text unpaired, so I cannot tell the quoted text from the instruction, and nothing was done. Quote the text without backslashes, using «» or backticks when it holds quotes of its own."
        )
    );
}

#[test]
fn g71_an_opening_quote_that_never_closes_is_declined_in_the_request_language() {
    let english = drive("Replace 'x with y in f.txt.", &[("f.txt", "x\n")]);
    assert!(english.tools.is_empty(), "{:?}", english.tools);
    assert_eq!(
        english.answer.as_deref(),
        Some(
            "A quote in this request opens and never closes (`'x with y in f.txt.`), so I cannot tell the quoted text from the instruction, and nothing was done. Close the quote, or use «» or backticks when the text holds quotes of its own."
        )
    );
    let russian = drive("Замени 'x на y в f.txt.", &[("f.txt", "x\n")]);
    assert_eq!(
        russian.answer.as_deref(),
        Some(
            "Кавычка в этом запросе открывается и не закрывается (`'x на y в f.txt.`), поэтому я не могу отличить цитату от инструкции, и ничего не сделано. Закройте кавычку или используйте «» или обратные апострофы, если в тексте есть свои кавычки."
        )
    );
}

#[test]
fn g71_quotes_that_pair_apostrophes_and_code_literals_are_not_faults() {
    for prompt in [
        "Replace '\\' with '/' in p.txt.",
        "Replace 'C:\\dir\\' with 'D:\\' in p.txt.",
        "Replace `\\'` with `'` in f.js.",
        "What's in the users' files? Don't guess.",
        "Insert the line «a 'b' \"c\"» after the line 'x' in f.md.",
        "Append the line '- G1 \"Delete the line 'x'.\" -' to gaps.md.",
        "The 5\" screen and the 6' wall.",
        "rock 'n' roll",
    ] {
        assert_eq!(quote_fault(prompt), None, "{prompt}");
    }
}

#[test]
fn g71_the_same_request_with_guillemets_is_carried_out() {
    let run = drive(
        "Insert the line «| T210 | asked 'set the contents of note.txt to hello' | Pass |» after the line containing 'T209' in ledger.md.",
        &[("ledger.md", LEDGER)],
    );
    assert_eq!(
        run.files["ledger.md"],
        format!("{LEDGER}| T210 | asked 'set the contents of note.txt to hello' | Pass |\n")
    );
    assert!(!run.files.contains_key("note.txt"));
}

/// The `contraction` / `expansion` pairs `data/seed/languages.lino` seeds.
fn seeded_contractions() -> Vec<(String, String)> {
    let languages = formal_ai::seed::seed_files()
        .into_iter()
        .find(|(path, _)| *path == "data/seed/languages.lino")
        .map(|(_, text)| text)
        .expect("languages.lino is seeded");
    let mut pairs = Vec::new();
    let mut contracted = None;
    for line in languages.lines() {
        let Some((key, value)) = line.trim().split_once(' ') else {
            continue;
        };
        let value = value.trim_matches('"').to_owned();
        match key {
            "contraction" => contracted = Some(value),
            "expansion" => {
                if let Some(pair) = contracted.take() {
                    pairs.push((pair, value));
                }
            }
            _ => {}
        }
    }
    pairs
}

#[test]
fn g70_a_provider_denial_written_with_a_contraction_is_a_failure() {
    let tools = ["bash", "read", "webfetch", "websearch"];
    let mut messages = vec![ChatMessage::user("Fetch https://example.test/x")];
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &tools) else {
        panic!("the fetch is planned");
    };
    assert_eq!(calls[0].tool, "webfetch");
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "c0",
        "webfetch",
        calls[0].arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(
        "c0",
        "webfetch",
        "You can't perform that action at this time.",
    ));
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &tools) else {
        panic!("the denial is answered");
    };
    assert!(
        answer.starts_with("The command failed: You can't perform that action at this time."),
        "{answer}"
    );
}

#[test]
fn g70_no_seed_surface_holds_a_seeded_contraction_written_apart() {
    let pairs = seeded_contractions();
    assert!(
        pairs
            .iter()
            .any(|(contracted, expansion)| contracted == "can t" && expansion == "cannot")
    );
    let mut contracted = Vec::new();
    for (path, text) in formal_ai::seed::seed_files() {
        if path == "data/seed/languages.lino" {
            continue;
        }
        for (index, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            let Some(value) = ["text ", "phrase ", "cue ", "followup_request "]
                .iter()
                .find_map(|key| trimmed.strip_prefix(key))
                .and_then(|value| value.strip_prefix('"')?.strip_suffix('"'))
            else {
                continue;
            };
            let padded = format!(" {value} ");
            if pairs
                .iter()
                .any(|(pair, _)| padded.contains(&format!(" {pair} ")))
            {
                contracted.push(format!("{path}:{}", index + 1));
            }
        }
    }
    assert!(contracted.is_empty(), "{contracted:?}");
}

#[test]
fn g69_an_addition_that_quotes_no_text_earns_a_question() {
    for prompt in [
        "add hello to config.txt",
        "append hello to config.txt",
        "add hello to the end of config.txt",
    ] {
        let run = drive(prompt, &[("config.txt", "a = 1\n")]);
        assert!(run.tools.is_empty(), "{prompt}: {:?}", run.tools);
        assert_eq!(run.files["config.txt"], "a = 1\n", "{prompt}");
        assert_eq!(
            run.answer.as_deref(),
            Some(
                "What exact text should go into `config.txt`, and where? The request names the addition without quoting it, so I cannot tell the text to add from words that describe it, and nothing was changed. Quote the text and say where it goes, for example: Append the line «…» to config.txt."
            ),
            "{prompt}"
        );
    }
    let named = drive(
        "Create a test for add in t.mjs and run it.",
        &[("m.mjs", "export const add = (a, b) => a + b;\n")],
    );
    assert!(
        !named
            .tools
            .iter()
            .any(|tool| tool == "write" || tool == "edit"),
        "{:?}",
        named.tools
    );
    assert!(!named.files.contains_key("t.mjs"));
    let quoted = drive(
        "Append the line 'b = 2' to config.txt.",
        &[("config.txt", "a = 1\n")],
    );
    assert_eq!(quoted.files["config.txt"], "a = 1\nb = 2\n");
}

const REQ: &str = "# req\nrow x here\n";

#[test]
fn g63_a_sentence_break_followed_by_backtick_spans_keeps_the_whole_replacement() {
    let run = drive(
        "Replace `x` with `a `data/meta/p.lino` is b. It has c, each with an `e`: `any`, d.` in req.md.",
        &[("req.md", REQ)],
    );
    assert_eq!(run.tools, ["read", "edit", "bash"]);
    assert_eq!(run.commands, ["sha256sum -- req.md"]);
    assert_eq!(
        run.files["req.md"],
        "# req\nrow a `data/meta/p.lino` is b. It has c, each with an `e`: `any`, d. here\n"
    );
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "Replaced `x` with ``a `data/meta/p.lino` is b. It has c, each with an `e`: `any`, d.`` in `req.md` and observed the result."
        )
    );
}

#[test]
fn g63_a_payload_ending_in_a_backtick_span_names_no_target_file() {
    let run = drive(
        "Replace `x` with `a is b. It has `p.lino`` in req.md.",
        &[("req.md", REQ)],
    );
    assert_eq!(
        run.files["req.md"],
        "# req\nrow a is b. It has `p.lino` here\n"
    );
    assert!(!run.files.contains_key("p.lino"));
}

#[test]
fn t181_a_quiet_edit_reply_names_the_replacement() {
    let run = drive(
        "In greeting.txt, change hello to goodbye",
        &[("greeting.txt", "hello world\n")],
    );
    assert_eq!(run.tools, ["read", "edit"]);
    assert_eq!(run.files["greeting.txt"], "goodbye world\n");
    assert_eq!(
        run.answer.as_deref(),
        Some("Replaced `hello` with `goodbye` in `greeting.txt`; the edit tool reported no error.")
    );
}

#[test]
fn g78_a_replace_whose_new_text_holds_the_old_is_a_verified_edit() {
    for (prompt, name, source, expected, answer) in [
        (
            "In s.txt replace 'key 3' with 'key 33'",
            "s.txt",
            "a\nkey 3\nb\n",
            "a\nkey 33\nb\n",
            "Replaced `key 3` with `key 33` in `s.txt` and observed the result.",
        ),
        (
            "Replace `run();` with `guard(); run();` in s.mjs.",
            "s.mjs",
            "if (a) {\n  run();\n}\n",
            "if (a) {\n  guard(); run();\n}\n",
            "Replaced `run();` with `guard(); run();` in `s.mjs` and observed the result.",
        ),
    ] {
        let run = drive(prompt, &[(name, source)]);
        assert_eq!(run.tools, ["read", "edit", "bash"], "{prompt}");
        assert_eq!(run.files[name], expected, "{prompt}");
        assert_eq!(run.answer.as_deref(), Some(answer), "{prompt}");
    }
}

#[test]
fn g64_an_append_whose_payload_names_paths_and_positions_appends_it() {
    let run = drive(
        "Append the line 'D a.rs (one line at the start of x)' to c.md.",
        &[("c.md", "# claims\n"), ("a.rs", "fn a() {}\n")],
    );
    assert_eq!(run.tools, ["read", "edit", "bash"]);
    assert_eq!(run.commands, ["sha256sum -- c.md"]);
    assert_eq!(
        run.files["c.md"],
        "# claims\nD a.rs (one line at the start of x)\n"
    );
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "Appended `D a.rs (one line at the start of x)` to the end of `c.md` and observed the result."
        )
    );
}

#[test]
fn g72_an_anchor_line_found_more_than_once_is_named() {
    let source = "a\nanchor\nb\nanchor\nc\n";
    let run = drive(
        "In g.txt, insert the line 'new' after the line 'anchor'.",
        &[("g.txt", source)],
    );
    assert_eq!(run.tools, ["read"]);
    assert_eq!(run.files["g.txt"], source);
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "The line `anchor` occurs 2 times in `g.txt`, so I cannot tell which one places the insert, and nothing was changed. Quote more of the line, or name it by its line number."
        )
    );
}

#[test]
fn g15_the_plan_event_step_says_it_writes_the_composed_plan() {
    let run = drive("Write hello to x.txt", &[]);
    assert_eq!(run.files["x.txt"], "hello");
    assert!(
        run.files[".formal-ai/general-change-plan.lino"].contains(
            "\n    action \"write the composed plan to .formal-ai/general-change-plan.lino\"\n"
        ),
        "{}",
        run.files[".formal-ai/general-change-plan.lino"]
    );
}

#[test]
fn g73_a_whole_line_anchor_that_also_ends_another_line_is_edited_once() {
    let run = drive(
        "Insert the line '    2,' before the line '];' in t.rs.",
        &[(
            "t.rs",
            "pub const A: &[&str] = &[`x`];\npub const T: &[u8] = &[\n    1,\n];\n",
        )],
    );
    assert_eq!(run.tools, ["read", "edit", "bash"]);
    assert_eq!(
        run.files["t.rs"],
        "pub const A: &[&str] = &[`x`];\npub const T: &[u8] = &[\n    1,\n    2,\n];\n"
    );
    assert_eq!(
        run.answer.as_deref(),
        Some("Inserted `    2,` before `];` in `t.rs` and observed the result.")
    );
}

#[test]
fn g74_an_inserted_line_followed_by_an_empty_line_keeps_it() {
    let run = drive(
        "Insert the line 'pub mod cache;' followed by an empty line before the line 'use std::collections::BTreeMap;' in r.rs.",
        &[("r.rs", "pub mod a;\nuse std::collections::BTreeMap;\n")],
    );
    assert_eq!(
        run.files["r.rs"],
        "pub mod a;\npub mod cache;\n\nuse std::collections::BTreeMap;\n"
    );
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "Inserted `pub mod cache;` before `use std::collections::BTreeMap;` in `r.rs` and observed the result.\nAdded an empty line to `r.rs` and observed the result."
        )
    );
}

#[test]
fn g25_a_test_with_no_expected_result_is_a_question() {
    let run = drive(
        "Create a Python test for add in test_m.py and run it.",
        &[("m.py", "def add(a, b):\n    return a + b\n")],
    );
    assert!(run.tools.is_empty(), "{:?}", run.tools);
    assert_eq!(run.files.keys().collect::<Vec<_>>(), ["m.py"]);
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "What should the test in `test_m.py` check: which call, and what result? The request states no expected result, so nothing was written or run. Say what the function should return for which inputs."
        )
    );
}

/// The answer to `Run <command>` once the command ran with no output.
fn quiet_run(command: &str) -> Option<String> {
    let mut messages = vec![ChatMessage::user(format!("Run {command}"))];
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
        panic!("{command} is planned");
    };
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("tool arguments are JSON");
    assert_eq!(arguments["command"], command);
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "c0",
        "bash",
        calls[0].arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result("c0", "bash", ""));
    match plan_chat_step(&messages, &TOOLS) {
        Some(AgenticPlan::Final(answer)) => Some(answer),
        _ => None,
    }
}

#[test]
fn g75_a_command_is_a_listing_by_its_own_word_only() {
    assert_eq!(
        quiet_run("sed -i '' '/^a/d' allowlist.txt").as_deref(),
        Some("The command completed successfully without output.")
    );
    assert_eq!(
        quiet_run("ls empty").as_deref(),
        Some("This folder is empty.")
    );
}

#[test]
fn g51_a_file_inserted_after_an_unquoted_anchor_that_follows_another() {
    let run = drive(
        "Insert the contents of rows.txt after the line row a that follows the line table u in f.lino.",
        &[
            ("f.lino", "table t\n  row a\nend\ntable u\n  row a\nend\n"),
            ("rows.txt", "  row b\n  row c\n"),
        ],
    );
    assert_eq!(
        run.files["f.lino"],
        "table t\n  row a\nend\ntable u\n  row a\n  row b\n  row c\nend\n"
    );
    assert_eq!(
        run.answer.as_deref(),
        Some("Inserted `  row b\n  row c` after `row a` in `f.lino` and observed the result.")
    );
}

const SCOPED_ROWS: &str = "const C = {\n  slug: \"c\",\n  run: \"./main\",\n};\nconst RUST = {\n  slug: \"rust\",\n  run: \"./main\",\n};\n";

#[test]
fn g79_a_scoped_replace_of_a_repeated_text_is_declined_naming_the_part() {
    let run = drive(
        "In rows.mjs, in the row whose slug is rust, replace `run: \"./main\",` with `run: \"\",`",
        &[("rows.mjs", SCOPED_ROWS)],
    );
    assert_eq!(run.files["rows.mjs"], SCOPED_ROWS);
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "`run: \"./main\",` occurs 2 times in `rows.mjs`, and I cannot tell which of them lie in the row whose slug is rust, so nothing was changed. Quote more of the text around the one to replace."
        )
    );
}

#[test]
fn g79_a_scoped_replace_of_a_text_found_once_is_made() {
    let run = drive(
        "In rows.mjs, in the row whose slug is rust, replace `slug: \"rust\",` with `slug: \"rs\",`",
        &[("rows.mjs", SCOPED_ROWS)],
    );
    assert_eq!(
        run.files["rows.mjs"],
        SCOPED_ROWS.replace("slug: \"rust\"", "slug: \"rs\"")
    );
}

#[test]
fn g79_words_that_name_no_part_are_no_scope() {
    let run = drive(
        "Please, in rows.mjs, replace `run: \"./main\",` with `run: \"\",`",
        &[("rows.mjs", SCOPED_ROWS)],
    );
    assert_eq!(
        run.files["rows.mjs"],
        SCOPED_ROWS.replace("run: \"./main\",", "run: \"\",")
    );
}

#[test]
fn g80_listed_lines_are_replaced_as_one_block() {
    let run = drive(
        "Replace the three lines `check: some(`, `\"kotlinc Main.kt\",` and `),` with the line `check: none,` in k.mjs",
        &[(
            "k.mjs",
            "const KOTLIN = {\n  check: some(\n    \"kotlinc Main.kt\",\n  ),\n  run: \"java\",\n};\n",
        )],
    );
    assert_eq!(
        run.files["k.mjs"],
        "const KOTLIN = {\n  check: none,\n  run: \"java\",\n};\n"
    );
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "Replaced `check: some(`, `\"kotlinc Main.kt\",`, `),` with `check: none,` in `k.mjs` and observed the result."
        )
    );
}

#[test]
fn g13_an_assertion_is_added_in_the_files_assert_form_and_run() {
    let source = "import assert from 'node:assert/strict';\nimport { add } from './m.mjs';\n\ntest('add', () => {\n  assert.equal(add(1, 2), 3);\n});\n";
    let run = drive(
        "Add an assertion that add(2, 2) equals 4 to m.test.mjs.",
        &[("m.test.mjs", source)],
    );
    assert_eq!(
        run.files["m.test.mjs"],
        source.replace("3);\n", "3);\n  assert.equal(add(2, 2), 4);\n")
    );
    assert_eq!(run.tools, ["read", "edit", "bash"]);
    assert_eq!(run.commands, ["node --test m.test.mjs"]);
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "Added `assert.equal(add(2, 2), 4);` to `m.test.mjs` and ran `node --test m.test.mjs`.\n\nThe `node --test m.test.mjs` command completed. Output:\n\n```text\n# tests 1\n# pass 1\n# fail 0\n```"
        )
    );
}

#[test]
fn g13_an_assertion_is_added_in_the_files_expect_form() {
    let source = "import { expect, test } from 'bun:test';\nimport { add } from './m.mjs';\n\ntest('add', () => {\n    expect(add(1, 2)).toBe(3);\n});\n";
    let run = drive(
        "Add an assertion that add(2, 2) equals 4 to m.test.mjs.",
        &[("m.test.mjs", source)],
    );
    assert_eq!(
        run.files["m.test.mjs"],
        source.replace("3);\n", "3);\n    expect(add(2, 2)).toBe(4);\n")
    );
}

#[test]
fn g25_a_missing_javascript_test_file_is_written_around_the_stated_assertion() {
    let run = drive(
        "Create a test for add in m.test.mjs that add(2, 3) returns 5, and run it.",
        &[("m.mjs", "export function add(a, b) {\n  return a + b;\n}\n")],
    );
    assert_eq!(
        run.files["m.test.mjs"],
        "import { test } from 'node:test';\nimport assert from 'node:assert/strict';\nimport { add } from './m.mjs';\n\ntest('add', () => {\n  assert.equal(add(2, 3), 5);\n});\n"
    );
    assert_eq!(run.tools, ["read", "write", "bash"]);
    assert_eq!(run.commands, ["node --test m.test.mjs"]);
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "Wrote `m.test.mjs` with `assert.equal(add(2, 3), 5);` and ran `node --test m.test.mjs`.\n\nThe `node --test m.test.mjs` command completed. Output:\n\n```text\n# tests 1\n# pass 1\n# fail 0\n```"
        )
    );
}

#[test]
fn g25_a_missing_python_test_file_imports_from_the_module_its_name_names() {
    let run = drive(
        "Create a Python test for add in test_m.py that add(2, 3) returns 5, and run it.",
        &[("m.py", "def add(a, b):\n    return a + b\n")],
    );
    assert_eq!(
        run.files["test_m.py"],
        "from m import add\n\n\ndef test_add():\n    assert add(2, 3) == 5\n"
    );
    assert_eq!(
        run.commands.first().map(String::as_str),
        Some("python3 -m pytest test_m.py")
    );
}

#[test]
fn g25_a_shell_word_is_a_command_only_when_named_as_one() {
    let run = drive("Please run the command ls in the terminal.", &[]);
    assert_eq!(run.commands.first().map(String::as_str), Some("ls"));
}

#[test]
fn g76_a_leading_code_span_is_the_command_and_the_words_after_it_are_prose() {
    let run = drive(
        "Run `node --test x.test.js` and tell me whether every test passes.",
        &[],
    );
    assert_eq!(
        run.commands.first().map(String::as_str),
        Some("node --test x.test.js")
    );
}

#[test]
fn g77_a_call_killed_at_the_timeout_is_stated_without_an_exit_code() {
    let run = drive("Run `node --test slow.test.mjs`.", &[]);
    assert_eq!(run.commands, ["node --test slow.test.mjs"]);
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "The command failed: Signal: SIGTERM\nTimeout: 60000 ms\n\nI detected a failure while working on this request. Would you like me to prepare an issue report with the diagnostic context? Reply `Report issue`."
        )
    );
}

#[test]
fn g81_a_line_is_moved_so_that_it_follows_another() {
    let run = drive(
        "In f.md move the line that starts with «- alpha» so that it follows the line that starts with «- gamma».",
        &[("f.md", "# T\n\n- alpha one\n- beta two\n- gamma three\n")],
    );
    assert_eq!(
        run.files["f.md"],
        "# T\n\n- beta two\n- gamma three\n- alpha one\n"
    );
    assert_eq!(
        run.answer.as_deref(),
        Some("Moved `- alpha` after `- gamma` in `f.md` and observed the result.")
    );
}

#[test]
fn g83_a_replace_in_a_file_whose_name_holds_line_is_a_text_replace() {
    let run = drive(
        "In x-line.lino replace «file» with «zzz».",
        &[("x-line.lino", "a check-file-size b\nrun check\n")],
    );
    assert_eq!(run.files["x-line.lino"], "a check-zzz-size b\nrun check\n");
    assert_eq!(
        run.answer.as_deref(),
        Some("Replaced `file` with `zzz` in `x-line.lino` and observed the result.")
    );
}

#[test]
fn g82_a_copy_then_its_edits_are_planned_sentence_by_sentence() {
    let run = drive(
        "Copy a.lino to b.lino. In b.lino replace «x» with «y», replace «p» with «q», and replace «m» with «n».",
        &[("a.lino", "x one\np two\nm three\n")],
    );
    assert_eq!(run.files["a.lino"], "x one\np two\nm three\n");
    assert_eq!(run.files["b.lino"], "y one\nq two\nn three\n");
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "Completed the action `cp a.lino b.lino` and verified it with `test -e b.lino`, `test -e a.lino`.\n\nMade 3 replacements in `b.lino`, in order: `x` → `y`, `p` → `q`, `m` → `n`; and observed the result."
        )
    );
}
