//! Iterative, seeded validation of repository-file summarization quality
//! (issue #563's original ask, re-opened as issue #893).
//!
//! Issue #563 asked for more than "a function that summarizes a file". It asked
//! for a *protocol*: take two random repository files, check the summaries,
//! generalize, then take two more, and keep going until the result is stable on
//! files nobody optimized for — at a quality bar of at least 80%. This module is
//! that protocol, expressed as pure, deterministic code:
//!
//! 1. **Sample.** [`SamplingProtocol`] fixes a seed, a per-iteration file count
//!    (two, as the issue asks) and an iteration bound. The corpus is permuted
//!    once with a seeded `splitmix64` Fisher-Yates shuffle, so the same seed and
//!    the same corpus always draw the same files in the same order, and no file
//!    is validated twice inside one run. The draw is then stratified over the
//!    one case the issue names explicitly — see
//!    [`SamplingProtocol::stratified_sampling_order`].
//! 2. **Validate.** Every sampled file goes through the *production* summarizer
//!    ([`formalize_repository_file`] and
//!    [`RepositoryFileFormalization::summary`](super::file::RepositoryFileFormalization::summary)),
//!    never a test-only reimplementation, and is scored against the published
//!    criteria in [`CRITERIA`].
//! 3. **Iterate until stable.** The loop stops as soon as
//!    [`SamplingProtocol::stability_window`] consecutive iterations all clear the
//!    ratchet and stay within [`SamplingProtocol::stability_tolerance_percent`]
//!    of each other — the issue's "2-3 times similar summarization" — and the
//!    run has taken at least [`DEFAULT_MINIMUM_ITERATIONS`] samples. Otherwise it
//!    stops at the iteration bound and reports that bound plainly, rather than
//!    claiming a stability it never observed.
//! 4. **Ratchet.** [`QUALITY_RATCHET_PERCENT`] is the 80% floor. A committed
//!    baseline records the last measured score, and the score may never fall
//!    below either the floor or the baseline.
//!
//! Scores are exact integer ratios (`passed` criteria over *applicable*
//! criteria), micro-averaged over files and iterations, and floored when
//! rendered as a percentage — 79.6% is reported and gated as 79%, never rounded
//! up into a pass.

mod baseline;
mod criteria;
mod prose;
mod sampling;

pub use baseline::{
    ratchet_violations, QualityBaseline, BASELINE_PATH, BASELINE_RECORD, HONESTY_POLICY,
    RATCHET_POLICY, RATCHET_RUNNER,
};
pub use criteria::COMPRESSION_FLOOR_BYTES;
pub use prose::sentence as quality_sentence;
pub use sampling::SamplingProtocol;

use criteria::{
    carries_embedded_grammar, check_compression, check_content_grounded, check_content_retained,
    check_determinism, check_embedded_grammars, check_format, check_identity, check_meta_language,
    check_mode_ladder, check_size,
};

use crate::links_format::push_lino_node;

use super::file::formalize_repository_file;
use super::SummarizationConfig;

/// Minimum quality percentage the repository-summarization protocol must reach.
/// This is the 80% bar issue #563 set ("at least 80% perfect") and issue #893
/// re-opened as a ratchet.
pub const QUALITY_RATCHET_PERCENT: u32 = 80;

/// Default sampling seed. `563` names the issue this protocol answers, so a run
/// reported in a review can be reproduced from the issue number alone.
pub const DEFAULT_SAMPLING_SEED: u64 = 563;

/// Files validated per iteration — "we first take 2 random files" (issue #563).
pub const DEFAULT_FILES_PER_ITERATION: usize = 2;

/// Iteration bound. Reaching it without stability is reported, never hidden.
pub const DEFAULT_MAX_ITERATIONS: usize = 24;

/// Consecutive similar iterations required to call the result stable —
/// the issue's "until we will actually have 2-3 times similar summarization".
pub const DEFAULT_STABILITY_WINDOW: usize = 3;

/// How far apart, in percentage points, two iteration scores may be and still
/// count as "similar".
pub const DEFAULT_STABILITY_TOLERANCE_PERCENT: u32 = 5;

/// Iterations that must run before stability may be declared at all.
///
/// The stability window alone would let a healthy corpus stop after three
/// iterations — six files. Three consecutive perfect iterations over six files
/// is not evidence about a corpus of ten thousand, and a gate that only ever
/// looks at six files stops being able to notice a regression. The bound is
/// halved rather than removed: a run samples at least twenty-four files, then
/// stops as soon as the window is satisfied.
pub const DEFAULT_MINIMUM_ITERATIONS: usize = 12;

/// One published quality criterion.
///
/// `applicable` is decided per file: a criterion that cannot apply to a file
/// (embedded-grammar recursion on a file with no fenced blocks, for instance)
/// is left out of that file's denominator instead of being scored as a free
/// pass or an unfair failure.
///
/// Only the language-neutral `name` lives in Rust; the sentence that describes
/// the criterion is looked up from the seed (R379), so publishing the metric in
/// another language is a seed edit rather than a code change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Criterion {
    /// Stable machine-readable name, used in reports and the baseline artifact.
    pub name: &'static str,
}

impl Criterion {
    /// What the criterion checks, published verbatim in the case study.
    ///
    /// Read from `data/seed/multilingual-responses-summarization-quality.lino`
    /// under the intent `summarization_criterion_<name>`.
    #[must_use]
    pub fn description(&self) -> String {
        prose::sentence(&format!("{CRITERION_INTENT_PREFIX}{}", self.name), &[])
    }
}

/// Intent prefix under which each criterion's published description is seeded.
pub const CRITERION_INTENT_PREFIX: &str = "summarization_criterion_";

/// The published quality metric: every criterion a sampled file is scored on.
pub const CRITERIA: &[Criterion] = &[
    Criterion {
        name: "identity_names_path",
    },
    Criterion {
        name: "format_declared",
    },
    Criterion {
        name: "size_reported",
    },
    Criterion {
        name: "content_retained",
    },
    Criterion {
        name: "content_grounded",
    },
    Criterion {
        name: "compression",
    },
    Criterion {
        name: "embedded_grammar_recursion",
    },
    Criterion {
        name: "meta_language_evidence",
    },
    Criterion {
        name: "determinism",
    },
    Criterion {
        name: "mode_ladder",
    },
];

/// Outcome of one criterion on one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriterionOutcome {
    pub name: &'static str,
    /// `false` when the criterion does not apply to this file; such criteria are
    /// excluded from both numerator and denominator.
    pub applicable: bool,
    pub passed: bool,
    /// Human-readable evidence for the verdict.
    pub detail: String,
}

/// An exact `passed / applicable` ratio. Kept as integers so a score is
/// reproducible bit-for-bit across platforms.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct QualityScore {
    pub passed: usize,
    pub applicable: usize,
}

impl QualityScore {
    /// Add another score into this one (micro-average, not mean-of-means).
    pub const fn absorb(&mut self, other: Self) {
        self.passed += other.passed;
        self.applicable += other.applicable;
    }

    /// Floored percentage. An empty score is `0`, never a vacuous `100`.
    #[must_use]
    pub const fn percent(self) -> u32 {
        if self.applicable == 0 {
            return 0;
        }
        // A percentage of a ratio of counts never exceeds 100, so the narrowing
        // is exact regardless of how large the corpus grows.
        #[allow(clippy::cast_possible_truncation)]
        {
            ((self.passed * 100) / self.applicable) as u32
        }
    }

    /// Does this score clear the supplied floor?
    #[must_use]
    pub const fn meets(self, floor_percent: u32) -> bool {
        self.applicable > 0 && self.percent() >= floor_percent
    }
}

/// Quality report for a single sampled file.
#[derive(Debug, Clone)]
pub struct FileQualityReport {
    pub path: String,
    pub format: String,
    pub line_count: usize,
    pub byte_count: usize,
    /// Number of recursively formalized Markdown embedded grammar blocks.
    pub embedded_grammar_count: usize,
    pub outcomes: Vec<CriterionOutcome>,
    pub score: QualityScore,
}

impl FileQualityReport {
    /// Criteria that did not pass, in published order.
    #[must_use]
    pub fn failures(&self) -> Vec<&CriterionOutcome> {
        self.outcomes
            .iter()
            .filter(|outcome| outcome.applicable && !outcome.passed)
            .collect()
    }
}

/// Quality report for one iteration of the protocol (two files by default).
#[derive(Debug, Clone)]
pub struct IterationReport {
    /// Zero-based iteration index.
    pub index: usize,
    pub files: Vec<FileQualityReport>,
    pub score: QualityScore,
}

/// Result of the whole iterative validation run.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub protocol: SamplingProtocol,
    pub iterations: Vec<IterationReport>,
    /// `true` when the stability window closed before the iteration bound.
    pub stabilized: bool,
    /// `true` when the run stopped because it ran out of iterations or files.
    pub bound_reached: bool,
    /// Total Markdown embedded grammar blocks exercised across the run.
    pub embedded_grammar_blocks: usize,
    /// Files that carried at least one Markdown embedded grammar block.
    pub embedded_grammar_files: usize,
    pub score: QualityScore,
}

impl ValidationReport {
    /// Does the measured score clear the published 80% ratchet?
    #[must_use]
    pub const fn meets_ratchet(&self) -> bool {
        self.score.meets(QUALITY_RATCHET_PERCENT)
    }

    /// Every sampled path, in sampling order.
    #[must_use]
    pub fn sampled_paths(&self) -> Vec<&str> {
        self.iterations
            .iter()
            .flat_map(|iteration| iteration.files.iter().map(|file| file.path.as_str()))
            .collect()
    }

    /// Every failing criterion across the run, as `path::criterion` pairs.
    #[must_use]
    pub fn failures(&self) -> Vec<(&str, &CriterionOutcome)> {
        self.iterations
            .iter()
            .flat_map(|iteration| iteration.files.iter())
            .flat_map(|file| {
                file.failures()
                    .into_iter()
                    .map(move |outcome| (file.path.as_str(), outcome))
            })
            .collect()
    }

    /// Render the run as the committed Links Notation baseline document.
    ///
    /// This is exactly the artifact `formal-ai summarization validate --append`
    /// writes and `formal-ai summarization ratchet` reads back, so the committed
    /// baseline is never hand-maintained prose about a run — it is the run.
    #[must_use]
    pub fn to_links_notation(&self, ratchet_percent: u32) -> String {
        let ratchet_percent = ratchet_percent.max(QUALITY_RATCHET_PERCENT);
        let mut out = String::new();
        push_lino_node(&mut out, 0, BASELINE_RECORD, None);
        let field = |out: &mut String, name: &str, value: &str| {
            push_lino_node(out, 2, name, Some(value));
        };
        field(&mut out, "record_type", BASELINE_RECORD);
        field(&mut out, "ratchet_runner", RATCHET_RUNNER);
        field(&mut out, "ratchet_policy", RATCHET_POLICY);
        field(&mut out, "honesty_policy", HONESTY_POLICY);
        field(
            &mut out,
            "minimum_percent",
            &QUALITY_RATCHET_PERCENT.to_string(),
        );
        field(&mut out, "ratchet_percent", &ratchet_percent.to_string());
        field(&mut out, "seed", &self.protocol.seed.to_string());
        field(
            &mut out,
            "files_per_iteration",
            &self.protocol.files_per_iteration.to_string(),
        );
        field(
            &mut out,
            "max_iterations",
            &self.protocol.max_iterations.to_string(),
        );
        field(
            &mut out,
            "minimum_iterations",
            &self.protocol.minimum_iterations.to_string(),
        );
        field(
            &mut out,
            "stability_window",
            &self.protocol.stability_window.to_string(),
        );
        field(
            &mut out,
            "stability_tolerance_percent",
            &self.protocol.stability_tolerance_percent.to_string(),
        );
        field(&mut out, "iterations", &self.iterations.len().to_string());
        field(&mut out, "stabilized", &self.stabilized.to_string());
        field(&mut out, "bound_reached", &self.bound_reached.to_string());
        field(
            &mut out,
            "embedded_grammar_files",
            &self.embedded_grammar_files.to_string(),
        );
        field(
            &mut out,
            "embedded_grammar_blocks",
            &self.embedded_grammar_blocks.to_string(),
        );
        field(&mut out, "passed_criteria", &self.score.passed.to_string());
        field(
            &mut out,
            "applicable_criteria",
            &self.score.applicable.to_string(),
        );
        field(&mut out, "percent", &self.score.percent().to_string());
        for criterion in CRITERIA {
            push_lino_node(&mut out, 2, "criterion", Some(criterion.name));
        }
        for iteration in &self.iterations {
            push_lino_node(&mut out, 2, "iteration", None);
            push_lino_node(&mut out, 4, "index", Some(&iteration.index.to_string()));
            push_lino_node(
                &mut out,
                4,
                "percent",
                Some(&iteration.score.percent().to_string()),
            );
            for file in &iteration.files {
                push_lino_node(&mut out, 4, "file", None);
                push_lino_node(&mut out, 6, "path", Some(&file.path));
                push_lino_node(&mut out, 6, "format", Some(&file.format));
                push_lino_node(
                    &mut out,
                    6,
                    "percent",
                    Some(&file.score.percent().to_string()),
                );
                push_lino_node(
                    &mut out,
                    6,
                    "embedded_grammar_blocks",
                    Some(&file.embedded_grammar_count.to_string()),
                );
                for outcome in &file.outcomes {
                    if outcome.applicable && !outcome.passed {
                        push_lino_node(&mut out, 6, "failed_criterion", Some(outcome.name));
                    }
                }
            }
        }
        out
    }
}

/// A repository file the protocol may sample.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusFile {
    pub path: String,
    pub content: String,
}

impl CorpusFile {
    /// Build a corpus entry.
    pub fn new(path: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
        }
    }
}

/// Score one repository file through the production summarizer.
///
/// The formalization and the summary both come from
/// [`formalize_repository_file`]; nothing here re-implements summarization, so a
/// regression in the shipped pipeline shows up as a falling score.
#[must_use]
pub fn evaluate_file(path: &str, content: &str, config: &SummarizationConfig) -> FileQualityReport {
    let formalized = formalize_repository_file(path, content);
    let summary = formalized.summary(config);
    let mut outcomes = Vec::with_capacity(CRITERIA.len());

    outcomes.push(check_identity(path, &summary));
    outcomes.push(check_format(&formalized, &summary));
    outcomes.push(check_size(&formalized, &summary));
    outcomes.push(check_content_retained(&formalized, &summary, config));
    outcomes.push(check_content_grounded(path, content, &summary, &formalized));
    outcomes.push(check_compression(content, &summary));
    outcomes.push(check_embedded_grammars(&formalized, content, &summary));
    outcomes.push(check_meta_language(&formalized, &summary));
    outcomes.push(check_determinism(path, content, config, &summary));
    outcomes.push(check_mode_ladder(&formalized, config));

    let mut score = QualityScore::default();
    for outcome in &outcomes {
        if outcome.applicable {
            score.applicable += 1;
            if outcome.passed {
                score.passed += 1;
            }
        }
    }

    FileQualityReport {
        path: path.to_owned(),
        format: formalized.format.clone(),
        line_count: formalized.line_count,
        byte_count: formalized.byte_count,
        embedded_grammar_count: formalized.embedded_grammars.len(),
        outcomes,
        score,
    }
}

/// Run the iterative protocol over `corpus` until the result stabilizes or the
/// reported bound is reached.
///
/// A run is only allowed to declare stability once it has actually exercised
/// recursive Markdown embedded grammars, so the protocol cannot certify the
/// summarizer on a sample that never touched the hardest case.
#[must_use]
pub fn validate_repository_summarization(
    corpus: &[CorpusFile],
    protocol: &SamplingProtocol,
    config: &SummarizationConfig,
) -> ValidationReport {
    let ordered = protocol.stratified_sampling_order(corpus);
    let available = protocol.available_iterations(ordered.len());

    let mut iterations: Vec<IterationReport> = Vec::new();
    let mut score = QualityScore::default();
    let mut embedded_grammar_blocks = 0;
    let mut embedded_grammar_files = 0;
    let mut stabilized = false;

    for index in 0..available {
        let start = index * protocol.files_per_iteration;
        let mut files = Vec::with_capacity(protocol.files_per_iteration);
        let mut iteration_score = QualityScore::default();
        for path in ordered
            .iter()
            .skip(start)
            .take(protocol.files_per_iteration)
        {
            let Some(entry) = corpus.iter().find(|file| file.path == *path) else {
                continue;
            };
            let report = evaluate_file(&entry.path, &entry.content, config);
            iteration_score.absorb(report.score);
            embedded_grammar_blocks += report.embedded_grammar_count;
            if report.embedded_grammar_count > 0 {
                embedded_grammar_files += 1;
            }
            files.push(report);
        }
        score.absorb(iteration_score);
        iterations.push(IterationReport {
            index,
            files,
            score: iteration_score,
        });

        if embedded_grammar_blocks > 0 && is_stable(&iterations, protocol, available) {
            stabilized = true;
            break;
        }
    }

    ValidationReport {
        protocol: *protocol,
        iterations,
        stabilized,
        bound_reached: !stabilized,
        embedded_grammar_blocks,
        embedded_grammar_files,
        score,
    }
}

/// Are the last `stability_window` iterations all above the ratchet and within
/// the configured tolerance of one another?
fn is_stable(
    iterations: &[IterationReport],
    protocol: &SamplingProtocol,
    available: usize,
) -> bool {
    if protocol.stability_window == 0 || iterations.len() < protocol.stability_window {
        return false;
    }
    // A corpus too small to supply the minimum sample cannot be held to it, or
    // a run over a twelve-file fixture could never stabilize at all.
    if iterations.len() < protocol.minimum_iterations.min(available) {
        return false;
    }
    let window = &iterations[iterations.len() - protocol.stability_window..];
    let percents: Vec<u32> = window
        .iter()
        .map(|iteration| iteration.score.percent())
        .collect();
    let Some(&lowest) = percents.iter().min() else {
        return false;
    };
    let Some(&highest) = percents.iter().max() else {
        return false;
    };
    lowest >= QUALITY_RATCHET_PERCENT && highest - lowest <= protocol.stability_tolerance_percent
}
