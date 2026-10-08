//! PR #1188 dogfooding (T80): `Add an assertion that add(2, 2) equals 4 to
//! m.test.mjs.` replaced the whole test file with that sentence: the general
//! change plan read "an assertion that …" as the file's new bytes. When the
//! write verb is the seeded add action and the content comes before the file,
//! the request names an addition to that file, never its whole new content;
//! content a seeded content lead introduces (`containing`) is still bytes.
//! Twin of `rust/tests/web/pull-request-1188-addition-guard.test.mjs`.

use formal_ai::agentic_coding::general_planner::compose_general_change_plan;

#[test]
fn an_addition_named_before_its_file_is_not_a_whole_file_write() {
    assert!(
        compose_general_change_plan("Add an assertion that add(2, 2) equals 4 to m.test.mjs.")
            .is_none()
    );
    assert!(
        compose_general_change_plan("Добавь проверку, что add(2, 2) равно 4, в m.test.mjs.")
            .is_none()
    );
}

#[test]
fn content_a_content_lead_introduces_is_still_the_file() {
    let plan = compose_general_change_plan("Add a file notes.txt containing 'hello'.")
        .expect("a file creation with a content lead is a whole-file write");
    assert_eq!(
        (plan.target.as_str(), plan.content.as_str()),
        ("notes.txt", "hello")
    );
}

#[test]
fn a_write_verb_that_is_not_an_addition_still_writes_the_file() {
    let plan = compose_general_change_plan("Write 'hello' to notes.txt.")
        .expect("a plain write is a whole-file write");
    assert_eq!(
        (plan.target.as_str(), plan.content.as_str()),
        ("notes.txt", "hello")
    );
}
