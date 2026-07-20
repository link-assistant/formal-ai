use std::error::Error;
use std::fs;
use std::path::PathBuf;

use formal_ai::learning_cycle::{
    run_learning_cycle, LearningCycleRun, GOOGLE_TRENDS_FRONTIER,
    GOOGLE_TRENDS_FRONTIER_RECORD,
};
use formal_ai::promotion::render_promotion_proposals;
use formal_ai::{parse_frontier_record, FrontierItem};

/// Which recorded learning frontier `formal-ai learn cycle` replays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum LearnFrontier {
    /// The Google Trends frontier recorded by issues #498/#499.
    GoogleTrends,
}

/// Arguments for `formal-ai learn cycle` (issue #701, E59).
#[derive(Debug)]
pub struct LearnCycleArgs {
    /// The recorded frontier to run the cycle over.
    pub frontier: LearnFrontier,
    /// Read frontier items from this file instead of the committed record.
    pub from: Option<PathBuf>,
    /// Explicit acknowledgement that the run only proposes. Always true today.
    pub dry_run: bool,
    /// Print the promotion proposals as a `promotion_proposals` document that
    /// `formal-ai improve --promote --proposals -` consumes.
    pub proposals: bool,
}

/// Run one learning cycle and print its auditable record.
///
/// The cycle is proposal-only by construction: it derives candidate seed edits
/// from a recorded frontier, validates them against held-out prompts of the same
/// class, and prints promotion proposals for the human-gated issue-#656
/// protocol. It never writes a seed file and never runs the network, so
/// `--dry-run` is the default and the run is reproducible offline.
///
/// # Errors
///
/// Returns an error when `--from` cannot be read.
pub fn run_learn_cycle(args: &LearnCycleArgs) -> Result<(), Box<dyn Error>> {
    let (frontier, items) = load_frontier(args)?;
    let run = run_learning_cycle(&frontier, &items);

    if args.proposals {
        println!("{}", render_promotion_proposals(&run.proposals));
    } else {
        println!("{}", run.links_notation());
    }
    report(&run, args.dry_run);
    Ok(())
}

fn load_frontier(args: &LearnCycleArgs) -> Result<(String, Vec<FrontierItem>), Box<dyn Error>> {
    match &args.from {
        Some(path) => {
            let document = fs::read_to_string(path)?;
            Ok((
                String::from("custom"),
                parse_frontier_record(&document),
            ))
        }
        None => match args.frontier {
            LearnFrontier::GoogleTrends => Ok((
                String::from(GOOGLE_TRENDS_FRONTIER),
                parse_frontier_record(GOOGLE_TRENDS_FRONTIER_RECORD),
            )),
        },
    }
}

/// Summarise the run on stderr so the stdout document stays machine-readable.
fn report(run: &LearningCycleRun, dry_run: bool) {
    eprintln!(
        "Learning cycle over '{}': {} frontier item(s), {} validated candidate(s) of {}, \
         {} held-out test(s), {} proposal(s), {} blocked class(es).",
        run.frontier,
        run.frontier_items,
        run.validated_candidates().len(),
        run.candidates.len(),
        run.held_out_count(),
        run.proposals.len(),
        run.blocked.len()
    );
    if !dry_run {
        eprintln!(
            "Note: the cycle is proposal-only whether or not --dry-run is passed; adoption stays \
             behind the human-gated issue-#656 promotion protocol."
        );
    }
    for blocked in &run.blocked {
        eprintln!(
            "  blocked: {}/{} — {} (kept as a durable frontier record)",
            blocked.language, blocked.variation, blocked.reason
        );
    }
    eprintln!(
        "Proposal-only run; no seed file was written. Pipe `--proposals` into \
         `formal-ai improve --promote --proposals <file>` to replay the canonical gates."
    );
}
