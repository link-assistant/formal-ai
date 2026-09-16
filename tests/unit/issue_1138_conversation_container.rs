//! Issue #1138 B6 (plan 06, L14): a conversation's container survives idling and restarts (#937).
//!
//! An idle container stops with its state preserved and comes back with that
//! state intact. When both a snapshot and a replay exist they are *compared*,
//! and a divergence is reported rather than hidden behind whichever one was
//! consulted first.

use std::time::{Duration, Instant};

use formal_ai::execution_box::container::{ConversationContainer, SnapshotPolicy};

fn container(policy: SnapshotPolicy) -> ConversationContainer {
    ConversationContainer {
        conversation_id: String::from("issue-1138-container"),
        image: String::from("konard/box"),
        handle: None,
        idle_after: Duration::from_millis(10),
        restore: policy,
    }
}

/// Write a file, idle past the threshold, reattach: the file is still there.
#[test]
fn an_idle_container_stops_and_restores_its_state() {
    let mut conversation = container(SnapshotPolicy::Snapshot);
    {
        let boxed = conversation
            .attach()
            .expect("a conversation container attaches or refuses by name");
        boxed
            .run("open('state.txt', 'w').write('kept')\n", &[])
            .expect("the write runs inside the container");
    }

    conversation
        .stop_if_idle(Instant::now() + Duration::from_millis(50))
        .expect("an idle container stops, preserving its state");

    let boxed = conversation
        .attach()
        .expect("reattaching restores the stopped container");
    let observation = boxed
        .run("print(open('state.txt').read())\n", &[])
        .expect("the restored container still runs");
    assert!(
        observation.partial_output.contains("kept"),
        "state written before the container idled must survive the restore: {}",
        observation.partial_output
    );
}

/// Snapshot and replay are two restores of the same state. When both exist the
/// system compares them and reports the difference.
#[test]
fn replay_and_snapshot_are_compared_when_both_exist() {
    let mut conversation = container(SnapshotPolicy::Replay);
    let comparison = conversation
        .compare_restores()
        .expect("comparing the two restores is supported");
    assert!(
        comparison.snapshot_digest.is_some(),
        "a snapshot digest is taken so there is something to compare against"
    );
    assert!(
        comparison.replay_digest.is_some(),
        "a replay digest is taken so the comparison is possible"
    );
    assert_eq!(
        comparison.diverged,
        comparison.snapshot_digest != comparison.replay_digest,
        "divergence is derived from the two digests, never assumed away"
    );
}
