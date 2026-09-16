//! Mirror entry for `src/verifiable_task*` (issue #1138, plan 08).
//!
//! The module is a wave-T skeleton: every body is `todo!` until wave I8 fills it
//! in, so there is nothing private to mirror yet. What the mirror does carry
//! today is the source-placement contract — the declared file set and the
//! file-size ceiling — which is what plan 14's wave T asks of this directory.

#[path = "source_tests/verifiable_task/tests.rs"]
mod tests;
