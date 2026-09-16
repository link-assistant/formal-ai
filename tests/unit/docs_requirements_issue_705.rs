//! Issue #705 documentation and evidence traceability.

use std::fs;
use std::path::Path;

use formal_ai::sha256_hex;

const CASE: &str = "docs/case-studies/issue-705";
const SESSION: &str = "ses_04262d1beffeStZQyvlKMA8qCg";
const ARTIFACT_SHA256: &str = "a7d58aa0003dd965f4fc53d3e02f5fbeb93fece526c3803e000bb867ebdedb6f";

#[test]
fn issue_705_requirements_and_grounded_recipe_are_traceable() {
    let root = root();
    let requirements = read(root, "REQUIREMENTS.md");
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #705 Anticipatory Dreaming",
            "| R705-1 ",
            "| R705-2 ",
            "| R705-3 ",
            "| R705-4 ",
            "| R705-5 ",
            "| R705-6 ",
            "| R705-7 ",
            "| R705-8 ",
            "docs/case-studies/issue-705",
            "tests/unit/issue_705_anticipation.rs",
        ],
    );

    let recipe = read(root, "data/meta/dreaming-recipe.lino");
    assert_contains_all(
        "data/meta/dreaming-recipe.lino",
        &recipe,
        &[
            "order \"14\"",
            "id \"predict_request_classes\"",
            "order \"15\"",
            "id \"expand_and_probe_predictions\"",
            "order \"16\"",
            "id \"prelearn_consented_sources\"",
            "order \"17\"",
            "id \"record_anticipation_outcomes\"",
            "suite \"issue_705_anticipation\"",
        ],
    );

    let architecture = read(root, "ARCHITECTURE.md");
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "src/anticipation.rs",
            "prediction_hit",
            "seventeen-stage recipe",
        ],
    );
    let roadmap = read(root, "ROADMAP.md");
    assert_contains_all(
        "ROADMAP.md",
        &roadmap,
        &[
            "Anticipatory learning",
            "Done for #705",
            "honest later-hit ledger",
        ],
    );
}

#[test]
fn issue_705_case_study_preserves_inputs_design_and_verification() {
    let root = root();
    let readme = read(root, &format!("{CASE}/README.md"));
    assert_contains_all(
        "issue-705 README",
        &readme,
        &[
            "# Issue 705 Case Study",
            "## Before and after",
            "## Acceptance evidence",
            "## Runtime and safety boundaries",
            "frobulator705",
            "0 hits and 0 basis points",
            "proposal-only",
        ],
    );
    let design = read(root, &format!("{CASE}/transition-design.md"));
    assert_contains_all(
        "issue-705 transition design",
        &design,
        &[
            "A Markov state is an `IntentClass`",
            "Every variant is evaluated",
            "exact normalized alias",
            "absence of evidence is never converted into a positive score",
        ],
    );
    let research = read(root, &format!("{CASE}/raw-data/online-research.md"));
    assert_contains_all(
        "issue-705 research",
        &research,
        &["USENIX 1994", "USENIX 1997", "USENIX 2001", "USITS 1999"],
    );

    for relative in [
        "raw-data/issue-705.json",
        "raw-data/issue-705-comments.json",
        "raw-data/pr-887.json",
        "raw-data/pr-887-review-comments.json",
        "raw-data/pr-887-conversation-comments.json",
        "raw-data/pr-887-reviews.json",
        "raw-data/initial-ci-runs.json",
        "self-hosting-authorship/agent-cli.log",
        "self-hosting-authorship/formal-ai.log",
        "self-hosting-authorship/decomposition.lino",
        "self-hosting-authorship/failed-literal-wording/agent-cli.log",
        "self-hosting-authorship/failed-literal-wording/formal-ai.log",
    ] {
        assert!(
            root.join(CASE).join(relative).is_file(),
            "{relative} should remain in the issue #705 review trail",
        );
    }
}

#[test]
fn same_task_agent_cli_artifact_is_byte_pinned_and_honestly_counted() {
    let root = root();
    let artifact_path = root
        .join(CASE)
        .join("self-hosting-authorship/anticipation-invariant.lino");
    let artifact = fs::read(&artifact_path)
        .unwrap_or_else(|error| panic!("{}: {error}", artifact_path.display()));
    assert_eq!(artifact.len(), 540);
    assert_eq!(sha256_hex(&artifact), ARTIFACT_SHA256);
    assert!(artifact.starts_with(b"anticipation_contract\n"));
    assert!(
        !artifact.ends_with(b"\n"),
        "the exact authored bytes are pinned"
    );

    let agent_log = read(
        root,
        &format!("{CASE}/self-hosting-authorship/agent-cli.log"),
    );
    assert!(agent_log.contains(SESSION));
    let server_log = read(
        root,
        &format!("{CASE}/self-hosting-authorship/formal-ai.log"),
    );
    for transition in [
        "planned ToolCalls",
        "tool=write",
        "planned Final",
        "anticipation-invariant.lino",
    ] {
        assert!(
            server_log.contains(transition),
            "server trace lacks {transition}"
        );
    }

    let decomposition = read(
        root,
        &format!("{CASE}/self-hosting-authorship/decomposition.lino"),
    );
    assert_eq!(decomposition.matches("issue_705_smallest_leaf_").count(), 5);
    assert_eq!(
        decomposition
            .matches("authorship \"formal_ai_agent_cli\"")
            .count(),
        1
    );
    assert!(decomposition.contains(&format!("session \"{SESSION}\"")));
    assert!(decomposition.contains("formal_ai_authored_percent \"20\""));

    let failed = fs::read(
        root.join(CASE)
            .join("self-hosting-authorship/failed-literal-wording/anticipation-invariant.lino"),
    )
    .expect("retained first attempt");
    assert!(failed.starts_with(b"exactly\nanticipation_contract\n"));
}

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn read(root: &Path, relative: &str) -> String {
    let path = root.join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(content.contains(needle), "{label} should contain: {needle}");
    }
}
