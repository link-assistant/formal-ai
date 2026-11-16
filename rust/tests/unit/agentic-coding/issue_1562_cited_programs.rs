//! Issue #1562: a work item that only cites a Hello World repository (the
//! `test-hello-world-…` addresses of a bug report) or a source file name asks
//! for no program. No `main.*` is written, nothing is run or committed, and the
//! summary names the reported defect. A work item that asks for the program in
//! its own words ("Implement Hello World in Scala") still gets it.
//! Twin of `rust/tests/web/issue-1562-cited-programs.test.mjs`.
//!
//! The fixtures are the texts of hive-mind issues 4031, 4034 and 4035 (title,
//! blank line, body). Before the fix the native solver read the whole text,
//! normalized `…/test-hello-world-019f…` to "test hello world", and the
//! command reroute wrote `main.rs` (4031) or `main.js` (4034, 4035), ran it,
//! committed it, ran `gh pr ready` and answered "Created and verified". The
//! sessions replay the recorded Agent CLI request of PR #1493 through the
//! public chat-completion entry (the emulation of
//! `issue_1558_reference_resolution.rs`), which panics on any action other
//! than a read, a glob or a `find`.

use formal_ai::agentic_coding::work_item_citations::{
    cites_anything, is_citation, obligation_text,
};
use formal_ai::protocol::{ChatMessage, ToolCall, create_chat_completion_with_solver};
use serde_json::{Value, json};

use super::issue_1558_reference_resolution::{
    Session, Workspace, agent_solver, recorded_request, run_session, settled,
};

const REPOSITORY: &str = "link-assistant/hive-mind";
const ISSUES: [u32; 3] = [4031, 4034, 4035];

fn fixture(issue: u32) -> &'static str {
    match issue {
        4031 => include_str!("../../../../data/benchmarks/fixtures/issue-1562/hive-mind-4031.txt"),
        4034 => include_str!("../../../../data/benchmarks/fixtures/issue-1562/hive-mind-4034.txt"),
        4035 => include_str!("../../../../data/benchmarks/fixtures/issue-1562/hive-mind-4035.txt"),
        _ => panic!("no recorded issue {issue}"),
    }
}

fn title(text: &str) -> &str {
    text.split('\n').next().unwrap_or_default()
}

/// The first `count` characters of `text`.
fn prefix(text: &str, count: usize) -> String {
    text.chars().take(count).collect()
}

fn hello_world_issue(language: &str) -> String {
    let issues: Value = serde_json::from_str(include_str!(
        "../../fixtures/issue-1364-hello-world-issues.json"
    ))
    .expect("the Hello World issue fixtures");
    let issue = issues
        .as_array()
        .expect("an array of issues")
        .iter()
        .find(|issue| {
            issue["title"]
                .as_str()
                .is_some_and(|title| title.ends_with(language))
        })
        .expect("a Hello World issue of the language");
    format!(
        "{}\n\n{}",
        issue["title"].as_str().expect("a title"),
        issue["body"].as_str().expect("a body")
    )
}

#[test]
fn citations_are_blanked_byte_for_byte_and_the_work_item_keeps_its_own_words() {
    for issue in ISSUES {
        let text = fixture(issue);
        let obligations = obligation_text(text);
        assert_eq!(
            obligations.len(),
            text.len(),
            "hive-mind#{issue}: byte offsets are kept"
        );
        assert_eq!(
            obligations.lines().count(),
            text.lines().count(),
            "hive-mind#{issue}: lines are kept"
        );
        assert!(cites_anything(text), "hive-mind#{issue} cites evidence");
        assert!(
            !obligations.to_lowercase().contains("hello"),
            "hive-mind#{issue}: Hello World appears only in citations"
        );
        assert!(
            !obligations.contains("://"),
            "hive-mind#{issue}: no address is left"
        );
        let first = title(&obligations);
        for word in title(text)
            .split_whitespace()
            .filter(|word| !word.contains('`'))
        {
            assert!(
                first.contains(word),
                "hive-mind#{issue}: the title is an obligation: {word}"
            );
        }
    }
}

#[test]
fn each_kind_of_citation_is_blanked_and_a_quoted_phrase_in_a_sentence_is_kept() {
    let text = [
        "Implement a \"Hello World\" program in Scala.",
        "See https://github.com/konard/test-hello-world-01/pull/2 and <https://example.com/x>.",
        "The file `main.rs` and [the log](https://gist.example/raw/log.txt) and www.example.com.",
        "> Implement Hello World in Rust",
        "| main.rs | hello |",
        "```rust",
        "fn main() { println!(\"Hello, world!\"); }",
        "```",
        "Done — ünïcödé stays.",
    ]
    .join("\n");
    let obligations = obligation_text(&text);
    assert_eq!(obligations.len(), text.len());
    let lines = obligations.split('\n').collect::<Vec<_>>();
    assert_eq!(lines[0], "Implement a \"Hello World\" program in Scala.");
    assert_eq!(
        lines[1].split_whitespace().collect::<Vec<_>>(),
        ["See", "and", "."]
    );
    assert_eq!(
        lines[2].split_whitespace().collect::<Vec<_>>(),
        ["The", "file", "and", "and"]
    );
    for (index, line) in lines.iter().enumerate().take(8).skip(3) {
        assert!(line.trim().is_empty(), "line {} is a citation", index + 1);
    }
    assert_eq!(lines[8], "Done — ünïcödé stays.");
    assert_eq!(
        obligation_text("Implement Hello World in Scala"),
        "Implement Hello World in Scala"
    );
    assert!(!cites_anything("Implement Hello World in Scala"));
}

#[test]
fn a_requirement_span_wholly_inside_a_citation_is_a_citation() {
    let text = "Fix the guard.\n\n| a | b |\n\n```js\nx();\n```\n";
    let obligations = obligation_text(text);
    let span = |needle: &str| {
        let start = text.find(needle).expect("the needle");
        (start, start + needle.len())
    };
    assert!(!is_citation(&obligations, span("Fix the guard.")));
    assert!(is_citation(&obligations, span("| a | b |")));
    assert!(is_citation(&obligations, span("```js\nx();\n```")));
    assert!(
        !is_citation(&obligations, (3, 3)),
        "an empty span is no citation"
    );
    assert!(
        !is_citation(&obligations, (0, text.len() + 1)),
        "a span past the text is no citation"
    );
}

/// The obligation text of a report selects no program; the whole text did.
#[test]
fn the_solver_selects_a_program_only_from_the_cited_addresses() {
    for issue in ISSUES {
        let obligations = obligation_text(fixture(issue));
        assert_eq!(
            formal_ai::UniversalSolver::default()
                .solve(&obligations)
                .execution_recipe
                .map(|recipe| recipe.path),
            None,
            "hive-mind#{issue}: no obligation asks for a program"
        );
    }
}

/// "Implement Hello World in <language>" still selects its program from its own words.
#[test]
fn a_work_item_that_asks_for_the_program_keeps_its_recipe() {
    for (language, path) in [("Scala", "Main.scala"), ("Rust", "main.rs")] {
        let text = hello_world_issue(language);
        let whole = formal_ai::UniversalSolver::default().solve(&text);
        let obliged = formal_ai::UniversalSolver::default().solve(&obligation_text(&text));
        let whole = whole
            .execution_recipe
            .expect("the whole text selects the program");
        let obliged = obliged
            .execution_recipe
            .expect("the obligation text selects the program");
        assert_eq!(whole.path, obliged.path, "{language}: the same file");
        assert_eq!(
            whole.language, obliged.language,
            "{language}: the same language"
        );
        assert!(
            whole.path.eq_ignore_ascii_case(path),
            "{language}: {} is the program file",
            whole.path
        );
    }
}

/// The calls that must never be planned for a report that asks for no program.
fn assert_writes_runs_and_commits_nothing(issue: u32, session: &Session) {
    for call in &session.calls {
        assert!(
            ["read", "glob", "bash"].contains(&call.tool.as_str()),
            "hive-mind#{issue}: only lookups of the cited files: {} {}",
            call.tool,
            call.arguments
        );
        let arguments = call.arguments.to_string();
        for forbidden in ["main.rs", "main.js", "git commit", "gh pr ready"] {
            assert!(
                !arguments.contains(forbidden),
                "hive-mind#{issue}: {forbidden} in {arguments}"
            );
        }
    }
}

fn final_answer(session: &Session) -> &str {
    session
        .final_answer
        .as_deref()
        .expect("the session ends with an answer")
}

#[test]
fn the_three_reports_write_run_and_commit_nothing_and_name_the_defect() {
    let workspace = Workspace::checkout();
    for issue in ISSUES {
        let text = fixture(issue);
        let session = run_session(issue, REPOSITORY, &workspace, text, None);
        assert_writes_runs_and_commits_nothing(issue, &session);
        let answer = final_answer(&session);
        assert!(
            answer.contains(&prefix(title(text), 60)),
            "hive-mind#{issue}: the summary names the reported defect: {answer}"
        );
        assert!(
            !answer.contains("Created and verified"),
            "hive-mind#{issue}: {answer}"
        );
        let record = settled(&session);
        assert!(
            record["requirements"]
                .as_array()
                .is_some_and(|rows| rows.iter().any(|row| row["status"] == "citation")),
            "hive-mind#{issue}: the cited evidence is not an obligation: {record}"
        );
        // The program the whole text selected is named as not written.
        let unbound = record["unboundWrite"]["path"].as_str().unwrap_or_else(|| {
            panic!("hive-mind#{issue}: the unbound write is recorded: {record}")
        });
        assert!(
            unbound.to_ascii_lowercase().starts_with("main."),
            "hive-mind#{issue}: {unbound}"
        );
        assert_eq!(record["unboundWrite"]["reason"], "NamedByNoObligation");
        assert!(
            answer.contains(&format!(
                "`{unbound}` was not written: no obligation of the work item names it"
            )),
            "hive-mind#{issue}: the summary says why no program was written: {answer}"
        );
    }
}

/// Production order: the program was already written and run when the session
/// resumes. The file is named by no obligation, so the session neither commits
/// it nor marks the pull request ready nor claims completion.
#[test]
fn a_session_whose_only_written_file_is_named_by_no_obligation_commits_nothing() {
    let workspace = Workspace::checkout();
    let text = fixture(4031);
    let mut request = recorded_request(4031, REPOSITORY, &workspace, text, None);
    let path = format!("{}/main.rs", workspace.root);
    for (id, tool, arguments, output) in [
        (
            "call_write_main",
            "write",
            json!({"filePath": path, "content": "fn main() {\n    println!(\"Hello, World!\");\n}\n"}),
            "Wrote file successfully.",
        ),
        (
            "call_run_main",
            "bash",
            json!({"command": "rustc main.rs -o main && ./main", "description": "Run the program"}),
            "Hello, World!\n",
        ),
    ] {
        request
            .messages
            .push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                id,
                tool,
                arguments.to_string(),
            )]));
        request
            .messages
            .push(ChatMessage::tool_result(id, tool, output));
    }
    let solver = agent_solver();
    for _ in 0..40 {
        let completion = create_chat_completion_with_solver(&request, &solver);
        let message = completion.choices[0].message.clone();
        if message.tool_calls.is_empty() {
            let answer = message.content.plain_text();
            assert!(!answer.contains("Created and verified"), "{answer}");
            return;
        }
        for call in &message.tool_calls {
            let arguments = &call.function.arguments;
            assert!(
                !["write", "edit"].contains(&call.function.name.as_str()),
                "nothing more is written: {} {arguments}",
                call.function.name
            );
            for forbidden in ["git commit", "git push", "gh pr ready"] {
                assert!(
                    !arguments.contains(forbidden),
                    "an unbound file is never committed or reported: {arguments}"
                );
            }
        }
        request.messages.push(message.clone());
        for call in &message.tool_calls {
            let arguments: Value =
                serde_json::from_str(&call.function.arguments).expect("JSON arguments");
            let Some((content, _)) = workspace.answer(&call.function.name, &arguments) else {
                panic!(
                    "the session planned an action outside the emulation: {} {}",
                    call.function.name, call.function.arguments
                );
            };
            request.messages.push(ChatMessage::tool_result(
                call.id.clone(),
                call.function.name.clone(),
                content,
            ));
        }
    }
    panic!("the session did not end within 40 steps");
}
