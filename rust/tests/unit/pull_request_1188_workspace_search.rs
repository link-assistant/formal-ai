//! PR #1188 dogfooding (T90, gap G12): "Find all usages of add." searched the
//! web for the sentence, "Where is add used in this project?" grepped the
//! whole question, "Search for add in the files of this directory." listed
//! the directory and "Find all usages of add in this directory." looked for a
//! file named all-usages-of-add. Workspace content search now has its own
//! seeded vocabulary (`workspace_content_search_form`, every registered
//! language): the arm greps the named identifier or quoted literal -- the
//! client's grep tool when advertised, else `grep -rn` through the shell --
//! and answers with the file:line hits, or a seeded not-found naming the
//! pattern and the scope. Twin of
//! `rust/tests/web/pull-request-1188-workspace-search.test.mjs`.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};

const AGENT_CLI_TOOLS: [&str; 14] = [
    "bash",
    "batch",
    "codesearch",
    "edit",
    "glob",
    "grep",
    "list",
    "read",
    "task",
    "todoread",
    "todowrite",
    "webfetch",
    "websearch",
    "write",
];
const REDUCED_TOOLS: [&str; 4] = ["read", "write", "edit", "bash"];
const GREP_HITS: &str = concat!(
    "Found 2 matches\n",
    "/work/demo/m.mjs:\n",
    "  Line 1: export function add(a, b) {\n",
    "\n",
    "/work/demo/m.test.mjs:\n",
    "  Line 6:   assert.strictEqual(add(1, 2), 3);",
);
const SHELL_HITS: &str =
    "./m.mjs:1:export function add(a, b) {\n./m.test.mjs:6:  assert.strictEqual(add(1, 2), 3);\n";
const LISTED: &str =
    "- `m.mjs:1`: export function add(a, b) {\n- `m.test.mjs:6`: assert.strictEqual(add(1, 2), 3);";

/// The conversation after the user's request and, when given, one tool call
/// with its result.
fn conversation(prompt: &str, call: Option<(&str, &str, &str)>) -> Vec<ChatMessage> {
    let mut messages = vec![
        ChatMessage::new("system", "<env>\n  Working directory: /work/demo\n</env>"),
        ChatMessage::user(prompt),
    ];
    if let Some((tool, arguments, result)) = call {
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "c0",
            tool,
            arguments.to_owned(),
        )]));
        messages.push(ChatMessage::tool_result("c0", tool, result));
    }
    messages
}

/// The first planned call (tool, arguments) and the answer given after
/// `result` comes back.
fn search_once(prompt: &str, tools: &[&str], result: &str) -> (String, serde_json::Value, String) {
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&conversation(prompt, None), tools)
    else {
        panic!("{prompt} planned no tool call");
    };
    let call = calls[0].clone();
    let next = conversation(prompt, Some((&call.tool, &call.arguments, result)));
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&next, tools) else {
        panic!("{prompt} did not answer from the search result");
    };
    let arguments = serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
    (call.tool, arguments, answer)
}

#[test]
fn every_phrasing_greps_the_named_identifier_and_lists_its_hits() {
    for prompt in [
        "Find all usages of add.",
        "Find all usages of add in this directory.",
        "Where is add used in this project?",
        "Search for add in the files of this directory.",
        "Grep for add.",
    ] {
        let (tool, arguments, answer) = search_once(prompt, &AGENT_CLI_TOOLS, GREP_HITS);
        assert_eq!(tool, "grep", "{prompt}");
        assert_eq!(
            arguments,
            serde_json::json!({ "path": ".", "pattern": "\\badd\\b" }),
            "{prompt}"
        );
        assert_eq!(
            answer,
            format!("Found 2 line(s) mentioning `add` in `.`:\n{LISTED}"),
            "{prompt}"
        );
    }
}

#[test]
fn every_registered_language_asks_in_its_own_words() {
    for (prompt, expected) in [
        (
            "Найди все использования add.",
            format!("Найдено строк с `add` в `.`: 2.\n{LISTED}"),
        ),
        (
            "इस निर्देशिका में add के सभी उपयोग खोजें।",
            format!("`.` में `add` वाली 2 पंक्ति(याँ) मिलीं:\n{LISTED}"),
        ),
        (
            "查找 add 的所有用法。",
            format!("在 `.` 中找到 2 行提到 `add`：\n{LISTED}"),
        ),
        (
            "¿Dónde se usa add?",
            format!("Se encontraron 2 línea(s) que mencionan `add` en `.`:\n{LISTED}"),
        ),
    ] {
        let (tool, arguments, answer) = search_once(prompt, &AGENT_CLI_TOOLS, GREP_HITS);
        assert_eq!(tool, "grep", "{prompt}");
        assert_eq!(arguments["pattern"], "\\badd\\b", "{prompt}");
        assert_eq!(answer, expected, "{prompt}");
    }
}

#[test]
fn a_pattern_found_nowhere_is_a_seeded_not_found() {
    let (_, _, answer) = search_once(
        "Find all usages of sub.",
        &AGENT_CLI_TOOLS,
        "No files found",
    );
    assert_eq!(answer, "No line in `.` mentions `sub`.");
}

#[test]
fn without_a_grep_tool_the_shell_runs_grep() {
    let (tool, arguments, answer) =
        search_once("Find all usages of add.", &REDUCED_TOOLS, SHELL_HITS);
    assert_eq!(tool, "bash");
    assert_eq!(
        arguments["command"],
        "grep -rnHw --exclude-dir=.git -- 'add' '.'"
    );
    assert_eq!(
        answer,
        format!("Found 2 line(s) mentioning `add` in `.`:\n{LISTED}")
    );
    let (_, _, none) = search_once("Find all usages of sub.", &REDUCED_TOOLS, "");
    assert_eq!(none, "No line in `.` mentions `sub`.");
}

/// G36: a scope phrase after the searched word is the scope, not part of
/// the query (`rg -n 'foo this'` before).
#[test]
fn a_scope_phrase_after_the_searched_word_is_not_part_of_the_query() {
    for prompt in [
        "Search for 'foo' in this project.",
        "Search for foo in the workspace.",
    ] {
        let Some(AgenticPlan::ToolCalls(calls)) =
            plan_chat_step(&conversation(prompt, None), &REDUCED_TOOLS)
        else {
            panic!("{prompt} planned no tool call");
        };
        let arguments: serde_json::Value =
            serde_json::from_str(&calls[0].arguments).expect("tool arguments are JSON");
        assert_eq!(
            arguments["command"], "grep -rnHw --exclude-dir=.git -- 'foo' '.'",
            "{prompt}"
        );
    }
}

#[test]
fn a_quoted_literal_is_searched_as_fixed_text_within_the_named_folder() {
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(
        &conversation("Grep for 'return a' in lib/", None),
        &REDUCED_TOOLS,
    ) else {
        panic!("the literal search planned no tool call");
    };
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("tool arguments are JSON");
    assert_eq!(
        arguments["command"],
        "grep -rnHF --exclude-dir=.git -- 'return a' 'lib'"
    );
}

#[test]
fn a_named_file_is_the_scope_and_a_request_to_change_the_uses_is_not_a_search() {
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(
        &conversation("Count the lines containing foo in f.txt.", None),
        &REDUCED_TOOLS,
    ) else {
        panic!("the scoped search planned no tool call");
    };
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("tool arguments are JSON");
    assert_eq!(
        arguments["command"],
        "grep -rnHw --exclude-dir=.git -- 'foo' 'f.txt'"
    );
    for prompt in [
        "Rename all usages of add to plus.",
        "Remove all uses of add.",
    ] {
        if let Some(AgenticPlan::ToolCalls(calls)) =
            plan_chat_step(&conversation(prompt, None), &REDUCED_TOOLS)
        {
            assert!(!calls[0].arguments.contains("grep -rn"), "{prompt}");
        }
    }
}

#[test]
fn a_file_name_request_still_locates_the_file() {
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(
        &conversation("Find the file m.mjs in this directory.", None),
        &AGENT_CLI_TOOLS,
    ) else {
        panic!("the file-name request planned no tool call");
    };
    assert_eq!(calls[0].tool, "bash");
    assert!(
        calls[0].arguments.contains("find "),
        "{}",
        calls[0].arguments
    );
}

#[test]
fn a_search_the_request_sends_to_the_web_is_not_grepped() {
    let plan = plan_chat_step(
        &conversation("Find usages of add online.", None),
        &AGENT_CLI_TOOLS,
    );
    if let Some(AgenticPlan::ToolCalls(calls)) = plan {
        assert_ne!(calls[0].tool, "grep");
    }
}

/// T91 (gap G11): a named test file runs with the runtime its extension
/// needs, from the seeded `test_file_runners` group of
/// `data/seed/shell-intents.lino`, never the whole-suite command a marker
/// file in the server's own directory chose.
#[test]
fn a_named_test_file_runs_with_its_own_runtime() {
    for (prompt, command) in [
        ("Run the tests in m.test.mjs.", "node --test m.test.mjs"),
        ("Запусти тесты в m.test.mjs.", "node --test m.test.mjs"),
        ("Run the tests in test_m.py.", "python3 -m pytest test_m.py"),
    ] {
        let Some(AgenticPlan::ToolCalls(calls)) =
            plan_chat_step(&conversation(prompt, None), &REDUCED_TOOLS)
        else {
            panic!("{prompt} planned no tool call");
        };
        assert_eq!(calls[0].tool, "bash", "{prompt}");
        let arguments: serde_json::Value =
            serde_json::from_str(&calls[0].arguments).expect("tool arguments are JSON");
        assert_eq!(arguments["command"], command, "{prompt}");
    }
}

/// T92 (gap G10): a function whose body is stated as an expression
/// (`returns a - b`) is the seeded integer operation that meets the
/// specification at every sample pair, and the returned value is the longest
/// expression right after the verb, not the rest of the clause (`… to m.mjs`).
#[test]
fn a_function_whose_body_is_stated_as_an_expression_is_added() {
    const MODULE: &str = "export function add(a, b) {\n  return a + b;\n}\n";
    let mut file = MODULE.to_owned();
    let mut messages = conversation(
        "Add a function sub(a, b) that returns a - b to m.mjs.",
        None,
    );
    let mut tools = Vec::new();
    let mut answer = String::new();
    for step in 0..6 {
        let calls = match plan_chat_step(&messages, &REDUCED_TOOLS) {
            Some(AgenticPlan::ToolCalls(calls)) => calls,
            Some(AgenticPlan::Final(text)) => {
                answer = text;
                break;
            }
            _ => break,
        };
        let call = calls[0].clone();
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
        let result = match call.tool.as_str() {
            "read" => file.clone(),
            "write" => {
                file = arguments["content"].as_str().unwrap_or_default().to_owned();
                String::new()
            }
            _ => String::new(),
        };
        tools.push(call.tool.clone());
        let id = format!("s{step}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(id, call.tool.clone(), result));
    }
    assert_eq!(tools, ["read", "write", "bash"]);
    assert_eq!(
        file,
        format!("{MODULE}\nexport function sub(a, b) {{\n  return a - b;\n}}\n")
    );
    assert!(
        answer.starts_with("Created and verified `m.mjs`"),
        "answered {answer:?}"
    );
}

const BUGGY_MODULE: &str = "export function add(a, b) {\n  return a + b;\n}\n\nexport function mul(a, b) {\n  return a * b;\n}\n";
const ADD_PROBE: &str =
    "node --input-type=module -e \"import { add } from './m.mjs'; console.log(add(2, 3));\"";

/// Read returns the module, the shell returns `printed`: the calls (a shell
/// call by its command) and the final answer.
fn check(prompt: &str, printed: &str) -> (Vec<String>, Option<String>) {
    let mut messages = conversation(prompt, None);
    let mut calls = Vec::new();
    for step in 0..4 {
        let planned = match plan_chat_step(&messages, &REDUCED_TOOLS) {
            Some(AgenticPlan::ToolCalls(planned)) => planned,
            Some(AgenticPlan::Final(answer)) => return (calls, Some(answer)),
            _ => break,
        };
        let call = planned[0].clone();
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
        calls.push(if call.tool == "bash" {
            arguments["command"].as_str().unwrap_or_default().to_owned()
        } else {
            call.tool.clone()
        });
        let result = if call.tool == "read" {
            BUGGY_MODULE
        } else {
            printed
        };
        let id = format!("e{step}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(id, call.tool.clone(), result));
    }
    (calls, None)
}

/// T93 (gap G14): a bug report with a stated expectation is checked first --
/// the module read, the function run at the contract's samples through its
/// seeded probe -- and a function that meets it is no defect found.
#[test]
fn a_function_that_meets_the_stated_expectation_is_no_defect_found() {
    let (calls, answer) = check("Fix the bug in m.mjs: add should return the sum.", "5\n");
    assert_eq!(calls, ["read", ADD_PROBE]);
    assert_eq!(
        answer.as_deref(),
        Some(
            "No defect found: `add(2, 3)` in `m.mjs` returned `5`, the expected `5`, so `m.mjs` is unchanged."
        )
    );
}

#[test]
fn a_function_that_misses_the_stated_expectation_is_a_confirmed_defect() {
    let (_, answer) = check("Fix the bug in m.mjs: add should return a * b.", "5\n");
    assert_eq!(
        answer.as_deref(),
        Some(
            "The defect is confirmed: `add(2, 3)` in `m.mjs` returned `5`, but the request expects `6`. `m.mjs` is unchanged."
        )
    );
}

#[test]
fn every_registered_language_states_the_expectation_in_its_own_words() {
    let (calls, answer) = check(
        "Исправь ошибку в m.mjs: add должна возвращать сумму.",
        "5\n",
    );
    assert_eq!(calls, ["read", ADD_PROBE]);
    assert_eq!(
        answer.as_deref(),
        Some(
            "Ошибка не найдена: `add(2, 3)` в `m.mjs` вернула `5` — ожидаемое `5`, поэтому `m.mjs` не изменён."
        )
    );
    let (calls, _) = check("m.mjs में बग ठीक करें: add को योग लौटाना चाहिए।", "5\n");
    assert_eq!(calls, ["read", ADD_PROBE]);
}

/// Run `prompt` over the in-memory `files`; the tools act as the Agent CLI's
/// do. Returns each call (`tool path`, or a shell command) and the answer.
fn run_over(
    prompt: &str,
    files: &mut std::collections::BTreeMap<String, String>,
) -> (Vec<String>, Option<String>) {
    let mut messages = conversation(prompt, None);
    let mut calls = Vec::new();
    for step in 0..6 {
        let planned = match plan_chat_step(&messages, &REDUCED_TOOLS) {
            Some(AgenticPlan::ToolCalls(planned)) => planned,
            Some(AgenticPlan::Final(answer)) => return (calls, Some(answer)),
            _ => break,
        };
        let call = planned[0].clone();
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
        let path = arguments["filePath"]
            .as_str()
            .or_else(|| arguments["path"].as_str())
            .unwrap_or_default()
            .to_owned();
        let result = match call.tool.as_str() {
            "read" => files
                .get(&path)
                .cloned()
                .unwrap_or_else(|| format!("Error: File not found: {path}")),
            "write" => {
                let content = arguments["content"].as_str().unwrap_or_default();
                files.insert(path.clone(), content.to_owned());
                String::new()
            }
            "bash" => {
                let command = arguments["command"].as_str().unwrap_or_default();
                files
                    .get(command.trim_start_matches("cat "))
                    .cloned()
                    .unwrap_or_default()
            }
            _ => String::new(),
        };
        calls.push(if call.tool == "bash" {
            arguments["command"].as_str().unwrap_or_default().to_owned()
        } else {
            format!("{} {path}", call.tool)
        });
        let id = format!("w{step}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(id, call.tool.clone(), result));
    }
    (calls, None)
}

/// T96 (the backstop behind G13, G17 and T29): unless the first write verb is
/// a seeded whole-file write, a literal-file plan reads its target first, and
/// an existing non-empty file is replaced only when the request says so.
#[test]
fn an_edit_misread_as_a_whole_file_write_reads_the_file_and_keeps_it() {
    const MODULE: &str = "export function add(a, b) {\n  return a + b;\n}\n";
    let mut files = std::collections::BTreeMap::from([("m.mjs".to_owned(), MODULE.to_owned())]);
    let (calls, answer) = run_over("Replace all uses of add with plus in m.mjs.", &mut files);
    assert_eq!(calls, ["read m.mjs"]);
    assert_eq!(
        answer.as_deref(),
        Some(
            "Left `m.mjs` unchanged: it already has content, and the request does not say to replace the whole file."
        )
    );
    assert_eq!(files["m.mjs"], MODULE);
}

#[test]
fn a_missing_target_is_created_after_the_read_finds_nothing() {
    let mut files = std::collections::BTreeMap::new();
    let (calls, _) = run_over("Put 'hello' into notes.txt.", &mut files);
    assert_eq!(
        calls,
        [
            "read notes.txt",
            "write .formal-ai/general-change-plan.lino",
            "write notes.txt",
            "cat notes.txt",
        ]
    );
    assert_eq!(files["notes.txt"], "hello");
}

#[test]
fn an_existing_target_is_kept_for_that_verb_and_replaced_for_a_whole_file_write() {
    let mut kept = std::collections::BTreeMap::from([("notes.txt".to_owned(), "old".to_owned())]);
    let (_, answer) = run_over("Put 'hello' into notes.txt.", &mut kept);
    assert_eq!(kept["notes.txt"], "old");
    assert!(
        answer
            .as_deref()
            .is_some_and(|text| text.starts_with("Left `notes.txt` unchanged")),
        "answered {answer:?}"
    );
    let mut replaced = std::collections::BTreeMap::from([("a.txt".to_owned(), "old".to_owned())]);
    let (calls, _) = run_over("Create a file a.txt containing hello", &mut replaced);
    assert_eq!(calls[0], "write .formal-ai/general-change-plan.lino");
    assert_eq!(replaced["a.txt"], "hello");
}

/// T98 (gap G29): a listing request lists the directory it names -- the
/// path-shaped word after a seeded place preposition -- unless the word
/// belongs to a seeded scope phrase.
#[test]
fn a_listing_request_lists_the_directory_it_names() {
    for (prompt, tools, tool, arguments) in [
        (
            "List the files in src.",
            &AGENT_CLI_TOOLS[..],
            "list",
            serde_json::json!({ "path": "src" }),
        ),
        (
            "Покажи файлы в src.",
            &AGENT_CLI_TOOLS[..],
            "list",
            serde_json::json!({ "path": "src" }),
        ),
        (
            "List the files in this directory.",
            &AGENT_CLI_TOOLS[..],
            "list",
            serde_json::json!({ "path": "." }),
        ),
        (
            "List the files in src.",
            &REDUCED_TOOLS[..],
            "bash",
            serde_json::json!({ "command": "ls 'src'" }),
        ),
        (
            "List the files in the current folder.",
            &REDUCED_TOOLS[..],
            "bash",
            serde_json::json!({ "command": "ls" }),
        ),
    ] {
        let Some(AgenticPlan::ToolCalls(calls)) =
            plan_chat_step(&conversation(prompt, None), tools)
        else {
            panic!("{prompt} planned no tool call");
        };
        assert_eq!(calls[0].tool, tool, "{prompt}");
        let planned: serde_json::Value =
            serde_json::from_str(&calls[0].arguments).expect("tool arguments are JSON");
        assert_eq!(planned, arguments, "{prompt}");
    }
}

/// T99 (gap G26): a question about what a module exports is answered from the
/// functions it declares behind a seeded export marker.
#[test]
fn a_question_about_what_a_module_exports_is_answered_from_its_declarations() {
    const MODULE: &str = "export function add(a, b) {\n  return a + b;\n}\n\nfunction helper() {}\n\nexport async function mul(a, b) {\n  return a * b;\n}\n";
    for (prompt, expected) in [
        (
            "Which functions does src/m.mjs export?",
            "`src/m.mjs` exports 2 function(s): `add`, `mul`.",
        ),
        (
            "Какие функции экспортирует src/m.mjs?",
            "`src/m.mjs` экспортирует функции (2): `add`, `mul`.",
        ),
    ] {
        let Some(AgenticPlan::ToolCalls(calls)) =
            plan_chat_step(&conversation(prompt, None), &REDUCED_TOOLS)
        else {
            panic!("{prompt} planned no tool call");
        };
        assert_eq!(calls[0].tool, "read", "{prompt}");
        let next = conversation(prompt, Some((&calls[0].tool, &calls[0].arguments, MODULE)));
        assert_eq!(
            plan_chat_step(&next, &REDUCED_TOOLS),
            Some(AgenticPlan::Final(expected.to_owned())),
            "{prompt}"
        );
    }
}

/// T100 (gap G27): a summary of a named file hands the file's text to the
/// summarization handlers instead of answering with its first line.
#[test]
fn a_summary_of_a_named_file_summarizes_its_text() {
    const TEXT: &str = "The river flooded the town after three days of rain. Residents moved to the school on the hill. Volunteers brought food and blankets.\n";
    let prompt = "Summarize README.md in one sentence.";
    let Some(AgenticPlan::ToolCalls(calls)) =
        plan_chat_step(&conversation(prompt, None), &REDUCED_TOOLS)
    else {
        panic!("the summary planned no read");
    };
    assert_eq!(calls[0].tool, "read");
    let next = conversation(prompt, Some((&calls[0].tool, &calls[0].arguments, TEXT)));
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&next, &REDUCED_TOOLS) else {
        panic!("the summary did not answer");
    };
    assert!(
        answer.starts_with("The river flooded the town after three days of rain."),
        "{answer}"
    );
    assert!(!answer.contains("Volunteers"), "{answer}");
}
