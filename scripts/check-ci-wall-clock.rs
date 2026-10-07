#!/usr/bin/env rust-script
//! The pull-request pipeline's wall-clock ceiling is recorded and respected.
//!
//! R1085-16 item 5. The critical path through PR-triggered jobs is the sum of
//! `timeout-minutes` along the named chain of jobs in
//! `data/meta/ci-wall-clock.lino`. The ceiling in that file must not be
//! exceeded by the sum of the named jobs' timeouts from
//! `.github/workflows/release.yml`.
//!
//! The `critical_path_jobs` row names the jobs on the critical path (those that
//! run on pull_request events and form the longest serial chain). Only those
//! jobs' timeouts are summed; push-only jobs such as auto-release, manual-release
//! and deploy-pages are excluded.
//!
//! Usage:
//!   rust-script scripts/check-ci-wall-clock.rs
//!   rust-script --test scripts/check-ci-wall-clock.rs
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const WALL_CLOCK_FILE: &str = "data/meta/ci-wall-clock.lino";
const WORKFLOW_FILE: &str = ".github/workflows/release.yml";

/// Parse the wall-clock record. Returns (measured_minutes, ceiling_minutes, critical_path_jobs).
fn parse_wall_clock(content: &str) -> Result<(u32, u32, Vec<String>), String> {
    let mut measured: Option<u32> = None;
    let mut ceiling: Option<u32> = None;
    let mut jobs: Vec<String> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(v) = trimmed.strip_prefix("measured_minutes ") {
            measured = v.trim().parse().ok();
        }
        if let Some(v) = trimmed.strip_prefix("ceiling_minutes ") {
            ceiling = v.trim().parse().ok();
        }
        if let Some(v) = trimmed.strip_prefix("critical_path_jobs ") {
            let raw = v.trim().trim_matches('"');
            jobs = raw.split_whitespace().map(|s| s.to_string()).collect();
        }
    }
    match (measured, ceiling) {
        (Some(m), Some(c)) if !jobs.is_empty() => Ok((m, c, jobs)),
        (None, _) | (_, None) => Err(format!(
            "{WALL_CLOCK_FILE}: expected `measured_minutes` and `ceiling_minutes` rows"
        )),
        _ => Err(format!(
            "{WALL_CLOCK_FILE}: expected a `critical_path_jobs` row listing the PR-triggered jobs"
        )),
    }
}

/// Parse `timeout-minutes` values for top-level jobs from a workflow YAML.
/// Returns a map from job id to timeout. Takes the first timeout-minutes seen
/// per job (the job-level timeout, not inner step timeouts).
pub fn parse_job_timeouts(yaml: &str) -> HashMap<String, u32> {
    let mut map = HashMap::new();
    let mut current_job: Option<String> = None;
    let mut in_jobs = false;
    for line in yaml.lines() {
        if line == "jobs:" {
            in_jobs = true;
            continue;
        }
        if !in_jobs {
            continue;
        }
        // A two-space-indented line ending in ':' starting with a letter is a job id.
        if line.starts_with("  ") && !line.starts_with("   ") {
            let trimmed = line.trim();
            if trimmed.ends_with(':')
                && trimmed
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            {
                current_job = Some(trimmed.trim_end_matches(':').to_string());
            }
        }
        if let Some(job) = &current_job {
            if !map.contains_key(job) {
                if let Some(rest) = line.trim().strip_prefix("timeout-minutes:") {
                    if let Ok(n) = rest.trim().parse::<u32>() {
                        map.insert(job.clone(), n);
                    }
                }
            }
        }
    }
    map
}

/// Sum the timeout-minutes for the named critical path jobs. Returns the sum
/// and a list of any job names not found in the workflow.
pub fn critical_path_sum(
    jobs: &[String],
    timeouts: &HashMap<String, u32>,
) -> (u32, Vec<String>) {
    let mut total = 0u32;
    let mut missing = Vec::new();
    for job in jobs {
        match timeouts.get(job) {
            Some(&t) => total = total.saturating_add(t),
            None => missing.push(job.clone()),
        }
    }
    (total, missing)
}

#[cfg(not(test))]
fn repository_root() -> PathBuf {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .expect("git rev-parse");
    PathBuf::from(String::from_utf8_lossy(&output.stdout).trim())
}

#[cfg(not(test))]
fn main() {
    let root = repository_root();
    let wall_clock_content = match fs::read_to_string(root.join(WALL_CLOCK_FILE)) {
        Ok(c) => c,
        Err(e) => {
            println!("::error::{WALL_CLOCK_FILE}: {e}");
            std::process::exit(1);
        }
    };
    let (measured, ceiling, jobs) = match parse_wall_clock(&wall_clock_content) {
        Ok(v) => v,
        Err(e) => {
            println!("::error::{e}");
            std::process::exit(1);
        }
    };
    if measured > ceiling {
        println!(
            "::error::{WALL_CLOCK_FILE}: measured_minutes ({measured}) exceeds \
             ceiling_minutes ({ceiling}); update the ceiling or reduce job timeouts"
        );
        std::process::exit(1);
    }
    let workflow_content = match fs::read_to_string(root.join(WORKFLOW_FILE)) {
        Ok(c) => c,
        Err(e) => {
            println!("::error::{WORKFLOW_FILE}: {e}");
            std::process::exit(1);
        }
    };
    let timeouts = parse_job_timeouts(&workflow_content);
    let (path_sum, missing) = critical_path_sum(&jobs, &timeouts);
    if !missing.is_empty() {
        println!(
            "::error::{WALL_CLOCK_FILE}: critical_path_jobs names jobs not found in \
             {WORKFLOW_FILE}: {}",
            missing.join(", ")
        );
        std::process::exit(1);
    }
    if path_sum > ceiling {
        println!(
            "::error::The sum of critical-path job timeouts is {path_sum} min ({}), \
             which exceeds the recorded ceiling of {ceiling} min in {WALL_CLOCK_FILE}. \
             Update ceiling_minutes and measured_minutes, or reduce a job timeout.",
            jobs.iter()
                .map(|j| format!("{j}({})", timeouts.get(j).copied().unwrap_or(0)))
                .collect::<Vec<_>>()
                .join("+")
        );
        std::process::exit(1);
    }
    println!(
        "CI wall-clock ceiling ok: critical path {path_sum} min ({}) <= ceiling {ceiling} min.",
        jobs.iter()
            .map(|j| format!("{j}({})", timeouts.get(j).copied().unwrap_or(0)))
            .collect::<Vec<_>>()
            .join("+")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_wall_clock_reads_all_fields() {
        let content = "ci_wall_clock pr\n  measured_minutes 140\n  ceiling_minutes 140\n  \
            critical_path_jobs \"a b c\"\n";
        let (m, c, jobs) = parse_wall_clock(content).unwrap();
        assert_eq!(m, 140);
        assert_eq!(c, 140);
        assert_eq!(jobs, vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_wall_clock_rejects_missing_ceiling() {
        let content = "ci_wall_clock pr\n  measured_minutes 140\n  critical_path_jobs \"a\"\n";
        assert!(parse_wall_clock(content).is_err());
    }

    #[test]
    fn parse_wall_clock_rejects_missing_jobs() {
        let content = "ci_wall_clock pr\n  measured_minutes 140\n  ceiling_minutes 140\n";
        assert!(parse_wall_clock(content).is_err());
    }

    #[test]
    fn critical_path_sum_adds_known_jobs() {
        let mut timeouts = HashMap::new();
        timeouts.insert("a".to_string(), 5u32);
        timeouts.insert("b".to_string(), 20u32);
        timeouts.insert("c".to_string(), 90u32);
        let jobs: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let (sum, missing) = critical_path_sum(&jobs, &timeouts);
        assert_eq!(sum, 115);
        assert!(missing.is_empty());
    }

    #[test]
    fn critical_path_sum_reports_missing_jobs() {
        let timeouts: HashMap<String, u32> = HashMap::new();
        let jobs = vec!["unknown".to_string()];
        let (_, missing) = critical_path_sum(&jobs, &timeouts);
        assert_eq!(missing, vec!["unknown".to_string()]);
    }

    #[test]
    fn parse_job_timeouts_finds_job_level_entries() {
        let yaml = "jobs:\n  detect-changes:\n    timeout-minutes: 5\n  \
            test:\n    timeout-minutes: 90\n        timeout-minutes: 45\n";
        let t = parse_job_timeouts(yaml);
        // The job-level timeout (first seen) is 5 for detect-changes and 90 for test.
        assert_eq!(t.get("detect-changes"), Some(&5));
        assert_eq!(t.get("test"), Some(&90));
    }

    #[test]
    fn the_recorded_ceiling_covers_the_real_workflow() {
        // Read the actual release.yml and ci-wall-clock.lino from the repository
        // root and verify the recorded critical path is not exceeded.
        // This is the integration assertion that runs in CI.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let wall_clock_path = root.join(WALL_CLOCK_FILE);
        let workflow_path = root.join(WORKFLOW_FILE);
        if !wall_clock_path.exists() || !workflow_path.exists() {
            // Skip when run outside the repository.
            return;
        }
        let wall_clock_content = fs::read_to_string(&wall_clock_path).unwrap();
        let (measured, ceiling, jobs) = parse_wall_clock(&wall_clock_content).unwrap();
        assert!(
            measured <= ceiling,
            "measured_minutes ({measured}) exceeds ceiling_minutes ({ceiling})"
        );
        let workflow_content = fs::read_to_string(&workflow_path).unwrap();
        let timeouts = parse_job_timeouts(&workflow_content);
        let (path_sum, missing) = critical_path_sum(&jobs, &timeouts);
        assert!(
            missing.is_empty(),
            "critical_path_jobs names jobs not in workflow: {missing:?}"
        );
        assert!(
            path_sum <= ceiling,
            "critical path sum {path_sum} min exceeds ceiling {ceiling} min"
        );
    }
}
