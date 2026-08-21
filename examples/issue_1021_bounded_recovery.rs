//! What does an unattended run actually do when it gets stuck, or when the next
//! version of itself does not compile?
//!
//! Issue #1021 carries #947 (E95) and #946 (E94) as one pair of questions about
//! the same run: how long may it work unwatched, and what does it fall back to
//! when its own next version is broken. This probe answers both against real
//! machinery -- an injected clock the probe advances by hand, and a real `rustc`
//! compiling a file that is wrong on purpose.
//!
//! Run with `cargo run --example issue_1021_bounded_recovery`.

use std::fs;
use std::time::Duration;

use formal_ai::bounded_autonomy::{
    AutonomyMode, AutonomyPolicy, LoopStep, ManualClock, RecoveryLoop, Resolution, ResolutionOption,
};
use formal_ai::memory_revision::{AttemptOutcome, RevisionLedger, rustc_verdict};

fn main() {
    stuck_run();
    println!();
    broken_version();
    println!();
    delegated_choice();
}

/// A step that resolves nothing, taken until the limit says stop. The limit is
/// sixty seconds rather than the default hour only so the probe finishes; the
/// arithmetic is the arithmetic the hour uses.
fn stuck_run() {
    let clock = ManualClock::new();
    let policy = AutonomyPolicy {
        mode: AutonomyMode::FullAutonomous,
        full_trust: false,
        stuck_recovery_limit: Duration::from_mins(1),
    };
    let mut run = RecoveryLoop::new(policy, &clock);
    run.record("rebuild_after_the_failing_test");
    run.record("rerun_the_failing_test");

    println!("=== a run that never resolves");
    let mut steps = 0_u32;
    loop {
        match run.step() {
            LoopStep::Continue => {
                steps += 1;
                clock.advance(Duration::from_secs(15));
                println!("-- step {steps}: continue");
            }
            LoopStep::AskPermission {
                request,
                plan,
                elapsed,
                limit,
            } => {
                println!(
                    "-- stopped after {steps} steps: {} ({}s of {}s)",
                    request.slug(),
                    elapsed.as_secs(),
                    limit.as_secs()
                );
                for entry in &plan {
                    println!("   plan: {entry}");
                }
                break;
            }
        }
    }
}

/// A version of itself that a real compiler rejects, and the state the ledger
/// puts back.
fn broken_version() {
    let root = std::env::temp_dir().join(format!(
        "formal-ai-issue-1021-bounded-recovery-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("workspace");
    let stable = "pub fn answer() -> u32 {\n    41\n}\n";
    fs::write(root.join("version.rs"), stable).expect("stable version");

    let tracked = vec![String::from("version.rs")];
    let mut ledger = RevisionLedger::open(&root, &tracked, &[]);

    println!("=== a next version of itself that does not compile");
    let outcome = ledger
        .attempt(
            |root| {
                fs::write(
                    root.join("version.rs"),
                    "pub fn answer() -> u32 {\n    \"42\"\n}\n",
                )
            },
            |root| {
                let verdict = rustc_verdict(root, "version.rs", 1);
                for line in verdict.diagnostics.lines().take(3) {
                    println!("-- rustc: {line}");
                }
                verdict
            },
        )
        .expect("attempt");

    match outcome {
        AttemptOutcome::Adopted { revision } => println!("-- adopted {revision}"),
        AttemptOutcome::RolledBack {
            restored, reason, ..
        } => println!("-- {} -> restored {restored}", reason.slug()),
    }
    let restored = fs::read_to_string(root.join("version.rs")).expect("version");
    println!(
        "-- workspace matches the last stable version: {}",
        restored == stable
    );
    let _ = fs::remove_dir_all(&root);
}

/// The same choice, asked and answered, under the two trust settings.
fn delegated_choice() {
    let options = vec![
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
    ];

    println!("=== two viable resolutions");
    for full_trust in [false, true] {
        let clock = ManualClock::new();
        let policy = AutonomyPolicy {
            mode: AutonomyMode::FullAutonomous,
            full_trust,
            stuck_recovery_limit: Duration::from_mins(1),
        };
        let mut run = RecoveryLoop::new(policy, &clock);
        match run.resolve(&options) {
            Resolution::Ask(request) => {
                println!("-- full_trust={full_trust}: {}", request.slug());
            }
            Resolution::Chose(choice) => println!(
                "-- full_trust={full_trust}: {} (net {} against runner-up {:?})",
                choice.option.id, choice.net_weight, choice.runner_up
            ),
        }
    }
}
