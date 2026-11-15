//! Unit tests of rust/src/agentic_coding/work_item_leads.rs (issue #1483),
//! installed as its nested `tests` module so private helpers stay private.
//! Issue #1191: they were an inline `mod tests` in src/, which main's
//! `src_has_no_inline_unit_tests` rejects; no pull-request check saw it.
use super::{symbol_matches, work_item_leads, work_item_symbols};

#[test]
fn leads_split_attachments_documents_and_related_pages() {
    let task = "See ![shot](https://github.com/user-attachments/assets/abc) and the log \
                https://example.com/run.log. Related: https://github.com/o/r/issues/2 \
                and https://github.com/o/r/blob/main/src/a.rs";
    let leads = work_item_leads(task, "https://github.com/o/r/issues/1");
    assert_eq!(
        leads.attachments,
        ["https://github.com/user-attachments/assets/abc"]
    );
    assert_eq!(leads.documents, ["https://example.com/run.log"]);
    assert_eq!(leads.related, ["https://github.com/o/r/issues/2"]);
}

#[test]
fn symbols_keep_compound_identifiers_only() {
    let task =
        "The `planned_not_executed` reason in `Progress::scan()` and `word` and `camelCase`, `x.y`";
    assert_eq!(
        work_item_symbols(task),
        ["planned_not_executed", "scan", "camelCase"]
    );
}

#[test]
fn symbol_matches_rank_sources_first_then_depth() {
    let output = "./docs/notes.md\n/w/src/deep/a.rs:3: x\nsrc/b.mjs\nnode_modules/x/y.js\n";
    assert_eq!(
        symbol_matches(output, &["/w".to_owned()]),
        ["src/b.mjs", "src/deep/a.rs", "docs/notes.md"]
    );
}
