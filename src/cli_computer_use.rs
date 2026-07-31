use std::error::Error;

#[derive(Debug, clap::Args)]
pub struct ComputerUseArgs {
    #[arg(long)]
    prompt: String,

    /// Grant the complete computer-use primitive set for this invocation.
    #[arg(long, default_value_t = false)]
    agent_mode: bool,

    /// Confirm the plan's write, move, POST, command, and archive effects.
    #[arg(long, default_value_t = false)]
    confirm_effects: bool,

    /// Re-run independent verification over the recorded outcome.
    #[arg(long, default_value_t = false)]
    replay: bool,
}

pub fn run_computer_use(args: ComputerUseArgs) -> Result<(), Box<dyn Error>> {
    let ComputerUseArgs {
        prompt,
        agent_mode,
        confirm_effects,
        replay,
    } = args;
    if !agent_mode {
        return Err("computer_use_refused: --agent-mode is required".into());
    }
    if !confirm_effects {
        return Err("computer_use_refused: --confirm-effects is required".into());
    }
    let outcome = formal_ai::computer_use::run_verified_plan(&prompt)?;
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
