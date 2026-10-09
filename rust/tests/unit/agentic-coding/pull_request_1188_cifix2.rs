//! PR #1188 CIFIX2: the routing regressions behind the Rust CI failures on
//! 6e1c539fc (dogfood ledger rows T310-T319).
//!
//! The JavaScript twin is `rust/tests/web/pull-request-1188-cifix2.test.mjs`.

#[path = "../../fixtures/first-workspace-call.rs"]
mod first_workspace_call;
/// The first planned call: its tool and parsed arguments.
#[path = "../../fixtures/observed-plan-tools.rs"]
mod observed_plan_tools;
fn first_call(prompt: &str, tools: &[&str]) -> (String, serde_json::Value) {
    first_workspace_call::first_workspace_call(prompt, tools)
}

const WRITE_TOOLS: [&str; 3] = ["read_file", "write_file", "exec_command"];

/// T310: a file noun before the path and the content after it declare the
/// file the request writes, so it is written first, whatever the verb.
#[test]
fn a_request_that_declares_its_file_writes_it_unread() {
    for prompt in [
        "add file note.txt containing hello",
        "add a new file src/lib.rs with content pub fn ready() {}",
        "new file: notes.txt, contents: hello",
        "record file note.txt containing hello",
        "добавь файл note.txt с текстом hello",
        "जोड़ो फ़ाइल note.txt सामग्री के साथ hello",
        "添加 文件 note.txt 内容为 hello",
        "añade archivo note.txt con el texto hello",
    ] {
        let (tool, arguments) = first_call(prompt, &WRITE_TOOLS);
        assert_eq!(tool, "write_file", "{prompt}");
        let plan = formal_ai::agentic_coding::general_planner::compose_general_change_plan(prompt)
            .expect("original literal plan");
        assert_eq!(arguments["path"], plan.target, "{prompt}");
        assert_eq!(arguments["content"], plan.content, "{prompt}");
    }
}

/// T310: a destination ahead of the file noun keeps the read-first guard.
#[test]
fn a_destination_before_the_file_keeps_the_read_first_guard() {
    for prompt in [
        "add to the file note.txt containing hello",
        "set note.txt containing hello",
    ] {
        let (tool, arguments) = first_call(prompt, &WRITE_TOOLS);
        assert_eq!(tool, "read_file", "{prompt}");
        assert_eq!(arguments["path"], "note.txt", "{prompt}");
    }
}

/// T311: an explicit grep runs as written.
#[test]
fn an_explicit_grep_runs_as_written() {
    let (tool, arguments) = first_call("execute grep TODO note.txt", &["exec_command"]);
    assert_eq!(tool, "exec_command");
    assert_eq!(arguments["command"], "grep TODO note.txt");
}

/// T311: a local search request greps the workspace for the whole word.
#[test]
fn a_search_request_greps_the_workspace_for_the_whole_word() {
    let (_, arguments) = first_call("search for TODO in the code", &["exec_command"]);
    assert_eq!(
        arguments["command"],
        "grep -rnHw --exclude-dir=.git -- 'TODO' '.'"
    );
    let (_, grep) = first_call("search the code for RouteIntent", &["grep_search"]);
    assert_eq!(grep["pattern"], "\\bRouteIntent\\b");
}

/// T312: an article after a place preposition is no directory.
#[test]
fn an_article_after_a_place_preposition_is_no_directory() {
    let (tool, arguments) = first_call("qué hay en la carpeta actual", &["exec_command"]);
    assert_eq!(tool, "exec_command");
    assert_eq!(arguments["command"], "ls");
    let (_, named) = first_call("List the files in src.", &["exec_command"]);
    assert_eq!(named["command"], "ls 'src'");
}
