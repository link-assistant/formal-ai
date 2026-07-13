use std::fs;

use formal_ai::agentic_coding::run_agentic_task;

const TASK: &str = "Create file self-coding-result.txt containing self-coding=passed";

#[test]
fn self_coding_session_replays() {
    let root = env!("CARGO_MANIFEST_DIR");
    let dir = format!("{root}/docs/case-studies/issue-651/self-coding-run");
    let plan =
        fs::read_to_string(format!("{dir}/general-change-plan.lino")).expect("captured plan");
    let diff = fs::read_to_string(format!("{dir}/result.diff")).expect("captured diff");
    assert!(plan.contains("self-coding-result.txt"));
    assert!(plan.contains("capability \"Run\""));
    assert!(diff.contains("+self-coding=passed"));
    let committed = fs::read_to_string(format!("{dir}/session.json")).expect("session");
    let fresh = run_agentic_task(TASK).expect("offline replay");
    assert_eq!(
        committed.trim(),
        serde_json::to_string_pretty(&fresh.session_json())
            .expect("session JSON")
            .trim()
    );
}

#[test]
fn self_coding_capture_contains_every_layer() {
    let root = env!("CARGO_MANIFEST_DIR");
    let dir = format!("{root}/docs/case-studies/issue-651/self-coding-run");
    for artifact in [
        "hive-mind-dispatch.log",
        "agent-stream.jsonl",
        "formal-ai.log",
        "general-change-plan.lino",
        "result.diff",
        "session.json",
    ] {
        assert!(
            fs::metadata(format!("{dir}/{artifact}")).is_ok(),
            "missing {artifact}"
        );
    }
}
