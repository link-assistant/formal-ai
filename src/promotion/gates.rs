//! Canonical, executable promotion gates.
//!
//! Proposal documents name changes, never commands or observed results. This
//! module owns the allow-listed commands and derives evidence only from their
//! fresh process output, preventing a proposal from promoting itself by
//! claiming fabricated counts or substituting a benign runner.

use std::path::Path;
use std::process::Command;

use crate::engine::stable_id;

use super::{PromotionProposal, PromotionRatchet};

/// Captured result of one canonical benchmark command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateCommandOutput {
    /// Whether the command exited successfully.
    pub succeeded: bool,
    /// Captured standard output.
    pub stdout: String,
    /// Captured standard error.
    pub stderr: String,
}

impl GateCommandOutput {
    /// A successful fixture or executor result.
    #[must_use]
    pub fn success(stdout: impl Into<String>) -> Self {
        Self {
            succeeded: true,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    /// A failed fixture or executor result.
    #[must_use]
    pub fn failure(stdout: impl Into<String>, stderr: impl Into<String>) -> Self {
        Self {
            succeeded: false,
            stdout: stdout.into(),
            stderr: stderr.into(),
        }
    }
}

/// Replay all required promotion gates with an injectable command executor.
///
/// Each canonical command runs once and its immutable evidence is cloned onto
/// every proposal considered in this batch. Successful commands without a
/// parseable pass/fail report are rejected as malformed evidence; failed
/// commands become ordinary blocking evidence so proposals remain durable.
pub fn replay_promotion_gates_with<F>(
    mut proposals: Vec<PromotionProposal>,
    mut execute: F,
) -> Result<Vec<PromotionProposal>, String>
where
    F: FnMut(&str) -> Result<GateCommandOutput, String>,
{
    let mut evidence = Vec::new();
    for mut gate in required_gates() {
        let output = execute(&gate.runner).map_err(|error| {
            format!(
                "could not execute canonical gate `{}`: {error}",
                gate.suite_id
            )
        })?;
        let joined = format!(
            "suite={}\nrunner={}\nsucceeded={}\nstdout:\n{}\nstderr:\n{}",
            gate.suite_id, gate.runner, output.succeeded, output.stdout, output.stderr
        );
        gate.evidence_digest = Some(stable_id("promotion_gate_output", &joined));
        gate.command_succeeded = output.succeeded;
        if output.succeeded {
            let (passed, failed) =
                parse_pass_fail(&output.stdout, &output.stderr).ok_or_else(|| {
                    format!(
                        "canonical gate `{}` succeeded without a parseable pass/fail report",
                        gate.suite_id
                    )
                })?;
            gate.passed = passed;
            gate.failed = failed;
        } else {
            gate.passed = 0;
            gate.failed = 1;
        }
        evidence.push(gate);
    }
    for proposal in &mut proposals {
        proposal.gates.clone_from(&evidence);
    }
    Ok(proposals)
}

/// Replay canonical gates as subprocesses rooted at `workspace_root`.
///
/// Commands are not sourced from proposal data. They come exclusively from the
/// checked-in manifests (plus the fixed unit-specification command), so invoking
/// the shell here cannot introduce proposal-controlled command execution.
pub fn replay_promotion_gates(
    proposals: Vec<PromotionProposal>,
    workspace_root: &Path,
) -> Result<Vec<PromotionProposal>, String> {
    replay_promotion_gates_with(proposals, |runner| {
        let output = Command::new("sh")
            .arg("-c")
            .arg(runner)
            .current_dir(workspace_root)
            .output()
            .map_err(|error| format!("failed to start `{runner}`: {error}"))?;
        Ok(GateCommandOutput {
            succeeded: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    })
}

pub(super) fn required_gates() -> Vec<PromotionRatchet> {
    let mut gates = vec![
        PromotionRatchet::coding_modification(0, 1),
        PromotionRatchet::industry(0, 1),
        PromotionRatchet::unit_specs(0, 1),
    ];
    for gate in &mut gates {
        gate.command_succeeded = false;
    }
    gates
}

fn parse_pass_fail(stdout: &str, stderr: &str) -> Option<(usize, usize)> {
    let combined = format!("{stdout}\n{stderr}");
    for line in combined.lines().rev() {
        if let (Some(passed), Some(failed)) =
            (count_after(line, "passed="), count_after(line, "failed="))
        {
            return Some((passed, failed));
        }
    }
    // Cargo's one-test harness summary follows custom benchmark output. Use it
    // only for the unit gate; named benchmark case counts take precedence.
    for line in combined.lines().rev() {
        if line.contains("test result:") {
            let passed = count_before(line, " passed;");
            let failed = count_before(line, " failed;");
            if let (Some(passed), Some(failed)) = (passed, failed) {
                return Some((passed, failed));
            }
        }
    }
    None
}

fn count_after(line: &str, marker: &str) -> Option<usize> {
    let tail = line.split_once(marker)?.1;
    let digits: String = tail.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

fn count_before(line: &str, marker: &str) -> Option<usize> {
    let head = line.split_once(marker)?.0;
    head.split_whitespace().next_back()?.parse().ok()
}
