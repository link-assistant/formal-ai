//! Resolve file-scoped release defaults without changing historical measurements.

use std::path::Path;

use super::{ReleaseRow, ledger_storage};

pub fn read_release_rows(ledger: &Path) -> Result<Vec<ReleaseRow>, String> {
    let text = ledger_storage::read(ledger)
        .map_err(|error| format!("could not read {}: {error}", ledger.display()))?;
    let mut rows = Vec::new();
    let mut fields = Vec::new();
    let mut defaults = Vec::new();
    let mut in_defaults = false;
    let mut seen_defaults = false;
    for line in text.lines() {
        if line.trim() == "release"
            || matches!(line, "self_hosting_ledger" | "self-hosting-ledger-archive")
        {
            if !fields.is_empty() {
                rows.push(parse_release_row(&fields, &defaults)?);
                fields.clear();
            }
            if !line.starts_with("  ") {
                defaults.clear();
                seen_defaults = false;
            }
            in_defaults = false;
        } else if line == "  release-defaults" {
            if !fields.is_empty() || seen_defaults {
                return Err("release defaults must precede their records".to_owned());
            }
            in_defaults = true;
            seen_defaults = true;
        } else if line.starts_with("    ") {
            if in_defaults {
                defaults.push(line.trim().to_owned());
            } else {
                fields.push(line.trim().to_owned());
            }
        } else if line.starts_with("  ") {
            in_defaults = false;
        }
    }
    if !fields.is_empty() {
        rows.push(parse_release_row(&fields, &defaults)?);
    }
    Ok(rows)
}

fn parse_release_row(fields: &[String], defaults: &[String]) -> Result<ReleaseRow, String> {
    let value = |key: &str| -> Result<String, String> {
        let prefix = format!("{key} \"");
        fields
            .iter()
            .chain(defaults)
            .find_map(|field| {
                field
                    .strip_prefix(&prefix)
                    .and_then(|value| value.strip_suffix('"'))
                    .map(str::to_owned)
            })
            .ok_or_else(|| format!("release row is missing {key}"))
    };
    let number = |key: &str| -> Result<u64, String> {
        value(key)?
            .parse::<u64>()
            .map_err(|error| format!("release row has invalid {key}: {error}"))
    };
    let optional_number = |key: &str| -> Result<Option<u64>, String> {
        let Ok(raw) = value(key) else {
            return Ok(None);
        };
        raw.parse::<u64>()
            .map(Some)
            .map_err(|error| format!("release row has invalid {key}: {error}"))
    };
    Ok(ReleaseRow {
        // Explicit file-scoped defaults share structure. Legacy files without
        // a marker retain epoch 1; a row's own fields always take precedence.
        metric_version: optional_number("metric_version")?.unwrap_or(1),
        tag: value("tag")?,
        since: value("since")?,
        until: value("until")?,
        self_authored_lines: number("self_authored_lines")?,
        changed_lines: number("changed_lines")?,
        self_authored_commits: number("self_authored_commits")?,
        commits: number("commits")?,
        percentage_basis_points: number("percentage_basis_points")?,
        trailing_window: usize::try_from(number("trailing_window")?)
            .map_err(|_| "trailing_window does not fit usize".to_owned())?,
        trailing_percentage_basis_points: number("trailing_percentage_basis_points")?,
        target_percentage_basis_points: optional_number("target_percentage_basis_points")?,
        target_override_basis_points: optional_number("target_override_basis_points")?,
        self_authored_pull_requests: fields
            .iter()
            .filter_map(|field| {
                field
                    .strip_prefix("self_authored_pull_request \"")
                    .and_then(|value| value.strip_suffix('"'))
                    .map(str::to_owned)
            })
            .collect(),
        self_authored_pull_request_authors: fields
            .iter()
            .filter_map(|field| {
                field
                    .strip_prefix("self_authored_pull_request_author \"")
                    .and_then(|value| value.strip_suffix('"'))
                    .map(str::to_owned)
            })
            .collect(),
    })
}
