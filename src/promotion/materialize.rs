//! Formal-AI-authored promotion materialization.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::agentic_coding::run_agentic_task;
use crate::engine::stable_id;

use super::{AppliedSeedEdit, PromotionApplyOutcome, PromotionOutcome, PromotionRun};

/// Materialize accepted edits using the same Formal AI agentic task path used
/// by Agent CLI sessions, without ever pushing.
///
/// Edits targeting the same seed file are first coalesced into one complete
/// desired file. Formal AI then plans and executes a literal file-creation task
/// in its isolated agent workspace. Only the exact `write_file` arguments from
/// that deterministic session are accepted for the real promotion workspace.
/// A mismatched path or byte is a fail-closed protocol error.
pub fn apply_promotions(
    run: &PromotionRun,
    workspace_root: &Path,
) -> io::Result<PromotionApplyOutcome> {
    let mut targets: BTreeMap<String, (String, usize)> = BTreeMap::new();
    let mut rejected = Vec::new();
    for record in &run.records {
        if record.outcome == PromotionOutcome::Rejected {
            rejected.push(record.proposal.id.clone());
            continue;
        }
        let edit = &record.proposal.edit;
        validate_seed_path(&edit.seed_file)?;
        let target_path = workspace_root.join(&edit.seed_file);
        let existing = match fs::read_to_string(&target_path) {
            Ok(existing) => existing,
            Err(error) if error.kind() == io::ErrorKind::NotFound => String::new(),
            Err(error) => {
                return Err(io::Error::new(
                    error.kind(),
                    format!(
                        "could not read existing seed {}: {error}",
                        target_path.display()
                    ),
                ));
            }
        };
        let entry = targets
            .entry(edit.seed_file.clone())
            .or_insert((existing, 0));
        if !entry.0.is_empty() && !entry.0.ends_with('\n') {
            entry.0.push('\n');
            entry.1 = entry.1.saturating_add(1);
        }
        entry.0.push_str(&edit.lino);
        entry.1 = entry.1.saturating_add(edit.lino.len());
    }

    let has_targets = !targets.is_empty();
    let mut authored_edits = Vec::new();
    let mut agent_session_ids = Vec::new();
    for (seed_file, (desired, bytes_written)) in targets {
        let task = format!("Create file {seed_file} containing\n{desired}");
        let outcome = run_agentic_task(&task).map_err(invalid_data)?;
        if outcome.hit_turn_cap {
            return Err(invalid_data("Formal AI Agent hit its turn cap"));
        }
        let (authored_path, authored_content) = authored_write(&outcome.steps)?;
        if authored_path != seed_file || authored_content != desired {
            return Err(invalid_data(format!(
                "Formal AI Agent write did not match the promotion target exactly: expected {seed_file} ({} bytes), got {authored_path} ({} bytes)",
                desired.len(),
                authored_content.len()
            )));
        }

        let session = serde_json::to_string(&outcome.session_json()).map_err(invalid_data)?;
        agent_session_ids.push(stable_id("promotion_agent_session", &session));
        authored_edits.push((seed_file, authored_path, authored_content, bytes_written));
    }

    // Agent planning is deliberately side-effect free. Create the review branch only
    // after every proposed write has been authored and byte-validated, so a routing or
    // protocol failure leaves the caller's branch untouched.
    if has_targets {
        prepare_local_branch(workspace_root, &run.branch_plan().branch)?;
    }

    let mut applied = Vec::new();
    for (seed_file, authored_path, authored_content, bytes_written) in authored_edits {
        let path = workspace_root.join(&authored_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, authored_content.as_bytes())?;
        applied.push(AppliedSeedEdit {
            seed_file,
            path,
            bytes_written,
        });
    }

    Ok(PromotionApplyOutcome {
        applied,
        rejected,
        branch_plan: run.branch_plan(),
        agent_session_ids,
    })
}

fn prepare_local_branch(workspace_root: &Path, branch: &str) -> io::Result<()> {
    let inside = git(workspace_root, &["rev-parse", "--is-inside-work-tree"]).map_err(|_| {
        invalid_data(
            "promotion apply requires a Git worktree so edits exist only on a review branch",
        )
    })?;
    if inside.trim() != "true" {
        return Err(invalid_data(
            "promotion apply requires a Git worktree so edits exist only on a review branch",
        ));
    }
    let status = git(workspace_root, &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        return Err(invalid_data(
            "promotion apply requires a clean Git worktree before creating its branch",
        ));
    }
    git(workspace_root, &["switch", "-c", branch])?;
    Ok(())
}

fn git(workspace_root: &Path, args: &[&str]) -> io::Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace_root)
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(invalid_data(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn authored_write(steps: &[crate::agentic_coding::DriverToolStep]) -> io::Result<(String, String)> {
    steps
        .iter()
        .filter(|step| step.tool == "write_file")
        .find_map(|step| {
            let args: Value = serde_json::from_str(&step.arguments).ok()?;
            let path = args.get("path")?.as_str()?.to_owned();
            if path == crate::agentic_coding::general_planner::PLAN_PATH {
                return None;
            }
            Some((path, args.get("content")?.as_str()?.to_owned()))
        })
        .ok_or_else(|| invalid_data("Formal AI Agent emitted no seed write"))
}

fn validate_seed_path(path: &str) -> io::Result<()> {
    let candidate = Path::new(path);
    let safe = !candidate.is_absolute()
        && candidate
            .extension()
            .is_some_and(|extension| extension == "lino")
        && candidate.starts_with("data/seed")
        && !candidate
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir));
    if safe {
        Ok(())
    } else {
        Err(invalid_data(format!(
            "promotion target must be a relative data/seed/*.lino path: {path}"
        )))
    }
}

fn invalid_data(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}
