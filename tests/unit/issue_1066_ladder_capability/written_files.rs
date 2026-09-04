//! The bytes a run puts in the file it was asked to write (issue #1066).
//!
//! A request that names a file is only served when the file arrives with the
//! contents the request stated -- spelled out literally, opened on the line the
//! same request pinned, and carrying the work rather than the name of the work.
//! These pins record the ways the ladder got that wrong. They reach the planning
//! helpers in [`super`] through `super::` (a child module may use an ancestor's
//! private items).

#[test]
fn a_written_file_starts_with_the_line_the_same_request_pinned() {
    // A request can state two things about one file: what it has to contain and
    // how it has to begin. The literal-write parser read the first and ignored
    // the second, so it composed bytes out of the prose and wrote a file that
    // broke a constraint stated three sentences earlier. A literal write is only
    // literal when it satisfies every stated constraint on the file it writes.
    //
    // The three prompts reach the file by three different routes, and that is
    // deliberate: the pinned line is a property of the request, so no route may
    // be the one that happens to honour it. The first two are composed
    // documents and reach `evidence_record`; the third states its bytes
    // outright and reaches the literal-write repair in `general_planner`.
    for (prompt, target, pinned) in [
        (
            "Draft a handover memo containing the migration status, the outstanding \
             blockers, and the on-call owner. Leave the memo in `handover/2026-q3.md`. \
             The first line must be exactly `handover=q3`.",
            "handover/2026-q3.md",
            "handover=q3",
        ),
        (
            "Create file `release-checklist.md` containing verify the tag, publish \
             the crate, push the image. The first line must be exactly \
             `checklist=v2`.",
            "release-checklist.md",
            "checklist=v2",
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
        let writes = super::planned_writes_to(prompt, target);
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

#[test]
fn an_unquoted_pinned_line_stops_before_the_following_body_clause() {
    // The real CLI ladder states a machine-readable opening line without
    // Markdown quotes, then coordinates a second requirement with "and". The
    // line parser used to swallow that body requirement into the header.
    let prompt = "Inspect the task model and record the concrete result. Leave supporting \
        evidence in `.agent-ladder/node-proof.md`. The first line must be exactly \
        node_path=1.1.1.1.1 and the body must state the concrete result.";

    let writes = super::planned_writes_to(prompt, ".agent-ladder/node-proof.md");
    assert!(
        writes
            .iter()
            .any(|content| content.lines().next() == Some("node_path=1.1.1.1.1")),
        "the coordinated body clause was included in the exact first line: {writes:#?}",
    );
}

#[test]
fn issue_1069_delivery_keeps_each_artifacts_own_constraints() {
    let prompt = "Inspect the existing queue data model and identify where a queue stores its \
        pending entries. Create `audit-effects/queue.lino` with these exact field lines: \
        `subject=queue`, `kind=inspection`, and `result=` followed by the observed result. \
        Leave supporting evidence in `.audit/queue-proof.md`. The first line must be exactly \
        `proof_for=queue`.";

    let proof = super::planned_writes_to(prompt, ".audit/queue-proof.md");
    assert!(
        proof
            .iter()
            .any(|content| content.starts_with("proof_for=queue\n")),
        "the proof inherited no opening-line constraint: {proof:#?}",
    );

    let effects = super::planned_writes_to(prompt, "audit-effects/queue.lino");
    assert!(
        effects.iter().any(|content| {
            content.lines().any(|line| line == "subject=queue")
                && content.lines().any(|line| line == "kind=inspection")
                && content.lines().any(|line| line.starts_with("result="))
        }),
        "the structured effect lost its exact fields: {effects:#?}",
    );
}

#[test]
fn nested_delivery_carries_the_observation_into_the_outer_effect() {
    // One investigation feeds two outputs in the real ladder: a human-readable
    // proof and a machine-checked effect. Once the proof write succeeded, its
    // "Recorded the findings" status used to become the effect's `result`,
    // discarding the observation that both artifacts were meant to record.
    let prompt = "Inspect the existing task-decomposition data model and identify where a node \
        stores its children. Create `audit-effects/decomposition.lino` with these exact field \
        lines: `subject=decomposition`, `kind=inspection`, and `result=` followed by the \
        observed result. Leave supporting evidence in `.audit/decomposition-proof.md`. The \
        first line must be exactly `proof_for=decomposition`.";
    let mut messages = vec![formal_ai::ChatMessage::user(prompt)];
    let mut effect = None;

    for turn in 0..super::LADDER_TURN_CAP {
        let Some(formal_ai::agentic_coding::AgenticPlan::ToolCalls(calls)) =
            formal_ai::agentic_coding::plan_chat_step(&messages, &super::LADDER_TOOLS)
        else {
            break;
        };
        for (index, call) in calls.iter().enumerate() {
            if let Ok(arguments) = serde_json::from_str::<serde_json::Value>(&call.arguments)
                && super::argument(
                    &arguments,
                    &["path", "filePath", "file_path", "absolute_path"],
                )
                .is_some_and(|path| path.ends_with("audit-effects/decomposition.lino"))
            {
                effect =
                    super::argument(&arguments, &["content", "contents", "text", "new_string"]);
            }
            let id = format!("nested-delivery-{turn}-{index}");
            messages.push(formal_ai::ChatMessage::assistant_tool_calls(vec![
                formal_ai::protocol::ToolCall::function(&id, &call.tool, call.arguments.clone()),
            ]));
            let result = if call.tool == "grep" {
                "src/task_decomposition.rs:79: pub children: Vec<Self>"
            } else {
                "ok"
            };
            messages.push(formal_ai::ChatMessage::tool_result(id, &call.tool, result));
        }
    }

    let effect = effect.expect("the nested delivery must produce its outer effect");
    assert!(
        effect
            .lines()
            .any(|line| line.starts_with("result=") && line.contains("children: Vec<Self>")),
        "the proof writer's status replaced the observed task result: {effect:?}",
    );
    assert!(!effect.contains("Recorded the findings"), "{effect}");
}

#[test]
fn every_named_input_is_read_before_a_structured_result_is_written() {
    // A composed result has one observation per named input. Completing the
    // investigation after the first successful read both skipped the remaining
    // input and copied Agent's display-only `<file>` wrapper and line numbers
    // into the durable result. The delivery route must wait for every input and
    // carry only the files' undecorated `result=` values forward.
    let prompt = "Read the committed regional summaries in `inputs/east.lino` and \
        `inputs/west.lino`. Inspect both files before writing anything. Treat only the \
        single line beginning exactly `result=` as each regional result. Create \
        `outputs/combined.lino` with these exact field lines: `kind=combined`, \
        `east_result=` followed by the exact east child `result=` value, `west_result=` \
        followed by the exact west child `result=` value, and `result=` followed by at \
        least four words that include both exact regional results and state how they \
        compose. Leave supporting evidence in \
        `.audit/combined-proof.md`. The first line must be exactly `proof_for=combined`.";
    // Agent's `read` tool abbreviates a physical line after 1,000 columns. The
    // root of the real 63-node ladder receives child results longer than that,
    // so an exact-field read must use a byte-preserving extraction rather than
    // treating the display abbreviation as file contents.
    let east = format!("East queue drains before noon {}", "east".repeat(320));
    let west = format!("West queue drains after noon {}", "west".repeat(320));
    let decorated = |region: &str, result: &str| {
        let visible = &result[..960];
        format!(
            "<file>\n00001| region={region}\n00002| kind=summary\n00003| result={visible} \
             ... [omitted columns 1001..{} of line 3] ...\n\n\
             (End of file - total 3 lines)\n</file>",
            result.len()
        )
    };
    let mut messages = vec![formal_ai::ChatMessage::user(prompt)];
    let mut observed_paths = Vec::new();
    let mut writes = Vec::new();

    for turn in 0..super::LADDER_TURN_CAP {
        let Some(formal_ai::agentic_coding::AgenticPlan::ToolCalls(calls)) =
            formal_ai::agentic_coding::plan_chat_step(&messages, &super::LADDER_TOOLS)
        else {
            break;
        };
        for (index, call) in calls.iter().enumerate() {
            let arguments = serde_json::from_str::<serde_json::Value>(&call.arguments)
                .expect("planned tool arguments");
            let path = super::argument(
                &arguments,
                &["path", "filePath", "file_path", "absolute_path"],
            );
            let content =
                super::argument(&arguments, &["content", "contents", "text", "new_string"]);
            let command = super::argument(&arguments, &["command", "cmd"]);
            if call.tool == "read"
                && let Some(path) = path.as_deref()
            {
                observed_paths.push(path.to_owned());
            }
            if call.tool == "bash"
                && let Some(command) = command.as_deref()
            {
                for input in ["inputs/east.lino", "inputs/west.lino"] {
                    if command.contains(input) {
                        observed_paths.push(input.to_owned());
                    }
                }
            }
            if let (Some(path), Some(content)) = (path.as_deref(), content) {
                assert!(
                    observed_paths
                        .iter()
                        .any(|read| read.ends_with("inputs/east.lino"))
                        && observed_paths
                            .iter()
                            .any(|read| read.ends_with("inputs/west.lino")),
                    "planned a durable write before observing every named input: \
                     reads={observed_paths:?}, write={path:?}"
                );
                writes.push((path.to_owned(), content));
            }
            let id = format!("multi-read-delivery-{turn}-{index}");
            messages.push(formal_ai::ChatMessage::assistant_tool_calls(vec![
                formal_ai::protocol::ToolCall::function(&id, &call.tool, call.arguments.clone()),
            ]));
            let result = match (path.as_deref(), command.as_deref()) {
                (_, Some(command))
                    if call.tool == "bash" && command.contains("inputs/east.lino") =>
                {
                    east.clone()
                }
                (_, Some(command))
                    if call.tool == "bash" && command.contains("inputs/west.lino") =>
                {
                    west.clone()
                }
                (Some(path), _) if call.tool == "read" && path.ends_with("inputs/east.lino") => {
                    decorated("east", &east)
                }
                (Some(path), _) if call.tool == "read" && path.ends_with("inputs/west.lino") => {
                    decorated("west", &west)
                }
                _ => "ok".to_owned(),
            };
            messages.push(formal_ai::ChatMessage::tool_result(id, &call.tool, result));
        }
    }

    let effect = writes
        .iter()
        .find(|(path, _)| path.ends_with("outputs/combined.lino"))
        .map(|(_, content)| content)
        .expect("the structured combined result");
    assert!(
        effect
            .lines()
            .any(|line| line == format!("east_result={east}"))
    );
    assert!(
        effect
            .lines()
            .any(|line| line == format!("west_result={west}"))
    );
    let combined = effect
        .lines()
        .find_map(|line| line.strip_prefix("result="))
        .expect("the combined result field");
    assert_eq!(
        effect
            .lines()
            .filter(|line| line.starts_with("result="))
            .count(),
        1,
        "a quoted source-field reference became a duplicate output field: {effect}"
    );
    assert!(
        combined.contains(&east) && combined.contains(&west),
        "{effect}"
    );
    assert!(
        !effect.contains("<file>") && !effect.contains("00003|"),
        "tool display decorations leaked into the durable result: {effect}"
    );
}

#[test]
fn a_local_observation_satisfies_nested_artifacts_before_optional_web_research() {
    // A harness can permit web research as a fallback after asking about the
    // checkout. Once grep has answered that local question, the fallback is no
    // longer work to perform: both requested artifacts must be grounded in the
    // result already in hand. Re-planning the residual before consuming that
    // result sent the run to the web and left both artifacts unwritten.
    let prompt = "Review the existing readiness check and record the observable completion \
        contract for workers. Create `audit-effects/readiness.lino` with these exact field \
        lines: `subject=readiness`, `kind=inspection`, and `result=` followed by the \
        observed result. Leave supporting evidence in `.audit/readiness-proof.md`. The \
        first line must be exactly `proof_for=readiness`. Use web research when it \
        materially improves factual accuracy.";
    let observed = concat!(
        "Found 3 matches\n",
        "/tmp/work/src/work.rs:\n",
        "  Line 70:     pub completion_criterion: String,\n",
        "  Line 89:             && !self.completion_criterion.starts_with(\"unresolved_\")\n",
        "  Line 130:                 self.completion_criterion.clone(),",
    );
    let mut messages = vec![
        formal_ai::ChatMessage::user(prompt),
        formal_ai::ChatMessage::assistant_tool_calls(vec![
            formal_ai::protocol::ToolCall::function(
                "search-readiness",
                "grep",
                r#"{"pattern":"completion_criterion"}"#.to_owned(),
            ),
        ]),
        formal_ai::ChatMessage::tool_result("search-readiness", "grep", observed),
    ];
    let mut writes = Vec::new();

    for turn in 0..super::LADDER_TURN_CAP {
        let Some(formal_ai::agentic_coding::AgenticPlan::ToolCalls(calls)) =
            formal_ai::agentic_coding::plan_chat_step(&messages, &super::LADDER_TOOLS)
        else {
            break;
        };
        for (index, call) in calls.iter().enumerate() {
            assert_ne!(
                call.tool, "websearch",
                "a successful local observation must not fall through to optional web research"
            );
            if let Ok(arguments) = serde_json::from_str::<serde_json::Value>(&call.arguments)
                && let (Some(path), Some(content)) = (
                    super::argument(
                        &arguments,
                        &["path", "filePath", "file_path", "absolute_path"],
                    ),
                    super::argument(&arguments, &["content", "contents", "text", "new_string"]),
                )
            {
                writes.push((path, content));
            }
            let id = format!("local-delivery-{turn}-{index}");
            messages.push(formal_ai::ChatMessage::assistant_tool_calls(vec![
                formal_ai::protocol::ToolCall::function(&id, &call.tool, call.arguments.clone()),
            ]));
            messages.push(formal_ai::ChatMessage::tool_result(id, &call.tool, "ok"));
        }
    }

    assert!(
        writes.iter().any(|(path, content)| {
            path.ends_with("audit-effects/readiness.lino")
                && content.contains("result=Line 89:")
                && content.contains("!self.completion_criterion.starts_with")
        }),
        "the local condition was not written to the structured effect: {writes:#?}",
    );
    assert!(
        writes.iter().any(|(path, content)| {
            path.ends_with(".audit/readiness-proof.md")
                && content.starts_with("proof_for=readiness\n")
                && content.contains("!self.completion_criterion.starts_with")
        }),
        "the local condition was not written to the proof: {writes:#?}",
    );
    let proof = writes
        .iter()
        .find(|(path, _)| path.ends_with(".audit/readiness-proof.md"))
        .map(|(_, content)| content)
        .expect("the proof write");
    assert!(
        !proof.contains("Line 70:") && !proof.contains("Line 130:"),
        "the evidence copied the entire search response instead of its concrete result: {proof}",
    );
}

#[test]
fn a_terse_source_fact_still_produces_a_non_hollow_result() {
    // A one-token sentinel is a complete answer to some repository questions,
    // but the ladder deliberately rejects an effect whose `result` is only a
    // copied line number and that token. Preserve the exact observed fact while
    // rendering it as a concrete result with enough natural-language context.
    let prompt = "Verify that a leaf without an observable completion contract is never treated \
        as independently checkable. Create `agent-ladder-effects/node.lino` with these exact \
        field lines: `node_path=1.1`, `node_depth=5`, `node_kind=leaf`, and `result=` followed by \
        at least four words that state the task result actually observed in this checkout. \
        Leave supporting evidence in `.agent-ladder/node-proof.md`. The first line must be \
        exactly `node_path=1.1`.";
    let mut messages = vec![
        formal_ai::ChatMessage::user(prompt),
        formal_ai::ChatMessage::assistant_tool_calls(vec![
            formal_ai::protocol::ToolCall::function(
                "search-contract",
                "grep",
                r#"{"include":"src/**/*","pattern":"unresolved_single_need"}"#.to_owned(),
            ),
        ]),
        formal_ai::ChatMessage::tool_result(
            "search-contract",
            "grep",
            "src/task_decomposition.rs:385: \"unresolved_single_need\",",
        ),
    ];
    let mut writes = Vec::new();

    for turn in 0..super::LADDER_TURN_CAP {
        let Some(formal_ai::agentic_coding::AgenticPlan::ToolCalls(calls)) =
            formal_ai::agentic_coding::plan_chat_step(&messages, &super::LADDER_TOOLS)
        else {
            break;
        };
        for (index, call) in calls.iter().enumerate() {
            if let Ok(arguments) = serde_json::from_str::<serde_json::Value>(&call.arguments)
                && let (Some(path), Some(content)) = (
                    super::argument(
                        &arguments,
                        &["path", "filePath", "file_path", "absolute_path"],
                    ),
                    super::argument(&arguments, &["content", "contents", "text", "new_string"]),
                )
            {
                writes.push((path, content));
            }
            let id = format!("terse-delivery-{turn}-{index}");
            messages.push(formal_ai::ChatMessage::assistant_tool_calls(vec![
                formal_ai::protocol::ToolCall::function(&id, &call.tool, call.arguments.clone()),
            ]));
            messages.push(formal_ai::ChatMessage::tool_result(id, &call.tool, "ok"));
        }
    }

    let effect = writes
        .iter()
        .find(|(path, _)| path.ends_with("agent-ladder-effects/node.lino"))
        .map(|(_, content)| content)
        .expect("the structured leaf effect");
    let result = effect
        .lines()
        .find_map(|line| line.strip_prefix("result="))
        .expect("the result field");
    assert!(
        result.split_whitespace().count() >= 4,
        "the result remained hollow: {effect:?}",
    );
    assert!(result.contains("unresolved_single_need"), "{effect}");

    let proof = writes
        .iter()
        .find(|(path, _)| path.ends_with(".agent-ladder/node-proof.md"))
        .map(|(_, content)| content)
        .expect("the supporting proof");
    let body = proof.lines().skip(1).collect::<Vec<_>>().join(" ");
    assert!(
        body.split_whitespace().count() >= 4,
        "the proof remained hollow: {proof:?}",
    );
    assert!(body.contains("unresolved_single_need"), "{proof}");
}

#[test]
fn a_source_condition_ending_in_a_colon_is_not_an_unmade_list() {
    // Python block headers are complete source observations even though their
    // final colon has the same shape as prose promising a list. Preserve the
    // exact matched condition, but finish the rendered claim so the generic
    // proof judge does not mistake source punctuation for a hollow promise.
    let prompt = "Verify a single node can be selected by dotted binary path for focused \
        debugging. Create `agent-ladder-effects/node.lino` with these exact field lines: \
        `node_path=2.1.2.1.2`, `node_depth=5`, `node_kind=leaf`, and `result=` followed by \
        at least four words that state the task result actually observed in this checkout. \
        Leave supporting evidence in `.agent-ladder/node-proof.md`. The first line must be \
        exactly `node_path=2.1.2.1.2`.";
    let marker = "if depth == level and (not filt or node == filt):";
    let mut messages = vec![
        formal_ai::ChatMessage::user(prompt),
        formal_ai::ChatMessage::assistant_tool_calls(vec![
            formal_ai::protocol::ToolCall::function(
                "search-node-filter",
                "grep",
                r#"{"include":"experiments/**/*","pattern":"node == filt"}"#.to_owned(),
            ),
        ]),
        formal_ai::ChatMessage::tool_result(
            "search-node-filter",
            "grep",
            format!("Line 148:         {marker}"),
        ),
    ];
    let mut writes = Vec::new();

    for turn in 0..super::LADDER_TURN_CAP {
        let Some(formal_ai::agentic_coding::AgenticPlan::ToolCalls(calls)) =
            formal_ai::agentic_coding::plan_chat_step(&messages, &super::LADDER_TOOLS)
        else {
            break;
        };
        for (index, call) in calls.iter().enumerate() {
            if let Ok(arguments) = serde_json::from_str::<serde_json::Value>(&call.arguments)
                && let (Some(path), Some(content)) = (
                    super::argument(
                        &arguments,
                        &["path", "filePath", "file_path", "absolute_path"],
                    ),
                    super::argument(&arguments, &["content", "contents", "text", "new_string"]),
                )
            {
                writes.push((path, content));
            }
            let id = format!("colon-source-delivery-{turn}-{index}");
            messages.push(formal_ai::ChatMessage::assistant_tool_calls(vec![
                formal_ai::protocol::ToolCall::function(&id, &call.tool, call.arguments.clone()),
            ]));
            messages.push(formal_ai::ChatMessage::tool_result(id, &call.tool, "ok"));
        }
    }

    for (path, content) in &writes {
        if path.ends_with(".agent-ladder/node-proof.md")
            || path.ends_with("agent-ladder-effects/node.lino")
        {
            assert!(
                content.contains(marker),
                "lost the exact observation: {content:?}"
            );
            assert!(
                !content.trim_end().ends_with(':'),
                "source punctuation looked like an unmade list: {content:?}"
            );
        }
    }
    assert!(
        writes
            .iter()
            .any(|(path, _)| path.ends_with(".agent-ladder/node-proof.md")),
        "the proof was not written: {writes:#?}"
    );
    assert!(
        writes
            .iter()
            .any(|(path, _)| path.ends_with("agent-ladder-effects/node.lino")),
        "the effect was not written: {writes:#?}"
    );
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
        let writes = super::planned_writes(prompt);
        assert!(
            writes.iter().any(|content| content.contains(expected)),
            "expected the literal bytes {expected:?} for {prompt:?}, planned {writes:?}"
        );
    }
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
        // Both prompts above ask about task structure, which the agentic router
        // now answers on its own -- so they no longer reach the symbolic engine
        // and no longer pin this route. These two ask something no agentic route
        // answers, which is exactly the state that made this gap visible: the
        // router has no plan, the engine does, and the caller asked for a file.
        (
            "What is 17 multiplied by 23? Record the answer in `math/product.md`. \
             The first line must be exactly `math=product`.",
            "math/product.md",
            "math=product",
        ),
        (
            "What is 480 divided by 15? Save the result in `arith/quotient.md`. The \
             first line must be exactly `arith=quotient`.",
            "arith/quotient.md",
            "arith=quotient",
        ),
    ] {
        let paths = super::planned_paths(prompt);
        assert!(
            paths.iter().any(|path| path == target),
            "nothing was planned to be written to {target} for {prompt:?}: {paths:?}"
        );
        let writes = super::planned_writes(prompt);
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
        let paths = super::planned_paths(prompt);
        assert!(
            paths.iter().any(|path| path == target),
            "nothing was planned to be written to {target} for {prompt:?}: {paths:?}"
        );
        let writes = super::planned_writes(prompt);
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
fn a_payload_that_names_the_work_product_is_not_written_as_the_body() {
    // "Save the result to FILE" says where an answer goes; it does not say what
    // the answer is. Read as a literal write, the phrase naming the work product
    // became the file's contents, so the ladder's proof files contained the word
    // "result" and nothing else -- non-empty, and evidence of nothing.
    //
    // The guard belongs to the destination-led shape, the one that infers its
    // payload from position: the bytes are whatever sits between the write
    // action and the file clause, which here is only the work product's name. A
    // request that states no work has nothing to record, so the whole write is
    // declined rather than written hollow -- an empty list of writes, not an
    // empty file.
    for (prompt, target) in [
        ("Save the answer to `out/e.md`.", "out/e.md"),
        ("Save the result to `counts.md`.", "counts.md"),
    ] {
        assert_eq!(
            super::planned_writes_to(prompt, target),
            Vec::<String>::new(),
            "wrote the name of the work product as the work product for {prompt:?}"
        );
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
        let planned = super::planned_writes(polite);
        assert!(
            !planned.is_empty(),
            "the politely phrased write planned nothing: {polite:?}"
        );
        assert_eq!(
            planned.len(),
            super::planned_writes(familiar).len(),
            "politeness changed how much was planned for {polite:?}"
        );
        for (polite_plan, familiar_plan) in planned.iter().zip(super::planned_writes(familiar)) {
            assert_eq!(
                super::without_derived_ids(polite_plan, polite),
                super::without_derived_ids(&familiar_plan, familiar),
                "politeness changed the plan for {polite:?}"
            );
        }
    }
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
        for content in super::planned_writes_to(prompt, target) {
            assert!(
                !content.contains(intruder),
                "wrote the delivery instruction into the document for {prompt:?}: {content:?}"
            );
        }
    }
}

#[test]
fn a_semicolon_inside_a_payload_does_not_end_it() {
    // The sentence splitter that bounds a literal payload was written for
    // command policy, where a semicolon separates one command from the next, so
    // it read `;` as the end of the statement. In prose a semicolon joins two
    // clauses into one sentence, and half the payload was thrown away: issue
    // #918's minimal-core invariant reached its file ending at "host surface;"
    // with the clause naming where domain knowledge goes cut off (issue #1066).
    let prompt = "Create file `policy/retention.md` containing Logs are kept for ninety \
                  days; backups are kept for a year.";
    assert_eq!(
        super::planned_writes_to(prompt, "policy/retention.md"),
        vec!["Logs are kept for ninety days; backups are kept for a year.".to_owned()]
    );
}
