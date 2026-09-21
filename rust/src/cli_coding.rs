//! Explicit fragment-cache maintenance for the source-driven composer.

use std::error::Error;
use std::path::PathBuf;

use clap::{Args, Subcommand};
use formal_ai::fragment_catalog::{FragmentCatalog, FragmentLedger};

#[derive(Debug, Args)]
pub struct CodingArgs {
    #[command(subcommand)]
    pub action: CodingAction,
}

#[derive(Debug, Subcommand)]
pub enum CodingAction {
    /// Delete rediscovered fragments, never the shipped bootstrap or memory.
    ForgetFragments {
        #[arg(long, env = "FORMAL_AI_CACHE_DIR", default_value = "data/cache")]
        cache_dir: PathBuf,
        /// Required acknowledgement because this removes cache records.
        #[arg(long, default_value_t = false)]
        confirm: bool,
    },
    /// Rebuild the local ledger from provenance-bearing offline captures.
    RediscoverFragments {
        #[arg(long, env = "FORMAL_AI_CACHE_DIR", default_value = "data/cache")]
        cache_dir: PathBuf,
        /// Directory containing a `coding-fragments/` capture ledger.
        #[arg(long)]
        captures: PathBuf,
    },
}

pub fn run_coding(args: CodingArgs) -> Result<(), Box<dyn Error>> {
    match args.action {
        CodingAction::ForgetFragments { cache_dir, confirm } => {
            if !confirm {
                return Err("forget-fragments requires --confirm".into());
            }
            let removed = FragmentLedger::new(cache_dir).forget_all()?;
            println!("forgotten_fragments={removed}");
        }
        CodingAction::RediscoverFragments {
            cache_dir,
            captures,
        } => {
            let captured =
                FragmentCatalog::default().with_rediscovered(&FragmentLedger::new(captures));
            if captured.fragments().is_empty() {
                return Err("no provenance-bearing fragment captures were found".into());
            }
            let destination = FragmentLedger::new(cache_dir);
            for fragment in captured.fragments() {
                destination.remember(fragment)?;
            }
            println!(
                "rediscovered_fragments={} content_id={}",
                captured.fragments().len(),
                captured.content_id()
            );
        }
    }
    Ok(())
}
