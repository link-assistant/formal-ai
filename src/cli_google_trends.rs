use std::error::Error;
use std::path::{Path, PathBuf};

use clap::Args as ClapArgs;

use formal_ai::google_trends::{
    render_prompt_suite_from_rss, TrendPromptSuiteConfig, GOOGLE_TRENDS_US_RSS_URL,
};

/// CLI options for converting a saved Google Trends RSS feed into Formal AI request cases.
#[derive(Debug, Clone, ClapArgs)]
pub struct GoogleTrendsOptions {
    /// Saved RSS input, for example docs/case-studies/issue-499/raw-data/google-trends-us-rss.xml.
    #[arg(long)]
    input: PathBuf,

    /// Destination `.lino` file. Omit or pass `-` to write to stdout.
    #[arg(long)]
    output: Option<PathBuf>,

    /// Number of ranked trends to convert.
    #[arg(long, default_value_t = 10)]
    top: usize,

    /// Trends geography code recorded in the generated fixture.
    #[arg(long, default_value = "US")]
    geo: String,

    /// Capture timestamp recorded in the generated fixture.
    #[arg(long = "captured-at", default_value = "unknown")]
    captured_at: String,

    /// Source feed URL recorded in the generated fixture.
    #[arg(long, default_value = GOOGLE_TRENDS_US_RSS_URL)]
    source_url: String,

    /// Snapshot path recorded in the generated fixture.
    #[arg(
        long,
        default_value = "docs/case-studies/issue-499/raw-data/google-trends-us-rss.xml"
    )]
    source_snapshot: String,
}

pub fn run_google_trends(options: GoogleTrendsOptions) -> Result<(), Box<dyn Error>> {
    let config = TrendPromptSuiteConfig {
        geo: options.geo,
        captured_at: options.captured_at,
        source_url: options.source_url,
        source_snapshot: options.source_snapshot,
        top_n: options.top,
    };
    render_google_trends_file(&options.input, options.output.as_deref(), &config)
}

fn render_google_trends_file(
    input: &Path,
    output: Option<&Path>,
    config: &TrendPromptSuiteConfig,
) -> Result<(), Box<dyn Error>> {
    let xml = std::fs::read_to_string(input)?;
    let rendered = render_prompt_suite_from_rss(&xml, config)?;
    match output {
        Some(path) if path != Path::new("-") => {
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            std::fs::write(path, rendered)?;
            eprintln!("wrote Google Trends prompt suite to {}", path.display());
        }
        _ => print!("{rendered}"),
    }
    Ok(())
}
