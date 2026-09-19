//! Issue #1138 B6 (plan 06, L11, L13): a box reports, it never pretends.
//!
//! A timeout carries both numbers and the partial output; the descending-N
//! ladder records every N it tried and what happened at each; network is denied
//! unless the task's contract requires it; and a missing container daemon is a
//! refusal, never a silent pass.

use std::time::Duration;

use formal_ai::box_language_projects::{box_image_survey, box_language_contract};
use formal_ai::execution_box::{
    BoxError, BoxPolicy, ExecutionBackend, ExecutionBox, NetworkPolicy, backend_from_configuration,
    disposable_container_invocation,
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
    let boxed = ExecutionBox::open(
        &ExecutionBackend::HostSandbox,
        &host_policy(Duration::from_millis(50)),
    )
    .expect("the host sandbox is always available");
    let observation = boxed
        .run(
            "import time\nprint('started', flush=True)\ntime.sleep(5)\n",
            &[],
        )
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
    let boxed = ExecutionBox::open(
        &ExecutionBackend::HostSandbox,
        &host_policy(Duration::from_millis(100)),
    )
    .expect("the host sandbox is always available");
    let rungs = boxed
        .halving_ladder("total = sum(range({N}))\nprint(total)\n", 1_000_000_000)
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

/// The two toolchains that motivate prerequisite recovery have no dedicated
/// image in the pinned registry survey. They must therefore remain explicit
/// deferred contracts rather than silently borrowing the multi-gigabyte image
/// or pretending execution was observed.
#[test]
fn kotlin_and_scala_are_deferred_when_the_survey_has_no_dedicated_image() {
    let contract = box_language_contract();
    let survey = box_image_survey();
    for (language, repository) in [("kotlin", "box-kotlin"), ("scala", "box-scala")] {
        let project = contract
            .deferred
            .iter()
            .find(|project| project.language == language)
            .unwrap_or_else(|| panic!("{language}: deferred project must be declared"));
        assert!(
            project.reason.contains(repository),
            "{language}: the reason must name the missing image"
        );
        assert!(
            survey.missing.iter().any(|missing| missing == repository),
            "{language}: the pinned survey must corroborate the deferral"
        );
    }
}

/// The Docker image's two environment declarations form one permission: both
/// are required, `docker` is observed as the isolation, and the configured
/// runner remains exact argv rather than becoming host-shell text.
#[test]
fn start_runner_configuration_honors_the_declared_isolation() {
    let backend = backend_from_configuration(
        Some("docker"),
        Some("$ --isolated docker --auto-remove-docker-container --"),
        None,
    )
    .expect("the Dockerfile configuration is valid")
    .expect("the pair selects a backend");
    assert_eq!(
        backend,
        ExecutionBackend::StartRunner {
            program: String::from("$"),
            arguments: vec![
                String::from("--isolated"),
                String::from("docker"),
                String::from("--auto-remove-docker-container"),
                String::from("--"),
            ],
            isolation: String::from("docker"),
        }
    );

    for incomplete in [
        backend_from_configuration(Some("docker"), None, None),
        backend_from_configuration(None, Some("$ --isolated docker --"), None),
        backend_from_configuration(Some("host"), Some("runner"), None),
    ] {
        assert!(
            matches!(incomplete, Err(BoxError::InvalidConfiguration { .. })),
            "incomplete or non-isolated runner configuration must refuse"
        );
    }
}

/// User-controlled argv follows the fixed container program as positional
/// data. Shell punctuation cannot modify the extraction/working-directory
/// command, and the default network denial is visible in argv.
#[test]
fn disposable_container_command_is_hermetic_and_network_denied() {
    let invocation = disposable_container_invocation(
        "konard/box-python:2.4.0",
        NetworkPolicy::Denied,
        "python3",
        &["name; touch escaped", "$(false)"],
    );
    assert_eq!(invocation.program, "docker");
    assert!(
        invocation
            .arguments
            .windows(2)
            .any(|pair| pair == ["--network", "none"]),
        "the default invocation must lower network denial to Docker"
    );
    assert_eq!(
        &invocation.arguments[invocation.arguments.len() - 3..],
        ["python3", "name; touch escaped", "$(false)"],
        "program and arguments stay distinct positional argv"
    );
    assert!(
        invocation
            .arguments
            .iter()
            .any(|argument| argument == "exec \"$@\"" || argument.contains("exec \"$@\"")),
        "the only shell program executes positional argv"
    );
}

/// Language-to-image selection reads the shared box contract. A published
/// language resolves dynamically; a surveyed deferral refuses with its data
/// reason instead of borrowing a different image.
#[test]
fn box_language_backend_is_selected_from_the_shared_contract() {
    let python = backend_from_configuration(None, None, Some("box-language:python"))
        .expect("python has a published box contract")
        .expect("the contract selects a backend");
    let ExecutionBackend::Box { image } = python else {
        panic!("a published language must select its box image");
    };
    assert!(image.starts_with("konard/box-python:"));

    let kotlin = backend_from_configuration(None, None, Some("box-language:kotlin"));
    let Err(BoxError::InvalidConfiguration { detail }) = kotlin else {
        panic!("a deferred language must refuse rather than borrow an image");
    };
    assert!(detail.contains("box-kotlin"));

    assert!(matches!(
        backend_from_configuration(None, None, Some("box:--privileged")),
        Err(BoxError::InvalidConfiguration { .. })
    ));
}
