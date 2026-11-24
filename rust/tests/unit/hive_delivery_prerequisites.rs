//! The full-delivery tests must execute their real Git and compiler operations.
//! Their shared harness supports optional local runs; this CI target requires all
//! tools so an unavailable compiler cannot silently turn a case into a pass.

#[cfg(unix)]
#[test]
fn full_delivery_replays_have_their_required_toolchains() {
    assert!(
        crate::offline_delivery::available(&[
            "git", "node", "jq", "python3", "rustc", "timeout", "kotlinc", "java",
        ]),
        "the focused native lane requires every full-delivery toolchain"
    );
}
