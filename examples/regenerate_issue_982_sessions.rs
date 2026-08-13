//! Regenerate the committed issue-982 self-hosting session fixtures so they
//! replay byte-for-byte against the current agent (see
//! `tests/integration/issue_982_self_hosting.rs`).

use formal_ai::agentic_coding::run_agentic_task;

fn main() {
    let cases = [
        (
            "Create file memory-upgrade-contract.md containing memory upgrade preflight and explicit migration contract verified",
            "docs/case-studies/issue-982/self-hosting/contract/session.json",
        ),
        (
            "Create file memory-upgrade-rollback.md containing rollback restores the byte-exact schema-1 backup",
            "docs/case-studies/issue-982/self-hosting/rollback/session.json",
        ),
    ];

    for (task, path) in cases {
        let outcome = run_agentic_task(task).expect("run agentic task");
        let rendered =
            serde_json::to_string_pretty(&outcome.session_json()).expect("serialize session JSON");
        std::fs::write(path, format!("{rendered}\n")).expect("write session fixture");
        println!("wrote {path}");
    }
}
