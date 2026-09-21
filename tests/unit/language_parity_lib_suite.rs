//! The five-language parity census library, compiled into the unit suite.
//!
//! Issue #1081 (tests/unit/ci-cd/issue_1081.rs) counts a script's inline
//! suite as covered when it is compiled into the test crate or run by a gate
//! with `--test`. `scripts/language-parity-lib.rs` is a library script -- it
//! has no `fn main`, so `rust-script --test` cannot execute it directly, and
//! the two consumers that `#[path]`-include it are gated with `--test`, which
//! runs their own suites through the included module only as a side effect of
//! their binaries. Compiling it here makes the unit suite the second, direct
//! home for its tests, and keeps `scripts/test-scripts.sh`'s derived
//! selection equal to the test's expectation.
//!
//! The census below reads the same `DEBT_FILE` the parity gate reads, so a
//! drift between this module and the gate's expectations surfaces here.

// `run` is the entry point of the two script consumers that include this
// library; this crate reads only the census, so the unused entry point is
// expected here and must not fail the no-warnings build.
#[allow(dead_code)]
#[path = "../../scripts/language-parity-lib.rs"]
mod language_parity_lib;

use std::path::Path;

#[test]
fn the_parity_census_library_counts_the_committed_debt_file() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(root.join(language_parity_lib::DEBT_FILE).is_file());
    let gaps = language_parity_lib::current_gap_count(root)
        .expect("the committed parity debt file parses with the gate's own reader");
    // The debt ratchet (data/meta/debt-ratchet.lino, `language_parity_gaps`)
    // holds this measure at 915 and only allows it to fall. The bound here
    // mirrors the ratchet so this suite is also a tripwire for a regression
    // the gate would catch one job later.
    assert!(
        gaps <= 915,
        "the five-language parity debt must stay under the ratchet's 915; got {gaps} gaps"
    );
}
