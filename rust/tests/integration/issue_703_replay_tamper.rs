//! Issue #703 R703-7: replay rejects an edited, truncated, reordered, or
//! wrongly parented session by name.
//!
//! The JavaScript twin (`js/agentic/crate/orchestration_replay.mjs`) is pinned
//! on the same committed sessions and the same tampering by
//! `rust/tests/web/issue-0703-session-replay.test.mjs`.

use formal_ai::orchestration::{
    AgentSession, AgentStatus, ReplayError, replay_continuation, replay_session, session_sha256,
};

const HIVE: &[u8] = include_bytes!(
    "../../../docs/case-studies/issue-921/formal-ai-to-hive-mind/orchestration-session.json"
);
const CONTROLLER: &[u8] = include_bytes!(
    "../../../docs/case-studies/issue-703/followup-authorship/controller-session.json"
);
const CORRECTED: &[u8] = include_bytes!(
    "../../../docs/case-studies/issue-703/followup-authorship/corrected-session.json"
);
const FINAL: &[u8] =
    include_bytes!("../../../docs/case-studies/issue-703/followup-authorship/final-session.json");

/// Parse, edit, and re-render canonically, the way a careful tamperer would.
fn edited(bytes: &[u8], edit: impl FnOnce(&mut AgentSession)) -> Vec<u8> {
    let mut session: AgentSession =
        serde_json::from_slice(bytes).expect("the committed session parses");
    edit(&mut session);
    let mut rendered = serde_json::to_vec_pretty(&session).expect("the edited session renders");
    rendered.push(b'\n');
    rendered
}

fn rejection(result: Result<AgentSession, ReplayError>) -> String {
    result
        .expect_err("the tampered session must not replay")
        .to_string()
}

#[test]
fn committed_sessions_replay_and_bind_their_fields_to_the_event_chain() {
    let hive = replay_session(HIVE).expect("the committed session replays");
    let kinds: Vec<&str> = hive
        .events
        .iter()
        .map(|event| event.kind.as_str())
        .collect();
    assert_eq!(
        kinds,
        [
            "permission_granted",
            "adapter_selected",
            "process_started",
            "process_succeeded",
            "verification_started",
            "verification_passed",
            "workspace_effect",
        ]
    );
    assert_eq!(hive.status, AgentStatus::Succeeded);
    for bytes in [CONTROLLER, CORRECTED, FINAL] {
        replay_session(bytes).expect("every session of the correction chain replays");
    }
}

#[test]
fn edits_truncation_and_reordering_are_rejected_by_name() {
    let text = String::from_utf8(HIVE.to_vec()).expect("the session is UTF-8");
    let detail = text.replace(
        "\"detail\": \"formal-ai-to-hive-mind.txt\"",
        "\"detail\": \"elsewhere.txt\"",
    );
    assert_eq!(
        rejection(replay_session(detail.as_bytes())),
        "event_digest:6"
    );
    assert_eq!(
        rejection(replay_session(&edited(HIVE, |session| {
            session.events.swap(4, 5);
        }))),
        "event_sequence:4"
    );
    assert_eq!(
        rejection(replay_session(&edited(HIVE, |session| {
            session.events.remove(4);
        }))),
        "event_sequence:4"
    );
    // Dropping the last event leaves a valid chain; the effect it stood for is
    // still listed in `changes`, so the truncation shows as a binding failure.
    assert_eq!(
        rejection(replay_session(&edited(HIVE, |session| {
            session.events.pop();
        }))),
        "event_binding:effects"
    );
    assert_eq!(
        rejection(replay_session(&edited(HIVE, |session| {
            session.status = AgentStatus::Failed;
        }))),
        "event_binding:status"
    );
    assert_eq!(
        rejection(replay_session(&edited(HIVE, |session| {
            session.changes[0].path = "elsewhere.txt".to_owned();
        }))),
        "event_binding:effects"
    );
    assert_eq!(
        rejection(replay_session(&edited(CORRECTED, |session| {
            session.continuation = None;
        }))),
        "event_binding:continuation"
    );
    assert_eq!(
        rejection(replay_session(&edited(HIVE, |session| {
            session.schema = "formal-ai-agent-session-v0".to_owned();
        }))),
        "unsupported_session_schema"
    );
    let mut trailing = HIVE.to_vec();
    trailing.push(b'\n');
    assert_eq!(
        rejection(replay_session(&trailing)),
        "non_canonical_session"
    );
    assert!(matches!(
        replay_session(&HIVE[..HIVE.len() / 2]),
        Err(ReplayError::Json(_))
    ));
}

#[test]
fn a_correction_replays_only_against_the_exact_parent_bytes_it_names() {
    let corrected = replay_continuation(CONTROLLER, CORRECTED).expect("the first correction");
    let parent = replay_session(CONTROLLER).expect("the parent replays");
    assert_eq!(
        corrected
            .continuation
            .map(|continuation| continuation.parent_session_sha256),
        Some(session_sha256(&parent).expect("the parent digest"))
    );
    replay_continuation(CORRECTED, FINAL).expect("the second correction");
    assert_eq!(
        rejection(replay_continuation(CONTROLLER, FINAL)),
        "broken_ancestry"
    );
    assert_eq!(
        rejection(replay_continuation(HIVE, HIVE)),
        "broken_ancestry"
    );
    // Free text such as stdout is outside the event chain: an edited parent
    // still replays alone, and the child's parent digest is what exposes it.
    let edited_parent = edited(CONTROLLER, |session| session.stdout.push_str("edited"));
    replay_session(&edited_parent).expect("an stdout edit alone keeps the chain valid");
    assert_eq!(
        rejection(replay_continuation(&edited_parent, CORRECTED)),
        "broken_ancestry"
    );
}
