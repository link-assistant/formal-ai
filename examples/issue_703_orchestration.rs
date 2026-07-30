//! Scripted walkthrough of the issue #703 orchestration library API.
//!
//! Run with:
//! `cargo run --example issue_703_orchestration -- /tmp/formal-ai-agent-demo`

use formal_ai::orchestration::{
    replay_session, run_agent, write_session, AgentCommand, AgentRunConfig, AgentRunPermission,
};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let workspace = std::env::args_os().nth(1).map_or_else(
        || std::env::temp_dir().join("formal-ai-agent-demo"),
        PathBuf::from,
    );
    fs::create_dir_all(&workspace)?;
    fs::write(workspace.join("README.md"), "# Demonstration\n")?;

    let command = AgentCommand::new("sh")
        .arg("-c")
        .arg("printf '\\n[![Formal AI](https://img.shields.io/badge/Formal-AI-blue)]\\n' >> README.md; printf 'scripted agent complete\\n'");
    let mut config = AgentRunConfig::new("codex", "add a README badge", &workspace)
        .with_permission(AgentRunPermission::grant_for(&workspace))
        .with_command(command);
    config.allowlisted_agent_commands.insert("sh".to_string());
    let session = run_agent(&config)?;
    let session_path = workspace.join("agent-session.json");
    write_session(&session_path, &session)?;

    let bytes = fs::read(&session_path)?;
    let replayed = replay_session(&bytes)?;
    println!(
        "status={:?} changes={} session={}",
        replayed.status,
        replayed.changes.len(),
        session_path.display()
    );
    Ok(())
}
