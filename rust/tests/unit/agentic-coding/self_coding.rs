use std::fs;

use formal_ai::agentic_coding::run_agentic_task;

const TASK: &str = "Create file self-coding-result.txt containing self-coding=passed";

#[test]
fn self_coding_keeps_server_memory_out_of_the_agent_workspace() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let script = fs::read_to_string(format!("{root}/rust/examples/self-coding/run.sh"))
        .expect("self-coding replay script");

    assert!(
        script.contains("FORMAL_AI_MEMORY_PATH=\"$work/.git/formal-ai-memory/memory.lino\""),
        "server-private .lino and binary .links state must stay below .git so Agent snapshots cannot mistake it for an authored repository effect",
    );
    assert!(
        !script.contains("FORMAL_AI_MEMORY_PATH=\"$work/memory.lino\""),
        "the Agent worktree must contain only fixture inputs and Agent-authored effects",
    );
}

#[test]
fn self_coding_session_replays() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    let dir = format!("{root}/docs/case-studies/issue-651/self-coding-run");
    let plan =
        fs::read_to_string(format!("{dir}/general-change-plan.lino")).expect("captured plan");
    let diff = fs::read_to_string(format!("{dir}/result.diff")).expect("captured diff");
    assert!(plan.contains("self-coding-result.txt"));
    assert!(plan.contains("capability \"Run\""));
    assert!(diff.contains("+self-coding=passed"));
    let legacy =
        fs::read_to_string(format!("{dir}/session.json")).expect("immutable legacy session");
    let legacy: serde_json::Value = serde_json::from_str(&legacy).expect("legacy JSON");
    assert_eq!(legacy["task"], TASK);
    let committed = fs::read_to_string(format!(
        "{root}/docs/case-studies/pull-request-1188/native-protocol-captures/8abb066db/self-coding.json"
    )).expect("actual CI native session oracle");
    let observed: serde_json::Value = serde_json::from_str(&committed).expect("observed JSON");
    assert_eq!(observed["task"], legacy["task"]);
    let target = legacy["steps"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .find(|step| step["tool"] == "write_file")
        .unwrap();
    assert!(
        observed["steps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|step| step["tool"] == "write_file"
                && step["arguments"]["path"] == target["arguments"]["path"]
                && step["arguments"]["content"] == target["arguments"]["content"])
    );
    let verification = legacy["steps"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .find(|step| step["tool"] == "run_command")
        .unwrap();
    let actual = observed["steps"]
        .as_array()
        .unwrap()
        .iter()
        .rev()
        .find(|step| step["tool"] == "run_command")
        .unwrap();
    let receipt: serde_json::Value =
        serde_json::from_str(actual["result"].as_str().unwrap()).unwrap();
    assert_eq!(
        actual["arguments"]["command"],
        verification["arguments"]["command"]
    );
    assert_eq!(receipt["schema"], "command-execution-receipt/v1");
    assert_eq!(receipt["command"], verification["arguments"]["command"]);
    assert_eq!(receipt["stdout"], verification["result"]);
    assert_eq!(receipt["exit_code"], 0);
    assert_eq!(receipt["complete"], true);
    assert_eq!(receipt["timed_out"], false);
    assert_eq!(receipt["truncated"], false);
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
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
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
