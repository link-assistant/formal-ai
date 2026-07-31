use std::error::Error;

#[derive(Debug, clap::Args)]
pub struct ComputerUseArgs {
    #[arg(long, default_value_t = String::new())]
    prompt: String,

    /// Print the plan schemas auto-learned from the benchmark corpus, in Links
    /// Notation, and exit without executing anything.
    #[arg(long, default_value_t = false)]
    learn: bool,

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
        learn,
    } = args;
    if learn {
        print!("{}", formal_ai::computer_use::learned().links_notation());
        return Ok(());
    }
    if prompt.is_empty() {
        return Err("computer_use_refused: --prompt is required".into());
    }
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
