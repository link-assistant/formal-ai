use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use clap::Subcommand;
use formal_ai::algorithm_discovery::{discover_algorithms, traces_from_memory_events};
use formal_ai::learning_cycle::{
    GOOGLE_TRENDS_FRONTIER, LearningCycleRun, recorded_frontier, recorded_frontiers,
    run_learning_cycle,
};
use formal_ai::promotion::render_promotion_proposals;
use formal_ai::{FrontierItem, MemoryStore, parse_frontier_record, render_response};

/// Auto-learning commands.
#[derive(Debug, Subcommand)]
pub enum LearnAction {
    /// Derive candidate knowledge from a recorded frontier, validate it against
    /// held-out prompts of the same class, and emit promotion proposals in the
    /// issue-#656 shape. Proposal-only and offline: no seed file is written and
    /// no network call is made, so the run is deterministic and reproducible.
    Cycle {
        /// The slug of the recorded frontier to replay. Resolved through
        /// `learning_cycle::recorded_frontiers()`, so registering a new
        /// frontier record makes it selectable here without a code change to
        /// the argument parser.
        #[arg(long)]
        frontier: Option<String>,

        /// Read a frontier report or all pages of a split report in order.
        #[arg(long, value_name = "PATH", num_args = 1..)]
        from: Vec<PathBuf>,

        /// Explicit acknowledgement that the cycle only proposes. The cycle is
        /// proposal-only either way; the flag documents the intent at the call
        /// site and keeps the acceptance-criteria invocation literal.
        #[arg(long, default_value_t = false)]
        dry_run: bool,

        /// Print the `promotion_proposals` document instead of the full cycle
        /// record, ready to pipe into `formal-ai improve --promote`.
        #[arg(long, default_value_t = false)]
        proposals: bool,
    },
    /// Mine repeated tool/event sequences from portable memory, infer shared
    /// parameters, and validate candidates on held-out occurrences. The output
    /// is proposal-only and cannot execute without promotion.
    Algorithms {
        /// Portable `demo_memory` or full-memory bundle containing observations.
        #[arg(long, value_name = "PATH")]
        from: PathBuf,

        /// Write the discovery artifact to this path instead of stdout.
        #[arg(long, value_name = "PATH")]
        output: Option<PathBuf>,
    },
}

/// Arguments for `formal-ai learn cycle` (issue #701, E59).
#[derive(Debug)]
pub struct LearnCycleArgs {
    /// The slug of the recorded frontier to run the cycle over.
    pub frontier: Option<String>,
    /// Read frontier items from these files in the supplied order.
    pub from: Vec<PathBuf>,
    /// Explicit acknowledgement that the run only proposes. Always true today.
    pub dry_run: bool,
    /// Print the promotion proposals as a `promotion_proposals` document that
    /// `formal-ai improve --promote --proposals -` consumes.
    pub proposals: bool,
}

/// Dispatch one auto-learning command.
///
/// # Errors
///
/// Returns an error when a cycle has missing, unreadable, or conflicting reports.
pub fn run_learn_action(action: LearnAction) -> Result<(), Box<dyn Error>> {
    match action {
        LearnAction::Cycle {
            frontier,
            from,
            dry_run,
            proposals,
        } => run_learn_cycle(&LearnCycleArgs {
            frontier,
            from,
            dry_run,
            proposals,
        }),
        LearnAction::Algorithms { from, output } => {
            run_algorithm_learning(&from, output.as_deref())
        }
    }
}

fn run_algorithm_learning(from: &Path, output: Option<&Path>) -> Result<(), Box<dyn Error>> {
    if !from.exists() {
        let path = from.display().to_string();
        let message = render_response("algorithm_observation_missing", "en", &[("path", &path)])
            .unwrap_or_default();
        return Err(message.into());
    }
    let store = MemoryStore::load_from_file(from)?;
    let run = discover_algorithms(&traces_from_memory_events(store.events()));
    let document = run.links_notation();
    if let Some(path) = output {
        formal_ai::memory::write_locked_atomic(path, &document)?;
    } else {
        print!("{document}");
    }
    let traces = run.trace_count.to_string();
    let candidates = run.candidates.len().to_string();
    let validated = run.validated_candidates().len().to_string();
    let summary = render_response(
        "algorithm_discovery_summary",
        "en",
        &[
            ("traces", &traces),
            ("candidates", &candidates),
            ("validated", &validated),
        ],
    )
    .unwrap_or_default();
    formal_ai::diagnostic!("debug"; "{summary}");
    Ok(())
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
/// Returns an error when `--from` is missing, unreadable, or combines different frontiers.
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
    if !args.from.is_empty() {
        let mut frontier = args.frontier.clone();
        let mut items = Vec::new();
        for path in &args.from {
            let document = fs::read_to_string(path)?;
            let tree = formal_ai::seed::parse_lino(&document);
            for root in &tree.children {
                let declared = if root.name == "google_trends_learning" {
                    GOOGLE_TRENDS_FRONTIER
                } else {
                    root.find_child_value("frontier")
                };
                if declared.is_empty() {
                    continue;
                }
                if frontier
                    .as_ref()
                    .is_some_and(|known| known.as_str() != declared)
                {
                    return Err(format!("{} declares a different frontier", path.display()).into());
                }
                frontier.get_or_insert_with(|| declared.to_owned());
            }
            items.extend(parse_frontier_record(&document));
        }
        let slug = frontier
            .as_deref()
            .and_then(recorded_frontier)
            .map_or("custom", |frontier| frontier.slug);
        return Ok((slug.to_owned(), items));
    }
    let requested = args.frontier.as_deref().unwrap_or(GOOGLE_TRENDS_FRONTIER);
    let Some(frontier) = recorded_frontier(requested) else {
        return Err(format!(
            "unknown frontier '{}'. Recorded frontiers: {}",
            requested,
            recorded_frontiers()
                .iter()
                .map(|frontier| format!("{} ({})", frontier.slug, frontier.summary))
                .collect::<Vec<_>>()
                .join(", ")
        )
        .into());
    };
    let message = render_response(
        "learning-frontier-report-required",
        "en",
        &[("frontier", frontier.slug)],
    )
    .unwrap_or_default();
    Err(message.into())
}

/// Summarise the run on stderr so the stdout document stays machine-readable.
fn report(run: &LearningCycleRun, dry_run: bool) {
    formal_ai::diagnostic!("debug";
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
        formal_ai::diagnostic!("debug";
            "Note: the cycle is proposal-only whether or not --dry-run is passed; adoption stays \
             behind the human-gated issue-#656 promotion protocol."
        );
    }
    for blocked in &run.blocked {
        formal_ai::diagnostic!("debug";
            "  blocked: {}/{} — {} (kept as a durable frontier record)",
            blocked.language, blocked.variation, blocked.reason
        );
    }
    formal_ai::diagnostic!("debug";
        "Proposal-only run; no seed file was written. Pipe `--proposals` into \
         `formal-ai improve --promote --proposals <file>` to replay the canonical gates."
    );
}
