//! The committed ledger of honest upstream benchmark results (issue #698).
//!
//! `data/benchmarks/external-results.lino` records one `external_benchmark_suite`
//! record per upstream suite (provenance, license, and the monotonic
//! `minimum_pass_count` floor), one `external_benchmark_result` row per
//! scheduled run, and one `benchmark_unavailable` row whenever a suite could
//! not run — never a repository-local substitute.

use std::collections::BTreeMap;
use std::fmt::Write as _;

/// A `.lino` record: a bare id line plus two-space indented `key "value"` lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerRecord {
    pub name: String,
    pub fields: Vec<(String, String)>,
}

impl LedgerRecord {
    #[must_use]
    pub fn field(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }

    #[must_use]
    pub fn record_type(&self) -> &str {
        self.field("record_type").unwrap_or_default()
    }

    #[must_use]
    pub fn usize_field(&self, key: &str) -> Option<usize> {
        self.field(key).and_then(|value| value.parse().ok())
    }
}

/// A suite's provenance and its monotonic ratchet floor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiteEntry {
    pub id: String,
    pub license: String,
    pub minimum_pass_count: usize,
    pub ratchet_slice: usize,
}

/// One recorded run of one suite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultEntry {
    pub suite: String,
    pub date: String,
    pub slice: usize,
    pub passed: usize,
    pub failed: usize,
    pub total: usize,
    pub solver_version: String,
}

/// One recorded reason a suite could not run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnavailableEntry {
    pub suite: String,
    pub date: String,
    pub reason: String,
}

/// The parsed ledger.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ledger {
    pub records: Vec<LedgerRecord>,
}

impl Ledger {
    /// Parse the two-space indented `.lino` ledger.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut records: Vec<LedgerRecord> = Vec::new();
        for (number, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            if line.starts_with("  ") {
                let record = records
                    .last_mut()
                    .ok_or_else(|| format!("line {}: field before any record id", number + 1))?;
                let (key, value) = parse_field(line.trim()).ok_or_else(|| {
                    format!(
                        "line {}: expected `key \"value\"`, got `{line}`",
                        number + 1
                    )
                })?;
                record.fields.push((key, value));
            } else if line.starts_with(char::is_whitespace) {
                return Err(format!("line {}: unexpected indentation", number + 1));
            } else {
                records.push(LedgerRecord {
                    name: line.trim().to_string(),
                    fields: Vec::new(),
                });
            }
        }
        Ok(Self { records })
    }

    #[must_use]
    pub fn suites(&self) -> BTreeMap<String, SuiteEntry> {
        self.records
            .iter()
            .filter(|record| record.record_type() == "external_benchmark_suite")
            .filter_map(|record| {
                let id = record.field("id")?.to_string();
                Some((
                    id.clone(),
                    SuiteEntry {
                        id,
                        license: record.field("license").unwrap_or_default().to_string(),
                        minimum_pass_count: record.usize_field("minimum_pass_count")?,
                        ratchet_slice: record.usize_field("ratchet_slice")?,
                    },
                ))
            })
            .collect()
    }

    #[must_use]
    pub fn results(&self) -> Vec<ResultEntry> {
        self.records
            .iter()
            .filter(|record| record.record_type() == "external_benchmark_result")
            .filter_map(|record| {
                Some(ResultEntry {
                    suite: record.field("suite")?.to_string(),
                    date: record.field("date")?.to_string(),
                    slice: record.usize_field("slice")?,
                    passed: record.usize_field("passed")?,
                    failed: record.usize_field("failed")?,
                    total: record.usize_field("total")?,
                    solver_version: record
                        .field("solver_version")
                        .unwrap_or_default()
                        .to_string(),
                })
            })
            .collect()
    }

    #[must_use]
    pub fn unavailable(&self) -> Vec<UnavailableEntry> {
        self.records
            .iter()
            .filter(|record| record.record_type() == "benchmark_unavailable")
            .filter_map(|record| {
                Some(UnavailableEntry {
                    suite: record.field("suite")?.to_string(),
                    date: record.field("date")?.to_string(),
                    reason: record.field("reason")?.to_string(),
                })
            })
            .collect()
    }

    /// Render back to `.lino`, preserving record and field order.
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::new();
        for record in &self.records {
            let _ = writeln!(out, "{}", record.name);
            for (key, value) in &record.fields {
                let _ = writeln!(out, "  {key} \"{}\"", escape(value));
            }
        }
        out
    }

    /// Append a result row, replacing an existing row for the same suite, date
    /// and slice so a rerun on the same day stays idempotent.
    pub fn upsert_result(&mut self, entry: &ResultEntry, runner: &str, note: &str) {
        let name = format!(
            "external_benchmark_result_{}_{}_{}",
            entry.suite,
            entry.date.replace('-', "_"),
            entry.slice
        );
        let record = LedgerRecord {
            name: name.clone(),
            fields: vec![
                ("record_type".into(), "external_benchmark_result".into()),
                ("suite".into(), entry.suite.clone()),
                ("date".into(), entry.date.clone()),
                ("slice".into(), entry.slice.to_string()),
                ("passed".into(), entry.passed.to_string()),
                ("failed".into(), entry.failed.to_string()),
                ("total".into(), entry.total.to_string()),
                ("solver_version".into(), entry.solver_version.clone()),
                ("runner".into(), runner.to_string()),
                ("note".into(), note.to_string()),
            ],
        };
        self.replace_or_push(&name, record);
    }

    /// Append (or refresh) a `benchmark_unavailable` row.
    pub fn upsert_unavailable(&mut self, entry: &UnavailableEntry, evidence: &str) {
        let name = format!(
            "benchmark_unavailable_{}_{}",
            entry.suite,
            entry.date.replace('-', "_")
        );
        let record = LedgerRecord {
            name: name.clone(),
            fields: vec![
                ("record_type".into(), "benchmark_unavailable".into()),
                ("suite".into(), entry.suite.clone()),
                ("date".into(), entry.date.clone()),
                ("reason".into(), entry.reason.clone()),
                ("evidence".into(), evidence.to_string()),
            ],
        };
        self.replace_or_push(&name, record);
    }

    /// Raise a suite's ratchet floor. The floor never falls: a lower value is
    /// ignored so an unlucky rerun cannot erase a recorded pass count.
    pub fn raise_floor(&mut self, suite: &str, passed: usize, slice: usize) {
        for record in &mut self.records {
            if record.record_type() != "external_benchmark_suite"
                || record.field("id") != Some(suite)
            {
                continue;
            }
            let ratchet_slice = record.usize_field("ratchet_slice").unwrap_or(slice);
            if ratchet_slice != slice {
                return;
            }
            let current = record.usize_field("minimum_pass_count").unwrap_or(0);
            if passed > current {
                for (key, value) in &mut record.fields {
                    if key == "minimum_pass_count" {
                        *value = passed.to_string();
                    }
                }
            }
            return;
        }
    }

    fn replace_or_push(&mut self, name: &str, record: LedgerRecord) {
        if let Some(existing) = self
            .records
            .iter_mut()
            .find(|candidate| candidate.name == name)
        {
            *existing = record;
        } else {
            self.records.push(record);
        }
    }
}

fn parse_field(line: &str) -> Option<(String, String)> {
    let (key, rest) = line.split_once(' ')?;
    let value = rest.trim();
    let unquoted = value.strip_prefix('"')?.strip_suffix('"')?;
    Some((key.to_string(), unescape(unquoted)))
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn unescape(value: &str) -> String {
    value.replace("\\\"", "\"").replace("\\\\", "\\")
}
