#!/usr/bin/env rust-script
//! CI wall-clock ceiling check for the PR pipeline. (R1085-16)
//!
//! Discovers all PR-triggered workflows under .github/workflows/, computes the
//! critical path through each workflow's needs: DAG weighted by
//! timeout-minutes, handles reusable workflows (uses: ./.github/workflows/X)
//! by recursing, excludes jobs whose if: restricts them to push /
//! workflow_dispatch / release / schedule only (with no output-based escape),
//! and fails if any reachable job has no timeout-minutes (unbounded).
//!
//! The ceiling is read from data/meta/ci-wall-clock.lino. The measured maximum
//! must equal measured_minutes and must not exceed ceiling_minutes.
//!
//! Usage:
//!   rust-script scripts/check-ci-wall-clock.rs
//!   rust-script --test scripts/check-ci-wall-clock.rs
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! [dependencies]
//! serde_yaml = "0.9"
//! ```

use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::process::Command;

const WALL_CLOCK_FILE: &str = "data/meta/ci-wall-clock.lino";
const GITHUB_DEFAULT_TIMEOUT: u32 = 360;

// ─── .lino parser ───────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub struct WallClockRecord {
    pub measured_minutes: u32,
    pub ceiling_minutes: u32,
}

pub fn parse_wall_clock(content: &str) -> Result<WallClockRecord, String> {
    let mut measured = None;
    let mut ceiling = None;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with('#') || t.is_empty() {
            continue;
        }
        if let Some(v) = t.strip_prefix("measured_minutes ") {
            measured = v.trim().parse::<u32>().ok();
        }
        if let Some(v) = t.strip_prefix("ceiling_minutes ") {
            ceiling = v.trim().parse::<u32>().ok();
        }
    }
    match (measured, ceiling) {
        (Some(m), Some(c)) => Ok(WallClockRecord {
            measured_minutes: m,
            ceiling_minutes: c,
        }),
        _ => Err(format!(
            "{WALL_CLOCK_FILE}: missing measured_minutes or ceiling_minutes"
        )),
    }
}

// ─── YAML helpers ────────────────────────────────────────────────────────────

/// Return the value of the `on:` key (which YAML 1.1 parsers may read as
/// boolean `true`, while YAML 1.2 reads it as the string `"on"`).
fn get_on(wf: &Value) -> Option<&Value> {
    if let Some(v) = wf.get("on") {
        return Some(v);
    }
    // Some parsers represent the `on:` YAML key as a bool true.
    if let Value::Mapping(m) = wf {
        for (k, v) in m {
            if k == &Value::Bool(true) {
                return Some(v);
            }
        }
    }
    None
}

/// Returns true if the workflow has a `pull_request` trigger.
pub fn workflow_has_pr_trigger(wf: &Value) -> bool {
    let on = match get_on(wf) {
        Some(v) => v,
        None => return false,
    };
    match on {
        Value::String(s) => s == "pull_request",
        Value::Sequence(seq) => seq.iter().any(|v| v.as_str() == Some("pull_request")),
        Value::Mapping(m) => m
            .keys()
            .any(|k| k.as_str() == Some("pull_request")),
        _ => false,
    }
}

/// Returns true when a job's `if:` condition is restricted purely to non-PR
/// events and has no output-based alternative that would allow it to run on a
/// pull_request.
///
/// Logic: a job is excluded only when its condition contains one of the
/// push / workflow_dispatch / release / schedule event-name checks AND does
/// NOT contain `needs.X.outputs.` patterns (which make the job runnable on
/// any event that produces those outputs).
pub fn is_job_excluded_on_pr(if_condition: &str) -> bool {
    if if_condition.is_empty() {
        return false;
    }
    // Explicitly names pull_request: not excluded.
    if if_condition.contains("pull_request") {
        return false;
    }
    // Has output-based checks: can fire on any event including pull_request.
    if if_condition.contains(".outputs.") {
        return false;
    }
    // Condition names a non-PR event without any escape path.
    let non_pr = [
        "github.event_name == 'push'",
        "github.event_name == \"push\"",
        "github.event_name == 'workflow_dispatch'",
        "github.event_name == \"workflow_dispatch\"",
        "github.event_name == 'release'",
        "github.event_name == \"release\"",
        "github.event_name == 'schedule'",
        "github.event_name == \"schedule\"",
    ];
    non_pr.iter().any(|pat| if_condition.contains(pat))
}

/// Try to resolve a `${{ matrix.X }}` expression to the maximum integer value
/// found across the job's `strategy.matrix.include` rows.
/// Returns `None` when the matrix is dynamic or the key is nested (`matrix.X.Y`).
pub fn resolve_matrix_timeout(t_expr: &str, job: &Value) -> Option<u32> {
    let trimmed = t_expr.trim();
    if !trimmed.starts_with("${{") || !trimmed.ends_with("}}") {
        return None;
    }
    let inner = trimmed
        .trim_start_matches("${{")
        .trim_end_matches("}}")
        .trim();
    let key = inner.strip_prefix("matrix.")?;
    // Nested like matrix.leg.cap → dynamic, can't resolve.
    if key.contains('.') {
        return None;
    }
    let include = job
        .get("strategy")
        .and_then(|s| s.get("matrix"))
        .and_then(|m| m.get("include"))?;
    let vals: Vec<u32> = include
        .as_sequence()?
        .iter()
        .filter_map(|row| {
            row.get(key)
                .and_then(|v| v.as_u64())
                .map(|n| n as u32)
        })
        .collect();
    vals.into_iter().max()
}

// ─── Needs-DAG critical-path computation ────────────────────────────────────

type ReusableCache = HashMap<String, (u32, Vec<String>)>;

/// Return the weight (minutes) and label string for a single job.
pub fn job_weight(
    job: &Value,
    job_name: &str,
    workflows_dir: &Path,
    cache: &mut ReusableCache,
    depth: u32,
) -> (u32, String) {
    // Reusable workflow: recurse into it (or contribute 0 if not found).
    if let Some(uses_val) = job.get("uses") {
        let uses = uses_val.as_str().unwrap_or("");
        let rw_path = resolve_reusable_path(uses, workflows_dir);
        let (cp, label) = match rw_path {
            Some(p) => {
                let (c, _) = reusable_critical_path(&p, workflows_dir, cache, depth);
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                (c, format!("{job_name}(->reusable:{name}:{c}m)"))
            }
            None => (0, format!("{job_name}(->unresolved:{uses}:0m)")),
        };
        return (cp, label);
    }
    // Static or matrix-expression timeout.
    match job.get("timeout-minutes") {
        None => (
            GITHUB_DEFAULT_TIMEOUT,
            format!("{job_name}(UNBOUNDED->{GITHUB_DEFAULT_TIMEOUT}m)"),
        ),
        Some(t_val) => {
            if let Some(n) = t_val.as_u64() {
                (n as u32, format!("{job_name}({}m)", n))
            } else if let Some(expr) = t_val.as_str() {
                if let Some(resolved) = resolve_matrix_timeout(expr, job) {
                    (resolved, format!("{job_name}(matrix-max:{resolved}m)"))
                } else {
                    // Truly dynamic expression: treat as unbounded.
                    (
                        GITHUB_DEFAULT_TIMEOUT,
                        format!("{job_name}(DYNAMIC:{expr}->{GITHUB_DEFAULT_TIMEOUT}m)"),
                    )
                }
            } else {
                (
                    GITHUB_DEFAULT_TIMEOUT,
                    format!("{job_name}(UNBOUNDED->{GITHUB_DEFAULT_TIMEOUT}m)"),
                )
            }
        }
    }
}

/// Resolve a `uses: ./.github/workflows/x.yml` reference to an absolute path.
fn resolve_reusable_path(uses: &str, workflows_dir: &Path) -> Option<PathBuf> {
    if uses.starts_with("./") {
        let candidate = workflows_dir
            .parent()
            .and_then(|p| p.parent())
            .map(|repo| repo.join(&uses[2..]))?;
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

/// Compute the critical path of a reusable workflow, with caching.
fn reusable_critical_path(
    wf_path: &Path,
    workflows_dir: &Path,
    cache: &mut ReusableCache,
    depth: u32,
) -> (u32, Vec<String>) {
    let key = wf_path.to_string_lossy().into_owned();
    if let Some(cached) = cache.get(&key) {
        return cached.clone();
    }
    if depth > 5 {
        return (0, vec!["(max recursion depth)".to_string()]);
    }
    let result = fs::read_to_string(wf_path)
        .ok()
        .and_then(|content| serde_yaml::from_str::<Value>(&content).ok())
        .map(|wf| {
            let jobs = wf.get("jobs").cloned().unwrap_or(Value::Null);
            let (cp, chain) = critical_path_of_jobs(&jobs, workflows_dir, cache, depth + 1);
            let name = wf_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?");
            let prefixed: Vec<String> = chain.iter().map(|c| format!("{name}::{c}")).collect();
            (cp, prefixed)
        })
        .unwrap_or((0, vec![format!("(could not parse {})", wf_path.display())]));
    cache.insert(key, result.clone());
    result
}

/// Compute the critical path through the jobs' needs: DAG.
/// Returns (max_minutes, chain_of_labels).
pub fn critical_path_of_jobs(
    jobs: &Value,
    workflows_dir: &Path,
    cache: &mut ReusableCache,
    depth: u32,
) -> (u32, Vec<String>) {
    let mapping = match jobs.as_mapping() {
        Some(m) => m,
        None => return (0, vec![]),
    };
    // Build needs map.
    let mut needs_map: HashMap<String, Vec<String>> = HashMap::new();
    for (k, v) in mapping {
        let jname = match k.as_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        let needs = parse_needs(v);
        needs_map.insert(jname, needs);
    }
    // Topological sort.
    let topo = topological_sort(&needs_map);
    // DP over topological order.
    let mut dist: HashMap<String, (u32, Vec<String>)> = HashMap::new();
    for jname in &topo {
        let jbody = match mapping.get(jname.as_str()) {
            Some(v) => v,
            None => continue,
        };
        // Exclude push-only jobs.
        let if_cond = jbody
            .get("if")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if is_job_excluded_on_pr(if_cond) {
            dist.insert(jname.clone(), (0, vec![]));
            continue;
        }
        let (w, label) = job_weight(jbody, jname, workflows_dir, cache, depth);
        // Best predecessor.
        let (max_pred, max_chain) = needs_map
            .get(jname)
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|dep| dist.get(dep))
            .max_by_key(|(d, _)| *d)
            .cloned()
            .unwrap_or((0, vec![]));
        let mut chain = max_chain;
        chain.push(label);
        dist.insert(jname.clone(), (max_pred + w, chain));
    }
    dist.into_values()
        .max_by_key(|(d, _)| *d)
        .unwrap_or((0, vec![]))
}

/// Parse the `needs:` field from a job value into a Vec of job names.
fn parse_needs(job: &Value) -> Vec<String> {
    match job.get("needs") {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Sequence(seq)) => seq
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => vec![],
    }
}

/// Kahn's algorithm for topological sort.
fn topological_sort(needs_map: &HashMap<String, Vec<String>>) -> Vec<String> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    for (n, deps) in needs_map {
        in_degree.entry(n).or_insert(0);
        for d in deps {
            if needs_map.contains_key(d.as_str()) {
                *in_degree.entry(n).or_insert(0) += 0; // ensure entry
                *in_degree.entry(n.as_str()).or_insert(0) += 0; // noop
            }
        }
    }
    // Rebuild properly.
    let mut in_deg: HashMap<String, usize> = needs_map.keys().map(|k| (k.clone(), 0)).collect();
    for (n, deps) in needs_map {
        for d in deps {
            if needs_map.contains_key(d.as_str()) {
                *in_deg.entry(n.clone()).or_insert(0) += 1;
            }
        }
    }
    let mut queue: std::collections::VecDeque<String> = in_deg
        .iter()
        .filter(|&(_, &d)| d == 0)
        .map(|(k, _)| k.clone())
        .collect();
    // Sort for determinism.
    let mut sorted_queue: Vec<String> = queue.drain(..).collect();
    sorted_queue.sort();
    let mut queue: std::collections::VecDeque<String> = sorted_queue.into();
    let mut result = Vec::new();
    while let Some(n) = queue.pop_front() {
        result.push(n.clone());
        // Find nodes that depend on n (n is a dep of them).
        let mut next: Vec<String> = needs_map
            .iter()
            .filter(|(_, deps)| deps.contains(&n))
            .map(|(k, _)| k.clone())
            .collect();
        next.sort();
        for m in next {
            if let Some(d) = in_deg.get_mut(&m) {
                *d = d.saturating_sub(1);
                if *d == 0 {
                    queue.push_back(m);
                }
            }
        }
    }
    // Append any remaining (cycle safety).
    for k in needs_map.keys() {
        if !result.contains(k) {
            result.push(k.clone());
        }
    }
    result
}

// ─── Unbounded-job detection ─────────────────────────────────────────────────

/// Find PR-reachable jobs with no timeout-minutes in a workflow.
/// Returns job names (for diagnostic output).
pub fn find_unbounded_jobs(jobs: &Value) -> Vec<String> {
    let mapping = match jobs.as_mapping() {
        Some(m) => m,
        None => return vec![],
    };
    let mut unbounded = Vec::new();
    for (k, v) in mapping {
        let jname = match k.as_str() {
            Some(s) => s,
            None => continue,
        };
        let if_cond = v.get("if").and_then(|c| c.as_str()).unwrap_or("");
        if is_job_excluded_on_pr(if_cond) {
            continue;
        }
        // Reusable workflows delegate timeout to their own jobs.
        if v.get("uses").is_some() {
            continue;
        }
        match v.get("timeout-minutes") {
            None => unbounded.push(jname.to_string()),
            Some(t) => {
                // Check for truly dynamic expression (not resolvable from static include).
                if let Some(expr) = t.as_str() {
                    if resolve_matrix_timeout(expr, v).is_none() {
                        unbounded.push(format!("{jname}(DYNAMIC:{expr})"));
                    }
                }
            }
        }
    }
    unbounded
}

// ─── Main entry point ────────────────────────────────────────────────────────

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
    let wall_clock_path = root.join(WALL_CLOCK_FILE);
    let wall_clock_content = match fs::read_to_string(&wall_clock_path) {
        Ok(c) => c,
        Err(e) => {
            println!("::error::{WALL_CLOCK_FILE}: {e}");
            std::process::exit(1);
        }
    };
    let record = match parse_wall_clock(&wall_clock_content) {
        Ok(r) => r,
        Err(e) => {
            println!("::error::{e}");
            std::process::exit(1);
        }
    };

    let workflows_dir = root.join(".github/workflows");
    let mut cache: ReusableCache = HashMap::new();
    let mut all_unbounded: Vec<(String, String)> = Vec::new();
    let mut results: Vec<(u32, Vec<String>, String)> = Vec::new(); // (minutes, chain, wf_name)

    let mut wf_files: Vec<_> = fs::read_dir(&workflows_dir)
        .expect("read workflows dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("yml"))
        .collect();
    wf_files.sort();

    for wf_file in &wf_files {
        let content = match fs::read_to_string(wf_file) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let wf: Value = match serde_yaml::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if !workflow_has_pr_trigger(&wf) {
            continue;
        }
        let jobs = wf.get("jobs").cloned().unwrap_or(Value::Null);
        let ub = find_unbounded_jobs(&jobs);
        if !ub.is_empty() {
            let wf_name = wf_file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            for j in ub {
                all_unbounded.push((wf_name.clone(), j));
            }
        }
        let (cp, chain) = critical_path_of_jobs(&jobs, &workflows_dir, &mut cache, 0);
        let wf_name = wf_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        results.push((cp, chain, wf_name));
    }

    // Fail on unbounded jobs.
    if !all_unbounded.is_empty() {
        for (wf, job) in &all_unbounded {
            println!(
                "::error::{wf}: job `{job}` is reachable on pull_request with no \
                 timeout-minutes; add a timeout or restrict the job to non-PR events"
            );
        }
        std::process::exit(1);
    }

    // Find the maximum critical path.
    let max = results.iter().max_by_key(|(cp, _, _)| *cp);
    let (measured, chain_str, wf_name) = match max {
        Some((cp, chain, wf)) => (*cp, chain.join(" -> "), wf.as_str()),
        None => {
            println!("::error::no PR-triggered workflows found under .github/workflows/");
            std::process::exit(1);
        }
    };

    // Verify against recorded ceiling.
    if measured != record.measured_minutes {
        println!(
            "::error::{WALL_CLOCK_FILE}: measured_minutes is {}, but the discovered \
             maximum is {measured} min (from {wf_name}: {chain_str}); \
             update measured_minutes to {measured}",
            record.measured_minutes
        );
        std::process::exit(1);
    }
    if measured > record.ceiling_minutes {
        println!(
            "::error::{WALL_CLOCK_FILE}: measured_minutes ({measured}) exceeds \
             ceiling_minutes ({}); reduce job timeouts or raise the ceiling",
            record.ceiling_minutes
        );
        std::process::exit(1);
    }
    println!(
        "CI wall-clock ceiling ok: {wf_name} critical path {measured} min \
         ({chain_str}) <= ceiling {} min.",
        record.ceiling_minutes
    );
}

// ─── Unit tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Inline YAML fixtures.
    const FIXTURE_MAIN: &str = r#"
on:
  pull_request:
  push:

jobs:
  setup:
    timeout-minutes: 5
    runs-on: ubuntu-latest
    steps: []

  build:
    needs: [setup]
    timeout-minutes: 20
    runs-on: ubuntu-latest
    steps: []

  test:
    needs: [setup, build]
    timeout-minutes: 45
    runs-on: ubuntu-latest
    steps: []

  deploy:
    # push-only: excluded from PR critical path
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    needs: [test]
    timeout-minutes: 10
    runs-on: ubuntu-latest
    steps: []

  reuse-job:
    # calls a reusable workflow
    needs: [setup]
    uses: ./.github/workflows/fixture-reusable.yml

  finish:
    needs: [test, reuse-job, deploy]
    timeout-minutes: 5
    runs-on: ubuntu-latest
    steps: []
"#;

    const FIXTURE_REUSABLE: &str = r#"
on:
  workflow_call:

jobs:
  inner:
    timeout-minutes: 30
    runs-on: ubuntu-latest
    steps: []
"#;

    const FIXTURE_UNBOUNDED: &str = r#"
on:
  pull_request:

jobs:
  check:
    timeout-minutes: 10
    runs-on: ubuntu-latest
    steps: []

  no-timeout-job:
    # no timeout-minutes → unbounded → gate must fail
    needs: [check]
    runs-on: ubuntu-latest
    steps: []
"#;

    fn parse_yaml(s: &str) -> Value {
        serde_yaml::from_str(s).expect("fixture YAML parse")
    }

    #[test]
    fn workflow_has_pr_trigger_recognises_pr() {
        let wf = parse_yaml(FIXTURE_MAIN);
        assert!(workflow_has_pr_trigger(&wf));
    }

    #[test]
    fn workflow_has_pr_trigger_rejects_push_only() {
        let wf = parse_yaml(
            "on:\n  push:\n\njobs:\n  x:\n    timeout-minutes: 5\n    runs-on: ubuntu-latest\n",
        );
        assert!(!workflow_has_pr_trigger(&wf));
    }

    #[test]
    fn push_only_job_is_excluded() {
        assert!(is_job_excluded_on_pr(
            "github.event_name == 'push' && github.ref == 'refs/heads/main'"
        ));
    }

    #[test]
    fn job_with_output_check_is_not_excluded() {
        // Has workflow_dispatch but also an output check → can run on PR.
        assert!(!is_job_excluded_on_pr(
            "!cancelled() && (\n  github.event_name == 'workflow_dispatch' ||\n  \
             needs.detect-changes.outputs.any-code-changed == 'true'\n)\n"
        ));
    }

    #[test]
    fn job_with_pull_request_is_not_excluded() {
        assert!(!is_job_excluded_on_pr(
            "github.event_name == 'pull_request' && github.ref == 'refs/heads/main'"
        ));
    }

    #[test]
    fn workflow_dispatch_only_job_is_excluded() {
        assert!(is_job_excluded_on_pr(
            "github.event_name == 'workflow_dispatch' && github.event.inputs.mode == 'release'"
        ));
    }

    #[test]
    fn resolve_matrix_timeout_finds_max() {
        let job_yaml = r#"
timeout-minutes: "${{ matrix.capmin }}"
strategy:
  matrix:
    include:
      - {os: ubuntu, capmin: 40}
      - {os: macos, capmin: 50}
      - {os: windows, capmin: 50}
"#;
        let job = parse_yaml(job_yaml);
        let t = job
            .get("timeout-minutes")
            .and_then(|v| v.as_str())
            .unwrap();
        assert_eq!(resolve_matrix_timeout(t, &job), Some(50));
    }

    #[test]
    fn resolve_matrix_timeout_returns_none_for_nested_key() {
        // ${{ matrix.leg.cap }} — nested key, can't resolve statically.
        let job = parse_yaml("timeout-minutes: \"${{ matrix.leg.cap }}\"\n");
        let t = job
            .get("timeout-minutes")
            .and_then(|v| v.as_str())
            .unwrap();
        assert_eq!(resolve_matrix_timeout(t, &job), None);
    }

    #[test]
    fn find_unbounded_jobs_detects_missing_timeout() {
        let wf = parse_yaml(FIXTURE_UNBOUNDED);
        let jobs = wf.get("jobs").unwrap();
        let ub = find_unbounded_jobs(jobs);
        assert!(ub.contains(&"no-timeout-job".to_string()), "got: {ub:?}");
        assert!(!ub.contains(&"check".to_string()), "check should be bounded");
    }

    #[test]
    fn find_unbounded_jobs_skips_push_only() {
        let yaml = r#"
jobs:
  push-only:
    if: "github.event_name == 'push'"
    runs-on: ubuntu-latest
    steps: []
"#;
        let wf = parse_yaml(yaml);
        let jobs = wf.get("jobs").unwrap();
        assert!(find_unbounded_jobs(jobs).is_empty());
    }

    #[test]
    fn critical_path_follows_needs_dag() {
        let wf = parse_yaml(FIXTURE_MAIN);
        let jobs = wf.get("jobs").unwrap();
        let workflows_dir = PathBuf::from(".github/workflows");
        let mut cache = ReusableCache::new();
        let (cp, chain) = critical_path_of_jobs(jobs, &workflows_dir, &mut cache, 0);
        // Expected chain (ignoring reuse-job path which calls a reusable workflow
        // from a non-existent file in tests → 0m):
        // setup(5) -> build(20) -> test(45) -> finish(5) = 75m
        // deploy is excluded (push-only).
        // reuse-job → fixture-reusable.yml doesn't exist → 0m.
        assert_eq!(cp, 75, "chain: {chain:?}");
    }

    #[test]
    fn reusable_workflow_contributes_its_own_critical_path() {
        // Write fixtures to a temporary directory so the reusable-workflow
        // path resolution can find fixture-reusable.yml on disk.
        use std::fs;
        let dir = std::env::temp_dir().join(format!("check-ci-wall-clock-{}", std::process::id()));
        let wf_dir = dir.join(".github").join("workflows");
        fs::create_dir_all(&wf_dir).unwrap();
        fs::write(wf_dir.join("fixture-reusable.yml"), FIXTURE_REUSABLE).unwrap();
        let wf = parse_yaml(FIXTURE_MAIN);
        let jobs = wf.get("jobs").unwrap();
        let mut cache = ReusableCache::new();
        let (cp, chain) = critical_path_of_jobs(jobs, &wf_dir, &mut cache, 0);
        let _ = fs::remove_dir_all(&dir);
        // reuse-job: setup(5)+inner(30m)+finish(5)=40m.
        // Critical path via test: setup(5)+build(20)+test(45)+finish(5)=75m.
        assert_eq!(cp, 75, "chain: {chain:?}");
    }

    #[test]
    fn parse_wall_clock_reads_measured_and_ceiling() {
        let content =
            "ci_wall_clock pr\n  measured_minutes 225\n  ceiling_minutes 225\n  workflow \".github/workflows/issue-1028-agent-ladder.yml\"\n";
        let r = parse_wall_clock(content).unwrap();
        assert_eq!(r.measured_minutes, 225);
        assert_eq!(r.ceiling_minutes, 225);
    }

    #[test]
    fn parse_wall_clock_rejects_missing_fields() {
        let content = "ci_wall_clock pr\n  measured_minutes 100\n";
        assert!(parse_wall_clock(content).is_err());
    }

    #[test]
    fn parse_wall_clock_rejects_missing_measured() {
        let content = "ci_wall_clock pr\n  ceiling_minutes 100\n";
        assert!(parse_wall_clock(content).is_err());
    }
}
