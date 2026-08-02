use std::error::Error;
use std::fs;
use std::io;
use std::path::PathBuf;

use clap::Args;
use formal_ai::file_legality::{check_file_legality_with, FileLegalityConfig};

#[derive(Debug, Clone, Args)]
pub struct FileLegalityArgs {
    /// File to inspect without reproducing its content in the report.
    path: PathBuf,

    /// JSON policy and detector-evidence configuration.
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,

    /// Jurisdiction code to assess. Repeat for multi-jurisdiction output.
    #[arg(long = "jurisdiction", value_name = "CODE")]
    jurisdictions: Vec<String>,

    /// Indent the JSON report.
    #[arg(long)]
    pretty: bool,
}

pub fn run_file_legality(args: &FileLegalityArgs) -> Result<(), Box<dyn Error>> {
    let mut config = load_config(args.config.as_ref())?;
    if !args.jurisdictions.is_empty() {
        config.jurisdictions.clone_from(&args.jurisdictions);
    }
    let report = check_file_legality_with(&args.path, &config)?;
    let rendered = if args.pretty {
        serde_json::to_string_pretty(&report)?
    } else {
        serde_json::to_string(&report)?
    };
    println!("{rendered}");
    Ok(())
}

fn load_config(path: Option<&PathBuf>) -> Result<FileLegalityConfig, Box<dyn Error>> {
    let Some(path) = path else {
        return Ok(FileLegalityConfig::default());
    };
    let input = fs::read_to_string(path)?;
    serde_json::from_str(&input).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: {error}", path.display()),
        )
        .into()
    })
}
