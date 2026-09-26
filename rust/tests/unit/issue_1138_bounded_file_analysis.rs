//! A higher-order file request collects bounded evidence instead of echoing a
//! repository into the model context (issue #1138 self-use checkpoint).

use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};

const AUDIT_PROMPT: &str = "Audit and read `notes/alpha-plan.md`, `notes/beta-plan.md`, and \
                            `.github/workflows/check.yml`; report concrete remaining gaps.";

fn calls(messages: &[ChatMessage], tools: &[&str]) -> Vec<PlannedToolCall> {
    match plan_chat_step(messages, tools) {
        Some(AgenticPlan::ToolCalls(calls)) => calls,
        other => panic!("expected tool calls, got {other:?}"),
    }
}

fn answer(messages: &mut Vec<ChatMessage>, call: &PlannedToolCall, result: &str) {
    let id = format!("call_{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.clone(),
        call.tool.clone(),
        call.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(id, &call.tool, result));
}

fn arguments(call: &PlannedToolCall) -> serde_json::Value {
    serde_json::from_str(&call.arguments).expect("tool arguments are JSON")
}

#[test]
fn multi_file_audit_collects_seed_derived_gap_evidence_before_reading_files() {
    let messages = vec![ChatMessage::user(AUDIT_PROMPT)];
    let planned = calls(&messages, &["read", "grep"]);

    assert_eq!(planned.len(), 3);
    assert!(planned.iter().all(|call| call.tool == "grep"));
    let paths = planned
        .iter()
        .map(|call| arguments(call)["path"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            "notes/alpha-plan.md",
            "notes/beta-plan.md",
            ".github/workflows/check.yml",
        ]
    );
    for call in &planned {
        let pattern = arguments(call)["pattern"].as_str().unwrap().to_owned();
        assert!(pattern.contains("TODO"), "seeded marker missing: {pattern}");
        assert!(
            pattern.contains("pending"),
            "seeded marker missing: {pattern}"
        );
        assert!(
            pattern.contains("\\["),
            "unchecked-box marker missing: {pattern}"
        );
    }
}

#[test]
fn seeded_audit_actions_route_to_bounded_analysis_in_all_supported_languages() {
    for prompt in [
        "Review and read `a.md` and `b.md` for remaining work.",
        "Проверь и прочитай `a.md` и `b.md`, найди оставшуюся работу.",
        "समीक्षा करें और `a.md` तथा `b.md` फ़ाइलें पढ़ें।",
        "审查并读取 `a.md` 和 `b.md` 中的剩余工作。",
        "Audita y lee `a.md` y `b.md` para encontrar trabajo pendiente.",
    ] {
        let planned = calls(&[ChatMessage::user(prompt)], &["read", "grep"]);
        assert_eq!(planned.len(), 2, "{prompt}");
        assert!(planned.iter().all(|call| call.tool == "grep"), "{prompt}");
    }
}

#[test]
fn multi_file_audit_composes_findings_and_marks_absent_evidence_honestly() {
    let mut messages = vec![ChatMessage::user(AUDIT_PROMPT)];
    let planned = calls(&messages, &["read", "grep"]);
    for (call, result) in planned.iter().zip([
        "Found 1 matches\nnotes/alpha-plan.md:\n  Line 12: - [ ] implement the generic adapter",
        "No files found",
        "Found 1 matches\n.github/workflows/check.yml:\n  Line 44: # TODO publish the verified artifact",
    ]) {
        answer(&mut messages, call, result);
    }

    let Some(AgenticPlan::Final(report)) = plan_chat_step(&messages, &["read", "grep"]) else {
        panic!("completed evidence collection should produce a report");
    };
    assert!(report.contains("alpha-plan.md"));
    assert!(report.contains("implement the generic adapter"));
    assert!(report.contains("beta-plan.md"));
    assert!(report.contains("No explicit gap marker"));
    assert!(report.contains("publish the verified artifact"));
    assert!(report.contains("does not prove completion"));
}

#[test]
fn read_only_clients_receive_explicit_bounds_and_oversized_results_stay_bounded() {
    let mut messages = vec![ChatMessage::user(AUDIT_PROMPT)];
    let planned = calls(&messages, &["read"]);
    assert_eq!(planned.len(), 3);
    for call in &planned {
        assert_eq!(call.tool, "read");
        assert_eq!(arguments(call)["offset"], 0);
        assert_eq!(arguments(call)["limit"], 160);
        assert_eq!(arguments(call)["columnOffset"], 0);
        assert_eq!(arguments(call)["columnLimit"], 320);
    }

    let huge_finding = format!("TODO {}", "x".repeat(20_000));
    for call in &planned {
        answer(&mut messages, call, &huge_finding);
    }
    let Some(AgenticPlan::Final(report)) = plan_chat_step(&messages, &["read"]) else {
        panic!("bounded reads should produce a report");
    };
    assert!(
        report.len() < 4_000,
        "report was not bounded: {} bytes",
        report.len()
    );
    assert!(report.contains("TODO"));
    assert!(report.contains("does not prove completion"));
}

#[test]
fn explicit_content_requests_keep_the_full_read_contract() {
    let messages = vec![ChatMessage::user(
        "read `a.txt` and `b.md` and show their contents",
    )];
    let planned = calls(&messages, &["read"]);
    assert_eq!(planned.len(), 2);
    for call in planned {
        let args = arguments(&call);
        assert!(args.get("offset").is_none());
        assert!(args.get("limit").is_none());
    }
}

#[test]
fn shell_only_clients_receive_the_same_line_and_column_bounds() {
    let planned = calls(&[ChatMessage::user(AUDIT_PROMPT)], &["run_command"]);
    assert_eq!(planned.len(), 3);
    for call in planned {
        assert_eq!(call.tool, "run_command");
        let command = arguments(&call)["command"].as_str().unwrap().to_owned();
        assert!(command.starts_with("sed -n '1,160p' "), "{command}");
        assert!(command.ends_with(" | cut -c 1-320"), "{command}");
    }
}
