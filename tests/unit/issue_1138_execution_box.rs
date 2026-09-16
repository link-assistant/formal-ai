//! Issue #1138 B6 (plan 06, L11, L13): a box reports, it never pretends.
//!
//! A timeout carries both numbers and the partial output; the descending-N
//! ladder records every N it tried and what happened at each; network is denied
//! unless the task's contract requires it; and a missing container daemon is a
//! refusal, never a silent pass.

use std::time::Duration;

use formal_ai::execution_box::{
    BoxError, BoxPolicy, ExecutionBackend, ExecutionBox, NetworkPolicy,
};

fn host_policy(deadline: Duration) -> BoxPolicy {
    BoxPolicy {
        network: NetworkPolicy::Denied,
        deadline,
    }
}

/// Exceeding the deadline is a reported failure carrying the deadline, the
/// elapsed time and whatever was printed before it — never a pass and never a
/// silent truncation.
#[test]
fn a_timeout_is_a_reported_failure_with_both_numbers() {
    let mut boxed = ExecutionBox::open(
        &ExecutionBackend::HostSandbox,
        &host_policy(Duration::from_millis(50)),
    )
    .expect("the host sandbox is always available");
    let observation = boxed
        .run("import time\nprint('started', flush=True)\ntime.sleep(5)\n", &[])
        .expect("a timeout is an observation, not an error");

    assert!(
        observation.timed_out,
        "the run exceeded its deadline and must say so"
    );
    assert_eq!(
        observation.deadline,
        Duration::from_millis(50),
        "the deadline it was measured against is reported"
    );
    assert!(
        observation.elapsed >= observation.deadline,
        "the elapsed time is reported beside the deadline"
    );
    assert!(
        observation.partial_output.contains("started"),
        "whatever was printed before the deadline is shown, not discarded"
    );
    assert_ne!(
        observation.exit_code,
        Some(0),
        "a timed-out run may never report success"
    );
}

/// #930's ladder halves the iteration bound and retries, reporting which N timed
/// out and which N stopped timing out. Nothing is hidden.
#[test]
fn the_halving_ladder_records_every_n_it_tried() {
    let mut boxed = ExecutionBox::open(
        &ExecutionBackend::HostSandbox,
        &host_policy(Duration::from_millis(100)),
    )
    .expect("the host sandbox is always available");
    let rungs = boxed
        .halving_ladder("total = sum(range(N))\nprint(total)\n", 1_000_000_000)
        .expect("the ladder reports its rungs");

    assert!(
        rungs.len() >= 2,
        "a ladder that halved at least once records at least two rungs"
    );
    for pair in rungs.windows(2) {
        assert_eq!(
            pair[1].n,
            pair[0].n / 2,
            "each rung halves the previous N, and every N is recorded"
        );
    }
    assert!(
        rungs.iter().any(|rung| rung.timed_out),
        "the ladder records which N timed out"
    );
    assert!(
        rungs.iter().any(|rung| !rung.timed_out),
        "the ladder records which N stopped timing out"
    );
}

/// `--network none` is the default; only the task contract may lift it.
#[test]
fn network_is_denied_unless_the_contract_requires_it() {
    assert_eq!(
        NetworkPolicy::default(),
        NetworkPolicy::Denied,
        "the default network policy is deny"
    );
    let boxed = ExecutionBox::open(
        &ExecutionBackend::Box {
            image: String::from("konard/box"),
        },
        &BoxPolicy::default(),
    );
    match boxed {
        Ok(opened) => assert_eq!(
            opened.network(),
            NetworkPolicy::Denied,
            "a box opened under the default policy has no network"
        ),
        Err(BoxError::NoDaemon { .. } | BoxError::BackendNotConfigured { .. }) => {}
        Err(other) => panic!("opening a box must either succeed or refuse honestly, got {other:?}"),
    }
}

/// With no container runtime there is no output to show. The honest answer is a
/// refusal that names what is missing.
#[test]
fn a_missing_docker_daemon_is_a_refusal_not_a_skip() {
    let outcome = ExecutionBox::open(
        &ExecutionBackend::SweBenchImage {
            instance_id: String::from("astropy__astropy-12907"),
        },
        &BoxPolicy::default(),
    );
    match outcome {
        Err(BoxError::NoDaemon { detail }) => assert!(
            !detail.trim().is_empty(),
            "the refusal names what was observed when the daemon was contacted"
        ),
        Err(BoxError::BackendNotConfigured { backend }) => assert!(
            !backend.trim().is_empty(),
            "the refusal names the backend that is not configured"
        ),
        Ok(_) => {
            // A machine that really has the image may open it; the point is that
            // absence is never silently downgraded to success.
        }
        Err(other) => panic!("an absent backend must refuse by name, got {other:?}"),
    }
}
