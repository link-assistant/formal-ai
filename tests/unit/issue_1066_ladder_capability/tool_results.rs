//! What a run says about a result it already has (issue #1066).
//!
//! These pins all watch the same seam: a turn has come back, and the planner has
//! to decide what that answer means before it decides what to do next. Every
//! ladder failure they record was a misreading at that seam -- a search that had
//! matched a hundred lines reported as the command that failed, an observation
//! already made replaced by a verdict nobody asked for, an answer in hand filed
//! as a step that did not work. They live beside the helpers in
//! [`super`], which they reach through `super::` (a child module may use an
//! ancestor's private items).

use formal_ai::ChatMessage;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::ToolCall;

#[test]
fn a_delivered_answer_is_the_answer_and_not_a_report_of_a_failed_step() {
    // What made the ladder's 63 green nodes hollow: the file existed, opened
    // with the pinned marker, and its body was the error text of the read that
    // should never have run. A proof file made of a failure report is not
    // evidence, so the delivered body has to be the answer to the question the
    // same request asked.
    let writes = super::planned_writes(
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
        let writes = super::planned_writes(prompt);
        assert!(
            writes.is_empty(),
            "invented an evidence file for {prompt:?}: {writes:?}"
        );
    }
}

#[test]
fn an_observation_already_made_is_reported_rather_than_replaced_by_a_verdict() {
    // A request to look at the workspace is planned as a repository search, and
    // the turn after it only has to report what came back. The task-structure
    // route sits ahead of the route that reports a tool result and needs no tool
    // itself, so it answered the same request a second time from thinking alone
    // and the observation was discarded: thirty-one of the ladder's thirty-two
    // leaves recorded a decomposition of their own instructions where their
    // evidence should have been (issue #1066).
    //
    // Standing aside once a tool has run is the rule the two routes on either
    // side of it already follow. An answer that needs no evidence may not
    // overrule evidence that was gathered for the same request.
    let prompt = "Audit the shipment_scheduler module and locate where it splits a \
                  delivery task into sub-tasks.";
    let observed = "src/shipment_scheduler.rs:214: fn split_delivery(task: &Task) -> [Leg; 2]";
    let messages = vec![
        ChatMessage::user(prompt),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "search-0",
            "grep",
            r#"{"pattern":"shipment_scheduler","query":"shipment_scheduler"}"#.to_owned(),
        )]),
        ChatMessage::tool_result("search-0", "grep", observed),
    ];
    let plan = plan_chat_step(&messages, &super::LADDER_TOOLS);
    let Some(AgenticPlan::Final(answer)) = plan else {
        panic!("expected the search result to be reported, planned {plan:?}");
    };
    assert_eq!(
        answer,
        "The `grep` command completed. Output:\n\n```text\n\
         src/shipment_scheduler.rs:214: fn split_delivery(task: &Task) -> [Leg; 2]\n```"
    );
}

#[test]
fn a_workspace_search_that_ran_is_not_reported_as_a_lookup_that_returned_nothing() {
    // A completed tool call belongs to whichever route made it. The research
    // route decides what to do next by looking at the last one, and when that
    // last one was a workspace search it had not made, it took the search for a
    // round of its own that had come back empty and composed a report saying so
    // -- over the top of the result the agent was already holding. Five of the
    // ladder's thirty-two leaves recorded exactly that as their evidence:
    // "Research completed for ..., but the tool returned no content.", with the
    // matching `grep` output unused in the same transcript.
    let after_a_search = |request: &str| {
        vec![
            ChatMessage::user(request),
            ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                "search-0",
                "grep",
                r#"{"pattern":"two_node","query":"two_node"}"#.to_owned(),
            )]),
            ChatMessage::tool_result(
                "search-0",
                "grep",
                "tests/decomposition.rs:1: fn two_node_tree_has_two_leaves() {}",
            ),
        ]
    };

    // This request names the open web itself, so reaching it is still right --
    // by issuing the search, which is the one thing the old arm never did.
    let plan = plan_chat_step(
        &after_a_search("Search the web for how the Dvorak keyboard layout was standardised."),
        &super::LADDER_TOOLS,
    );
    let Some(AgenticPlan::ToolCalls(calls)) = plan else {
        panic!("expected the stated search to be issued, planned {plan:?}");
    };
    let issued = calls
        .iter()
        .map(|call| format!("{} {}", call.tool, call.arguments))
        .collect::<Vec<_>>();
    assert_eq!(
        issued,
        vec![r#"websearch {"query":"how the dvorak keyboard layout was standardised"}"#.to_owned()]
    );

    // This one states its subject in the first block and only places the worker
    // in the second, so nothing sends it to the open web. The search that ran is
    // the answer.
    let plan = plan_chat_step(
        &after_a_search(
            "Add or verify regression coverage for a two-node decomposition at depth one.\n\n\
             This is recursive binary-tree node 1.2.1.2.1 at depth 5. Solve only this node's \
             task in this fresh temporary repository. Use web research when it materially \
             improves factual accuracy. Do not claim success without evidence.",
        ),
        &super::LADDER_TOOLS,
    );
    let Some(AgenticPlan::Final(answer)) = plan else {
        panic!("expected the search result to be reported, planned {plan:?}");
    };
    assert_eq!(
        answer,
        "The `grep` command completed. Output:\n\n```text\n\
         tests/decomposition.rs:1: fn two_node_tree_has_two_leaves() {}\n```"
    );
}

#[test]
fn a_matched_line_that_quotes_an_error_is_not_this_command_s_failure() {
    // A search answers with other files' text. When one of those files happens
    // to contain the words an installer prints when something is missing, the
    // words are that file's, not the search's -- but the failure lexicon read
    // them as this step's diagnosis, and the whole matched listing went out
    // under "The command failed:". Issue #1066's offline run caught it on a node
    // whose only evidence was a `grep` that had matched fifty lines; the proof
    // it left behind reported the search as broken.
    let matched = "src/billing/legacy_rates.rs:41:    // the historical rate table \
                   was not found, so the default applies\n\
                   src/billing/invoice_total.rs:88: fn invoice_total(lines: &[Line]) -> Money {";
    let messages = vec![
        ChatMessage::user(
            "Inspect the existing invoice_total helper and record how it rounds a half cent.",
        ),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "search-0",
            "grep",
            r#"{"pattern":"invoice_total","query":"invoice_total"}"#.to_owned(),
        )]),
        ChatMessage::tool_result("search-0", "grep", matched),
    ];
    let plan = plan_chat_step(&messages, &super::LADDER_TOOLS);
    let Some(AgenticPlan::Final(answer)) = plan else {
        panic!("expected the matched lines to be reported, planned {plan:?}");
    };
    assert_eq!(
        answer,
        format!("The `grep` command completed. Output:\n\n```text\n{matched}\n```")
    );

    // What separates them is where the citation falls, not the vocabulary: a
    // harness reporting its own refusal names a path and a reason and cites no
    // line at all, and that is still a failure.
    let messages = vec![
        ChatMessage::user(
            "Inspect the existing invoice_total helper and record how it rounds a half cent.",
        ),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "search-1",
            "grep",
            r#"{"pattern":"invoice_total","query":"invoice_total"}"#.to_owned(),
        )]),
        ChatMessage::tool_result(
            "search-1",
            "grep",
            "grep: /etc/shadow: Permission denied, and the index was not found",
        ),
    ];
    let plan = plan_chat_step(&messages, &super::LADDER_TOOLS);
    let Some(AgenticPlan::Final(answer)) = plan else {
        panic!("expected the refusal to be reported, planned {plan:?}");
    };
    assert!(
        answer.starts_with("The command failed:"),
        "a harness refusal stopped being read as a failure, answered {answer:?}"
    );
}

#[test]
fn a_search_that_announces_its_matches_before_quoting_them_is_not_a_failure() {
    // The same search renders two ways. One harness prefixes every hit with the
    // file and the line it came from; `@link-assistant/agent` counts the hits
    // first, then groups them under the file they came from and numbers each
    // quoted line inside the group. A rule that looked at the opening line for a
    // located hit therefore recognised one shape and not the other, so the live
    // #1066 ladder opened its first node's proof with "The command failed: Found
    // 100 matches" -- the changelog that search had matched happens to talk
    // about tasks that fail.
    //
    // A result's own voice is what it says before it starts quoting: here, a
    // count and a heading. Nothing in that framing reports a failure, and the
    // words that do report one belong to the two files being quoted.
    let matched = concat!(
        "Found 3 matches\n",
        "/tmp/tmp.Qh7Kx2/CHANGELOG.md:\n",
        "  Line 65: - Restock planning: a reorder that cannot be filled is split, so a failed leg is retried on its own\n",
        "\n",
        "/tmp/tmp.Qh7Kx2/warehouse/restock_threshold.rs:\n",
        "  Line 12: /// The reorder point was not found in the ledger, so the floor stands.\n",
        "  Line 31: pub fn restock_threshold(ledger: &Ledger) -> Units {",
    );
    let messages = vec![
        ChatMessage::user(
            "Inspect the existing restock_threshold helper and identify where the \
             reorder point is stored.",
        ),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "hits-0",
            "grep",
            r#"{"pattern":"restock_threshold","query":"restock_threshold"}"#.to_owned(),
        )]),
        ChatMessage::tool_result("hits-0", "grep", matched),
    ];
    let plan = plan_chat_step(&messages, &super::LADDER_TOOLS);
    let Some(AgenticPlan::Final(answer)) = plan else {
        panic!("expected the grouped matches to be reported, planned {plan:?}");
    };
    assert_eq!(
        answer,
        format!("The `grep` command completed. Output:\n\n```text\n{matched}\n```")
    );

    // One cited number is not a body of quotations. `HTTP/1.1 404: Not Found`
    // puts a number before a colon exactly as a numbered quotation does, and it
    // quotes nobody -- it is the whole of what the fetch has to say, so it is
    // still read as the failure it is.
    let messages = vec![
        ChatMessage::user(
            "Look up the published reorder point for the winter restock and report it.",
        ),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "fetch-0",
            "webfetch",
            r#"{"url":"https://example.invalid/winter-restock"}"#.to_owned(),
        )]),
        ChatMessage::tool_result("fetch-0", "webfetch", "HTTP/1.1 404: Not Found"),
    ];
    let plan = plan_chat_step(&messages, &super::LADDER_TOOLS);
    let Some(AgenticPlan::Final(answer)) = plan else {
        panic!("expected the status line to be reported, planned {plan:?}");
    };
    assert!(
        answer.starts_with("The command failed:"),
        "a status line stopped being read as a failure, answered {answer:?}"
    );
}

#[test]
fn a_recorded_workspace_result_prefers_the_line_that_answers_the_question() {
    // Agent groups grep results by file, and a broad module search can put a
    // release-note hit ahead of the source declaration the caller requested.
    // The delivery field must carry the grounded answer, not merely the first
    // quotation in transport order.
    let prompt = "Atomic task L01: Inspect the existing task_decomposition data model and \
                  identify where a node stores its children. Record the finding in \
                  `audit/result.lino` with the exact field line `result=`.";
    let matched = concat!(
        "Found 4 matches\n",
        "/tmp/work/CHANGELOG.md:\n",
        "  Line 65: - Failure-driven splitting gained a task decomposition hook.\n",
        "\n",
        "/tmp/work/src/task_decomposition.rs:\n",
        "  Line 12: //! A task decomposition is a recursive tree.\n",
        "  Line 79:     pub children: Vec<Self>,\n",
        "  Line 106:         1 + self.children.iter().map(Self::node_count).sum::<usize>()",
    );
    let messages = vec![
        ChatMessage::user(prompt),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "search-children",
            "grep",
            r#"{"pattern":"task_decomposition","query":"task_decomposition"}"#.to_owned(),
        )]),
        ChatMessage::tool_result("search-children", "grep", matched),
    ];

    let plan = plan_chat_step(&messages, &super::LADDER_TOOLS);
    let Some(AgenticPlan::ToolCalls(calls)) = plan else {
        panic!("expected the grounded result to be written, planned {plan:?}");
    };
    let contents = calls
        .iter()
        .filter_map(|call| serde_json::from_str::<serde_json::Value>(&call.arguments).ok())
        .filter_map(|arguments| super::argument(&arguments, &["content", "contents", "text"]))
        .collect::<Vec<_>>();
    assert!(
        contents
            .iter()
            .any(|content| content.contains("result=Line 79:     pub children: Vec<Self>,")),
        "the recorded field did not select the requested source fact: {contents:?}"
    );
    assert!(
        contents
            .iter()
            .all(|content| !content.contains("Failure-driven splitting")),
        "a release-note match was recorded as the task result: {contents:?}"
    );
}

#[test]
fn a_workspace_result_prefers_the_authoritative_bounded_field() {
    // Live L02 searched broadly for the task-decomposition depth limit. The
    // first result was an example's `//! Usage: ... [max_depth]` comment. Its
    // label-and-colon shape looked like a declaration and its filename repeated
    // more query terms than the actual field, so it displaced `pub max_depth`.
    let prompt = "Atomic task L02: Inspect the existing task-decomposition recursion and \
                  record how depth limits are represented. Record the finding in \
                  `audit/result.lino` with the exact field line `result=`.";
    let matched = concat!(
        "Found 6 matches\n",
        "/tmp/work/src/solver_handlers/task_decomposition.rs:\n",
        "  Line 65:     max_depth: u8,\n",
        "\n",
        "/tmp/work/examples/dump_task_decomposition.rs:\n",
        "  Line 3: //! Usage: `cargo run --example dump_task_decomposition -- \\\"<task>\\\" [max_depth]`\n",
        "  Line 12:     let max_depth: u8 = args.next().unwrap().parse()?;\n",
        "  Line 220:         \"a generous depth must reach atomic leaves: {:?}\",\n",
        "\n",
        "/tmp/work/src/task_decomposition.rs:\n",
        "  Line 72:     pub depth: u8,\n",
        "  Line 180:     pub max_depth: u8,",
    );
    let messages = vec![
        ChatMessage::user(prompt),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "search-depth",
            "grep",
            r#"{"pattern":"task_decomposition|max_depth","query":"task_decomposition|max_depth"}"#
                .to_owned(),
        )]),
        ChatMessage::tool_result("search-depth", "grep", matched),
    ];

    let plan = plan_chat_step(&messages, &super::LADDER_TOOLS);
    let Some(AgenticPlan::ToolCalls(calls)) = plan else {
        panic!("expected the grounded result to be written, planned {plan:?}");
    };
    let contents = calls
        .iter()
        .filter_map(|call| serde_json::from_str::<serde_json::Value>(&call.arguments).ok())
        .filter_map(|arguments| super::argument(&arguments, &["content", "contents", "text"]))
        .collect::<Vec<_>>();
    assert!(
        contents
            .iter()
            .any(|content| content.contains("result=Line 180:     pub max_depth: u8,")),
        "the recorded field selected a usage comment instead of the declaration: {contents:?}"
    );
}

#[test]
fn a_check_result_prefers_the_condition_over_its_field_declaration() {
    // A narrow identifier search can return both the model field and the
    // predicate that checks it. When the requested source artifact is a check,
    // recording the field declaration describes storage but not the condition
    // the caller asked to observe.
    let prompt = "Review the existing readiness check and inspect the observable completion \
                  contract for workers. Record the finding in `audit/result.lino` with the \
                  exact field line `result=`.";
    let matched = concat!(
        "Found 5 matches\n",
        "/tmp/work/tests/readiness.rs:\n",
        "  Line 179:         \"  Line 89: && !self.completion_criterion.starts_with(\\\"unresolved_\\\")\\n\",\n",
        "  Line 229:             && content.contains(\"!self.completion_criterion.starts_with\")\n",
        "/tmp/work/src/work.rs:\n",
        "  Line 70:     pub completion_criterion: String,\n",
        "  Line 89:             && !self.completion_criterion.starts_with(\"unresolved_\")\n",
        "  Line 130:                 self.completion_criterion.clone(),",
    );
    let messages = vec![
        ChatMessage::user(prompt),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "search-contract",
            "grep",
            r#"{"pattern":"completion_criterion","query":"readiness"}"#.to_owned(),
        )]),
        ChatMessage::tool_result("search-contract", "grep", matched),
    ];

    let plan = plan_chat_step(&messages, &super::LADDER_TOOLS);
    let Some(AgenticPlan::ToolCalls(calls)) = plan else {
        panic!("expected the checked condition to be written, planned {plan:?}");
    };
    let contents = calls
        .iter()
        .filter_map(|call| serde_json::from_str::<serde_json::Value>(&call.arguments).ok())
        .filter_map(|arguments| super::argument(&arguments, &["content", "contents", "text"]))
        .collect::<Vec<_>>();
    assert!(
        contents.iter().any(|content| content.contains(
            "result=Line 89:             && !self.completion_criterion.starts_with(\"unresolved_\")"
        )),
        "the recorded field did not select the checked condition: {contents:?}"
    );
}
