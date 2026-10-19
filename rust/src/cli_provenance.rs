//! Native source snapshots use the canonical shared policy and bounded CPU adapters.
use clap::{Args, Subcommand};
use formal_ai::provenance_snapshot::{SnapshotLimits, snapshot_repository};
use std::error::Error;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct ProvenanceArgs {
    #[command(subcommand)]
    action: ProvenanceAction,
}
#[derive(Debug, Subcommand)]
enum ProvenanceAction {
    /// Snapshot every tracked working source file without exclusions.
    Snapshot {
        /// Physical repository root, not a nested directory.
        directory: PathBuf,
        /// auto selects CPU; gpu explicitly refuses until a real backend exists.
        #[arg(long, default_value = "auto", value_parser = ["auto", "cpu", "gpu"])]
        backend: String,
        #[arg(long, default_value_t = 50_000)]
        max_files: usize,
        #[arg(long, default_value_t = 4 * 1024 * 1024)]
        max_file_bytes: u64,
        #[arg(long, default_value_t = 32 * 1024 * 1024)]
        max_total_bytes: u64,
    },
}
pub fn run_provenance(arguments: ProvenanceArgs) -> Result<(), Box<dyn Error>> {
    match arguments.action {
        ProvenanceAction::Snapshot {
            directory,
            backend,
            max_files,
            max_file_bytes,
            max_total_bytes,
        } => {
            let result = snapshot_repository(
                &directory,
                &backend,
                SnapshotLimits {
                    max_files,
                    max_file_bytes,
                    max_total_bytes,
                },
            )?;
            println!("{}", serde_json::to_string(&result)?);
        }
    }
    Ok(())
}
