//! PR #1188 G32: the words of a quoted payload never choose a shell command.
//! `Add the line '- run tests' at the end of the section '## Usage' in
//! README.md.` ran the workspace test suite, because the shell intent table
//! matched its cue `run tests` inside the quotes. A request about text inside
//! a file now reads the intent cues outside its quotes only, while a quoted
//! command (`Run 'npm test'.`) still runs. Twin of
//! `rust/tests/web/pull-request-1188-quoted-payload-command.test.mjs`.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::ChatMessage;

const TOOLS: [&str; 4] = ["read", "edit", "bash", "write"];

/// The tool and its command or file argument for each planned call.
fn planned(prompt: &str) -> Vec<(String, String)> {
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&[ChatMessage::user(prompt)], &TOOLS)
    else {
        return Vec::new();
    };
    calls
        .iter()
        .map(|call| {
            let arguments: serde_json::Value =
                serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
            let argument = arguments["command"]
                .as_str()
                .or_else(|| arguments["filePath"].as_str())
                .unwrap_or_default();
            (call.tool.clone(), argument.to_owned())
        })
        .collect()
}

fn commands(prompt: &str) -> Vec<String> {
    planned(prompt)
        .into_iter()
        .filter(|(tool, _)| tool == "bash")
        .map(|(_, command)| command)
        .collect()
}

#[test]
fn a_line_to_add_that_names_a_test_run_runs_nothing() {
    assert!(
        commands("Add the line '- run tests' at the end of the section '## Usage' in README.md.")
            .is_empty()
    );
}

#[test]
fn an_insert_whose_line_names_a_test_run_reads_the_file_to_edit_it() {
    assert_eq!(
        planned("Insert the line 'run the tests' after the line 'x' in notes.txt."),
        [("read".to_owned(), "notes.txt".to_owned())]
    );
}

/// G66: intent cues match whole words, so the `move` cue inside `removed` no
/// longer plans `mv`.
#[test]
fn a_cue_matches_whole_words_only() {
    let planned = commands("Append the line 'a' then removed from f.md to g2.md.");
    assert!(
        !planned.iter().any(|command| command.starts_with("mv ")),
        "{planned:?}"
    );
}

#[test]
fn a_quoted_command_still_runs() {
    assert_eq!(commands("Run 'npm test'."), ["npm test"]);
}
