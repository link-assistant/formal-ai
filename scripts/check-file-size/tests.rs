use super::*;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("check-file-size-{name}-{nanos}"));
    fs::create_dir_all(&path).unwrap();
    path
}

fn write_rust_file_with_lines(path: &Path, line_count: usize) {
    write_file_with_lines(path, line_count);
}

fn write_lino_file_with_lines(path: &Path, line_count: usize) {
    write_file_with_lines(path, line_count);
}

fn write_js_file_with_lines(path: &Path, line_count: usize) {
    write_file_with_lines(path, line_count);
}

fn write_file_with_lines(path: &Path, line_count: usize) {
    let mut content = String::new();
    for line in 1..=line_count {
        writeln!(&mut content, "// line {line}").unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn classifies_warning_band_without_blocking() {
    let rust_limit = &FILE_LIMITS[0];
    assert_eq!(
        classify_line_count(rust_limit.warn_lines, rust_limit),
        LineStatus::WithinLimit
    );
    assert_eq!(
        classify_line_count(rust_limit.warn_lines + 1, rust_limit),
        LineStatus::Warning
    );
    assert_eq!(
        classify_line_count(rust_limit.max_lines, rust_limit),
        LineStatus::Warning
    );
}

#[test]
fn maintained_text_is_measured_and_captures_are_not() {
    assert_eq!(
        file_limit(Path::new("js/app/main.jsx")),
        Some(&MAINTAINED_LIMIT)
    );
    assert_eq!(
        file_limit(Path::new("CHANGELOG.md")),
        Some(&MAINTAINED_LIMIT)
    );
    assert_eq!(
        file_limit(Path::new("js/styles.css")),
        Some(&MAINTAINED_LIMIT)
    );
    assert_eq!(file_limit(Path::new("vscode/package-lock.json")), None);
    assert_eq!(file_limit(Path::new("js/vendor.bundle.js")), None);
    assert_eq!(
        file_limit(Path::new("data/cache/wikidata/lexeme/L3302.json")),
        None
    );
    assert_eq!(
        file_limit(Path::new(
            "rust/tests/fixtures/coding-discovery/python-docs/stdtypes.html"
        )),
        None
    );
}

#[test]
fn classifies_hard_limit_violations() {
    let rust_limit = &FILE_LIMITS[0];
    assert_eq!(
        classify_line_count(rust_limit.max_lines + 1, rust_limit),
        LineStatus::Violation
    );
}

#[test]
fn check_directory_reports_warning_and_violation_separately() {
    let repo = temp_dir("thresholds");
    let src_dir = repo.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let rust_limit = FILE_LIMITS[0];
    write_rust_file_with_lines(&src_dir.join("near_limit.rs"), rust_limit.warn_lines + 1);
    write_rust_file_with_lines(&src_dir.join("over_limit.rs"), rust_limit.max_lines + 1);
    write_rust_file_with_lines(&src_dir.join("small.rs"), rust_limit.warn_lines);

    let result = check_directory(&repo);

    assert_eq!(
        result.warnings,
        vec![Finding {
            file: "src/near_limit.rs".to_string(),
            lines: rust_limit.warn_lines + 1,
            max_lines: rust_limit.max_lines,
            warn_lines: rust_limit.warn_lines,
            label: rust_limit.label,
        }]
    );
    assert_eq!(
        result.violations,
        vec![Finding {
            file: "src/over_limit.rs".to_string(),
            lines: rust_limit.max_lines + 1,
            max_lines: rust_limit.max_lines,
            warn_lines: rust_limit.warn_lines,
            label: rust_limit.label,
        }]
    );
}

#[test]
fn warning_annotations_are_limited_to_growing_files() {
    let warnings = vec![
        Finding {
            file: "src/unchanged.rs".to_string(),
            lines: 910,
            max_lines: 1_000,
            warn_lines: 900,
            label: "Rust",
        },
        Finding {
            file: "src/changed.rs".to_string(),
            lines: 920,
            max_lines: 1_000,
            warn_lines: 900,
            label: "Rust",
        },
    ];

    assert_eq!(
        warnings_for_growing_paths(&warnings, &["src/changed.rs".to_string()]),
        vec![&warnings[1]]
    );
}

#[test]
fn numstat_selects_only_files_with_net_line_growth() {
    let numstat = "12\t0\tsrc/growing.rs\n3\t3\tsrc/unchanged.rs\n1\t8\tsrc/shrinking.rs\n-\t-\tassets/binary.png\n";

    assert_eq!(
        growing_paths_from_numstat(numstat),
        vec!["src/growing.rs".to_string()]
    );
}

#[test]
fn check_directory_enforces_lino_limit() {
    let repo = temp_dir("lino-thresholds");
    let data_dir = repo.join("data");
    fs::create_dir_all(&data_dir).unwrap();
    let lino_limit = FILE_LIMITS[1];
    write_lino_file_with_lines(&data_dir.join("oversized.lino"), lino_limit.max_lines + 1);

    let result = check_directory(&repo);

    assert_eq!(
        result.violations,
        vec![Finding {
            file: "data/oversized.lino".to_string(),
            lines: lino_limit.max_lines + 1,
            max_lines: lino_limit.max_lines,
            warn_lines: lino_limit.warn_lines,
            label: lino_limit.label,
        }]
    );
}

#[test]
fn check_directory_enforces_split_worker_js_limit() {
    let repo = temp_dir("worker-js-thresholds");
    let worker_dir = repo.join("js/worker");
    fs::create_dir_all(&worker_dir).unwrap();
    write_js_file_with_lines(
        &worker_dir.join("formal_ai_worker_seed_responses_and_language.js"),
        WORKER_JS_LIMIT.max_lines + 1,
    );

    let result = check_directory(&repo);

    assert_eq!(
        result.violations,
        vec![Finding {
            file: "js/worker/formal_ai_worker_seed_responses_and_language.js".to_string(),
            lines: WORKER_JS_LIMIT.max_lines + 1,
            max_lines: WORKER_JS_LIMIT.max_lines,
            warn_lines: WORKER_JS_LIMIT.warn_lines,
            label: WORKER_JS_LIMIT.label,
        }]
    );
}

#[test]
fn check_directory_applies_the_maintained_limit_to_app_js() {
    let repo = temp_dir("app-js-thresholds");
    let app_dir = repo.join("js/app");
    fs::create_dir_all(&app_dir).unwrap();
    write_js_file_with_lines(&app_dir.join("main.js"), MAINTAINED_LIMIT.max_lines + 1);

    let result = check_directory(&repo);

    assert_eq!(
        result.violations,
        vec![Finding {
            file: "js/app/main.js".to_string(),
            lines: MAINTAINED_LIMIT.max_lines + 1,
            max_lines: MAINTAINED_LIMIT.max_lines,
            warn_lines: MAINTAINED_LIMIT.warn_lines,
            label: MAINTAINED_LIMIT.label,
        }]
    );
}

#[test]
fn check_directory_rejects_embedded_worker_lino_data() {
    let repo = temp_dir("embedded-worker-lino");
    let worker_dir = repo.join("js/worker");
    fs::create_dir_all(&worker_dir).unwrap();
    fs::write(
        worker_dir.join("formal_ai_worker_seed_responses_and_language.js"),
        "const MEANINGS_LINO = [\n  \"meaning fact\",\n].join(\"\\n\");\n",
    )
    .unwrap();

    let result = check_directory(&repo);

    assert_eq!(
        result.embedded_data_violations,
        vec![EmbeddedDataFinding {
            file: "js/worker/formal_ai_worker_seed_responses_and_language.js".to_string(),
            line: 1,
            message: "Worker JavaScript must load Links Notation data from data/seed via seed_loader.js, not embed _LINO arrays or template literals.".to_string(),
        }]
    );
}

/// Issue #812: `.github/workflows/**` matched the `.git` exclusion as a
/// substring, so every workflow was invisible to this gate and
/// `release.yml` grew to 1824 lines with CI green.
#[test]
fn check_directory_measures_github_workflows() {
    let repo = temp_dir("github-workflows");
    let workflows = repo.join(".github/workflows");
    fs::create_dir_all(&workflows).unwrap();
    write_file_with_lines(
        &workflows.join("release.yml"),
        WORKFLOW_YAML_LIMIT.max_lines + 1,
    );

    let result = check_directory(&repo);

    assert_eq!(
        result.violations,
        vec![Finding {
            file: ".github/workflows/release.yml".to_string(),
            lines: WORKFLOW_YAML_LIMIT.max_lines + 1,
            max_lines: WORKFLOW_YAML_LIMIT.max_lines,
            warn_lines: WORKFLOW_YAML_LIMIT.warn_lines,
            label: WORKFLOW_YAML_LIMIT.label,
        }]
    );
}

/// Case studies quote other projects' pipelines verbatim as evidence;
/// trimming them to our ceiling would destroy what they document.
#[test]
fn check_directory_does_not_measure_quoted_case_study_workflows() {
    let repo = temp_dir("case-study-workflows");
    let case_study = repo.join("docs/case-studies/issue-561/template-comparison/js");
    fs::create_dir_all(&case_study).unwrap();
    write_file_with_lines(
        &case_study.join("release.yml"),
        WORKFLOW_YAML_LIMIT.max_lines + 1,
    );

    let result = check_directory(&repo);

    assert_eq!(result.violations, Vec::new());
    assert_eq!(result.warnings, Vec::new());
}

/// Issue #960 (R222-1): "each .lino file cannot be larger than 1500 lines"
/// applies to the generated Wikidata cache too — that is where the
/// generator writes, so that is where the cap has to bite. Oversized
/// responses are split into `<bucket>-partN.lino` instead of exempted.
#[test]
fn check_directory_enforces_lino_limit_inside_wikidata_cache() {
    let repo = temp_dir("wikidata-cache");
    let cache_dir = repo.join("data/cache/wikidata/entity");
    fs::create_dir_all(&cache_dir).unwrap();
    let lino_limit = FILE_LIMITS[1];
    write_lino_file_with_lines(&cache_dir.join("Q1860.lino"), lino_limit.max_lines + 1);

    let result = check_directory(&repo);

    assert_eq!(
        result.violations,
        vec![Finding {
            file: "data/cache/wikidata/entity/Q1860.lino".to_string(),
            lines: lino_limit.max_lines + 1,
            max_lines: lino_limit.max_lines,
            warn_lines: lino_limit.warn_lines,
            label: lino_limit.label,
        }]
    );
    assert_eq!(result.embedded_data_violations, Vec::new());
}

#[test]
fn check_directory_skips_preserved_development_evidence() {
    let repo = temp_dir("development-evidence");
    let evidence_dir = repo.join("dev/log/issues/798/templates");
    fs::create_dir_all(&evidence_dir).unwrap();
    let rust_limit = FILE_LIMITS[0];
    write_rust_file_with_lines(
        &evidence_dir.join("version-and-commit.rs"),
        rust_limit.max_lines + 1,
    );

    let result = check_directory(&repo);

    assert_eq!(result.violations, Vec::new());
    assert_eq!(result.warnings, Vec::new());
}

/// PR #1188: a session transcript git ignores sat at the repository root
/// and failed the gate locally although no commit could ever carry it.
#[test]
fn check_listed_files_measures_only_the_listed_files() {
    let repo = temp_dir("listed-files");
    let rust_limit = FILE_LIMITS[0];
    write_rust_file_with_lines(&repo.join("kept.rs"), rust_limit.max_lines + 1);
    write_rust_file_with_lines(&repo.join("ignored.rs"), rust_limit.max_lines + 1);
    let listed = BTreeSet::from(["kept.rs".to_string()]);

    let result = check_listed_files(&repo, Some(&listed));

    assert_eq!(
        result
            .violations
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        ["kept.rs"]
    );
}

#[test]
fn warning_annotation_uses_github_actions_format() {
    let rust_limit = FILE_LIMITS[0];
    let finding = Finding {
        file: "src/near_limit.rs".to_string(),
        lines: rust_limit.warn_lines + 1,
        max_lines: rust_limit.max_lines,
        warn_lines: rust_limit.warn_lines,
        label: rust_limit.label,
    };

    assert_eq!(
        warning_annotation(&finding),
        "::warning file=src/near_limit.rs::Rust file has 901 lines (approaching limit of 1000). Consider extracting content to keep at or below 900 lines and prevent review and merge conflicts."
    );
}
