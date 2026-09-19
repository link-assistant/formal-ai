//! Source-placement anchor for `src/coding/program_ir.rs` (issue #1138, plan 02
//! L2).
//!
//! The source mirror compiles a copy of each module beside its extracted unit
//! tests, so a module's tests live in `tests/source/source_tests/<module>/`
//! rather than inside the implementation file. The IR module's dependency
//! closure (`task_spec`, `procedure_text`, `fragment_catalog`) is not mirrored
//! yet, so this anchor carries the placement rules the mirror exists to enforce;
//! the implementation leaf that gives the IR real behaviour mirrors the module
//! itself.

#[path = "../source_tests/coding/program_ir/tests.rs"]
mod tests;
