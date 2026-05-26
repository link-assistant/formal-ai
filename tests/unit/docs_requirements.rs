use std::fs;
use std::path::Path;

use walkdir::{DirEntry, WalkDir};

#[test]
fn issue_12_vision_documents_are_present_and_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let vision = read(root.join("VISION.md"));
    assert_contains_all(
        "VISION.md",
        &vision,
        &[
            "# Vision",
            "associative operational space",
            "Links Data Store",
            "Add-only history",
            "dynamic type system",
        ],
    );

    let goals = read(root.join("GOALS.md"));
    assert_contains_all(
        "GOALS.md",
        &goals,
        &[
            "# Goals",
            "smallest useful seed dataset",
            "transparent reasoning",
            "chat-first",
            "isolated execution",
        ],
    );

    let non_goals = read(root.join("NON-GOALS.md"));
    assert_contains_all(
        "NON-GOALS.md",
        &non_goals,
        &[
            "# Non-Goals",
            "memoized answer cache",
            "GPU-required neural inference",
            "Hidden autonomous actions",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-12/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-12/README.md",
        &case_study,
        &[
            "# Issue 12 Case Study",
            "## Collected Data",
            "## Holistic Requirements",
            "## Solution Plan",
            "issue #1",
            "issue #4",
            "issue #6",
            "issue #8",
            "issue #10",
        ],
    );
}

#[test]
fn issue_16_followup_documents_capture_universal_seed_and_memory_migration() {
    // Pin the documentation surface that frames R105-R108 so the
    // requirement matrix, the architectural narrative, and the case study
    // cannot silently drift apart.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "| R105 ",
            "| R106 ",
            "| R107 ",
            "| R108 ",
            "src/web/seed/",
            "environments.lino",
            "formal-ai memory",
            "formal-ai bundle",
            "formal_ai_bundle",
        ],
    );

    let vision = read(root.join("VISION.md"));
    assert_contains_all(
        "VISION.md",
        &vision,
        &[
            "Self-Aware Environments",
            "Library-First Availability",
            "environments.lino",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-16/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-16/README.md",
        &case_study,
        &[
            "Self-Aware Environments and Cross-Surface Memory Migration",
            "environments.lino",
            "formal-ai environments",
            "demo_memory",
            "formal_ai_bundle",
        ],
    );
}

#[test]
fn issue_103_test_matrix_and_architecture_documents_are_present_and_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let architecture = read(root.join("ARCHITECTURE.md"));
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "# Architecture",
            "Links Notation",
            "Wikidata",
            "P-id",
            "Q-id",
            "temperature",
            "doublets-rs",
            "doublets-web",
            "Universal Problem Solver",
            "Transformation and Substitution Rules",
            "formal_ai_bundle",
        ],
    );

    let vision = read(root.join("VISION.md"));
    assert_contains_all(
        "VISION.md",
        &vision,
        &[
            "Formalization And Temperature",
            "Wikidata",
            "temperature",
            "doublets-rs",
            "ARCHITECTURE.md",
        ],
    );

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #103 Test-Matrix",
            "| R129 ",
            "| R130 ",
            "| R131 ",
            "| R132 ",
            "| R133 ",
            "| R134 ",
            "| R135 ",
            "| R136 ",
            "prompt_variations.rs",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-103/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-103/README.md",
        &case_study,
        &[
            "# Issue 103 Case Study",
            "## Collected Data",
            "## Requirements",
            "competitor-test-research.md",
            "ARCHITECTURE.md",
            "prompt_variations.rs",
        ],
    );
}

#[test]
fn issue_117_lino_i18n_catalog_documents_and_ci_rule_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #117 Lino I18n Catalog Requirements",
            "| R137 ",
            "| R138 ",
            "| R139 ",
            "| R140 ",
            "| R141 ",
            "| R142 ",
            "src/web/i18n-catalog.lino",
            "npm run --prefix tests/e2e check:i18n",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-117/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-117/README.md",
        &case_study,
        &[
            "# Issue 117 Case Study",
            "lino-i18n@0.1.1",
            "parseLinoCatalogs",
            "createI18n",
            "CI Rule",
            "104 keys",
        ],
    );

    let catalog = read(root.join("src/web/i18n-catalog.lino")).replace("\r\n", "\n");
    assert_contains_all(
        "src/web/i18n-catalog.lino",
        &catalog,
        &[
            "en\n  buttons",
            "ru\n  buttons",
            "zh\n  buttons",
            "hi\n  buttons",
            "\"\"\"",
            "label \"Language\"",
        ],
    );

    let workflow = read(root.join(".github/workflows/release.yml"));
    assert_contains_all(
        ".github/workflows/release.yml",
        &workflow,
        &[
            "Check i18n catalog coverage",
            "npm run --prefix tests/e2e check:i18n",
        ],
    );
}

#[test]
fn issue_115_github_log_collection_documents_are_present_and_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #115 GitHub Evidence Collection",
            "| R143 ",
            "| R144 ",
            "| R145 ",
            "| R146 ",
            "| R147 ",
            "| R148 ",
            "| R149 ",
            "formal-ai github-logs",
            "scripts/mine-hive-mind-dataset.rs",
        ],
    );

    let architecture = read(root.join("ARCHITECTURE.md"));
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "GitHub Evidence Collection",
            "src/github_logs.rs",
            "scripts/mine-hive-mind-dataset.rs",
            "manifest.json",
            "link-assistant/hive-mind",
            "not registered",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-115/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-115/README.md",
        &case_study,
        &[
            "# Issue 115 Case Study",
            "## Collected Data",
            "github-logs",
            "scripts/mine-hive-mind-dataset.rs",
            "hive-mind",
            "R143",
            "R149",
        ],
    );

    let tools = read(root.join("data/seed/tools.lino"));
    assert!(
        !tools.contains("tool_github_logs"),
        "GitHub dataset mining must stay an operator script/command, not a seed agent tool"
    );
}

#[test]
fn issue_63_definition_fusion_requirements_and_examples_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #63 Cross-Language Definition Fusion Requirements",
            "| R150 ",
            "| R151 ",
            "| R152 ",
            "| R153 ",
            "| R154 ",
            "| R155 ",
            "10-20 self-explanatory examples",
            "FORMAL_AI_DEFINITION_FUSION",
            "--definition-fusion",
            "tests/unit/specification/definition_fusion.rs",
        ],
    );

    let tests = read(root.join("tests/unit/specification/definition_fusion.rs"));
    assert_contains_all(
        "tests/unit/specification/definition_fusion.rs",
        &tests,
        &[
            "review requested 10-20 concrete examples",
            "Merge Wikipedia definitions of IIR",
            "Merge Wikipedia definitions of color",
            "Merge Wikipedia definitions of KISS principle",
            "Merge definitions of Telegram Ads",
            "Merge Wikipedia definitions of not-a-seeded-concept",
            "What is IIR?",
        ],
    );
}

#[test]
fn issue_80_software_project_dialogue_requirements_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #80 Software Project Request Requirements",
            "| R156 ",
            "| R157 ",
            "| R158 ",
            "| R159 ",
            "| R160 ",
            "| R161 ",
            "| R162 ",
            "| R163 ",
            "| R164 ",
            "at least 20 full dialogue examples",
            "Requirement model",
            "delivery_mode",
            "approval_gate",
            "software_project_dialogue_examples_formalize_plan_then_implement_after_approval",
        ],
    );

    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            "software artifact requests",
            "Links Notation meaning record",
            "requirement graph",
            "approval gates",
            "language-aware starter domain code",
            "after the user approves the plan",
        ],
    );

    let changelog =
        read(root.join("changelog.d/20260518_224500_issue_80_software_project_requests.md"));
    assert_contains_all(
        "changelog.d/20260518_224500_issue_80_software_project_requests.md",
        &changelog,
        &[
            "software_project_plan",
            "Links Notation meaning record",
            "requirement graph",
            "approval gates",
            "language-aware starter code after the user approves",
        ],
    );
}

#[test]
fn issue_207_natural_translation_documents_are_present_and_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #207 Natural Translation Pipeline",
            "| R213 ",
            "| R214 ",
            "| R215 ",
            "match_source_formatting",
            "TranslationPipeline",
            "src/translation/pipeline.rs",
            "src/translation/formatting.rs",
        ],
    );

    let architecture = read(root.join("ARCHITECTURE.md"));
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "Formalize → Meaning → Deformalize Pipeline",
            "match_source_formatting",
            "src/translation/pipeline.rs",
            "Resolution Order and Browser Fallback",
        ],
    );

    let vision = read(root.join("VISION.md"));
    assert_contains_all(
        "VISION.md",
        &vision,
        &[
            "preserve the source's surface signal",
            "match_source_formatting",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-207/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-207/README.md",
        &case_study,
        &[
            "# Issue 207 Case Study",
            "## Root Causes",
            "## Requirement Traceability",
            "## Fixes",
            "## Verification Plan",
            "formalize → meaning → deformalize",
        ],
    );
}

#[test]
fn issue_195_dind_telegram_runtime_documents_are_present_and_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #195 Docker-in-Docker Telegram Runtime",
            "| R220 ",
            "| R221 ",
            "| R222 ",
            "| R223 ",
            "| R224 ",
            "| R225 ",
            "konard/box-dind:2.1.1",
            "FORMAL_AI_START_RUNNER",
        ],
    );

    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            "Docker-in-Docker Telegram bot image",
            "konard/box-dind:2.1.1",
            "TELEGRAM_BOT_TOKEN",
            "--runtime=sysbox-runc",
            "Do not bind-mount the host",
            "$ --isolated docker --auto-remove-docker-container --",
            "verify-formal-ai-dind",
        ],
    );

    let architecture = read(root.join("ARCHITECTURE.md"));
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "Docker-in-Docker Telegram image",
            "konard/box-dind:2.1.1",
            "formal-ai telegram --mode polling",
            "/tmp/start-command/logs/",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-195/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-195/README.md",
        &case_study,
        &[
            "# Issue 195 Case Study",
            "## Collected Data",
            "## Online Facts",
            "## Root Causes",
            "## Verification Plan",
            "repro-before-docker-runtime.txt",
        ],
    );

    let research = read(root.join("docs/case-studies/issue-195/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-195/raw-data/online-research.md",
        &research,
        &[
            "https://github.com/link-foundation/box",
            "https://github.com/link-foundation/start",
            "konard/box-dind:2.1.1",
            "--isolated docker",
        ],
    );
}

#[test]
fn issue_278_default_native_doublets_store_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #278 Native Doublets Store Default Requirements",
            "| R231 ",
            "| R232 ",
            "| R233 ",
            "| R234 ",
            "| R235 ",
            "| R236 ",
            "doublets-rs as the native default",
        ],
    );

    let cargo = read(root.join("Cargo.toml"));
    assert_contains_all(
        "Cargo.toml",
        &cargo,
        &["default = [\"doublets-native\"]", "dep:doublets", "dep:mem"],
    );

    let link_store = read(root.join("src/link_store.rs"));
    assert_contains_all(
        "src/link_store.rs",
        &link_store,
        &[
            "default_native_link_store",
            "DefaultNativeLinkStore",
            "from_links_notation",
            "native_default_build_selects_doublets_rs_backend",
            "native_without_default_features_falls_back_to_lino_projection",
            "doublets_default_imports_full_lino_bundle_and_exports_deterministically",
        ],
    );

    let architecture = read(root.join("ARCHITECTURE.md"));
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "Default native doublets-rs",
            "--no-default-features",
            "formal_ai_bundle",
            "indexeddb-lino-mirror",
        ],
    );

    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            "Native Rust builds now select `doublets-rs`",
            "`doublets-native` feature",
            "`--no-default-features`",
        ],
    );

    let environments = read(root.join("data/seed/environments.lino"));
    assert_contains_all(
        "data/seed/environments.lino",
        &environments,
        &[
            "default native doublets-rs link store",
            "default_native_link_store()?.import_memory_links_notation",
        ],
    );
}

#[test]
fn repository_text_avoids_deferred_labels_requested_by_issue_103() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let phrase_space = ["proof", " of ", "concept"].concat();
    let phrase_hyphen = ["proof", "-of-", "concept"].concat();
    let compact_labels = [["m", "vp"].concat(), ["p", "oc"].concat()];
    let mut findings = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|entry| !is_skipped_tree(root, entry))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        let relative = relative_path(root, path);
        let lower_path = relative.to_lowercase();
        collect_for_haystack(
            &relative,
            "<path>",
            &lower_path,
            [&phrase_space, &phrase_hyphen],
            &compact_labels,
            &mut findings,
        );

        let bytes =
            fs::read(path).unwrap_or_else(|error| panic!("{relative} should be readable: {error}"));
        let Ok(content) = String::from_utf8(bytes) else {
            continue;
        };
        collect_for_haystack(
            &relative,
            "<content>",
            &content.to_lowercase(),
            [&phrase_space, &phrase_hyphen],
            &compact_labels,
            &mut findings,
        );
    }

    assert!(
        findings.is_empty(),
        "repository should not contain deferred implementation labels requested for removal:\n{}",
        findings.join("\n")
    );
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.as_ref().display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}"
        );
    }
}

fn is_skipped_tree(root: &Path, entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    if matches!(name.as_ref(), ".git" | "target" | "node_modules") {
        return true;
    }

    matches!(
        relative_path(root, entry.path()).as_str(),
        "ci-logs"
            | "logs"
            | "tests/e2e/playwright-report"
            | "tests/e2e/test-results"
            | "data/wikidata-cache"
            | "data/wiktionary-cache"
            | "data/http-cache"
            | "data/seed/api-cache"
    )
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn collect_for_haystack<'a>(
    relative: &str,
    source: &str,
    haystack: &str,
    phrase_labels: impl IntoIterator<Item = &'a String>,
    compact_labels: &[String],
    findings: &mut Vec<String>,
) {
    for label in phrase_labels {
        if haystack.contains(label) {
            findings.push(format!("{relative} {source}: {label}"));
        }
    }

    for label in compact_labels {
        if contains_compact_label(haystack, label) {
            findings.push(format!("{relative} {source}: {label}"));
        }
    }
}

fn contains_compact_label(haystack: &str, label: &str) -> bool {
    haystack
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|part| part == label)
}
