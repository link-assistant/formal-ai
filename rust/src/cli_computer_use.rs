use std::error::Error;
use std::fs;
use std::path::PathBuf;

use formal_ai::computer_use::{
    ComputerUsePolicy, execute_verified_plan, induce, parse_benchmark_tasks, plan_for_prompt,
    plan_request, replay_verified_plan, synthesize_with,
};

/// Each flag is an independent operator decision — grant, confirm, replay, or
/// inspect what was learned — so they stay separate switches rather than one
/// mode enum that would make illegal combinations expressible.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, clap::Args)]
pub struct ComputerUseArgs {
    #[arg(long, default_value_t = String::new())]
    prompt: String,

    /// Print the plan schemas learned from the supplied corpus, in Links
    /// Notation, and exit without executing anything.
    #[arg(long, requires = "from", conflicts_with = "plan")]
    learn: bool,

    /// Read teaching examples from this Links Notation file.
    #[arg(long)]
    from: Option<PathBuf>,

    /// Print an explicit plan prompt for a tool client without running the plan.
    #[arg(long)]
    plan: bool,

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
        from,
        plan,
    } = args;
    let tasks = from
        .as_ref()
        .map(|source| fs::read_to_string(source).map(|document| parse_benchmark_tasks(&document)))
        .transpose()?;
    if learn {
        let (Some(tasks), Some(source)) = (&tasks, &from) else {
            return Err("computer_use_refused: --from is required with --learn".into());
        };
        print!(
            "{}",
            induce(tasks).links_notation(&source.to_string_lossy())
        );
        return Ok(());
    }
    if prompt.is_empty() {
        return Err("computer_use_refused: --prompt is required".into());
    }
    let supplied = plan_request(&prompt);
    if plan && supplied.is_some() {
        println!("{prompt}");
        return Ok(());
    }
    let resolved = supplied
        .or_else(|| {
            let tasks = tasks.as_deref()?;
            plan_for_prompt(tasks, &prompt)
                .or_else(|| synthesize_with(&prompt, &induce(tasks)).map(|value| value.plan))
        })
        .ok_or("computer_use_input_required: supply a fenced computer-use plan or --from corpus")?;
    if plan {
        let steps = resolved
            .steps
            .iter()
            .map(|step| {
                serde_json::json!({
                    "primitive": step.primitive.name(),
                    "arguments": step.arguments,
                })
            })
            .collect::<Vec<_>>();
        println!(
            "{prompt}\n```computer-use\n{}\n```",
            serde_json::to_string(&steps)?
        );
        return Ok(());
    }
    if !agent_mode {
        return Err("computer_use_refused: --agent-mode is required".into());
    }
    if !confirm_effects {
        return Err("computer_use_refused: --confirm-effects is required".into());
    }
    let outcome = execute_verified_plan(resolved, ComputerUsePolicy::agent_mode_all())?;
    let replay_verified = replay.then(|| replay_verified_plan(&outcome)).transpose()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "outcome": outcome,
            "replay_verified": replay_verified
        }))?
    );
    Ok(())
}
