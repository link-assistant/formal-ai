#!/usr/bin/env rust-script
//! Fast consistency gate for the standalone issue #848 coding ladder.
//!
//! This reads the committed full result, ratchet, and workflow; it never runs
//! the 130 authoring tasks. The standalone workflow performs that expensive
//! measurement and invokes this checker against its temporary result.
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const RESULT: &str = "experiments/issue_847_coding_ladder/results.json";
const RATCHET: &str = "data/meta/ladder-ratchet.lino";
const WORKFLOW: &str = ".github/workflows/coding-ladder.yml";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Measurement {
    total: u64,
    passed: u64,
    l1_total: u64,
    l1_passed: u64,
}

fn number_after(text: &str, key: &str) -> Result<u64, String> {
    let marker = format!("\"{key}\"");
    let tail = text
        .find(&marker)
        .map(|index| &text[index + marker.len()..])
        .ok_or_else(|| format!("missing JSON field `{key}`"))?;
    let tail = tail
        .split_once(':')
        .map(|(_, value)| value.trim_start())
        .ok_or_else(|| format!("JSON field `{key}` has no value"))?;
    let digits = tail
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>();
    digits
        .parse()
        .map_err(|error| format!("JSON field `{key}`: {error}"))
}

fn full_result(text: &str) -> Result<Measurement, String> {
    for required in [
        "\"dataset_total\": 130",
        "\"measured_total\": 130",
        "\"complete\": true",
    ] {
        if !text.contains(required) {
            return Err(format!(
                "the committed result is not the full 130-task run: missing `{required}`"
            ));
        }
    }
    let summary = text
        .split_once("\"summary\"")
        .map(|(_, tail)| tail)
        .ok_or_else(|| "results have no summary".to_owned())?;
    let before_results = summary
        .split_once("\"results\"")
        .map(|(head, _)| head)
        .ok_or_else(|| "results have no task list after the summary".to_owned())?;
    let l1 = before_results
        .split_once("\"L1\"")
        .map(|(_, tail)| tail)
        .ok_or_else(|| "summary has no visible L1 row".to_owned())?;
    Ok(Measurement {
        total: number_after(before_results, "total")?,
        passed: number_after(before_results, "passed")?,
        l1_total: number_after(l1, "total")?,
        l1_passed: number_after(l1, "passed")?,
    })
}

fn ratchet_value(text: &str, field: &str) -> Result<u64, String> {
    text.lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix(&format!("{field} "))
                .map(str::trim)
        })
        .ok_or_else(|| format!("{RATCHET} has no `{field}`"))?
        .parse()
        .map_err(|error| format!("{RATCHET} `{field}`: {error}"))
}

fn ratchet_measurement(text: &str) -> Result<Measurement, String> {
    Ok(Measurement {
        total: ratchet_value(text, "coding_ladder_tasks")?,
        passed: ratchet_value(text, "coding_ladder_passing")?,
        l1_total: ratchet_value(text, "coding_ladder_l1_tasks")?,
        l1_passed: ratchet_value(text, "coding_ladder_l1_passing")?,
    })
}

fn compare_with_ratchet(
    observed: Measurement,
    recorded: Measurement,
    canonical_result: bool,
) -> Result<(), String> {
    if observed.total != recorded.total || observed.l1_total != recorded.l1_total {
        return Err(format!(
            "coding ladder shape changed: observed {observed:?}, recorded {recorded:?}"
        ));
    }
    if canonical_result {
        if observed != recorded {
            return Err(format!(
                "committed coding ladder result/ratchet mismatch: observed {observed:?}, recorded {recorded:?}"
            ));
        }
        if observed.l1_total != 16 || observed.l1_passed != 0 {
            return Err(format!(
                "the committed honest L1 frontier must remain visible as 0/16, observed {}/{}",
                observed.l1_passed, observed.l1_total
            ));
        }
    } else if observed.passed < recorded.passed || observed.l1_passed < recorded.l1_passed {
        return Err(format!(
            "coding ladder regressed below its floor: observed {observed:?}, recorded {recorded:?}"
        ));
    }
    Ok(())
}

fn validate_workflow(text: &str) -> Result<(), String> {
    for required in [
        "workflow_dispatch:",
        "schedule:",
        "cron:",
        "paths:",
        "experiments/issue_847_coding_ladder/run_coding_ladder.sh",
        "rust-script scripts/check-coding-ladder.rs --result \"$OUT\"",
        "actions/upload-artifact@v7",
    ] {
        if !text.contains(required) {
            return Err(format!("{WORKFLOW} is missing `{required}`"));
        }
    }
    Ok(())
}

fn base_ratchet(root: &Path, base: &str) -> Result<Option<Measurement>, String> {
    let output = Command::new("git")
        .args(["show", &format!("{base}:{RATCHET}")])
        .current_dir(root)
        .output()
        .map_err(|error| format!("git show {base}:{RATCHET}: {error}"))?;
    if !output.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&output.stdout);
    match ratchet_measurement(&text) {
        Ok(measurement) => Ok(Some(measurement)),
        Err(_) => Ok(None),
    }
}

fn check(root: &Path, result: &Path, base: Option<&str>) -> Result<(), String> {
    let observed = full_result(
        &fs::read_to_string(result).map_err(|error| format!("{}: {error}", result.display()))?,
    )?;
    let ratchet_text =
        fs::read_to_string(root.join(RATCHET)).map_err(|error| format!("{RATCHET}: {error}"))?;
    let recorded = ratchet_measurement(&ratchet_text)?;
    let canonical_result = result == root.join(RESULT);
    compare_with_ratchet(observed, recorded, canonical_result)?;
    validate_workflow(
        &fs::read_to_string(root.join(WORKFLOW)).map_err(|error| format!("{WORKFLOW}: {error}"))?,
    )?;
    if let Some(base) = base
        && let Some(previous) = base_ratchet(root, base)?
    {
        if recorded.total != previous.total || recorded.l1_total != previous.l1_total {
            return Err(format!(
                "coding ladder task counts differ from {base}: {recorded:?} versus {previous:?}"
            ));
        }
        if recorded.passed < previous.passed || recorded.l1_passed < previous.l1_passed {
            return Err(format!(
                "coding ladder floor fell from {previous:?} at {base} to {recorded:?}"
            ));
        }
    }
    Ok(())
}

fn arguments() -> Result<(PathBuf, PathBuf, Option<String>), String> {
    let mut root = env::current_dir().map_err(|error| error.to_string())?;
    let mut result = None;
    let mut base = None;
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--root" => root = PathBuf::from(args.next().ok_or("--root needs a path")?),
            "--result" => result = Some(PathBuf::from(args.next().ok_or("--result needs a path")?)),
            "--base" => base = Some(args.next().ok_or("--base needs a revision")?),
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    let result = result.unwrap_or_else(|| root.join(RESULT));
    Ok((root, result, base))
}

#[cfg(not(test))]
fn main() {
    let outcome =
        arguments().and_then(|(root, result, base)| check(&root, &result, base.as_deref()));
    match outcome {
        Ok(()) => println!("coding ladder: result, ratchet, and standalone workflow agree"),
        Err(error) => {
            eprintln!("coding ladder gate failed: {error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL: &str = r#"{
      "measurement":{"dataset_total": 130,"measured_total": 130,"complete": true},
      "summary":{"total": 130,"passed": 65,"by_level":{"L1":{"passed": 0,"total": 16}}},
      "results":[]
    }"#;

    #[test]
    fn parses_the_honest_full_frontier() {
        assert_eq!(
            full_result(FULL),
            Ok(Measurement {
                total: 130,
                passed: 65,
                l1_total: 16,
                l1_passed: 0,
            })
        );
    }

    #[test]
    fn partial_measurement_is_not_a_ratchet_input() {
        assert!(full_result(&FULL.replace("\"complete\": true", "\"complete\": false")).is_err());
    }

    #[test]
    fn a_fresh_full_run_may_improve_but_not_fall_below_the_floor() {
        let floor = Measurement {
            total: 130,
            passed: 65,
            l1_total: 16,
            l1_passed: 0,
        };
        assert!(
            compare_with_ratchet(
                Measurement {
                    passed: 66,
                    l1_passed: 1,
                    ..floor
                },
                floor,
                false,
            )
            .is_ok()
        );
        assert!(
            compare_with_ratchet(
                Measurement {
                    passed: 64,
                    ..floor
                },
                floor,
                false,
            )
            .is_err()
        );
    }
}
