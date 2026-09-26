//! The js -> ts -> rust cycle as CI structure (issue #1138, plan 16 L4).
//!
//! The maintainer's instruction (2026-09-21, quoted in the plan) is three
//! demands in one: each layer's CI runs only when its folder changed
//! (carry-forward -- a stabilized layer costs nothing), the layers are
//! ordered (`fail on ts only if all js checks pass, and fail on rust only if
//! all js and ts checks pass`), and the checks that span all layers stay
//! keyed on everything rather than on one layer's folder.
//!
//! These tests pin the structure that delivers those demands in
//! `.github/workflows/layered-ci.yml` and in
//! `scripts/detect-code-changes.rs`, following the repository's convention
//! of auditing workflows as text (`ci_gates.rs`, `workflow_fixtures.rs`).

use std::fs;

use super::workflow_fixtures::{job_block, unwrapped};

fn layered_workflow() -> String {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    fs::read_to_string(format!("{root}/.github/workflows/layered-ci.yml"))
        .expect("the layered CI workflow exists")
        .replace("\r\n", "\n")
}

fn release_workflow() -> String {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
    fs::read_to_string(format!("{root}/.github/workflows/release.yml"))
        .expect("the release workflow exists")
        .replace("\r\n", "\n")
}

/// The `if:` expression of one layered job, whitespace-folded so a condition
/// means the same thing however it wraps (the `unwrapped` convention).
fn tier_condition(workflow: &str, job: &str) -> String {
    let block = job_block(workflow, job);
    let line = block
        .lines()
        .find_map(|line| line.trim_start().strip_prefix("if: "))
        .unwrap_or_else(|| panic!("the {job} job declares a tier condition"));
    unwrapped(line)
}

/// The cycle's job graph: every tier declares its own path predicate, and
/// the predicates are cumulative downward -- the ts tier's condition names
/// only ts and the js result, the rust tier's only rust and the two results
/// above it.
#[test]
fn every_tier_declares_its_path_predicate_and_needs_the_tiers_above() {
    let workflow = layered_workflow();
    let names = super::workflow_fixtures::workflow_job_names(&workflow);
    for job in ["changes", "js", "ts", "rust"] {
        assert!(
            names.contains(&job),
            "the layered workflow declares the {job} job, found {names:?}"
        );
    }

    let js = tier_condition(&workflow, "js");
    assert_eq!(
        js, "needs.changes.outputs.js-changed == 'true'",
        "the js tier is gated on its own predicate and nothing else"
    );

    let ts = tier_condition(&workflow, "ts");
    assert!(
        ts.starts_with("needs.changes.outputs.ts-changed == 'true' && (needs.js.result == 'success' || needs.js.result == 'skipped')"),
        "the ts tier is gated on its own predicate plus the js tier being green or carried: {ts}"
    );

    let rust = tier_condition(&workflow, "rust");
    assert!(
        rust.starts_with("needs.changes.outputs.rust-changed == 'true' && (needs.js.result == 'success' || needs.js.result == 'skipped') && (needs.ts.result == 'success' || needs.ts.result == 'skipped')"),
        "the rust tier is gated on its own predicate plus js and ts each being green or carried: {rust}"
    );

    let ts_block = job_block(&workflow, "ts");
    assert!(
        ts_block.contains("needs: [changes, js]"),
        "ts structurally depends on js: {ts_block}"
    );
    let rust_block = job_block(&workflow, "rust");
    assert!(
        rust_block.contains("needs: [changes, js, ts]"),
        "rust structurally depends on js and ts: {rust_block}"
    );
}

/// The enforcement order, as a property of the conditions rather than of
/// prose: js red means the ts tier reports skipped-by-js, not its own red,
/// and either red above the rust tier means the same for rust. `always()`
/// would break exactly that -- it lets a job run (and fail) over a red tier
/// above it, which the instruction forbids.
#[test]
fn js_red_reports_skipped_by_js_not_the_tier_s_own_red() {
    let workflow = layered_workflow();
    for job in ["ts", "rust"] {
        let condition = tier_condition(&workflow, job);
        assert!(
            !condition.contains("always()"),
            "the {job} tier must not use always(): a red tier above it must surface as skipped, not as the {job} tier's own red"
        );
        assert!(
            condition.contains("|| needs.js.result == 'skipped'"),
            "the {job} tier admits js as carried (skipped by path), so a js-untouched change still runs it"
        );
    }
    let rust = tier_condition(&workflow, "rust");
    assert!(
        rust.contains("|| needs.ts.result == 'skipped'"),
        "the rust tier admits ts as carried too"
    );
}

/// No tier wakes on another tier's folder: the js job's condition mentions
/// no other tier's output, and the ts job's condition mentions no
/// rust-tier output. A condition that cross-wired the tiers would make a
/// rust-only change re-pay the js checks, which is the cost the cycle
/// exists to remove.
#[test]
fn no_tier_runs_when_only_another_tier_s_folder_changed() {
    let workflow = layered_workflow();
    let js = tier_condition(&workflow, "js");
    assert!(
        !js.contains("ts-changed") && !js.contains("rust-changed"),
        "the js tier must not wake on ts or rust folders: {js}"
    );
    let ts = tier_condition(&workflow, "ts");
    assert!(
        !ts.contains("rust-changed") && !ts.contains("js-changed"),
        "the ts tier is gated on its own predicate (js reaches it through the needs graph, not through its predicate): {ts}"
    );
}

/// The classifier feeds the tiers by name and sees the full push or pull
/// request diff (`fetch-depth: 0`), so a multi-commit push cannot hide a
/// tier's change behind an earlier commit in the same event.
#[test]
fn the_classifier_feeds_the_tiers_from_the_complete_diff() {
    let workflow = layered_workflow();
    let changes = job_block(&workflow, "changes");
    for output in ["js-changed", "ts-changed", "rust-changed"] {
        assert!(
            changes.contains(&format!(
                "{output}: ${{{{ steps.changes.outputs.{output} }}}}"
            )),
            "the changes job exports {output}"
        );
    }
    assert!(
        changes.contains("run: rust-script scripts/detect-code-changes.rs"),
        "the tier predicates are the detector's, not a second list in the workflow"
    );
    assert!(
        changes.contains("fetch-depth: 0"),
        "the classifier needs history to diff the whole event"
    );
}

/// The plan 16 L3 dogfood contract, wired as the rust tier's CI check:
/// delete the generated tree, regenerate it, and let one diff catch byte
/// drift, missing and extra files, and a transactional refusal (a refused
/// regeneration writes nothing, so the deletions stay and the diff is
/// fully red). A hand-edit of `ts/` is exactly the state this refuses.
#[test]
fn the_rust_tier_enforces_the_dogfood_regeneration_in_ci() {
    let workflow = layered_workflow();
    let rust = job_block(&workflow, "rust");
    for pin in [
        "find ts -name '*.ts' -type f -delete",
        "rust/target/release/formal-ai translate --from js --to ts --write",
        "git diff --exit-code -- ts/",
        "./.github/actions/formal-ai-binary",
    ] {
        assert!(
            rust.contains(pin),
            "the rust tier's dogfood gate pins {pin}"
        );
    }
}

/// The shared tier: the checks that span all layers must not be
/// layer-gated. `release.yml`'s pull request trigger carries no `paths:`
/// filter, so every pull request synchronize runs the spanning audits
/// regardless of which tier's folder changed -- a filter added there would
/// let a docs/ or data/ change skip the full pipeline, which is risk 3 in
/// the plan (carry-forward hiding a layer broken by a shared input).
#[test]
fn the_shared_tier_stays_ungated_in_the_pipeline() {
    let release = release_workflow();
    let on_at = release
        .find("\non:\n")
        .expect("the release workflow declares its trigger on its own line");
    let trigger = &release[on_at..release.find("\npermissions:").unwrap()];
    assert!(
        trigger.contains("pull_request:"),
        "the pipeline runs on pull requests"
    );
    assert!(
        !trigger.contains("paths:"),
        "the pipeline is the shared tier: it must run for any layer's change, \
         so its trigger carries no path filter"
    );
}

/// The tier predicates live in the classifier, not as a second list in the
/// workflow: a workflow-level `paths:` filter cannot express job-to-job
/// gating (ts must be skipped-by-js on a js red yet run on a ts-only
/// change), and two copies of the path lists would drift. The classifier's
/// own embedded tests -- which run in this suite through the
/// `detect_code_changes` module include -- hold the tier matrix, so this
/// pins the division of labor: no trigger-level filter in the workflow, the
/// cumulative rule functions in the script.
#[test]
fn the_tier_predicates_live_in_the_classifier_not_the_workflow() {
    let workflow = layered_workflow();
    // Anchored at line starts: a prose word like "instruction:" ends in
    // "on:" and would otherwise slice the header comment into the trigger.
    let on_at = workflow
        .find("\non:\n")
        .expect("the workflow declares its trigger on its own line");
    let trigger = &workflow[on_at..workflow.find("\npermissions:").unwrap()];
    assert!(
        !trigger.contains("paths:"),
        "the layered workflow must not carry a trigger-level path filter; the \
         classifier owns the tier predicates so they are tested in one place"
    );

    let script = fs::read_to_string(format!(
        "{}/../scripts/detect-code-changes.rs",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("the classifier is readable");
    for pin in [
        "let js_tier_input",
        "let ts_tier_input",
        "let rust_tier_input",
        "fn tier_gates_are_cumulative_downward_and_never_upward",
    ] {
        assert!(
            script.contains(pin),
            "the classifier carries the tier rule {pin}"
        );
    }
}
