//! Regression coverage for issue #1053: the test suite ran unoptimized.
//!
//! `[profile.test]` never set `opt-level`, so it inherited Cargo's default of
//! **0**. That default suits projects whose tests are I/O-bound; this one is
//! the opposite case. The seven tests over 60 seconds spawn no subprocesses at
//! all -- they are pure in-process computation, which is exactly the work an
//! optimizer removes.
//!
//! Measured on the whole unit suite, same 1945 tests either way:
//!
//! ```text
//! opt-level 0   104.64s
//! opt-level 2    28.25s     3.8x
//! ```
//!
//! The cost is about 40 seconds of extra compilation per job, paid once and
//! repaid many times over -- and on the macOS lane it is paid once in the
//! archive job while all eight slices run the faster binaries.
//!
//! The subtlety worth pinning: `opt-level` is independent of
//! `debug-assertions` and `overflow-checks`. Optimizing while silently
//! dropping those would change *what the tests check* -- `debug_assert!`
//! appears throughout `src/`, and an arithmetic overflow has to keep panicking
//! rather than wrapping. Both stay explicitly on.

use std::fs;

fn manifest() -> String {
    fs::read_to_string(format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR")))
        .expect("read Cargo.toml")
        .replace("\r\n", "\n")
}

fn test_profile() -> String {
    let manifest = manifest();
    let profile = manifest
        .split("[profile.test]")
        .nth(1)
        .expect("Cargo.toml declares [profile.test]");
    profile.split("\n[").next().unwrap_or(profile).to_string()
}

/// Tests are compiled with optimization.
#[test]
fn the_test_profile_is_optimized() {
    let profile = test_profile();

    let level: u32 = profile
        .lines()
        .find_map(|line| line.trim().strip_prefix("opt-level = "))
        .and_then(|value| value.trim().parse().ok())
        .expect(
            "[profile.test] must set opt-level; the default of 0 leaves the \
                 suite running unoptimized code",
        );

    assert!(
        level >= 2,
        "opt-level {level} is below 2. The suite's slowest tests are pure \
         computation -- 104.64s unoptimized against 28.25s at level 2 -- so \
         lowering this trades minutes of run time for seconds of compile time."
    );
}

/// Optimizing must not quietly weaken what the tests verify.
///
/// `opt-level`, `debug-assertions` and `overflow-checks` are independent
/// settings. Raising the first while leaving the others to follow a release-ish
/// default would turn off `debug_assert!` and let arithmetic overflow wrap
/// silently -- the suite would run faster and check less, which is worse than
/// running slowly.
#[test]
fn optimization_does_not_disable_the_checks_tests_rely_on() {
    let profile = test_profile();

    for (setting, why) in [
        (
            "debug-assertions = true",
            "`debug_assert!` appears throughout src/ and must keep firing under test",
        ),
        (
            "overflow-checks = true",
            "an arithmetic overflow must panic here, not wrap silently",
        ),
    ] {
        assert!(
            profile.contains(setting),
            "[profile.test] must state `{setting}` explicitly: {why}. Profile:\n{profile}"
        );
    }
}
