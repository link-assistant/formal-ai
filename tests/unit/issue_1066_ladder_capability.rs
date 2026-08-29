//! Capability pins for the recursive agent ladder (issue #1066).
//!
//! Issue #1066 requires `issue-1028-agent-ladder.yml` to complete, and states
//! that a failure at any depth "is a real capability gap, not a flake, and
//! should be fixed generically — never with a prompt-specific branch". These
//! tests therefore pin the *general* behaviour each observed ladder failure
//! exposed, and prove it with wording the ladder never uses.

use formal_ai::ChatMessage;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::ToolCall;

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

/// How many turns a replayed run is given to reach the file it was asked for.
///
/// The ladder runs its nodes with a turn budget too; this is a smaller one,
/// because a literal write that has not happened within a handful of turns is
/// the failure these tests exist to catch, not a run that needs longer.
const LADDER_TURN_CAP: usize = 6;

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
    for (prompt, target, pinned) in [
        (
            "Draft a handover memo containing the migration status, the outstanding \
             blockers, and the on-call owner. Leave the memo in `handover/2026-q3.md`. \
             The first line must be exactly `handover=q3`.",
            "handover/2026-q3.md",
            "handover=q3",
        ),
        (
            "Assemble a shift summary containing the machines serviced, the parts \
             replaced, and the hours logged. Record it in `shift/friday.md`. The first \
             line must be exactly `shift=friday`.",
            "shift/friday.md",
            "shift=friday",
        ),
    ] {
        // Only the caller's file is judged. A route may keep a record of its own
        // reasoning beside the work, and that record is not the file the
        // request pinned a first line on.
        let writes = planned_writes_to(prompt, target);
        assert!(
            !writes.is_empty(),
            "nothing was planned to be written to {target:?} for {prompt:?}"
        );
        for content in &writes {
            assert!(
                content.starts_with(pinned),
                "wrote a file that does not open with {pinned:?} for {prompt:?}: {content:?}"
            );
        }
    }
}

/// Every `content` the run writes to `target`, across the turns it takes to
/// reach it.
///
/// A literal write is not always the first thing a run does. The general change
/// route records the plan it composed before it executes that plan, so the
/// caller's file is written on a later turn, and judging turn one alone judges
/// the plan record instead of the file the request pinned a first line on. The
/// turns are therefore replayed the way the Agent CLI replays them, with each
/// planned call answered and fed back before the next one is asked for.
fn planned_writes_to(prompt: &str, target: &str) -> Vec<String> {
    let mut messages = vec![ChatMessage::user(prompt)];
    let mut written = Vec::new();
    for turn in 0..LADDER_TURN_CAP {
        let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &LADDER_TOOLS) else {
            break;
        };
        for (index, call) in calls.iter().enumerate() {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&call.arguments)
                && argument(&value, &["path", "filePath", "file_path", "absolute_path"])
                    .is_some_and(|path| path.ends_with(target))
                && let Some(content) =
                    argument(&value, &["content", "contents", "text", "new_string"])
            {
                written.push(content);
            }
            let id = format!("ladder-turn-{turn}-{index}");
            messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                &id,
                &call.tool,
                call.arguments.clone(),
            )]));
            messages.push(ChatMessage::tool_result(id, &call.tool, "ok"));
        }
    }
    written
}

/// The first of `keys` the tool arguments carry, as a string.
fn argument(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        value
            .get(*key)
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
    })
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

/// Every path a plan opens for reading, whatever key the tool names it under.
fn planned_reads(prompt: &str) -> Vec<String> {
    let Some(AgenticPlan::ToolCalls(calls)) = plan(prompt) else {
        return Vec::new();
    };
    calls
        .iter()
        .filter(|call| call.tool == "read" || call.tool == "cat")
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
fn a_file_the_request_asks_for_is_never_opened_for_reading() {
    // A request can ask for work and name where the answer goes, and constrain
    // how that file has to open. *First line* is a reading cue, so read the
    // prompt as one span and the cue captures the delivery path: the run opened
    // the file it was asked to create, got "No such file or directory", and
    // recorded that error as its evidence. Whatever cue accompanies it, a path
    // the request states as a write destination is not a file to read.
    for (prompt, target) in [
        (
            "Decide whether rewriting the deployment script is a single atomic task. \
             Leave the outcome in `triage/deploy.md`. The first line must be exactly \
             `triage=deploy`.",
            "triage/deploy.md",
        ),
        (
            "Judge whether the billing database migration is a problem you can break \
             down. Record the finding in `triage/billing.md`. The first line must be \
             exactly `triage=billing`.",
            "triage/billing.md",
        ),
        // A request can also be delivery and nothing else. Both prompts above
        // ask a question about a task, so the task-structure route answers them
        // and the read planner is never consulted -- which leaves the guard
        // unexercised by them. These two state only where the answer goes and
        // how it opens, so the read planner is exactly what decides them, and
        // the guard is what it decides with.
        (
            "Leave observable evidence in `audit/ledger-check.md`. The first line must \
             be exactly `audit=ledger-check`.",
            "audit/ledger-check.md",
        ),
        (
            "Put the outcome in `runs/nightly-42.md`. The first line must be exactly \
             `run=nightly-42`.",
            "runs/nightly-42.md",
        ),
    ] {
        let reads = planned_reads(prompt);
        assert!(
            !reads.iter().any(|path| path == target),
            "opened the file it was asked to write for {prompt:?}: {reads:?}"
        );
    }
}

#[test]
fn a_file_the_request_only_mentions_is_still_opened_for_reading() {
    // The gate above must cost nothing to a request that reads one file and
    // writes another: only the path a stated write names is protected, and the
    // pairing is read one sentence at a time so the read path keeps its meaning.
    let prompt = "Read the file `Cargo.toml`. Record the package name in `notes/report.md`.";
    let reads = planned_reads(prompt);
    assert!(
        reads.iter().any(|path| path.ends_with("Cargo.toml")),
        "the file to read was not opened for {prompt:?}: {reads:?}"
    );
}

#[test]
fn an_answer_only_the_symbolic_engine_reaches_is_still_delivered_to_the_named_file() {
    // The ladder's interior nodes ask a question about task structure and then
    // say where the answer goes. No tool answers that question -- Formal AI's
    // symbolic engine does -- so a delivery that only forwards what the agentic
    // router planned drops the obligation entirely and writes nothing. The
    // caller asked for a file; the file is what has to appear.
    for (prompt, target, pinned) in [
        (
            "Decide whether rewriting the deployment script is a single atomic task. \
             Leave the outcome in `triage/deploy.md`. The first line must be exactly \
             `triage=deploy`.",
            "triage/deploy.md",
            "triage=deploy",
        ),
        (
            "Judge whether the billing database migration is a problem you can break \
             down. Record the finding in `triage/billing.md`. The first line must be \
             exactly `triage=billing`.",
            "triage/billing.md",
            "triage=billing",
        ),
    ] {
        let paths = planned_paths(prompt);
        assert!(
            paths.iter().any(|path| path == target),
            "nothing was planned to be written to {target} for {prompt:?}: {paths:?}"
        );
        let writes = planned_writes(prompt);
        assert!(
            writes.iter().any(|content| content.starts_with(pinned)),
            "the delivered file does not open with {pinned:?} for {prompt:?}: {writes:?}"
        );
    }
}

#[test]
fn work_coordinated_into_its_delivery_sentence_is_not_thrown_away_with_it() {
    // Two sentences are the tidy way to ask for work and then say where it
    // goes, and the ladder's own nodes are written that way. English does not
    // require it: one sentence can state the work, say "and", and name the
    // destination. A reader that hands the whole sentence to delivery keeps the
    // destination and loses the work, so the request is answered in the
    // transcript and the named file never appears.
    for (prompt, target, pinned, subject) in [
        (
            "Break the customer import rewrite into sub-tasks and record what you work \
             out in `import-split.md`. The first line must be exactly \
             `plan_for=customer-import`.",
            "import-split.md",
            "plan_for=customer-import",
            "customer import",
        ),
        (
            "Split the invoice archiver migration into sub-tasks and write down what \
             you decide in `archive-steps.md`. The first line must be exactly \
             `plan_for=invoice-archiver`.",
            "archive-steps.md",
            "plan_for=invoice-archiver",
            "invoice archiver",
        ),
    ] {
        let paths = planned_paths(prompt);
        assert!(
            paths.iter().any(|path| path == target),
            "nothing was planned to be written to {target} for {prompt:?}: {paths:?}"
        );
        let writes = planned_writes(prompt);
        assert!(
            writes.iter().any(|content| content.starts_with(pinned)),
            "the delivered file does not open with {pinned:?} for {prompt:?}: {writes:?}"
        );
        assert!(
            writes
                .iter()
                .any(|content| content.to_lowercase().contains(subject)),
            "the delivered file says nothing about {subject:?}, so the work in front \
             of the delivery clause was dropped for {prompt:?}: {writes:?}"
        );
    }
}

#[test]
fn a_delivered_answer_is_the_answer_and_not_a_report_of_a_failed_step() {
    // What made the ladder's 63 green nodes hollow: the file existed, opened
    // with the pinned marker, and its body was the error text of the read that
    // should never have run. A proof file made of a failure report is not
    // evidence, so the delivered body has to be the answer to the question the
    // same request asked.
    let writes = planned_writes(
        "Decide whether rewriting the deployment script is a single atomic task. \
         Leave the outcome in `triage/deploy.md`. The first line must be exactly \
         `triage=deploy`.",
    );
    assert!(
        writes
            .iter()
            .any(|content| content.contains("sub-task") || content.contains("atomic")),
        "the delivered body does not answer the question that was asked: {writes:?}"
    );
    assert!(
        !writes
            .iter()
            .any(|content| content.contains("No such file or directory")),
        "recorded a failed step as the evidence: {writes:?}"
    );
}

#[test]
fn nothing_is_recorded_when_no_answer_was_reached() {
    // The delivery is only worth having while it stays honest: a residual the
    // engine cannot conclude anything about must leave the file unwritten
    // rather than fill it with the engine's inability to answer.
    for prompt in [
        "??? Leave the outcome in `triage/nothing.md`.",
        "asdkjhqwe zxcvbnm. Record the finding in `triage/noise.md`.",
    ] {
        let writes = planned_writes(prompt);
        assert!(
            writes.is_empty(),
            "invented an evidence file for {prompt:?}: {writes:?}"
        );
    }
}

#[test]
fn a_question_about_a_task_is_answered_by_thinking_about_the_task() {
    // Thirty of the ladder's sixty-three nodes ask what a task decomposes into.
    // Nothing on the open web knows the caller's task, and the recursion that
    // does know it is Formal AI's own -- but the agentic planner had no route to
    // it, so the question fell past every tool-using route to the research
    // routers and came back as a search for its own words. A question about a
    // task's structure needs no tool at all.
    for prompt in [
        "Break the customer import rewrite into sub-tasks.",
        "Разбей переработку импорта клиентов на подзадачи.",
    ] {
        let answer = final_answer(prompt)
            .unwrap_or_else(|| panic!("no answer was planned for {prompt:?}: {:?}", plan(prompt)));
        assert!(
            !answer.trim().is_empty(),
            "the answer to {prompt:?} was empty"
        );
        assert!(
            planned_queries(prompt).is_empty(),
            "searched the open web for a question about the caller's own task: {prompt:?}"
        );
    }
}

#[test]
fn a_path_that_closes_a_sentence_is_still_a_path() {
    // Prose nests its punctuation and the writer picks the nesting, so the same
    // path arrives as `` `Cargo.toml`. `` at the end of a sentence and as
    // `` `Cargo.toml`, `` inside one. Stripping each class once cleared the
    // outer layer and left the inner one, the token stopped being file-shaped,
    // and the plainest read request there is went to the open web.
    for prompt in [
        "Show me the contents of `Cargo.toml`.",
        "Read `Cargo.toml`, then stop.",
        "Open (`Cargo.toml`).",
    ] {
        let reads = planned_reads(prompt);
        assert!(
            reads.iter().any(|path| path.ends_with("Cargo.toml")),
            "the path was not recovered from {prompt:?}: {reads:?}"
        );
    }
}

#[test]
fn a_payload_that_names_the_work_product_is_not_written_as_the_body() {
    // "Save the result in FILE" says where an answer goes; it does not say what
    // the answer is. Read as a literal write, the phrase naming the work product
    // became the file's contents, so the ladder's proof files contained the word
    // "result" and nothing else -- non-empty, and evidence of nothing.
    for (prompt, deferred) in [
        (
            "Count the sub-tasks of the customer import rewrite and save the result in \
             `counts.md`.",
            "the result",
        ),
        (
            "Judge whether the customer import rewrite is one task and record the finding \
             in `verdict.md`.",
            "the finding",
        ),
    ] {
        for content in planned_writes(prompt) {
            assert!(
                content.trim() != deferred,
                "wrote the name of the work product as the work product for {prompt:?}"
            );
        }
    }
}

#[test]
fn a_politely_phrased_write_is_still_a_write() {
    // Russian states a request to a stranger in the plural imperative, and the
    // seed knew only the familiar singular, so every polite phrasing of the
    // plainest write missed the route entirely. The lexicon is the fix; this
    // pins that it stays.
    // The plan is compared against the familiar phrasing of the same request
    // rather than described here: what has to hold is that politeness changes
    // nothing, and stating the expected plan would pin this route's internals
    // instead.
    for (polite, familiar) in [
        (
            "Создайте файл `hello.txt` с текстом привет.",
            "Создай файл `hello.txt` с текстом привет.",
        ),
        (
            "Сохраните файл `notes.txt` с текстом заметка.",
            "Сохрани файл `notes.txt` с текстом заметка.",
        ),
        (
            "Сделайте `report.md` с текстом итог квартала.",
            "Сделай `report.md` с текстом итог квартала.",
        ),
    ] {
        let planned = planned_writes(polite);
        assert!(
            !planned.is_empty(),
            "the politely phrased write planned nothing: {polite:?}"
        );
        assert_eq!(
            planned.len(),
            planned_writes(familiar).len(),
            "politeness changed how much was planned for {polite:?}"
        );
        for (polite_plan, familiar_plan) in planned.iter().zip(planned_writes(familiar)) {
            assert_eq!(
                without_derived_ids(polite_plan, polite),
                without_derived_ids(&familiar_plan, familiar),
                "politeness changed the plan for {polite:?}"
            );
        }
    }
}

/// How many hexadecimal digits a content-derived identifier runs to.
const DERIVED_ID_DIGITS: usize = 16;

/// The same planned content with the request and every derived identifier
/// redacted.
///
/// A plan record names itself and its impulse after a hash of the request it
/// came from, so two phrasings of one request necessarily carry two sets of
/// identifiers. That is the record doing its job, and it is not what this test
/// is about: what has to hold is that politeness changes nothing else.
fn without_derived_ids(plan: &str, request: &str) -> String {
    let mut redacted = String::new();
    let mut digits = String::new();
    let flush = |digits: &mut String, out: &mut String| {
        if digits.len() >= DERIVED_ID_DIGITS {
            out.push_str("ID");
        } else {
            out.push_str(digits);
        }
        digits.clear();
    };
    for character in plan.replace(request, "REQUEST").chars() {
        if character.is_ascii_hexdigit() {
            digits.push(character);
        } else {
            flush(&mut digits, &mut redacted);
            redacted.push(character);
        }
    }
    flush(&mut digits, &mut redacted);
    redacted
}

#[test]
fn a_read_cue_selects_the_path_in_its_own_sentence() {
    // A request can name a file in passing and then ask for a different one to
    // be opened. Deciding the read target over the whole prompt takes the first
    // file-shaped token it finds, which is the one the caller only mentioned,
    // and the file they asked for is never opened. The cue and its path belong
    // to one sentence, and that is the scope the pairing is read at.
    let prompt = "The template lives at `docs/template.md`. Read `Cargo.toml` and tell me \
                  the package name.";
    let reads = planned_reads(prompt);
    assert!(
        reads.iter().any(|path| path.ends_with("Cargo.toml")),
        "opened a file the request only mentioned instead of the one it named: {reads:?}"
    );
}

#[test]
fn a_literal_payload_never_crosses_a_sentence_boundary() {
    // "Compose X containing A, B and C. Leave it in FILE." states the payload in
    // one sentence and the destination in the next. Bounding the payload by the
    // destination clause alone read straight through the full stop, so the words
    // of the delivery instruction became part of the document delivered: the
    // memo ended with "Leave the memo". A payload is something a sentence says,
    // and it stops where that sentence does.
    for (prompt, target, intruder) in [
        (
            "Draft a handover memo containing the migration status and the on-call \
             owner. Leave the memo in `handover/2026-q3.md`.",
            "handover/2026-q3.md",
            "Leave the memo",
        ),
        (
            "Assemble a shift summary containing the parts replaced and the hours \
             logged. Record it in `shift/friday.md`.",
            "shift/friday.md",
            "Record it",
        ),
    ] {
        for content in planned_writes_to(prompt, target) {
            assert!(
                !content.contains(intruder),
                "wrote the delivery instruction into the document for {prompt:?}: {content:?}"
            );
        }
    }
}
