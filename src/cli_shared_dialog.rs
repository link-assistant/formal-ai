use std::error::Error;
use std::path::PathBuf;

use clap::{Args as ClapArgs, Subcommand, ValueEnum};

use formal_ai::{convert_shared_dialog_to_demo_memory, SharedDialogFormat, SharedDialogMetadata};

#[derive(Debug, Subcommand)]
pub enum SharedDialogAction {
    Convert(SharedDialogConvertOptions),
}

#[derive(Debug, Clone, ClapArgs)]
pub struct SharedDialogConvertOptions {
    /// Source capture or transcript. Use `-` to read from stdin.
    #[arg(long)]
    input: PathBuf,

    /// Destination `.lino` file. Use `-` to write to stdout.
    #[arg(long, default_value = "-")]
    output: PathBuf,

    /// Input parser. `auto` detects `ChatGPT` HTML and normalized web-capture JSON before markdown.
    #[arg(long, value_enum, default_value = "auto")]
    format: SharedDialogCliFormat,

    /// Original URL for provenance in each exported event.
    #[arg(long)]
    source_url: Option<String>,

    /// Optional demo label stamped into each exported event.
    #[arg(long)]
    demo_label: Option<String>,

    /// Optional conversation id override.
    #[arg(long)]
    conversation_id: Option<String>,

    /// Optional conversation title override.
    #[arg(long)]
    conversation_title: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SharedDialogCliFormat {
    Auto,
    #[value(alias = "chatgpt")]
    ChatgptShareHtml,
    #[value(alias = "markdown")]
    MarkdownTranscript,
    #[value(alias = "web-capture", alias = "capture-json")]
    WebCaptureJson,
}

pub fn run_shared_dialog(action: SharedDialogAction) -> Result<(), Box<dyn Error>> {
    match action {
        SharedDialogAction::Convert(options) => {
            let input = crate::read_input(&options.input)?;
            let metadata = SharedDialogMetadata {
                source_url: options.source_url,
                demo_label: options.demo_label,
                conversation_id: options.conversation_id,
                conversation_title: options.conversation_title,
            };
            let text = convert_shared_dialog_to_demo_memory(
                &input,
                SharedDialogFormat::from(options.format),
                &metadata,
            )?;
            if options.output.as_os_str() == "-" {
                print!("{text}");
            } else {
                std::fs::write(&options.output, text)?;
                eprintln!("Wrote demo_memory to {}.", options.output.display());
            }
        }
    }
    Ok(())
}

impl From<SharedDialogCliFormat> for SharedDialogFormat {
    fn from(value: SharedDialogCliFormat) -> Self {
        match value {
            SharedDialogCliFormat::Auto => Self::Auto,
            SharedDialogCliFormat::ChatgptShareHtml => Self::ChatGptShareHtml,
            SharedDialogCliFormat::MarkdownTranscript => Self::MarkdownTranscript,
            SharedDialogCliFormat::WebCaptureJson => Self::WebCaptureJson,
        }
    }
}
