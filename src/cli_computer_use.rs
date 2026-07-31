use std::error::Error;

pub fn run_computer_use(
    prompt: &str,
    agent_mode: bool,
    confirm_effects: bool,
    replay: bool,
) -> Result<(), Box<dyn Error>> {
    if !agent_mode {
        return Err("computer_use_refused: --agent-mode is required".into());
    }
    if !confirm_effects {
        return Err("computer_use_refused: --confirm-effects is required".into());
    }
    let outcome = formal_ai::computer_use::run_verified_plan(prompt)?;
    let replay_verified = replay
        .then(|| formal_ai::computer_use::replay_verified_plan(&outcome))
        .transpose()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "outcome": outcome,
            "replay_verified": replay_verified
        }))?
    );
    Ok(())
}
