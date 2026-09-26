//! CLI execution surface for persisted natural-language procedures.

use std::error::Error;
use std::path::PathBuf;

use clap::{Args as ClapArgs, Subcommand};
use formal_ai::CompiledProcedure;

#[derive(ClapArgs, Debug)]
pub struct ProcedureArgs {
    #[command(subcommand)]
    action: ProcedureAction,
}

#[derive(Debug, Subcommand)]
enum ProcedureAction {
    /// Walk a persisted artifact with the deterministic, side-effect-free host.
    Conformance {
        /// Compiled procedure artifact written by the solver or Agent CLI.
        #[arg(long)]
        artifact: PathBuf,

        /// Explicit trigger value threaded into the first compiled step.
        #[arg(long)]
        trigger: String,
    },
}

pub fn run_procedure(args: ProcedureArgs) -> Result<(), Box<dyn Error>> {
    match args.action {
        ProcedureAction::Conformance { artifact, trigger } => {
            let document = std::fs::read_to_string(&artifact)?;
            let procedure = CompiledProcedure::from_artifact_links_notation(&document)?;
            print!("{}", procedure.conformance_links_notation(&trigger));
        }
    }
    Ok(())
}
