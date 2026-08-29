//! Capability pins for the recursive agent ladder (issue #1066).
//!
//! Issue #1066 requires `issue-1028-agent-ladder.yml` to complete, and states
//! that a failure at any depth "is a real capability gap, not a flake, and
//! should be fixed generically — never with a prompt-specific branch". These
//! tests therefore pin the *general* behaviour each observed ladder failure
//! exposed, and prove it with wording the ladder never uses.

use formal_ai::ChatMessage;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};

/// The fourteen tool names `@link-assistant/agent` advertises, in the order the
/// live ladder trace recorded them.
const LADDER_TOOLS: [&str; 14] = [
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

fn plan(prompt: &str) -> Option<AgenticPlan> {
    plan_chat_step(&[ChatMessage::user(prompt)], &LADDER_TOOLS)
}

/// Every path argument a plan carries, whatever key the tool names it under.
fn planned_paths(prompt: &str) -> Vec<String> {
    let Some(AgenticPlan::ToolCalls(calls)) = plan(prompt) else {
        return Vec::new();
    };
    calls
        .iter()
        .filter_map(|call| serde_json::from_str::<serde_json::Value>(&call.arguments).ok())
        .filter_map(|value| {
            ["path", "filePath", "file_path", "absolute_path"]
                .iter()
                .find_map(|key| {
                    value
                        .get(*key)
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
        })
        .collect()
}

#[test]
fn a_dotted_number_is_never_mistaken_for_a_file_to_read() {
    // The ladder addresses each node by its path in the tree — `1.1.1.1.1` for
    // the leftmost leaf at depth five. The planner used to split that on the
    // last dot, read `1` as an extension, and call `read("1.1.1.1.1")`, which
    // failed with "File not found" and ended the node with a fabricated answer.
    // A token made only of digits and dots is a number, not a file name.
    for prompt in [
        "Show me the first line of 1.1.1.1.1 please.",
        "Read 2.7.19 and tell me what it says.",
        "Open 192.168.0.14 and summarise it.",
        "What does 3.14159 contain?",
    ] {
        let paths = planned_paths(prompt);
        assert!(
            paths
                .iter()
                .all(|path| !path.chars().all(|c| c.is_ascii_digit() || c == '.')),
            "planned to open a dotted number as a file for {prompt:?}: {paths:?}"
        );
    }
}

#[test]
fn a_genuine_dotted_file_name_is_still_recognised() {
    // The fix above must not cost the planner its ordinary file recognition:
    // a name with a non-numeric part is still a file, with or without a
    // directory in front of it.
    for (prompt, expected) in [
        ("Read Cargo.toml and report the package name.", "Cargo.toml"),
        (
            "Show me what is inside src/lib.rs at the top.",
            "src/lib.rs",
        ),
        (
            "Open data/seed/roles.lino and list the first role.",
            "data/seed/roles.lino",
        ),
        ("Display the contents of v2.notes.md please.", "v2.notes.md"),
    ] {
        let paths = planned_paths(prompt);
        assert!(
            paths.iter().any(|path| path.ends_with(expected)),
            "expected a read of {expected} for {prompt:?}, planned {paths:?}"
        );
    }
}

/// The answer a plan settles on, when it settles on one without a tool call.
fn final_answer(prompt: &str) -> Option<String> {
    match plan(prompt)? {
        AgenticPlan::Final(answer) => Some(answer),
        AgenticPlan::ToolCalls(_) => None,
    }
}

/// Every `content` argument a plan carries, whatever key the tool names it under.
fn planned_writes(prompt: &str) -> Vec<String> {
    let Some(AgenticPlan::ToolCalls(calls)) = plan(prompt) else {
        return Vec::new();
    };
    calls
        .iter()
        .filter_map(|call| serde_json::from_str::<serde_json::Value>(&call.arguments).ok())
        .filter_map(|value| {
            ["content", "contents", "text", "new_string"]
                .iter()
                .find_map(|key| {
                    value
                        .get(*key)
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned)
                })
        })
        .collect()
}

#[test]
fn a_written_file_starts_with_the_line_the_same_request_pinned() {
    // A request can state two things about one file: what it has to contain and
    // how it has to begin. The literal-write parser read the first and ignored
    // the second, so it composed bytes out of the prose and wrote a file that
    // broke a constraint stated three sentences earlier. A literal write is only
    // literal when it satisfies every stated constraint on the file it writes.
    for (prompt, pinned) in [
        (
            "Draft a handover memo containing the migration status, the outstanding \
             blockers, and the on-call owner. Leave the memo in `handover/2026-q3.md`. \
             The first line must be exactly `handover=q3`.",
            "handover=q3",
        ),
        (
            "Assemble a shift summary containing the machines serviced, the parts \
             replaced, and the hours logged. Record it in `shift/friday.md`. The first \
             line must be exactly `shift=friday`.",
            "shift=friday",
        ),
    ] {
        let writes = planned_writes(prompt);
        assert!(
            !writes.is_empty(),
            "nothing was planned to be written for {prompt:?}"
        );
        for content in &writes {
            assert!(
                content.starts_with(pinned),
                "wrote a file that does not open with {pinned:?} for {prompt:?}: {content:?}"
            );
        }
    }
}

#[test]
fn spelled_out_bytes_are_still_written_literally() {
    // Guarding the pinned first line must not cost the planner the literal write
    // it already did well. A request that states no constraint on the opening
    // line has nothing to violate, so its bytes go to the file unchanged.
    for (prompt, expected) in [
        (
            "Create `list.txt` containing apples, bananas and cherries.",
            "apples, bananas and cherries",
        ),
        (
            "Write `greeting.txt` with the text hello from the harness.",
            "hello from the harness",
        ),
    ] {
        let writes = planned_writes(prompt);
        assert!(
            writes.iter().any(|content| content.contains(expected)),
            "expected the literal bytes {expected:?} for {prompt:?}, planned {writes:?}"
        );
    }
}

#[test]
fn a_document_specification_is_composed_instead_of_transcribed() {
    // "Produce a note containing A, B and C" names the headings a finished
    // document must have, not the bytes of a file. Transcribing the sentence
    // into the file answers the grammar and not the request; the parts have to
    // come back as the structure of a composed document.
    let answer = final_answer(
        "Prepare a briefing note containing the vendor name, the renewal date, and the \
         annual cost.",
    )
    .expect("a document specification should be answered by composing the document");
    for part in ["the vendor name", "the renewal date", "the annual cost"] {
        assert!(
            answer.contains(&format!("- {part}")),
            "composed note omits {part:?}: {answer}"
        );
    }
}

#[test]
fn a_composed_note_never_claims_what_the_session_did_not_observe() {
    // The honest deliverable for a specification nothing has answered yet is a
    // note that says so. Silence about the missing parts would read as success.
    let answer = final_answer(
        "Compose a release report containing the build identifier, the failing suites, \
         and the rollback owner.",
    )
    .expect("a document specification should be answered by composing the document");
    assert!(
        answer.contains("no tool result was recorded"),
        "a note composed from nothing should say so: {answer}"
    );
    assert!(
        answer.contains("No requested part above is backed by an observation"),
        "a note composed from nothing should list its parts as outstanding: {answer}"
    );
}

#[test]
fn a_document_specification_is_read_in_every_registered_language() {
    // The three signals the route reads are seed-declared, so a request that
    // carries them in Russian is the same request. Nothing in the module spells
    // a phrase out.
    let answer = final_answer(
        "Подготовьте отчёт с содержанием: уровень дерева, результаты тестов и \
         идентификатор сессии.",
    )
    .expect("a Russian document specification should be composed too");
    for part in [
        "уровень дерева",
        "результаты тестов",
        "идентификатор сессии",
    ] {
        assert!(
            answer.contains(&format!("- {part}")),
            "composed note omits {part:?}: {answer}"
        );
    }
}

#[test]
fn one_named_subject_is_a_question_rather_than_a_document_specification() {
    // Two or more parts is what makes a request a specification of a document's
    // structure. One is a thing to find out, and the ordinary routes answer it;
    // composing a one-bullet note in their place would replace an answer with a
    // form.
    for prompt in [
        "Prepare a summary containing the current exchange rate.",
        "Produce a note containing the population of Lisbon.",
    ] {
        let answer = final_answer(prompt).unwrap_or_default();
        assert!(
            !answer.contains("Requested parts:"),
            "composed a note for a single-subject question {prompt:?}: {answer}"
        );
    }
}

/// Every tool a plan calls, in the order it calls them.
fn planned_tools(prompt: &str) -> Vec<String> {
    let Some(AgenticPlan::ToolCalls(calls)) = plan(prompt) else {
        return Vec::new();
    };
    calls.iter().map(|call| call.tool.clone()).collect()
}

/// Every search subject a plan carries, whatever key the tool names it under.
fn planned_queries(prompt: &str) -> Vec<String> {
    let Some(AgenticPlan::ToolCalls(calls)) = plan(prompt) else {
        return Vec::new();
    };
    calls
        .iter()
        .filter_map(|call| serde_json::from_str::<serde_json::Value>(&call.arguments).ok())
        .filter_map(|value| {
            ["query", "pattern", "q", "command"].iter().find_map(|key| {
                value
                    .get(*key)
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned)
            })
        })
        .collect()
}

#[test]
fn a_question_about_the_repository_is_answered_by_reading_the_repository() {
    // The ladder's interior nodes ask the agent to look at the material it was
    // handed — "inspect the decomposition data model and identify where a node
    // stores its children". Nothing in that says *search*, and the repository
    // search route used to need the word, so the request reached the open-web
    // routers and the answer to a question about the code in front of the agent
    // was looked for on the internet.
    for (prompt, subject) in [
        (
            "Inspect the existing task_decomposition data model and identify where a node \
             stores its children.",
            "task_decomposition",
        ),
        (
            "Examine the retry-policy helper and confirm that it backs off between \
             attempts.",
            "retry_policy",
        ),
        (
            "Review how AgenticPlan is constructed and identify which branch returns no \
             tool call.",
            "AgenticPlan",
        ),
    ] {
        let queries = planned_queries(prompt);
        assert!(
            queries.iter().any(|query| query.contains(subject)),
            "expected the workspace to be searched for {subject:?} for {prompt:?}, \
             planned {queries:?}"
        );
    }
}

#[test]
fn a_question_the_workspace_cannot_answer_still_reaches_the_open_web() {
    // *Verify* and *check* are not local words by themselves. What decides is
    // the subject: a request whose subject is ordinary prose, or that names an
    // external source outright, has told the planner the answer is not in the
    // repository, and searching the repository for it would be a wrong answer
    // delivered confidently.
    for prompt in [
        "Verify the current exchange rate between the euro and the yen.",
        "Check on the web when the next total solar eclipse is visible from Iceland.",
    ] {
        let tools = planned_tools(prompt);
        assert!(
            !tools
                .iter()
                .any(|tool| tool == "grep" || tool == "codesearch"),
            "searched the workspace for an answer it does not hold for {prompt:?}: {tools:?}"
        );
    }
}
