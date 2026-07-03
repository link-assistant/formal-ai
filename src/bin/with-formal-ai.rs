use std::error::Error;

use formal_ai::{run_with_formal_ai, WithFormalAiArgs};
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
    let args = Args::parse();
    run_with_formal_ai(&args.with)
}
