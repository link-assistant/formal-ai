//! Issue #1138 B6 (plan 06, L14): detached conversation-container contracts.
//!
//! These checks are hermetic: they prove command construction and the
//! no-host-fallback boundary without claiming that Docker ran. The ignored live
//! recovery lane owns daemon-backed snapshot/replay evidence.

use std::time::Duration;

use formal_ai::execution_box::container::{ConversationContainer, SnapshotPolicy};
use formal_ai::execution_box::{
    BACKEND_ENV, BoxError, NetworkPolicy, conversation_container_name,
    conversation_create_invocation,
};

fn container(policy: SnapshotPolicy) -> ConversationContainer {
    ConversationContainer {
        conversation_id: String::from("issue-1138-container/user supplied ; $(false)"),
        image: String::from("konard/box:2.4.0"),
        handle: None,
        idle_after: Duration::from_millis(10),
        restore: policy,
    }
}

/// A conversation uses a detached Docker container with a stable safe name,
/// the declared image, and network disabled. User text is not shell source.
#[test]
fn a_conversation_builds_a_detached_network_denied_container() {
    let conversation = container(SnapshotPolicy::Snapshot);
    let invocation = conversation_create_invocation(
        &conversation.conversation_id,
        &conversation.image,
        NetworkPolicy::Denied,
    );
    let name = conversation_container_name(&conversation.conversation_id);

    assert_eq!(invocation.program, "docker");
    assert_eq!(
        &invocation.arguments[..4],
        ["run", "--detach", "--name", &name]
    );
    assert!(
        invocation
            .arguments
            .windows(2)
            .any(|pair| pair == ["--network", "none"])
    );
    assert!(
        invocation
            .arguments
            .iter()
            .any(|item| item == &conversation.image)
    );
    assert!(
        invocation
            .arguments
            .iter()
            .all(|item| !item.contains(&conversation.conversation_id)),
        "the conversation id is represented by a digest, never interpolated"
    );
}

/// A container request without the operator's Docker grant is a named refusal;
/// it is never silently executed by `python3` on the host.
#[test]
fn an_unconfigured_conversation_never_falls_back_to_the_host() {
    temp_env::with_var_unset(BACKEND_ENV, || {
        let mut conversation = container(SnapshotPolicy::Replay);
        assert!(matches!(
            conversation.attach(),
            Err(BoxError::BackendNotConfigured { .. })
        ));
        assert!(conversation.handle.is_none());
    });
}

/// The policy remains explicit data so a live run can measure snapshot and
/// replay independently; selecting replay does not alter Docker authorization.
#[test]
fn snapshot_and_replay_are_distinct_explicit_policies() {
    assert_ne!(SnapshotPolicy::Snapshot, SnapshotPolicy::Replay);
    assert_eq!(SnapshotPolicy::default(), SnapshotPolicy::Snapshot);
}
