//! The committed quality baseline and the two rules the ratchet enforces.
//!
//! Split out of [`super`] to keep each file inside the repository's own
//! thousand-line ceiling; the protocol is unchanged.

use super::{ValidationReport, QUALITY_RATCHET_PERCENT};

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
