use std::error::Error;

use clap::CommandFactory;
use formal_ai::{WithFormalAiArgs, delimit_tool_args, run_with_formal_ai};
use lino_arguments::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "with-formal-ai",
    version,
    about = "Run or permanently configure external CLIs against Formal AI"
)]
struct Args {
    #[command(flatten)]
    with: WithFormalAiArgs,
}

fn main() -> Result<(), Box<dyn Error>> {
    lino_arguments::init();
    // Everything after the tool name belongs to the wrapped client.
    let args = Args::parse_from(delimit_tool_args(
        std::env::args_os().collect(),
        &Args::command(),
    ));
    formal_ai::dialog_log::structured_log::configure(
        formal_ai::cli_env::flag_enabled("FORMAL_AI_SILENT"),
        false,
    );
    run_with_formal_ai(&args.with)
}
