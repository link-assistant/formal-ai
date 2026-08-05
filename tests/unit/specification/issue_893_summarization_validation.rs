//! Specification tests for issue #893 — iterative random-file summarization
//! validation and the 80% quality ratchet.
//!
//! One test per acceptance criterion in the issue, plus a whole-task test that
//! exercises the four together the way the issue asks for them:
//!
//! - a. a reproducible seeded sampling protocol over repository files,
//! - b. two files validated per iteration until the result stabilizes or a
//!   reported bound is reached,
//! - c. a published quality metric with an 80 percent minimum ratchet,
//! - d. recursive Markdown embedded grammars exercised through the *production*
//!   summarizer.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::summarization::validation::{
    evaluate_file, ratchet_violations, validate_repository_summarization, CorpusFile,
    QualityBaseline, QualityScore, SamplingProtocol, BASELINE_PATH, CRITERIA,
    DEFAULT_FILES_PER_ITERATION, DEFAULT_SAMPLING_SEED, DEFAULT_STABILITY_WINDOW,
    QUALITY_RATCHET_PERCENT,
};
use formal_ai::SummarizationConfig;

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// A small synthetic corpus: enough files for several iterations, one of which
/// is Markdown carrying fenced blocks in two different languages.
fn corpus() -> Vec<CorpusFile> {
    let mut files = vec![CorpusFile::new(
        "docs/embedded.md",
        "# Embedded grammars\n\nThe loader reads the manifest.\n\n```rust\nfn load() {}\n```\n\n\
         Then it runs the checker.\n\n```python\ndef check():\n    return True\n```\n",
    )];
    for index in 0..12 {
        files.push(CorpusFile::new(
            format!("src/module_{index}.rs"),
            format!(
                "//! Module {index} loads records.\n\npub fn load_{index}() -> usize {{\n    \
                 // The loader returns the record count.\n    {index}\n}}\n"
            ),
        ));
    }
    files
}

// --- a. reproducible seeded sampling protocol ------------------------------

#[test]
fn issue_893_seeded_sampling_is_reproducible_and_seed_dependent() {
    let files = corpus();
    let paths: Vec<&str> = files.iter().map(|file| file.path.as_str()).collect();

    let protocol = SamplingProtocol::default();
    let first = protocol.sampling_order(&paths);
    let second = protocol.sampling_order(&paths);
    assert_eq!(
        first, second,
        "the same seed over the same corpus must draw the same files in the same order"
    );

    // Input order must not change the draw: the protocol sorts before shuffling.
    let mut reversed = paths.clone();
    reversed.reverse();
    assert_eq!(
        first,
        protocol.sampling_order(&reversed),
        "the sampling order must not depend on the caller's corpus order"
    );

    let other = protocol.with_seed(DEFAULT_SAMPLING_SEED + 1).sampling_order(&paths);
    assert_ne!(
        first, other,
        "a different seed must draw a different order, or the seed is not doing anything"
    );

    // The draw is a permutation: every file exactly once, so no file is
    // validated twice while another is never validated at all.
    let mut sorted = first.clone();
    sorted.sort_unstable();
    let mut expected = paths.clone();
    expected.sort_unstable();
    assert_eq!(sorted, expected, "the sampling order must be a permutation");
}

// --- b. two files per iteration until stable or bounded --------------------

#[test]
fn issue_893_iterations_validate_two_files_each_until_stable_or_bounded() {
    let files = corpus();
    let paths: Vec<&str> = files.iter().map(|file| file.path.as_str()).collect();
    let protocol = SamplingProtocol::default();
    assert_eq!(
        protocol.files_per_iteration, DEFAULT_FILES_PER_ITERATION,
        "the issue asks for two files per iteration"
    );
    assert_eq!(DEFAULT_FILES_PER_ITERATION, 2);

    // Consecutive iterations draw disjoint slices of the same permutation.
    let first = protocol.iteration_paths(&paths, 0);
    let second = protocol.iteration_paths(&paths, 1);
    assert_eq!(first.len(), 2);
    assert_eq!(second.len(), 2);
    for path in &second {
        assert!(
            !first.contains(path),
            "iteration 1 re-drew {path} from iteration 0"
        );
    }

    let report = validate_repository_summarization(
        &files,
        &protocol,
        &SummarizationConfig::default(),
    );
    for iteration in &report.iterations {
        assert_eq!(
            iteration.files.len(),
            2,
            "iteration {} validated {} file(s), not two",
            iteration.index,
            iteration.files.len()
        );
    }
    assert!(
        report.stabilized,
        "the run neither stabilized nor reported a bound: {report:?}"
    );
    assert!(!report.bound_reached);
    assert_eq!(
        report.iterations.len(),
        DEFAULT_STABILITY_WINDOW.max(2),
        "stability must not be declared before the whole window is observed"
    );

    // A bound that cannot fit the stability window must be reported honestly
    // instead of claiming a stability the run never observed.
    let bounded = validate_repository_summarization(
        &files,
        &protocol.with_max_iterations(1),
        &SummarizationConfig::default(),
    );
    assert_eq!(bounded.iterations.len(), 1);
    assert!(!bounded.stabilized);
    assert!(
        bounded.bound_reached,
        "a run stopped by its bound must say so"
    );
}

// --- c. published quality metric with an 80% minimum ratchet ---------------

#[test]
fn issue_893_quality_metric_is_published_and_ratcheted_at_eighty_percent() {
    assert_eq!(QUALITY_RATCHET_PERCENT, 80);
    assert!(
        CRITERIA.len() >= 8,
        "the published metric must name its criteria, not hide behind one number"
    );
    for criterion in CRITERIA {
        assert!(!criterion.name.is_empty());
        assert!(
            criterion.description.len() > 20,
            "criterion {} has no published description",
            criterion.name
        );
    }

    // The score is an exact integer ratio, floored — 79.6% never rounds into a
    // pass.
    let score = QualityScore {
        passed: 199,
        applicable: 250,
    };
    assert_eq!(score.percent(), 79);
    assert!(!score.meets(QUALITY_RATCHET_PERCENT));
    assert!(QualityScore {
        passed: 200,
        applicable: 250
    }
    .meets(QUALITY_RATCHET_PERCENT));
    // An empty score is not a vacuous pass.
    assert_eq!(QualityScore::default().percent(), 0);
    assert!(!QualityScore::default().meets(QUALITY_RATCHET_PERCENT));

    let report = validate_repository_summarization(
        &corpus(),
        &SamplingProtocol::default(),
        &SummarizationConfig::default(),
    );
    assert!(
        report.meets_ratchet(),
        "measured {}% ({}/{}), failures: {:?}",
        report.score.percent(),
        report.score.passed,
        report.score.applicable,
        report.failures()
    );
    assert!(ratchet_violations(&report, None).is_empty());

    // The ratchet is monotonic against whatever the repository last committed.
    let higher = QualityBaseline {
        percent: report.score.percent() + 1,
        ..QualityBaseline::default()
    };
    let violations = ratchet_violations(&report, Some(&higher));
    assert!(
        violations.iter().any(|violation| violation.contains("regressed")),
        "a score below the committed baseline must be a violation: {violations:?}"
    );
}

#[test]
fn issue_893_committed_baseline_records_the_measured_run() {
    let path = repository_root().join(BASELINE_PATH);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    let baseline = QualityBaseline::parse(&text)
        .unwrap_or_else(|| panic!("{} is not a quality baseline document", path.display()));

    assert!(
        baseline.percent >= QUALITY_RATCHET_PERCENT,
        "the committed baseline records {}%, below the published {QUALITY_RATCHET_PERCENT}% floor",
        baseline.percent
    );
    assert_eq!(baseline.seed, DEFAULT_SAMPLING_SEED);
    assert!(baseline.applicable_criteria > 0);
    assert!(
        baseline.embedded_grammar_blocks > 0,
        "the recorded run never exercised a Markdown embedded grammar block"
    );
    for field in [
        "ratchet_runner",
        "ratchet_policy",
        "honesty_policy",
        "minimum_percent",
    ] {
        assert!(
            text.contains(field),
            "the committed baseline does not publish its {field}"
        );
    }
    for criterion in CRITERIA {
        assert!(
            text.contains(criterion.name),
            "the committed baseline does not publish criterion {}",
            criterion.name
        );
    }
}

// --- d. recursive Markdown embedded grammars through production ------------

#[test]
fn issue_893_markdown_embedded_grammars_run_through_the_production_summarizer() {
    let markdown = "# Loader\n\nThe loader reads the manifest.\n\n```rust\nfn load() {}\n```\n\n\
                    It then writes a report.\n\n~~~json\n{\"ok\": true}\n~~~\n";
    let report = evaluate_file("docs/loader.md", markdown, &SummarizationConfig::default());

    assert_eq!(report.format, "markdown");
    assert_eq!(
        report.embedded_grammar_count, 2,
        "both fenced blocks must be recursively formalized"
    );
    let recursion = report
        .outcomes
        .iter()
        .find(|outcome| outcome.name == "embedded_grammar_recursion")
        .expect("the published metric includes the embedded-grammar criterion");
    assert!(
        recursion.applicable,
        "a Markdown file with fenced blocks must be scored on recursion"
    );
    assert!(recursion.passed, "{}", recursion.detail);

    // A file with no fenced blocks is exempt rather than failed, so the metric
    // neither rewards nor punishes files the criterion cannot describe.
    let plain = evaluate_file(
        "docs/plain.md",
        "# Plain\n\nThe loader reads the manifest.\n",
        &SummarizationConfig::default(),
    );
    let plain_recursion = plain
        .outcomes
        .iter()
        .find(|outcome| outcome.name == "embedded_grammar_recursion")
        .expect("criterion present");
    assert!(!plain_recursion.applicable);
    assert!(!plain_recursion.passed);

    // A run that never reaches an embedded grammar cannot be certified.
    let no_markdown: Vec<CorpusFile> = corpus()
        .into_iter()
        .filter(|file| !file.path.ends_with(".md"))
        .collect();
    let run = validate_repository_summarization(
        &no_markdown,
        &SamplingProtocol::default(),
        &SummarizationConfig::default(),
    );
    assert_eq!(run.embedded_grammar_blocks, 0);
    assert!(
        !run.stabilized,
        "a run that never exercised the recursive case must not declare stability"
    );
    let violations = ratchet_violations(&run, None);
    assert!(
        violations
            .iter()
            .any(|violation| violation.contains("embedded grammar")),
        "the ratchet must reject a run that skipped the recursive case: {violations:?}"
    );
}

// --- whole task ------------------------------------------------------------

#[test]
fn issue_893_whole_task_validates_real_repository_files_against_the_ratchet() {
    // The whole task, on the repository's own files rather than a fixture: draw
    // seeded random files, validate two per iteration through the production
    // summarizer until stable, exercise embedded grammars, and clear 80%.
    let root = repository_root();
    let mut files = Vec::new();
    for relative in [
        "README.md",
        "ARCHITECTURE.md",
        "CONTRIBUTING.md",
        "REQUIREMENTS.md",
        "src/lib.rs",
        "src/summarization/mod.rs",
        "src/summarization/file.rs",
        "src/summarization/resource.rs",
        "src/summarization/validation.rs",
        "src/cli_summarization.rs",
        "docs/case-studies/issue-893/README.md",
        "Cargo.toml",
    ] {
        let path = root.join(relative);
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        files.push(CorpusFile::new(relative, content));
    }

    let protocol = SamplingProtocol::default();
    let report = validate_repository_summarization(
        &files,
        &protocol,
        &SummarizationConfig::default(),
    );

    assert!(
        report.stabilized && !report.bound_reached,
        "the protocol did not stabilize on real repository files: {:?}",
        report.failures()
    );
    assert!(
        report.embedded_grammar_blocks > 0 && report.embedded_grammar_files > 0,
        "no Markdown embedded grammar block was exercised"
    );
    assert!(
        report.meets_ratchet(),
        "measured {}% ({}/{}) on real files, failures: {:?}",
        report.score.percent(),
        report.score.passed,
        report.score.applicable,
        report.failures()
    );
    assert!(ratchet_violations(&report, None).is_empty());

    // The rendered baseline reads back as the run it describes.
    let rendered = report.to_links_notation();
    let parsed = QualityBaseline::parse(&rendered).expect("the rendered run parses as a baseline");
    assert_eq!(parsed.percent, report.score.percent());
    assert_eq!(parsed.seed, protocol.seed);
    assert_eq!(parsed.passed_criteria, report.score.passed);
    assert_eq!(parsed.applicable_criteria, report.score.applicable);
    assert_eq!(parsed.iterations, report.iterations.len());
    assert!(parsed.stabilized);
    for path in report.sampled_paths() {
        assert!(
            rendered.contains(path),
            "the baseline does not record sampled file {path}"
        );
    }
}
