//! Issue #1138 B6 (plan 06, L4–L5): a missing compiler is a requirement.
//!
//! `command not found` stops the run today. It must instead become a need of
//! kind `prerequisite` — but only when that is what it is: exit 126 is a
//! permission denial and is not installation consent, and an ordinary compiler
//! diagnostic is not a missing prerequisite at all. The platform is observed,
//! never inferred from the language the task was written in.

use formal_ai::meta_frame::NeedStatus;
use formal_ai::prerequisite::probe::ProbeVerdict;
use formal_ai::prerequisite::{Platform, PrerequisiteNeed, classify_failure, need_status};

const REQUIREMENT: &str =
    "Write a program in Zig that prints the sum of the numbers from one to ten, then actually \
compile and run it here and show me the real output.";

fn missing(program: &str) -> PrerequisiteNeed {
    PrerequisiteNeed {
        program: program.to_owned(),
        source_span: REQUIREMENT.to_owned(),
        observed: ProbeVerdict::Missing {
            exit_code: Some(127),
            stderr: format!("{program}: command not found"),
        },
        platform: Platform::observed(),
        requires: Vec::new(),
    }
}

/// Exit 127 with `command not found` names a program the system lacks.
#[test]
fn exit_127_becomes_a_need_not_an_error() {
    let need = classify_failure(
        "zig build-exe sum.zig",
        Some(127),
        "zig: command not found",
        REQUIREMENT,
    )
    .expect("exit 127 names a prerequisite");
    assert_eq!(
        need.program, "zig",
        "the need names the program that was not found"
    );
    assert_eq!(
        need.source_span, REQUIREMENT,
        "the need carries the requirement verbatim, in its original language"
    );
}

/// A permission denial says the file is there and we may not run it. That is not
/// consent to install anything.
#[test]
fn a_permission_denial_is_not_installation_consent() {
    let need = classify_failure(
        "./configure",
        Some(126),
        "./configure: Permission denied",
        REQUIREMENT,
    );
    if let Some(found) = need.as_ref() {
        assert!(
            matches!(found.observed, ProbeVerdict::Unusable { .. }),
            "exit 126 is unusable, never missing: {:?}",
            found.observed
        );
    }
    assert!(
        !matches!(
            need.as_ref().map(|found| &found.observed),
            Some(ProbeVerdict::Missing { .. })
        ),
        "a permission denial may never be classified as a missing program"
    );
}

/// A compiler that ran and rejected the source is not a missing prerequisite.
#[test]
fn an_ordinary_compile_error_is_not_a_missing_prerequisite() {
    let need = classify_failure(
        "rustc sum.rs",
        Some(1),
        "error[E0425]: cannot find value `total` in this scope",
        REQUIREMENT,
    );
    assert!(
        need.is_none(),
        "a compiler diagnostic is a task failure, not a prerequisite: {need:?}"
    );
}

/// The host is observed. A Kotlin task does not imply a JVM-shaped platform.
#[test]
fn the_platform_is_observed_not_inferred_from_the_language() {
    let need = classify_failure(
        "kotlinc sum.kt",
        Some(127),
        "kotlinc: command not found",
        "Напиши на Kotlin программу и запусти её здесь.",
    )
    .expect("exit 127 names a prerequisite");
    assert_eq!(
        need.platform,
        Platform::observed(),
        "the need's platform is the machine's, not the language's"
    );
    assert_ne!(
        need.platform,
        Platform::Unknown,
        "the platform must actually be observed, not left unknown"
    );
}

/// The need is `Blocked` while nothing can be done about it, `Planned` once a
/// procedure is selected, and `Satisfied` only after a re-probe returns
/// `Present`.
#[test]
fn a_prerequisite_need_is_blocked_until_the_reprobe_passes() {
    let need = missing("zig");
    assert_eq!(
        need_status(&need, None),
        NeedStatus::Blocked,
        "with no procedure selected the need is blocked, and says so"
    );
    assert_eq!(
        need_status(
            &need,
            Some(&ProbeVerdict::Missing {
                exit_code: Some(127),
                stderr: String::from("zig: command not found"),
            })
        ),
        NeedStatus::Planned,
        "an attempted but still-failing re-probe leaves the need planned"
    );
    assert_eq!(
        need_status(
            &need,
            Some(&ProbeVerdict::Present {
                version: String::from("0.0.0 as observed"),
            })
        ),
        NeedStatus::Satisfied,
        "only a re-probe returning Present may satisfy a prerequisite need"
    );
}
