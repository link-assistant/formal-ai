//! PR #1188 G85: verified line-range movement and preserved scalar file moves.
//! Uses the same driven-session harness as the remaining TEACH-F cases.

use super::pull_request_1188_teach_f::drive;

const G85_SOURCE: &str = "l1\nl2\nl3\nl4\nl5\n";

#[test]
fn g85_a_line_range_is_appended_to_the_destination_then_removed_from_the_source() {
    let run = drive(
        "Move lines 2-3 of a.md to the end of b.md.",
        &[("a.md", G85_SOURCE), ("b.md", "b1\n")],
    );
    assert_eq!(run.files["a.md"], "l1\nl4\nl5\n");
    assert_eq!(run.files["b.md"], "b1\nl2\nl3\n");
    assert!(
        run.commands
            .iter()
            .all(|command| !command.starts_with("mv ") && !command.starts_with("cp ")),
        "{:?}",
        run.commands
    );
    assert_eq!(
        run.answer.as_deref(),
        Some("Moved lines 2-3 of `a.md` to the end of `b.md` and observed the result.")
    );
}

#[test]
fn g85_the_destination_is_the_path_after_the_cue_wherever_the_source_is_named() {
    let run = drive(
        "In a.md move lines 2 to 3 to the start of b.md.",
        &[("a.md", G85_SOURCE), ("b.md", "b1\n")],
    );
    assert_eq!(run.files["a.md"], "l1\nl4\nl5\n");
    assert_eq!(run.files["b.md"], "l2\nl3\nb1\n");
    assert_eq!(
        run.answer.as_deref(),
        Some("Moved lines 2-3 of `a.md` to the start of `b.md` and observed the result.")
    );
}

#[test]
fn g85_one_line_moved_in_russian() {
    let run = drive(
        "Перенеси строку 4 из a.md в конец b.md.",
        &[("a.md", G85_SOURCE), ("b.md", "b1\n")],
    );
    assert_eq!(run.files["a.md"], "l1\nl2\nl3\nl5\n");
    assert_eq!(run.files["b.md"], "b1\nl4\n");
    assert_eq!(
        run.answer.as_deref(),
        Some("Перенёс строку 4 из `a.md` в конец `b.md` и проверил результат.")
    );
}

#[test]
fn g85_a_range_past_the_end_of_the_source_changes_nothing() {
    let run = drive(
        "Move lines 7-9 of a.md to the end of b.md.",
        &[("a.md", G85_SOURCE), ("b.md", "b1\n")],
    );
    assert_eq!(run.tools, ["read"]);
    assert_eq!(run.files["b.md"], "b1\n");
    assert_eq!(
        run.answer.as_deref(),
        Some("`a.md` has 5 lines, so it has no lines 7-9 to move, and nothing was changed.")
    );
}

#[test]
fn g85_a_file_move_without_line_numbers_is_still_a_file_move() {
    let run = drive("Move a.md to c.md.", &[("a.md", G85_SOURCE)]);
    assert!(
        run.commands.iter().any(|command| command == "test -e a.md"),
        "{:?}",
        run.commands
    );
}
