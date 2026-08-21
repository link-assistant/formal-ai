//! Issue #1021 / #947 (E95): the run stops on a clock, not on an exception.
//!
//! The review of this branch asked for the clock to be injected rather than
//! read, and that is the whole reason these tests exist at all: a one-hour limit
//! read from `Instant::now()` is a one-hour test, so nobody writes it and the
//! limit ships unexercised. [`ManualClock`] advances by hand, so the pathological
//! stuck scenario issue #947 describes -- a loop that never resolves -- runs here
//! in microseconds against the same arithmetic the default hour uses.

use std::time::Duration;

use formal_ai::bounded_autonomy::{
    weigh, AutonomyMode, AutonomyPolicy, Clock as _, LoopStep, ManualClock, PermissionRequest,
    RecoveryLoop, Resolution, ResolutionOption, SystemClock, AUTONOMY_MODE_VARIABLE,
    DEFAULT_STUCK_RECOVERY_LIMIT, FULL_AUTONOMOUS_VALUE, FULL_TRUST_VALUE, FULL_TRUST_VARIABLE,
    STUCK_RECOVERY_LIMIT_VARIABLE,
};

/// Run `body` with the three autonomy variables forced to `values` -- every one
/// of them, so a variable this call does not name is unset rather than
/// inherited -- and put the environment back afterwards, panic or not.
///
/// The policy is read from the process environment, so the modes cannot be
/// entered at once and every test that touches it must be serialised. That used
/// to be a mutex kept here; since edition 2024 made `std::env::set_var` unsafe
/// -- for exactly this reason -- and this crate forbids unsafe code, the
/// scoping is `temp-env`'s, whose reentrant lock is held for the closure and
/// serialises these tests against every other environment override in the
/// binary rather than only against each other.
fn with_environment(values: &[(&str, Option<&str>)], body: impl FnOnce()) {
    let mut scoped: Vec<(&str, Option<&str>)> = [
        AUTONOMY_MODE_VARIABLE,
        FULL_TRUST_VARIABLE,
        STUCK_RECOVERY_LIMIT_VARIABLE,
    ]
    .iter()
    .map(|name| (*name, None))
    .collect();
    for (name, value) in values {
        if let Some(slot) = scoped.iter_mut().find(|(scoped, _)| scoped == name) {
            slot.1 = *value;
        } else {
            scoped.push((name, *value));
        }
    }
    temp_env::with_vars(scoped, body);
}

const fn unattended(limit: Duration) -> AutonomyPolicy {
    AutonomyPolicy {
        mode: AutonomyMode::FullAutonomous,
        full_trust: false,
        stuck_recovery_limit: limit,
    }
}

#[test]
fn a_loop_that_never_resolves_stops_at_the_limit_and_asks() {
    let clock = ManualClock::new();
    let mut run = RecoveryLoop::new(unattended(Duration::from_mins(1)), &clock);
    run.record("retry_the_failing_build");

    // The pathological case: a step that changes nothing, taken forever.
    let mut taken = 0_usize;
    let stop = loop {
        match run.step() {
            LoopStep::Continue => {
                taken += 1;
                clock.advance(Duration::from_secs(10));
                assert!(taken < 100, "the loop should be bounded, not merely slow");
            }
            stop @ LoopStep::AskPermission { .. } => break stop,
        }
    };

    let LoopStep::AskPermission {
        request,
        plan,
        elapsed,
        limit,
    } = stop
    else {
        panic!("the loop should have stopped to ask");
    };
    assert_eq!(request, PermissionRequest::StuckRecoveryLimitReached);
    assert_eq!(plan, vec![String::from("retry_the_failing_build")]);
    assert_eq!(elapsed, Duration::from_mins(1));
    assert_eq!(limit, Duration::from_mins(1));
    assert_eq!(taken, 6);
}

#[test]
fn the_question_repeats_until_the_operator_answers_it() {
    let clock = ManualClock::new();
    let mut run = RecoveryLoop::new(unattended(Duration::from_secs(1)), &clock);
    clock.advance(Duration::from_secs(5));

    for _ in 0..3 {
        assert!(
            !run.step().proceeds(),
            "an unanswered question must not decay into permission"
        );
    }

    assert_eq!(run.questions(), 3);
}

#[test]
fn granting_more_time_resumes_the_run_from_where_it_stopped() {
    let clock = ManualClock::new();
    let mut run = RecoveryLoop::new(unattended(Duration::from_mins(1)), &clock);
    clock.advance(Duration::from_mins(1));
    assert!(!run.step().proceeds());

    run.grant(Duration::from_mins(1));

    assert!(run.step().proceeds(), "the granted hour should be usable");
    assert_eq!(run.deadline(), Duration::from_mins(2));
    clock.advance(Duration::from_mins(1));
    assert!(!run.step().proceeds(), "and then the limit applies again");
}

#[test]
fn the_default_limit_is_the_hour_the_issue_names() {
    let policy = AutonomyPolicy::default();

    assert_eq!(policy.stuck_recovery_limit, DEFAULT_STUCK_RECOVERY_LIMIT);
    // Spelled out in seconds rather than in the same unit the constant uses:
    // an assertion written as `from_hours(1)` against a constant that *is*
    // `from_hours(1)` would pass even if the unit were the thing that broke.
    assert_eq!(policy.stuck_recovery_limit.as_secs(), 3_600);
}

#[test]
fn the_default_mode_gates_every_command() {
    let policy = AutonomyPolicy::default();

    assert_eq!(policy.mode, AutonomyMode::PerCommandPermission);
    assert!(!policy.permits_unattended());
    assert!(
        !policy.full_trust,
        "full trust is opted into, never fallen into"
    );
}

#[test]
fn a_per_command_run_gates_each_command_and_records_the_approved_ones() {
    let clock = ManualClock::new();
    let mut run = RecoveryLoop::new(AutonomyPolicy::default(), &clock);

    let step = run.step_command("cargo test --test unit");
    let LoopStep::AskPermission { request, plan, .. } = step else {
        panic!("per-command mode should gate the command");
    };
    assert_eq!(request, PermissionRequest::CommandNotYetApproved);
    assert_eq!(plan, vec![String::from("cargo test --test unit")]);
    assert!(
        run.plan().is_empty(),
        "a command that was only proposed is not part of the plan yet"
    );

    run.approve("cargo test --test unit");

    assert_eq!(run.approvals(), 1);
    assert_eq!(run.plan(), [String::from("cargo test --test unit")]);
    assert!(
        !run.step_command("cargo clippy").proceeds(),
        "approval covers the command that was approved, not the next one"
    );
}

#[test]
fn a_full_autonomous_run_takes_commands_without_asking() {
    let clock = ManualClock::new();
    let mut run = RecoveryLoop::new(unattended(Duration::from_mins(1)), &clock);

    assert!(run.step_command("cargo test --test unit").proceeds());
    assert!(run.step_command("cargo clippy").proceeds());

    assert_eq!(run.questions(), 0);
    assert_eq!(run.plan().len(), 2);
}

#[test]
fn the_limit_outranks_the_mode() {
    let clock = ManualClock::new();
    let mut run = RecoveryLoop::new(unattended(Duration::from_mins(1)), &clock);
    clock.advance(Duration::from_mins(2));

    let step = run.step_command("cargo test --test unit");

    let LoopStep::AskPermission { request, .. } = step else {
        panic!("a spent limit stops a full-autonomous run too");
    };
    assert_eq!(
        request,
        PermissionRequest::StuckRecoveryLimitReached,
        "a run that has spent its hour asks for the hour, not for the command"
    );
}

fn options() -> Vec<ResolutionOption> {
    vec![
        ResolutionOption::new(
            "pin_the_old_major",
            vec![(String::from("builds_today"), 3)],
            vec![(String::from("carries_debt_forward"), 5)],
        ),
        ResolutionOption::new(
            "adopt_the_replacement_api",
            vec![
                (String::from("builds_today"), 3),
                (String::from("removes_the_debt"), 4),
            ],
            vec![(String::from("larger_diff"), 2)],
        ),
    ]
}

#[test]
fn without_full_trust_a_choice_goes_back_to_the_operator() {
    let clock = ManualClock::new();
    let mut run = RecoveryLoop::new(AutonomyPolicy::default(), &clock);

    let options = options();

    let resolution = run.resolve(&options);

    assert_eq!(
        resolution,
        Resolution::Ask(PermissionRequest::ChoiceNotDelegated)
    );
    assert_eq!(run.questions(), 1);
}

#[test]
fn full_trust_selects_the_heaviest_option_and_records_the_arithmetic() {
    let clock = ManualClock::new();
    let mut policy = unattended(Duration::from_mins(1));
    policy.full_trust = true;
    let mut run = RecoveryLoop::new(policy, &clock);
    let options = options();

    let resolution = run.resolve(&options);

    let Resolution::Chose(choice) = resolution else {
        panic!("full trust should choose: {resolution:?}");
    };
    assert_eq!(choice.option.id, "adopt_the_replacement_api");
    assert_eq!(choice.net_weight, 5);
    assert_eq!(choice.runner_up, Some(-2));
    assert_eq!(run.questions(), 0);
}

#[test]
fn a_tie_is_not_a_decision_full_trust_gets_to_make() {
    let tied = vec![
        ResolutionOption::new("left", vec![(String::from("a"), 2)], Vec::new()),
        ResolutionOption::new("right", vec![(String::from("b"), 2)], Vec::new()),
    ];

    assert_eq!(weigh(&tied), None);
    assert_eq!(weigh(&[]), None);
}

#[test]
fn the_environment_selects_the_mode_the_trust_and_the_limit() {
    with_environment(&[], || {
        assert_eq!(AutonomyPolicy::from_env(), AutonomyPolicy::default());
    });
    with_environment(
        &[
            (AUTONOMY_MODE_VARIABLE, Some(FULL_AUTONOMOUS_VALUE)),
            (FULL_TRUST_VARIABLE, Some(FULL_TRUST_VALUE)),
            (STUCK_RECOVERY_LIMIT_VARIABLE, Some("90")),
        ],
        || {
            let policy = AutonomyPolicy::from_env();
            assert_eq!(policy.mode, AutonomyMode::FullAutonomous);
            assert!(policy.full_trust);
            assert_eq!(policy.stuck_recovery_limit, Duration::from_secs(90));
        },
    );
}

#[test]
fn an_unreadable_or_zero_limit_falls_back_to_the_default_hour() {
    for value in ["", "0", "later", "-5"] {
        with_environment(&[(STUCK_RECOVERY_LIMIT_VARIABLE, Some(value))], || {
            assert_eq!(
                AutonomyPolicy::from_env().stuck_recovery_limit,
                DEFAULT_STUCK_RECOVERY_LIMIT,
                "a limit of {value:?} should not silently halt the run on step one"
            );
        });
    }
}

#[test]
fn full_trust_does_not_arrive_with_the_full_autonomous_mode() {
    with_environment(
        &[(AUTONOMY_MODE_VARIABLE, Some(FULL_AUTONOMOUS_VALUE))],
        || {
            let policy = AutonomyPolicy::from_env();
            assert!(policy.permits_unattended());
            assert!(
                !policy.full_trust,
                "delegating the commands is not delegating the choices"
            );
        },
    );
}

#[test]
fn the_real_clock_moves_forward() {
    let clock = SystemClock::started();
    let first = clock.elapsed();
    std::thread::sleep(Duration::from_millis(2));

    assert!(
        clock.elapsed() > first,
        "the injected clock is a seam, not a replacement for real time"
    );
}
