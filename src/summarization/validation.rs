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
//!    ([`formalize_repository_file`] and [`RepositoryFileFormalization::summary`]),
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

use std::collections::BTreeSet;

use crate::links_format::push_lino_node;

use super::file::{display_file_format, formalize_repository_file, RepositoryFileFormalization};
use super::{SummarizationConfig, SummarizationMode};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Criterion {
    /// Stable machine-readable name, used in reports and the baseline artifact.
    pub name: &'static str,
    /// What the criterion checks, published verbatim in the case study.
    pub description: &'static str,
}

/// The published quality metric: every criterion a sampled file is scored on.
pub const CRITERIA: &[Criterion] = &[
    Criterion {
        name: "identity_names_path",
        description: "The summary names the file it summarizes.",
    },
    Criterion {
        name: "format_declared",
        description: "The summary names the detected file format.",
    },
    Criterion {
        name: "size_reported",
        description: "The summary reports the file's line and byte counts.",
    },
    Criterion {
        name: "content_retained",
        description: "A file with content yields retained content statements in the summary.",
    },
    Criterion {
        name: "content_grounded",
        description:
            "Every identifier-shaped token in the summary occurs in the file's path or content.",
    },
    Criterion {
        name: "compression",
        description: "The summary is shorter than the file it summarizes.",
    },
    Criterion {
        name: "embedded_grammar_recursion",
        description: "Every Markdown fenced block is recursively formalized and its language is \
             named in the summary.",
    },
    Criterion {
        name: "meta_language_evidence",
        description:
            "A valid meta-language parse is reported with its label and syntax-link count.",
    },
    Criterion {
        name: "determinism",
        description: "Summarizing the same file twice returns byte-identical output.",
    },
    Criterion {
        name: "mode_ladder",
        description: "Short, Standard and Full summaries grow monotonically with the mode ladder.",
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

/// Record name of the committed quality baseline document.
pub const BASELINE_RECORD: &str = "summarization_quality_baseline";

/// Repository-relative path of the committed quality baseline.
pub const BASELINE_PATH: &str = "data/summarization/quality-baseline.lino";

/// The command that regenerates the baseline, recorded inside it so a reader
/// can reproduce the number without reading the workflow file.
pub const RATCHET_RUNNER: &str = "formal-ai summarization validate --append";

/// The monotonic rule the baseline enforces.
pub const RATCHET_POLICY: &str =
    "the measured percent may never fall below the committed ratchet_percent, which starts at \
     the published 80 percent minimum and may only ever be raised; the recorded percent is what \
     the committed run measured, and raising the floor to it is a deliberate reviewed edit";

/// What the recorded number is, and what it is not.
pub const HONESTY_POLICY: &str =
    "every number here is measured by running the production summarizer over seeded random \
     repository files; a run that reaches its iteration bound without stabilizing records \
     bound_reached true rather than claiming stability";

/// A previously committed baseline, read back for the monotonic comparison.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QualityBaseline {
    pub seed: u64,
    /// The percent the committed run actually measured — a record, not a rule.
    pub percent: u32,
    /// The percent the ratchet enforces. It starts at the published
    /// [`QUALITY_RATCHET_PERCENT`] and may only ever be raised.
    pub ratchet_percent: u32,
    pub passed_criteria: usize,
    pub applicable_criteria: usize,
    pub iterations: usize,
    pub stabilized: bool,
    pub embedded_grammar_blocks: usize,
}

impl QualityBaseline {
    /// Read a committed baseline document.
    ///
    /// The reader is line-oriented and tolerant of unknown fields, so adding a
    /// field to the artifact never breaks an older ratchet check.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        if !text.contains(BASELINE_RECORD) {
            return None;
        }
        let mut baseline = Self::default();
        let mut saw_percent = false;
        for line in text.lines() {
            let Some((name, value)) = split_field(line) else {
                continue;
            };
            match name {
                "seed" => baseline.seed = value.parse().unwrap_or_default(),
                "ratchet_percent" => baseline.ratchet_percent = value.parse().unwrap_or_default(),
                "percent" if !saw_percent => {
                    baseline.percent = value.parse().unwrap_or_default();
                    saw_percent = true;
                }
                "passed_criteria" => baseline.passed_criteria = value.parse().unwrap_or_default(),
                "applicable_criteria" => {
                    baseline.applicable_criteria = value.parse().unwrap_or_default();
                }
                "iterations" => baseline.iterations = value.parse().unwrap_or_default(),
                "stabilized" => baseline.stabilized = value == "true",
                // Only the header field counts: per-file `embedded_grammar_blocks`
                // lines appear later in the record and must not overwrite it.
                "embedded_grammar_blocks" if baseline.embedded_grammar_blocks == 0 => {
                    baseline.embedded_grammar_blocks = value.parse().unwrap_or_default();
                }
                _ => {}
            }
        }
        // A baseline written before the enforced floor was recorded separately,
        // or one whose floor was edited below the published minimum, still
        // enforces the published minimum.
        baseline.ratchet_percent = baseline.ratchet_percent.max(QUALITY_RATCHET_PERCENT);
        saw_percent.then_some(baseline)
    }
}

/// Every way `report` violates the ratchet, in report order. An empty result
/// means the ratchet holds.
///
/// Two rules apply, and both are absolute: the published 80% floor, and the
/// committed `ratchet_percent`, which starts at that floor and may only ever be
/// raised.
///
/// The enforced floor is deliberately *not* the last measured percent. The
/// corpus is every Git-tracked file, so it changes with every commit and the
/// seeded draw lands on different files; pinning the floor to a lucky 100% run
/// would turn an unlucky-but-honest draw into a red build. The measured percent
/// is recorded next to the floor so raising the floor stays a deliberate,
/// reviewable edit backed by evidence.
#[must_use]
pub fn ratchet_violations(
    report: &ValidationReport,
    baseline: Option<&QualityBaseline>,
) -> Vec<String> {
    let mut violations = Vec::new();
    let measured = report.score.percent();
    if report.score.applicable == 0 {
        violations.push(
            "no criterion was applicable: the run validated nothing and cannot be counted as a pass"
                .to_owned(),
        );
    }
    if !report.meets_ratchet() {
        violations.push(format!(
            "measured quality {measured}% is below the published minimum {QUALITY_RATCHET_PERCENT}%"
        ));
    }
    if report.embedded_grammar_blocks == 0 {
        violations.push(
            "no Markdown embedded grammar block was exercised: the run never reached the \
             recursive case the protocol exists to cover"
                .to_owned(),
        );
    }
    if let Some(baseline) = baseline {
        if measured < baseline.ratchet_percent {
            violations.push(format!(
                "measured quality {measured}% regressed below the committed ratchet {}%",
                baseline.ratchet_percent
            ));
        }
    }
    violations
}

fn split_field(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();
    let (name, rest) = trimmed.split_once(' ')?;
    let value = rest.trim();
    let delimiter = value.chars().next()?;
    if !matches!(delimiter, '"' | '\'') {
        return Some((name, value));
    }
    let unquoted = value
        .strip_prefix(delimiter)?
        .strip_suffix(delimiter)
        .unwrap_or(value);
    Some((name, unquoted))
}

/// The reproducible sampling protocol: which files are drawn, how many per
/// iteration, and when the loop is allowed to stop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SamplingProtocol {
    pub seed: u64,
    pub files_per_iteration: usize,
    pub max_iterations: usize,
    pub minimum_iterations: usize,
    pub stability_window: usize,
    pub stability_tolerance_percent: u32,
}

impl Default for SamplingProtocol {
    fn default() -> Self {
        Self {
            seed: DEFAULT_SAMPLING_SEED,
            files_per_iteration: DEFAULT_FILES_PER_ITERATION,
            max_iterations: DEFAULT_MAX_ITERATIONS,
            minimum_iterations: DEFAULT_MINIMUM_ITERATIONS,
            stability_window: DEFAULT_STABILITY_WINDOW,
            stability_tolerance_percent: DEFAULT_STABILITY_TOLERANCE_PERCENT,
        }
    }
}

impl SamplingProtocol {
    /// Builder helper pinning the seed.
    #[must_use]
    pub const fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Builder helper pinning the iteration bound.
    #[must_use]
    pub const fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Deterministic sampling order over `paths`.
    ///
    /// The corpus is sorted first, so a caller's directory-walk order cannot
    /// change the draw, then permuted with a seeded Fisher-Yates shuffle. The
    /// result is the sampling plan: iteration `i` validates the slice
    /// `[i * files_per_iteration, (i + 1) * files_per_iteration)`.
    #[must_use]
    pub fn sampling_order<'corpus>(&self, paths: &[&'corpus str]) -> Vec<&'corpus str> {
        let mut ordered: Vec<&str> = paths.to_vec();
        ordered.sort_unstable();
        ordered.dedup();
        let mut prng = Prng::seeded(self.seed);
        // Fisher-Yates from the back, the standard unbiased shuffle.
        for index in (1..ordered.len()).rev() {
            let swap = prng.below(index + 1);
            ordered.swap(index, swap);
        }
        ordered
    }

    /// Deterministic sampling order stratified over the recursive case.
    ///
    /// [`Self::sampling_order`] is a uniform permutation, which is the right
    /// draw for "files nobody optimized for" but the wrong one for a
    /// requirement that a *particular kind* of file be exercised. Markdown
    /// files carrying fenced blocks are a small minority of this repository, so
    /// a uniform draw bounded at `max_iterations * files_per_iteration` files
    /// can miss every one of them — that is not a hypothetical, it failed a CI
    /// run at 100% measured quality (see `docs/case-studies/issue-893/`).
    ///
    /// So the draw is stratified rather than enlarged: the seeded permutation
    /// is computed exactly as before, then the first entry that carries an
    /// embedded grammar is promoted to the front. Every other file keeps its
    /// seeded position, the result is still a permutation of the same corpus,
    /// and it is still a pure function of the seed and the file set — but
    /// iteration 0 now always reaches the recursive case, on any corpus that
    /// contains one.
    #[must_use]
    pub fn stratified_sampling_order<'corpus>(
        &self,
        corpus: &'corpus [CorpusFile],
    ) -> Vec<&'corpus str> {
        let paths: Vec<&str> = corpus.iter().map(|file| file.path.as_str()).collect();
        let mut ordered = self.sampling_order(&paths);
        let promote = ordered.iter().position(|path| {
            corpus
                .iter()
                .find(|file| file.path == *path)
                .is_some_and(|file| carries_embedded_grammar(&file.path, &file.content))
        });
        if let Some(index) = promote {
            let file = ordered.remove(index);
            ordered.insert(0, file);
        }
        ordered
    }

    /// Files drawn for one iteration, or an empty slice when the corpus is
    /// exhausted.
    #[must_use]
    pub fn iteration_paths<'corpus>(
        &self,
        paths: &[&'corpus str],
        iteration: usize,
    ) -> Vec<&'corpus str> {
        let ordered = self.sampling_order(paths);
        let start = iteration.saturating_mul(self.files_per_iteration);
        ordered
            .into_iter()
            .skip(start)
            .take(self.files_per_iteration)
            .collect()
    }

    /// How many iterations this corpus can supply under the bound.
    #[must_use]
    pub const fn available_iterations(&self, corpus_size: usize) -> usize {
        if self.files_per_iteration == 0 {
            return 0;
        }
        let supplied = corpus_size / self.files_per_iteration;
        if supplied < self.max_iterations {
            supplied
        } else {
            self.max_iterations
        }
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

fn check_identity(path: &str, summary: &str) -> CriterionOutcome {
    let passed = summary.contains(path);
    outcome("identity_names_path", true, passed, format!("path={path}"))
}

fn check_format(formalized: &RepositoryFileFormalization, summary: &str) -> CriterionOutcome {
    let label = display_file_format(&formalized.format);
    let passed = summary.contains(label);
    outcome(
        "format_declared",
        true,
        passed,
        format!("format={} label={label}", formalized.format),
    )
}

fn check_size(formalized: &RepositoryFileFormalization, summary: &str) -> CriterionOutcome {
    let passed = summary.contains(&formalized.line_count.to_string())
        && summary.contains(&formalized.byte_count.to_string());
    outcome(
        "size_reported",
        true,
        passed,
        format!(
            "lines={} bytes={}",
            formalized.line_count, formalized.byte_count
        ),
    )
}

fn check_content_retained(
    formalized: &RepositoryFileFormalization,
    summary: &str,
    config: &SummarizationConfig,
) -> CriterionOutcome {
    let applicable = !formalized.statements.is_empty();
    let retained = super::summarize(&formalized.statements, config);
    let rendered = super::deformalize(&retained);
    let passed = !rendered.trim().is_empty() && summary.contains(rendered.trim());
    outcome(
        "content_retained",
        applicable,
        passed,
        format!(
            "statements={} retained={}",
            formalized.statements.len(),
            retained.len()
        ),
    )
}

fn check_content_grounded(
    path: &str,
    content: &str,
    summary: &str,
    formalized: &RepositoryFileFormalization,
) -> CriterionOutcome {
    // Labels the summarizer itself introduces — the format name, the detected
    // meta-language, and embedded block languages — are metadata about the file
    // rather than claims quoted from it, so they are grounded by construction.
    let mut vocabulary: BTreeSet<&str> = BTreeSet::new();
    vocabulary.insert(display_file_format(&formalized.format));
    vocabulary.insert(formalized.format.as_str());
    if let Some(meta) = formalized.meta_language.as_ref() {
        vocabulary.insert(meta.label.as_str());
    }
    for block in &formalized.embedded_grammars {
        vocabulary.insert(block.language.as_str());
    }

    // Inline-code delimiters are markup, not content: a summary that renders
    // `Topic`/`Short` as Topic/Short quoted the file faithfully. Grounding is
    // therefore checked against the file with its code fences and code spans
    // unwrapped, so the criterion still catches invented or dropped text —
    // `crates.io-<version>-orange` summarized as `crates.io--orange` remains a
    // failure — without penalizing correct markup removal.
    let unwrapped: String = content.chars().filter(|ch| *ch != '`').collect();

    let ungrounded: Vec<String> = identifier_tokens(summary)
        .into_iter()
        .filter(|token| {
            !vocabulary.contains(token.as_str())
                && !unwrapped.contains(token.as_str())
                && !path.contains(token.as_str())
        })
        .collect();
    let detail = if ungrounded.is_empty() {
        "all identifier tokens grounded".to_owned()
    } else {
        format!("ungrounded={}", ungrounded.join(", "))
    };
    outcome("content_grounded", true, ungrounded.is_empty(), detail)
}

fn check_compression(content: &str, summary: &str) -> CriterionOutcome {
    // Tiny files legitimately summarize into something as long as themselves —
    // "x.txt is a text file with 1 lines and 3 bytes." is longer than "hi.".
    // The criterion applies once a file is big enough for compression to mean
    // something.
    let applicable = content.len() >= COMPRESSION_FLOOR_BYTES;
    let passed = summary.len() < content.len();
    outcome(
        "compression",
        applicable,
        passed,
        format!(
            "summary_bytes={} file_bytes={}",
            summary.len(),
            content.len()
        ),
    )
}

/// Files below this size are exempt from the compression criterion.
pub const COMPRESSION_FLOOR_BYTES: usize = 400;

fn check_embedded_grammars(
    formalized: &RepositoryFileFormalization,
    content: &str,
    summary: &str,
) -> CriterionOutcome {
    let expected = fenced_block_languages(content);
    let applicable = formalized.format == "markdown" && !expected.is_empty();
    let recorded: Vec<&str> = formalized
        .embedded_grammars
        .iter()
        .map(|block| block.language.as_str())
        .collect();
    let counted = recorded.len() == expected.len();
    let named: BTreeSet<&str> = recorded.iter().copied().collect();
    let listed = named.iter().all(|language| summary.contains(*language));
    let passed = counted && listed;
    outcome(
        "embedded_grammar_recursion",
        applicable,
        passed,
        format!(
            "fences={} recorded={} languages={}",
            expected.len(),
            recorded.len(),
            recorded.join(",")
        ),
    )
}

fn check_meta_language(
    formalized: &RepositoryFileFormalization,
    summary: &str,
) -> CriterionOutcome {
    let evidence = formalized
        .meta_language
        .as_ref()
        .filter(|meta| meta.is_valid());
    let applicable = evidence.is_some();
    let passed = evidence.is_some_and(|meta| {
        summary.contains(&meta.label) && summary.contains(&meta.syntax_link_count.to_string())
    });
    outcome(
        "meta_language_evidence",
        applicable,
        passed,
        evidence.map_or_else(
            || "no valid meta-language parse".to_owned(),
            |meta| format!("label={} links={}", meta.label, meta.syntax_link_count),
        ),
    )
}

fn check_determinism(
    path: &str,
    content: &str,
    config: &SummarizationConfig,
    summary: &str,
) -> CriterionOutcome {
    let repeated = formalize_repository_file(path, content).summary(config);
    let passed = repeated == summary;
    outcome(
        "determinism",
        true,
        passed,
        format!("summary_bytes={}", summary.len()),
    )
}

fn check_mode_ladder(
    formalized: &RepositoryFileFormalization,
    config: &SummarizationConfig,
) -> CriterionOutcome {
    let short = formalized.summary(&config.clone().with_mode(SummarizationMode::Short));
    let standard = formalized.summary(&config.clone().with_mode(SummarizationMode::Standard));
    let full = formalized.summary(&config.clone().with_mode(SummarizationMode::Full));
    let passed = short.len() <= standard.len() && standard.len() <= full.len();
    outcome(
        "mode_ladder",
        true,
        passed,
        format!(
            "short={} standard={} full={}",
            short.len(),
            standard.len(),
            full.len()
        ),
    )
}

const fn outcome(
    name: &'static str,
    applicable: bool,
    passed: bool,
    detail: String,
) -> CriterionOutcome {
    CriterionOutcome {
        name,
        applicable,
        // A criterion that does not apply is never counted as passed, so the
        // report cannot inflate itself with vacuous truths.
        passed: applicable && passed,
        detail,
    }
}

/// Does this file reach the recursive case the `embedded_grammar_recursion`
/// criterion scores?
///
/// This mirrors that criterion's applicability test, so the stratified draw
/// promotes a file the metric will actually be able to score rather than one
/// that merely looks like Markdown.
fn carries_embedded_grammar(path: &str, content: &str) -> bool {
    let markdown = std::path::Path::new(path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"));
    markdown && !fenced_block_languages(content).is_empty()
}

/// Independent `CommonMark` fence scanner used as the metric's oracle.
///
/// This deliberately does *not* call the summarizer's own fence scanner: a
/// criterion that asked the implementation to grade itself would pass by
/// construction.
fn fenced_block_languages(markdown: &str) -> Vec<String> {
    let mut languages = Vec::new();
    let mut open: Option<(char, usize)> = None;
    for line in markdown.lines() {
        let trimmed = line.trim_start();
        let marker = fence_marker(trimmed);
        match (open, marker) {
            (Some((ch, len)), Some((candidate_ch, candidate_len)))
                if candidate_ch == ch
                    && candidate_len >= len
                    && trimmed[candidate_len..].trim().is_empty() =>
            {
                open = None;
            }
            (None, Some((ch, len))) => {
                open = Some((ch, len));
                languages.push(fence_language(&trimmed[len..]));
            }
            // A non-closing marker inside an open block, or an ordinary line
            // outside one, is content rather than structure.
            _ => {}
        }
    }
    languages
}

fn fence_marker(trimmed_line: &str) -> Option<(char, usize)> {
    let ch = trimmed_line.chars().next()?;
    if ch != '`' && ch != '~' {
        return None;
    }
    let len = trimmed_line.chars().take_while(|c| *c == ch).count();
    (len >= 3).then_some((ch, len))
}

fn fence_language(info_string: &str) -> String {
    info_string
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

/// Identifier-shaped tokens (`snake_case`, `CamelCase`, `dotted.paths`) a
/// summary may only contain if the file or its path contains them too.
fn identifier_tokens(summary: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for raw in summary.split(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | '(' | ')')) {
        let token =
            raw.trim_matches(|c: char| matches!(c, '.' | ':' | '`' | '"' | '\'' | '!' | '?'));
        if token.len() < 4 {
            continue;
        }
        let looks_like_identifier = token.contains('_')
            || token.contains('/')
            || (token.contains('.') && !token.ends_with('.'))
            || is_camel_case(token);
        if looks_like_identifier && token.chars().all(is_identifier_char) {
            tokens.push(token.to_owned());
        }
    }
    tokens.sort();
    tokens.dedup();
    tokens
}

/// `CamelCase` in the strict sense: an interior capital that follows a
/// lower-case letter. A merely capitalized English word ("Markdown", "It") is
/// not an identifier and must not be demanded of the file's text.
fn is_camel_case(token: &str) -> bool {
    token
        .chars()
        .zip(token.chars().skip(1))
        .any(|(previous, next)| previous.is_lowercase() && next.is_uppercase())
}

const fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.' | '/' | '-' | ':')
}

/// Deterministic `splitmix64` stream: the sampling protocol's reproducible
/// randomness. Same seed, same permutation, on every platform and every run.
struct Prng {
    state: u64,
}

impl Prng {
    const fn seeded(seed: u64) -> Self {
        Self {
            state: seed ^ 0x9e37_79b9_7f4a_7c15,
        }
    }

    const fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            usize::try_from(self.next_u64() % bound as u64).unwrap_or(0)
        }
    }
}
