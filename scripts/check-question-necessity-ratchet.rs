#!/usr/bin/env rust-script
//! Keep the question-per-task ceiling monotonic.
//!
//! Usage:
//!   rust-script scripts/check-question-necessity-ratchet.rs
//!   rust-script scripts/check-question-necessity-ratchet.rs --base origin/main
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::env;
use std::fs;
use std::process::{Command, ExitCode};

const POLICY_PATH: &str = "data/seed/question-necessity.lino";

fn ratchet_maximum(seed: &str) -> Result<usize, String> {
    let mut in_ratchet = false;
    for line in seed.lines() {
        if !line.starts_with(' ') {
            in_ratchet = line.trim() == "question_necessity_ratchet";
            continue;
        }
        if in_ratchet {
            let trimmed = line.trim();
            if let Some(value) = trimmed.strip_prefix("maximum ") {
                return value
                    .trim_matches('"')
                    .parse()
                    .map_err(|error| format!("invalid ratchet maximum: {error}"));
            }
        }
    }
    Err(String::from("question necessity ratchet maximum is missing"))
}

fn validate_direction(seed: &str) -> Result<(), String> {
    if seed.contains("metric \"questions_per_100_tasks\"")
        && seed.contains("direction \"down\"")
    {
        Ok(())
    } else {
        Err(String::from(
            "question necessity ratchet must use questions_per_100_tasks with direction down",
        ))
    }
}

fn compare(current: usize, baseline: usize) -> Result<(), String> {
    if current <= baseline {
        Ok(())
    } else {
        Err(format!(
            "question necessity ratchet regressed from {baseline} to {current} questions per 100 tasks"
        ))
    }
}

fn base_seed(base: &str) -> Result<Option<String>, String> {
    let output = Command::new("git")
        .args(["show", &format!("{base}:{POLICY_PATH}")])
        .output()
        .map_err(|error| format!("could not invoke git: {error}"))?;
    if output.status.success() {
        String::from_utf8(output.stdout)
            .map(Some)
            .map_err(|error| format!("baseline policy is not UTF-8: {error}"))
    } else {
        Ok(None)
    }
}

fn run() -> Result<(), String> {
    let seed = fs::read_to_string(POLICY_PATH)
        .map_err(|error| format!("could not read {POLICY_PATH}: {error}"))?;
    validate_direction(&seed)?;
    let current = ratchet_maximum(&seed)?;

    let mut args = env::args().skip(1);
    let base = match args.next().as_deref() {
        None => None,
        Some("--base") => Some(
            args.next()
                .ok_or_else(|| String::from("--base requires a revision"))?,
        ),
        Some(other) => return Err(format!("unknown argument: {other}")),
    };
    if args.next().is_some() {
        return Err(String::from("unexpected extra arguments"));
    }

    if let Some(base) = base {
        if let Some(previous) = base_seed(&base)? {
            compare(current, ratchet_maximum(&previous)?)?;
        } else {
            println!("No question-necessity policy exists at {base}; establishing ceiling {current}.");
            return Ok(());
        }
    }
    println!("Question-necessity ratchet held at {current} questions per 100 tasks.");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_ratchet_maximum() {
        assert_eq!(ratchet_maximum("question_necessity_ratchet\n  maximum \"42\""), Ok(42));
    }

    #[test]
    fn permits_only_downward_or_equal_changes() {
        assert!(compare(40, 60).is_ok());
        assert!(compare(60, 60).is_ok());
        assert!(compare(61, 60).is_err());
    }
}
