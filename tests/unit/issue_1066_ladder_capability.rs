//! Capability pins for the recursive agent ladder (issue #1066).
//!
//! Issue #1066 requires `issue-1028-agent-ladder.yml` to complete, and states
//! that a failure at any depth "is a real capability gap, not a flake, and
//! should be fixed generically — never with a prompt-specific branch". These
//! tests therefore pin the *general* behaviour each observed ladder failure
//! exposed, and prove it with wording the ladder never uses.
//!
//! They are grouped by the seam each one guards: [`written_files`] for the bytes
//! a run puts in the file it was asked for, [`tool_results`] for what it makes
//! of an answer already in hand, and this module for the routing that decides
//! which of those a request is asking for. The planning helpers all three share
//! live here, and the two child modules reach them through `super::`.

use formal_ai::ChatMessage;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::ToolCall;

mod tool_results;
mod written_files;

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
fn a_specified_document_is_composed_even_when_the_request_names_a_file() {
    // A specification and a destination are two different sentences, and the
    // ladder's last leaf writes them that way. The literal-write parser saw a
    // cued path and the word "containing", took the words after it for the
    // bytes, and wrote the request's own wording into the file -- and, by
    // claiming the request, kept the route that composes documents from ever
    // running. Naming a file says where the document goes; it does not turn the
    // specification into the document.
    assert_eq!(
        planned_writes_to(
            "Draft a vendor brief containing the contract owner, the renewal window, \
             and the escalation path. Store it in `vendors/acme.md`.",
            "vendors/acme.md"
        ),
        vec![
            "Draft a vendor brief containing the contract owner, the renewal window, and the \
             escalation path\n\nRequested parts:\n- the contract owner\n- the renewal window\n- \
             the escalation path\n\nObserved in this session:\n- nothing: no tool result was \
             recorded before this note.\n\nNo requested part above is backed by an observation \
             from this session.\n"
                .to_owned()
        ]
    );
}

#[test]
fn a_label_that_calls_the_work_atomic_does_not_replace_it_with_a_verdict() {
    // The ladder hands each leaf a heading -- "Atomic task L32: ..." -- and the
    // heading alone carries both words the task-structure route reads, the
    // atomicity predicate and the task noun. So the route answered the question
    // the *label* posed, truthfully, and wrote "Yes — this task is atomic" where
    // the sentence after the colon had asked for a note. A request that names a
    // document to produce states work to do, not a task to classify.
    assert_eq!(
        planned_writes_to(
            "Atomic task 9: Assemble an intake summary containing the applicant name, the \
             referral source, and the interview date. Store it in `intake/monday.md`.",
            "intake/monday.md"
        ),
        vec![
            "Atomic task 9: Assemble an intake summary containing the applicant name, the \
             referral source, and the interview date\n\nRequested parts:\n- the applicant \
             name\n- the referral source\n- the interview date\n\nObserved in this session:\n- \
             nothing: no tool result was recorded before this note.\n\nNo requested part above \
             is backed by an observation from this session.\n"
                .to_owned()
        ]
    );
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

/// The decoded arguments of every planned tool call.
fn planned_arguments(prompt: &str) -> Vec<serde_json::Value> {
    let Some(AgenticPlan::ToolCalls(calls)) = plan(prompt) else {
        return Vec::new();
    };
    calls
        .iter()
        .filter_map(|call| serde_json::from_str(&call.arguments).ok())
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
fn a_source_artifact_noun_scopes_a_plain_inspection_subject() {
    // Not every source concept contains punctuation or capitals. In "retry
    // check", the artifact noun makes the adjacent plain word a safe local
    // subject just as "retry_policy helper" does; without that grammar the
    // inspection falls through to an answer that never reads the checkout.
    let queries = planned_queries(
        "Review the existing retry check and identify which condition accepts completed work.",
    );
    assert!(
        queries.iter().any(|query| query == "retry"),
        "expected a workspace search for the subject next to `check`, planned {queries:?}"
    );
}

#[test]
fn a_seed_mapped_source_fact_uses_its_narrow_expression() {
    // A natural-language property can have a canonical spelling in source.
    // Keeping every surrounding prose word in an OR expression fills a grep
    // result cap before that property appears, so an explicit seed mapping is
    // the complete search expression rather than one more broad alternative.
    let arguments = planned_arguments(
        "Review the existing readiness check and record the observable completion contract \
         for workers.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "readiness");
    assert_eq!(search["pattern"], "completion_criterion");
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("src/**/*"),
        "a condition inspection should not search generated traces or documentation: {search}"
    );
}

#[test]
fn a_missing_leaf_contract_uses_the_specific_fact() {
    // `completion criterion` has a useful broad source spelling, but the whole
    // phrase asks about the sentinel assigned to a leaf whose contract is
    // absent. The more specific fact must win even though it contains the
    // shorter mapped phrase verbatim.
    let arguments = planned_arguments(
        "Review the existing fallback check and verify that a leaf without an observable \
         completion criterion is not independently verifiable.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "fallback");
    assert_eq!(search["pattern"], "unresolved_single_need");
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("src/**/*")
    );
}

#[test]
fn a_named_format_rendering_is_a_workspace_subject() {
    // A source format can have an ordinary multi-word name with no identifier
    // punctuation. The adjacent artifact noun still makes it a local source
    // question, and generated documentation must not become its evidence.
    let arguments = planned_arguments(
        "Inspect the existing Links Notation rendering and record how child relationships are \
         serialized.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "Notation");
    assert_eq!(
        search["pattern"], r#""child""#,
        "the rendering search did not target the literal relationship key: {search}"
    );
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("src/**/*"),
        "an implementation rendering should exclude generated prose: {search}"
    );
}

#[test]
fn a_serialized_relationship_uses_its_literal_links_key() {
    // A source tree can contain thousands of broad prose hits for `record`,
    // `how`, and `child`. The relationship's serialized Links key is the
    // source-level fact: quoting it keeps a client's match cap from being
    // exhausted by unrelated child collections before any renderer appears.
    let arguments = planned_arguments(
        "Atomic task R17: Inspect the existing Wire Notation rendering and record how parent \
         relationships are encoded.\n\nThis is worker 4.2 in a fresh checkout. Record the observed \
         result and do not claim success without evidence.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["pattern"], r#""parent""#);
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("src/**/*")
    );
}

#[test]
fn a_plain_adapter_is_a_workspace_subject() {
    // Adapter names are often ordinary prose with no underscore or capital.
    // The artifact noun still makes the adjacent subsystem a local source
    // subject, just as `check` and `rendering` do for their own source kinds.
    let arguments = planned_arguments(
        "Inspect the existing transport adapter and identify where operations are dispatched.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "transport");
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("src/**/*")
    );
}

#[test]
fn recursive_tree_execution_targets_its_conversion_entry_point() {
    // Searching broad words such as `recursive`, `tree`, and `executed` fills
    // a result cap with descriptions of recursion. The adapter entry point is
    // the canonical source fact that reveals how the tree is converted.
    let arguments = planned_arguments(
        "Examine the recursive execution adapter and explain the decomposition tree execution.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["pattern"], "to_recursive_task");
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("src/**/*")
    );
}

#[test]
fn approved_task_strategies_target_the_ledger_entry_point() {
    // A hyphenated subsystem can name a concept without naming a source file.
    // Treating `task-strategy` as a module filter searched only nonexistent
    // `*task_strategy*` files, so the real ledger entry point in
    // `task_decomposition.rs` was excluded before grep could inspect it.
    let arguments = planned_arguments(
        "Review the task-strategy ledger and explain how reviewed decomposition strategies are \
         activated.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "task_strategy");
    assert_eq!(search["pattern"], "TaskStrategyLedger::shipped");
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("src/**/*"),
        "a canonical source fact must not be excluded by an inferred module filter: {search}"
    );
}

#[test]
fn a_regression_lower_bound_searches_test_source() {
    // A hyphenated scope such as `issue-scale` is prose, not necessarily a
    // module name. More importantly, this request explicitly names a
    // regression: its canonical fact belongs in test source, where the lower
    // bound is asserted, rather than in the production implementation.
    let arguments = planned_arguments(
        "Review the issue-scale decomposition regression and identify the minimum number of \
         independently verifiable leaves.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    assert_eq!(search["query"], "issue_scale");
    assert_eq!(search["pattern"], "leaves.*len.*>=");
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("tests/**/*"),
        "a regression fact must be sought in test source: {search}"
    );
}

#[test]
fn a_workspace_inspection_search_targets_the_fact_being_requested() {
    // Searching only for `task_decomposition` returns a hundred broad matches,
    // headed by release notes, before the field the caller asked about. The
    // code-shaped module remains useful context, but the grep pattern must name
    // the requested fact and the search must stay inside source code.
    let arguments = planned_arguments(
        "Inspect the existing task_decomposition data model and identify where a node \
         stores its children.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    let pattern = search["pattern"]
        .as_str()
        .expect("the grep pattern must be a string");
    assert!(
        pattern.contains("children"),
        "the search pattern omitted the fact being requested: {search}"
    );
    assert_eq!(
        search.get("include").and_then(serde_json::Value::as_str),
        Some("*task_decomposition*"),
        "a module-shaped subject should exclude unrelated changelogs and docs: {search}"
    );
}

#[test]
fn a_task_label_does_not_become_part_of_the_inspection_query() {
    // A harness may number and classify the task before the colon. Those words
    // describe the work item, not the repository fact after the colon. Letting
    // `atomic` and `task` into the grep filled the result cap and then outranked
    // the requested `children` declaration during evidence selection.
    let arguments = planned_arguments(
        "Atomic task L01: Inspect the existing task-decomposition data model and identify \
         where a node stores its children.",
    );
    let search = arguments
        .iter()
        .find(|arguments| arguments.get("pattern").is_some())
        .unwrap_or_else(|| panic!("no workspace search was planned: {arguments:?}"));
    let pattern = search["pattern"]
        .as_str()
        .expect("the grep pattern must be a string");
    assert!(
        pattern.split('|').any(|term| term == "children"),
        "{search}"
    );
    for label_word in ["atomic", "task", "l01:"] {
        assert!(
            !pattern.split('|').any(|term| term == label_word),
            "task-label word {label_word:?} leaked into the search: {search}"
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
fn a_client_that_speaks_no_computer_use_is_not_told_a_primitive_is_missing() {
    // The seed's computer-use plans are executed by the client that advertises
    // the primitives -- `fs.write`, `shell.run`, and the rest -- and each step
    // carries pre/postconditions that client checks. The Agent CLI advertises
    // none of them; it names its write tool `write`. Reporting the gap to it
    // claimed the request and spent the turn on a sentence about `fs.write`,
    // above the write route that would have written the file.
    let prompt = "Count the sub-tasks of the customer import rewrite and save the result in \
                  `counts.md`.";
    assert!(
        !matches!(plan(prompt), Some(AgenticPlan::Final(answer)) if answer.starts_with("capability_gap:")),
        "the capability gap was reported to a client that advertises no computer-use primitive"
    );
    // A client that *is* running computer-use plans and is missing one
    // primitive is told exactly which one, because there the gap is actionable.
    assert_eq!(
        plan_chat_step(&[ChatMessage::user(prompt)], &["fs.read"]),
        Some(AgenticPlan::Final(
            "capability_gap: required primitive fs.write was not advertised for plan \
             synthesized-computer_use_resource_customers-computer_use_count_lines, step \
             synthesized-computer_use_resource_customers-computer_use_count_lines-01."
                .to_owned()
        ))
    );
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
fn the_note_that_places_the_worker_does_not_name_the_subject_of_the_work() {
    // A request often arrives with a second block that says where the worker is
    // and how to report -- "you are shift 3 of the night-crew rota". That block
    // names nothing the request asks about. Scoring the whole prompt for its most
    // code-shaped word let the longest word of that note win, so a question about
    // the invoice totals searched the workspace for the rota instead.
    let queries = planned_queries(
        "Inspect the existing invoice-total helper and identify how it rounds a half \
         cent.\n\nYou are shift 3 of the night-crew-rota. Work only in this checkout.",
    );
    assert!(
        queries.iter().any(|query| query.contains("invoice_total")),
        "expected the workspace to be searched for the invoice-total helper, \
         planned {queries:?}"
    );
    assert!(
        !queries
            .iter()
            .any(|query| query.contains("night_crew_rota")),
        "took the subject from the block that only places the worker, \
         planned {queries:?}"
    );
}

#[test]
fn a_single_line_worker_contract_does_not_replace_the_inspection_subject() {
    // Agent's second compaction can flatten the original paragraph break. The
    // checkout question and its worker contract then occupy one request block,
    // but the machine-shaped completion token still is not the thing the user
    // asked to inspect.
    let queries = planned_queries(
        "Inspect the existing invoice-total helper and identify how it rounds a half cent. \
         This is validation job 7. Its completion criterion is new_audit_effect. Create \
         `audit-effects/job-7.lino` with the observed result.",
    );
    assert!(
        queries.iter().any(|query| query.contains("invoice_total")),
        "expected the stated helper to remain the inspection subject, planned {queries:?}",
    );
    assert!(
        !queries
            .iter()
            .any(|query| query.contains("new_audit_effect")),
        "searched for the worker contract instead of the inspection subject: {queries:?}",
    );
}

#[test]
fn a_permission_to_use_the_web_is_not_the_answer_to_the_question() {
    // "Use web research when it materially improves factual accuracy" grants a
    // tool; it does not say the answer is on the internet. Read as part of the
    // request's own words, it turned a question about the checkout into an
    // open-web query assembled from the whole prompt, and the evidence recorded
    // was the framing note itself with a line saying the lookup returned nothing.
    let writes = planned_writes(
        "Inspect the existing rounding rule and record when a half cent is kept.\n\n\
         You are worker 4. Use web research when it materially improves factual \
         accuracy. Record what you find in `notes/rounding.md`.",
    );
    assert!(
        !writes
            .iter()
            .any(|content| content.contains("materially improves")),
        "recorded the note that places the worker as the finding: {writes:?}"
    );
}

#[test]
fn an_open_web_query_stops_at_the_end_of_the_request() {
    // The subject of a search is stated in the request; the block after it says
    // where the worker is and how to report. Read as one string, the second
    // block became part of the query, blank line and all, and the ladder logged
    // what that costs: node 1.2.1.2.1 searched for "a two node decomposition at
    // depth one this is recursive binary tree node 1 2 1 2 1 at depth 5 solve
    // only this node s task in this fresh temporary repository its completion
    // criterion is observable evidence exists use web research when it
    // materially improves factual accuracy do not claim success without
    // evidence" -- a query no source on earth answers.
    //
    // Three different routes form an open-web query, and every one of them had
    // the same fault, so all three are pinned here with the query each must now
    // produce.
    for (prompt, expected) in [
        (
            "Verify the current exchange rate between the euro and the yen.\n\n\
             Work only in this checkout.",
            "Verify the current exchange rate between the euro and the yen",
        ),
        (
            "Search the web for how the Dvorak keyboard layout was standardised.\n\n\
             You are node 7 of the survey. Work only in this checkout.",
            "the web for how the dvorak keyboard layout was standardised",
        ),
        (
            "What is a hash-consed trie?\n\n\
             You are node 7 of the survey. Work only in this checkout.",
            "hash consed trie",
        ),
    ] {
        assert_eq!(
            planned_queries(prompt),
            vec![expected.to_owned()],
            "the query carried the block that only places the worker for {prompt:?}"
        );
    }
}
