//! Report rendering for `check-coverage-ratchet.rs`.
//!
//! Included by the script with `#[path]`. It lives in its own file because a
//! rust-script is a single file by construction and `scripts/check-file-size.rs`
//! caps a Rust file at 1000 lines; the rendering half is the part with no
//! decision logic in it, so it is the honest thing to lift out.

use super::*;

// ---------------------------------------------------------------------------
// Reports
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize)]
struct SummaryJson<'a> {
    denominator: &'a str,
    label: &'a str,
    lines: MetricJson,
    functions: MetricJson,
    tolerance_percent: f64,
    reviewed: &'a str,
    evidence: &'a str,
    status: &'a str,
    files: &'a [FileCoverage],
    #[serde(skip_serializing_if = "Option::is_none")]
    inventory: Option<&'a InventoryReport>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct MetricJson {
    found: usize,
    hit: usize,
    percent: f64,
    baseline_percent: f64,
    delta_percent: f64,
    status: String,
}

fn status_word(status: &RatchetStatus) -> &'static str {
    match status {
        RatchetStatus::Held => "held",
        RatchetStatus::Improved => "improved",
        RatchetStatus::Regressed => "regressed",
    }
}

fn metric_json(measurement: &Measurement, verdict: &MetricVerdict) -> MetricJson {
    let (found, hit) = match verdict.metric {
        "lines" => (measurement.lines_found, measurement.lines_hit),
        _ => (measurement.functions_found, measurement.functions_hit),
    };
    MetricJson {
        found,
        hit,
        percent: verdict.measured,
        baseline_percent: verdict.baseline,
        delta_percent: verdict.delta(),
        status: status_word(&verdict.status).to_string(),
    }
}

fn overall_status(verdicts: &[MetricVerdict], inventory: Option<&InventoryReport>) -> &'static str {
    if verdicts
        .iter()
        .any(|verdict| verdict.status == RatchetStatus::Regressed)
        || inventory.is_some_and(|report| !report.is_clean())
    {
        "failed"
    } else if verdicts
        .iter()
        .any(|verdict| verdict.status == RatchetStatus::Improved)
    {
        "improved"
    } else {
        "held"
    }
}

pub(super) fn render_summary_json(
    measurement: &Measurement,
    denominator: &Denominator,
    verdicts: &[MetricVerdict],
    inventory: Option<&InventoryReport>,
) -> String {
    let summary = SummaryJson {
        denominator: &measurement.denominator,
        label: &measurement.label,
        lines: metric_json(measurement, &verdicts[0]),
        functions: metric_json(measurement, &verdicts[1]),
        tolerance_percent: denominator.tolerance_percent,
        reviewed: &denominator.reviewed,
        evidence: &denominator.evidence,
        status: overall_status(verdicts, inventory),
        files: &measurement.files,
        inventory,
    };
    let mut serialized = serde_json::to_string_pretty(&summary).unwrap_or_default();
    serialized.push('\n');
    serialized
}

/// Human-readable report: the table a reviewer reads in the job summary, plus
/// the files where the next tests would pay off most.
pub(super) fn render_summary_markdown(
    measurement: &Measurement,
    verdicts: &[MetricVerdict],
    inventory: Option<&InventoryReport>,
) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "## Coverage — {} (`{}`)\n",
        measurement.label, measurement.denominator
    );
    let _ = writeln!(
        out,
        "| Metric | Covered | Total | Measured | Baseline | Delta | Status |"
    );
    let _ = writeln!(out, "| --- | ---: | ---: | ---: | ---: | ---: | --- |");
    for verdict in verdicts {
        let (found, hit) = match verdict.metric {
            "lines" => (measurement.lines_found, measurement.lines_hit),
            _ => (measurement.functions_found, measurement.functions_hit),
        };
        let _ = writeln!(
            out,
            "| {} | {hit} | {found} | {:.2}% | {:.2}% | {:+.2} pp | {} |",
            verdict.metric,
            verdict.measured,
            verdict.baseline,
            verdict.delta(),
            status_word(&verdict.status),
        );
    }
    let _ = writeln!(
        out,
        "\n{} file(s) measured in this denominator.",
        measurement.files.len()
    );

    let mut worst: Vec<&FileCoverage> = measurement
        .files
        .iter()
        .filter(|file| file.lines_missed() > 0)
        .collect();
    worst.sort_by(|left, right| {
        right
            .lines_missed()
            .cmp(&left.lines_missed())
            .then_with(|| left.path.cmp(&right.path))
    });
    if !worst.is_empty() {
        let _ = writeln!(out, "\n### Least-covered files\n");
        let _ = writeln!(out, "| File | Uncovered lines | Line % |");
        let _ = writeln!(out, "| --- | ---: | ---: |");
        for file in worst.iter().take(10) {
            let _ = writeln!(
                out,
                "| `{}` | {} | {:.2}% |",
                file.path,
                file.lines_missed(),
                percent(file.lines_hit, file.lines_found),
            );
        }
    }

    if let Some(report) = inventory {
        let _ = writeln!(out, "\n### Unmeasured-file inventory\n");
        if report.is_clean() {
            let _ = writeln!(
                out,
                "Every production file is either measured or listed with a reason."
            );
        } else {
            for (title, rows) in [
                ("Neither measured nor declared", &report.undeclared),
                ("Declared but no longer present", &report.missing),
                ("Declared but now measured — prune the row", &report.stale),
                ("Declared without a reason", &report.unexplained),
            ] {
                if rows.is_empty() {
                    continue;
                }
                let _ = writeln!(out, "- **{title}**: {}", rows.join(", "));
            }
        }
    }
    out
}
