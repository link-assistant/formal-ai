use std::fs;
use std::path::Path;

#[test]
fn issue_540_dreaming_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #540 Dreaming Memory Maintenance",
            "| R396 ",
            "| R397 ",
            "| R398 ",
            "| R399 ",
            "| R400 ",
            "| R401 ",
            "| R402 ",
            "| R403 ",
            "| R404 ",
            "| R405 ",
            "| R406 ",
            "| R407 ",
            "| R408 ",
            "| R409 ",
            "| R410 ",
            "| R411 ",
            "| R412 ",
            "| R413 ",
            "| R414 ",
            "| R415 ",
            "| R416 ",
            "| R417 ",
            "| R418 ",
            "| R419 ",
            "| R420 ",
            "| R421 ",
            "| R422 ",
            "src/dreaming.rs",
            "formal-ai memory dream",
            "desktop/lib/dreaming.cjs",
            "docs/case-studies/issue-540",
            "MetaAlgorithmAmendment",
            "data/meta/dreaming-recipe.lino",
        ],
    );

    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            "memory dream",
            "20%",
            "free-space reserve",
            "--apply --confirm",
            "real filesystem",
            "passing replay",
        ],
    );

    let architecture = read(root.join("ARCHITECTURE.md"));
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "Dreaming maintenance planner",
            "DreamingDurability",
            "RecomputableCache",
            "requires_bigger_storage",
            "FORMAL_AI_DESKTOP_DREAMING=off",
            "MetaAlgorithmAmendment",
            "ForgetCoveredSpecific",
            "meta_algorithm_amendment",
            "src/dreaming_runtime.rs",
            "src/storage_policy.rs",
            "multilingual cues",
        ],
    );

    let meta_algorithm = read(root.join("docs/meta-algorithm.md"));
    assert_contains_all(
        "docs/meta-algorithm.md",
        &meta_algorithm,
        &[
            "The dreaming meta-algorithm (issue #540)",
            "data/meta/dreaming-recipe.lino",
            "tests/unit/specification/dreaming_meta_algorithm.rs",
            "ForgetCoveredSpecific",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-540/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-540/README.md",
        &case_study,
        &[
            "# Issue 540 Case Study",
            "## 1. Collected Data",
            "## 2. Requirements",
            "## 3. Root Cause",
            "## 4. Implemented Design",
            "## 5. Prior Art And Existing Components",
            "## 6. Verification",
            "R396",
            "R407",
            "R412",
            "R413",
            "R421",
            "R422",
            "DreamingConfig",
            "desktop/lib/dreaming.cjs",
            "MetaAlgorithmAmendment",
            "data/meta/dreaming-recipe.lino",
        ],
    );

    let issue_requirements = read(root.join("docs/case-studies/issue-540/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-540/requirements.md",
        &issue_requirements,
        &[
            "R540-01",
            "R540-13",
            "R540-18",
            "R540-19",
            "R540-27",
            "R540-28",
            "FORMAL_AI_DESKTOP_DREAMING=off",
            "requires_bigger_storage",
            "MetaAlgorithmAmendment",
        ],
    );

    let solution_plans = read(root.join("docs/case-studies/issue-540/solution-plans.md"));
    assert_contains_all(
        "docs/case-studies/issue-540/solution-plans.md",
        &solution_plans,
        &[
            "Pure Planner First",
            "Explicit Apply",
            "Core And Desktop Background Scheduler",
            "Verified Learning And Application",
            "Real Storage And Persisted Consent",
            "Existing Component And Library Survey",
        ],
    );

    // Repo-wide terminology sweep (issue #540 §7): every source and document
    // must use memory-links terminology, not "memory graph". The sweep walks
    // the trees so a new file cannot reintroduce the phrase unnoticed.
    for tree in ["src", "tests", "docs"] {
        for entry in walkdir::WalkDir::new(root.join(tree)) {
            let entry = entry.expect("tree should be walkable");
            let path = entry.path();
            let sweepable = path
                .extension()
                .is_some_and(|ext| ext == "rs" || ext == "md" || ext == "lino");
            if !sweepable || path.ends_with("docs_requirements_issue_540.rs") {
                continue;
            }
            assert!(
                !read(path).to_lowercase().contains("memory graph"),
                "{} must use memory-links terminology",
                path.display(),
            );
        }
    }

    let research = read(root.join("docs/case-studies/issue-540/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-540/raw-data/online-research.md",
        &research,
        &[
            "RocksDB",
            "PostgreSQL",
            "requestIdleCallback",
            "Redis",
            "Library Survey And Selection",
            "fs2",
            "sysinfo::Disk",
            "redb::Database::compact",
            "src/memory.rs",
        ],
    );

    for relative in [
        "docs/case-studies/issue-540/raw-data/issue-540.json",
        "docs/case-studies/issue-540/raw-data/issue-540-comments.json",
        "docs/case-studies/issue-540/raw-data/issue-494.json",
        "docs/case-studies/issue-540/raw-data/issue-494-comments.json",
        "docs/case-studies/issue-540/raw-data/pr-645.json",
        "docs/case-studies/issue-540/raw-data/pr-645-conversation-comments.json",
        "docs/case-studies/issue-540/raw-data/pr-645-review-comments.json",
        "docs/case-studies/issue-540/raw-data/pr-645-reviews.json",
        "docs/case-studies/issue-540/raw-data/pr-645-amendment-2026-07-10.md",
        "docs/case-studies/issue-540/raw-data/recent-ci-runs.json",
        "docs/case-studies/issue-540/raw-data/recent-merged-related-prs.json",
        "docs/case-studies/issue-540/raw-data/code-search-memory.txt",
        "docs/case-studies/issue-540/raw-data/online-research.md",
        "changelog.d/20260708_223000_issue_540_dreaming.md",
        "changelog.d/20260709_090000_issue_540_dreaming_generalization.md",
        "data/meta/dreaming-recipe.lino",
        "docs/case-studies/issue-540/dreaming-gap-analysis.lino",
        "docs/case-studies/issue-540/agent-cli-session-dreaming-audit.json",
        "tests/unit/issue_540_agent_cli.rs",
        "tests/unit/specification/dreaming_meta_algorithm.rs",
    ] {
        let path = root.join(relative);
        assert!(
            path.is_file(),
            "{relative} should exist for issue #540 traceability",
        );
        // A zero-byte placeholder is not traceability evidence (issue #540 §7).
        assert!(
            path.metadata().map_or(0, |meta| meta.len()) > 0,
            "{relative} must not be empty for issue #540 traceability",
        );
    }

    // Every backticked snake_case identifier cited in the requirements table
    // must still exist in the live Rust sources: citing a renamed or deleted
    // test/function fails here instead of silently going stale (issue #540 §7).
    let live_sources = read_rust_sources(&[root.join("src"), root.join("tests")]);
    for identifier in cited_identifiers(&issue_requirements) {
        assert!(
            live_sources.contains(&identifier),
            "requirements.md cites `{identifier}` but it no longer appears in src/ or tests/",
        );
    }
}

/// Backticked tokens that look like plain `snake_case` identifiers (test or
/// function names). Paths, env vars, CLI flags, and type paths are skipped.
fn cited_identifiers(content: &str) -> Vec<String> {
    content
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|token| {
            token.contains('_')
                && token.chars().all(|character| {
                    character.is_ascii_lowercase() || character == '_' || character.is_ascii_digit()
                })
        })
        .map(str::to_owned)
        .collect()
}

fn read_rust_sources(roots: &[std::path::PathBuf]) -> String {
    let mut combined = String::new();
    for root in roots {
        for entry in walkdir::WalkDir::new(root) {
            let entry = entry.expect("source tree should be walkable");
            if entry.path().extension().is_some_and(|ext| ext == "rs") {
                combined.push_str(&read(entry.path()));
            }
        }
    }
    combined
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.as_ref().display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}",
        );
    }
}
