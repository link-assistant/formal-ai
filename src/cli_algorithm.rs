//! CLI inspection surface for learned execution-algorithm artifacts.

use std::collections::BTreeMap;
use std::error::Error;
use std::path::PathBuf;

use clap::{Args as ClapArgs, Subcommand};
use formal_ai::algorithm_discovery::AlgorithmCandidate;
use formal_ai::render_response;

#[derive(ClapArgs, Debug)]
pub struct AlgorithmArgs {
    #[command(subcommand)]
    action: AlgorithmAction,
}

#[derive(Debug, Subcommand)]
enum AlgorithmAction {
    /// Parse, integrity-check, and materialize a proposal without side effects.
    /// This does not approve or execute the candidate.
    Conformance {
        /// Discovery artifact written by `formal-ai learn algorithms`.
        #[arg(long)]
        artifact: PathBuf,

        /// Explicit value threaded into the conformance record.
        #[arg(long)]
        trigger: String,

        /// Parameter binding in `name=value` form. Repeat for each parameter.
        #[arg(long = "binding", value_name = "NAME=VALUE")]
        bindings: Vec<String>,
    },
}

pub fn run_algorithm(args: AlgorithmArgs) -> Result<(), Box<dyn Error>> {
    match args.action {
        AlgorithmAction::Conformance {
            artifact,
            trigger,
            bindings,
        } => {
            let document = std::fs::read_to_string(artifact)?;
            let candidate = AlgorithmCandidate::from_links_notation(&document)?;
            let bindings = parse_bindings(&bindings)?;
            print!(
                "{}",
                candidate.conformance_links_notation(&trigger, &bindings)?
            );
        }
    }
    Ok(())
}

fn parse_bindings(values: &[String]) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    values
        .iter()
        .map(|value| {
            let (name, value) = value.split_once('=').ok_or_else(|| {
                let binding = format!("{value:?}");
                render_response("algorithm_invalid_binding", "en", &[("binding", &binding)])
                    .unwrap_or_default()
            })?;
            if name.trim().is_empty() {
                let binding = format!("{value:?}");
                let message = render_response(
                    "algorithm_empty_binding_name",
                    "en",
                    &[("binding", &binding)],
                )
                .unwrap_or_default();
                return Err(message.into());
            }
            Ok((name.trim().to_owned(), value.to_owned()))
        })
        .collect()
}
