//! PR #1188 FIX-GAPS: the open edit gaps G90, G99, G102, G104, G106 and G107
//! of `experiments/formal_ai_subagent/gaps.md` (ledger rows T371, T377, T471,
//! T473, T720, T786, T793 and T794; the fixes are rows T840-T869 of
//! `docs/case-studies/pull-request-1188/formal-ai-dogfood.md`). Twin of
//! `rust/tests/web/pull-request-1188-fix-gaps.test.mjs`.

use super::pull_request_1188_teach_f::drive;

const MODULE: &str = "import { A } from \"x\";\nfoo(C);\n";
const LINES: &str = "a\nb\nc\nd\n";
const ROW: &str = "row A. one\nrow C two\n";
const SEVERAL_FILES: &str = "and one edit request changes one file, so nothing was changed. Ask for the edit once for each file.";
const NESTED: &str = "so I cannot tell where it ends, and nothing was done. Quote such a text with «», or with a mark it does not hold.";

#[test]
fn g104_files_listed_with_and_or_commas_are_all_named() {
    let run = drive(
        "In a.mjs and b.mjs, replace «A» with «B» and replace every «C» with «D».",
        &[("a.mjs", MODULE), ("b.mjs", MODULE)],
    );
    assert!(run.tools.is_empty(), "{:?}", run.tools);
    assert_eq!(run.files["a.mjs"], MODULE);
    assert_eq!(run.files["b.mjs"], MODULE);
    assert_eq!(
        run.answer.as_deref(),
        Some(format!("This edit names several files (`a.mjs`, `b.mjs`), {SEVERAL_FILES}").as_str())
    );
    let three = drive(
        "In a.txt, b.txt and c.txt, replace «x» with «y».",
        &[("a.txt", "x\n"), ("b.txt", "x\n"), ("c.txt", "x\n")],
    );
    assert!(three.tools.is_empty(), "{:?}", three.tools);
    assert_eq!(
        three.answer.as_deref(),
        Some(
            format!("This edit names several files (`a.txt`, `b.txt`, `c.txt`), {SEVERAL_FILES}")
                .as_str()
        )
    );
    // A path in a later sentence (the ladder leaf asking for evidence) is no
    // edit target.
    let ladder = drive(
        "In the file rust/src/core.rs, replace \"statement_negation_cue\" with \"statement_negation_marker\". Leave supporting evidence in .agent-ladder/node-2.1-proof.md.",
        &[(
            "rust/src/core.rs",
            "const statement_negation_cue: u8 = 1;\n",
        )],
    );
    assert_eq!(
        ladder.files["rust/src/core.rs"],
        "const statement_negation_marker: u8 = 1;\n"
    );
    assert!(
        !ladder
            .answer
            .as_deref()
            .unwrap_or_default()
            .starts_with("This edit names several files"),
        "{:?}",
        ladder.answer
    );
    let one = drive("In a.mjs, replace «A» with «B».", &[("a.mjs", MODULE)]);
    assert_eq!(one.files["a.mjs"], "import { B } from \"x\";\nfoo(C);\n");
    assert_eq!(
        one.answer.as_deref(),
        Some("Replaced `A` with `B` in `a.mjs` and observed the result.")
    );
}

#[test]
fn g99_edits_joined_by_a_sequence_cue_are_planned_step_by_step() {
    let run = drive(
        "Insert the line «b2» after the line «b» in f.txt, then delete the line «c» from f.txt.",
        &[("f.txt", LINES)],
    );
    assert_eq!(run.files["f.txt"], "a\nb\nb2\nd\n");
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "Inserted `b2` after `b` in `f.txt` and observed the result.\n\nRemoved `c` from `f.txt` and observed the result."
        )
    );
    let inherited = drive(
        "In f.txt, insert the line «b2» after the line «b» and then delete the line «c».",
        &[("f.txt", LINES)],
    );
    assert_eq!(inherited.files["f.txt"], "a\nb\nb2\nd\n");
    let russian = drive(
        "Вставь строку «b2» после строки «b» в f.txt, затем удали строку «c» из f.txt.",
        &[("f.txt", LINES)],
    );
    assert_eq!(russian.files["f.txt"], "a\nb\nb2\nd\n");
    assert_eq!(
        russian.answer.as_deref(),
        Some(
            "В `f.txt` после `b` вставлено `b2`, результат проверен.\n\nИз `f.txt` удалено `c`, результат проверен."
        )
    );
}

#[test]
fn g106_several_replaces_apply_all_or_decline() {
    let run = drive(
        "In r.md, replace «A.» with «B.» and then replace «C» with «D».",
        &[("r.md", ROW)],
    );
    assert_eq!(run.files["r.md"], "row B. one\nrow D two\n");
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "Replaced `A.` with `B.` in `r.md` and observed the result.\n\nReplaced `C` with `D` in `r.md` and observed the result."
        )
    );
    let declined = drive(
        "In r.md, replace «A.» with «B.» and replace C with D.",
        &[("r.md", ROW)],
    );
    assert!(declined.tools.is_empty(), "{:?}", declined.tools);
    assert_eq!(declined.files["r.md"], ROW);
    assert_eq!(
        declined.answer.as_deref(),
        Some(
            "This request asks for several edits, and I cannot read `replace C with D` as a replacement of quoted text, so nothing was changed. Quote its old and new texts, or ask for that edit on its own."
        )
    );
}

#[test]
fn g90_a_quoted_text_holding_its_own_quote_mark_is_declined() {
    let rust = "    (text.matches('`').count() % 2 == 0).then_some(text)\n";
    let run = drive(
        "Replace '(text.matches('`').count() % 2 == 0).then_some(text)' with 'text.matches('`').count().is_multiple_of(2).then_some(text)' in p.rs.",
        &[("p.rs", rust)],
    );
    assert!(run.tools.is_empty(), "{:?}", run.tools);
    assert_eq!(run.files["p.rs"], rust);
    assert_eq!(
        run.answer.as_deref(),
        Some(
            format!(
                "A quoted text in this request holds its own quote mark (``'(text.matches('`').count() % 2``), {NESTED}"
            )
            .as_str()
        )
    );
    let doubled = drive(
        "Replace \"a synthesized.\"\" with \"a holds.\"\" in f.lino.",
        &[("f.lino", "x = \"a synthesized.\"\n")],
    );
    assert!(doubled.tools.is_empty(), "{:?}", doubled.tools);
    assert_eq!(doubled.files["f.lino"], "x = \"a synthesized.\"\n");
    assert_eq!(
        doubled.answer.as_deref(),
        Some(
            format!(
                "A quoted text in this request holds its own quote mark (`\" with \"a holds.\"\" in f.lino.`), {NESTED}"
            )
            .as_str()
        )
    );
    let guillemets = drive(
        "Replace «\"a synthesized.\"» with «\"a holds.\"» in f.lino.",
        &[("f.lino", "x = \"a synthesized.\"\n")],
    );
    assert_eq!(guillemets.files["f.lino"], "x = \"a holds.\"\n");
}

#[test]
fn g107_a_backticked_payload_whose_inner_spans_pair_runs_to_its_own_close() {
    let run = drive(
        "In r.md, replace `x` with `a. `b` x`.",
        &[("r.md", "one x two\n")],
    );
    assert_eq!(run.files["r.md"], "one a. `b` x two\n");
    assert_eq!(
        run.answer.as_deref(),
        Some("Replaced `x` with ``a. `b` x`` in `r.md` and observed the result.")
    );
    let row = drive(
        "In r.md, replace `Remaining: x` with `In chat, \"f <u>\" (the `p_f` meaning of `d/m.lino`, in five) fetch (`r/p.rs`, `r/w.rs`; worker `j/w.js`); offline nothing. `t/u.rs` and `t/w.mjs` pin it. Remaining: x`",
        &[("r.md", "| U18 | Remaining: x, since y. |\n")],
    );
    assert_eq!(
        row.files["r.md"],
        "| U18 | In chat, \"f <u>\" (the `p_f` meaning of `d/m.lino`, in five) fetch (`r/p.rs`, `r/w.rs`; worker `j/w.js`); offline nothing. `t/u.rs` and `t/w.mjs` pin it. Remaining: x, since y. |\n"
    );
}

#[test]
fn g102_a_file_described_rather_than_given_is_never_written() {
    let run = drive(
        "Create the file abbreviations.test.mjs. It imports node:assert/strict as assert and node:test as test, and imports wordsOfIdentifier from «./scripts/measure-abbreviations.mjs». Write one test:\n1. test «a declared name splits at underscores»: assert.deepEqual(wordsOfIdentifier(«parseArgs»), [«parse», «args»]).",
        &[(
            "scripts/measure-abbreviations.mjs",
            "export function wordsOfIdentifier() {}\n",
        )],
    );
    // The planner plans no step: no `cat` of the fragments and no write of a
    // failed or refused step; the server's solver then answers.
    assert!(run.tools.is_empty(), "{:?}", run.tools);
    assert_eq!(run.answer, None);
    assert!(!run.files.contains_key("abbreviations.test.mjs"));
}

#[test]
fn g109_copy_then_comma_joined_edits_preserves_source() {
    let run = drive(
        "Copy a.lino to b.lino, then in b.lino replace every «x» with «y» and replace «m» with «n».",
        &[("a.lino", "x m\n")],
    );
    assert_eq!(run.files["a.lino"], "x m\n");
    assert_eq!(run.files["b.lino"], "y n\n");
}

#[test]
fn g108_scalar_backtick_comma_payload_is_fenced_whole() {
    let payload = "values `left`, `right` remain scalar";
    let request = format!("In f.txt replace «x» with «{payload}»");
    let run = drive(&request, &[("f.txt", "x")]);
    assert_eq!(run.files["f.txt"], payload);
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "Replaced `x` with ``values `left`, `right` remain scalar`` in `f.txt` and observed the result."
        )
    );
}

#[test]
fn g110_payload_paths_are_never_creation_targets() {
    let content = "record destination to rust/src/solver_handlers/policy_gates.rs";
    let request = format!("Create budget.lino with the content «{content}».");
    let run = drive(&request, &[]);
    assert_eq!(run.files["budget.lino"], content);
    assert!(
        !run.files
            .contains_key("rust/src/solver_handlers/policy_gates.rs")
    );
}

#[test]
fn g113_quoted_content_retains_all_whitespace() {
    let content = "  seed method-execution\n    bundle true\n";
    let request = format!("Create rows.lino with the content «{content}».");
    let run = drive(&request, &[]);
    assert_eq!(run.files["rows.lino"], content);
}

#[test]
fn g114_quoted_objective_is_preserved() {
    let content = "pub fn emit(\n    task: &str,\n) {}";
    let request = format!("In f.rs replace «old» with «{content}»");
    let run = drive(&request, &[("f.rs", "old")]);
    assert_eq!(run.files["f.rs"], content);
}

#[test]
fn g115_new_only_escape_is_source_text() {
    let old = "const lines = text;";
    let next = r"const lines = text.split('\n');";
    let request = format!("In f.mjs replace «{old}» with «{next}»");
    let run = drive(&request, &[("f.mjs", old)]);
    assert_eq!(run.files["f.mjs"], next);
}

#[test]
fn authored_fenced_source_cues_are_literal_data() {
    let content = "const inventory = [];\n// When `input` arrives then `output` follows.\n// task: Append to src/rules.mjs.\n// retain these words, in JavaScript.\n";
    let request = format!("Create output.mjs containing\n```javascript\n{content}```");
    let run = drive(&request, &[]);
    assert_eq!(run.files["output.mjs"], content);
}

#[test]
fn authored_whole_content_consent_overwrites_existing_source() {
    let content = "const inventory = [];\n// Copy a.mjs to b.mjs; Delete scratch.tmp.\n";
    let request = format!("Set the contents of output.mjs to «{content}»");
    let run = drive(&request, &[("output.mjs", "old\n")]);
    assert_eq!(run.files["output.mjs"], content);
}

#[test]
fn authored_new_literal_cannot_promote_replacement_to_whole_file_write() {
    let request = "In f.mjs replace «old» with «Create another.mjs with inventory []»";
    let run = drive(request, &[("f.mjs", "before old after\n")]);
    assert_eq!(
        run.files["f.mjs"],
        "before Create another.mjs with inventory [] after\n"
    );
}

#[test]
fn authored_failure_text_is_verification_data() {
    for content in [
        "Error: this is the diagnostic example.\n",
        "{\"z\":\"failed: recorded attempt\",\"a\":\"fixture\"}\n",
    ] {
        let request = format!("Set the contents of output.mjs to «{content}»");
        let run = drive(&request, &[("output.mjs", "previous\n")]);
        assert_eq!(run.files["output.mjs"], content);
        assert!(
            run.answer
                .as_deref()
                .is_some_and(|answer| answer.starts_with("Completed the general change request")),
            "{:?}",
            run.answer
        );
    }
}
