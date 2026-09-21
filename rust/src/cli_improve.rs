use std::error::Error;
use std::path::PathBuf;

use crate::cli_memory::load_memory_or_empty;
use formal_ai::promotion::open_draft_pull_request;
use formal_ai::{
    BundleInfo, MemoryStore, PromotionRun, agent_info, apply_promotions, export_memory_full,
    parse_promotion_proposals, replay_promotion_gates,
};

/// Explicit acknowledgement for destructive promotion materialization.
///
/// Keeping this state typed prevents callers from accidentally confusing the
/// confirmation bit with the other independent command modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestructiveConfirmation {
    /// The caller did not acknowledge the destructive action.
    Absent,
    /// The caller explicitly acknowledged the destructive action.
    Confirmed,
}

impl DestructiveConfirmation {
    const fn is_confirmed(self) -> bool {
        matches!(self, Self::Confirmed)
    }
}

impl From<bool> for DestructiveConfirmation {
    fn from(confirmed: bool) -> Self {
        if confirmed {
            Self::Confirmed
        } else {
            Self::Absent
        }
    }
}

/// Arguments for `formal-ai improve` (issue #656, E37).
#[derive(Debug)]
pub struct ImproveArgs {
    /// Run the benchmark-gated promotion protocol.
    pub promote: bool,
    /// Optional `promotion_proposals` Links Notation document.
    pub proposals: Option<PathBuf>,
    /// Workspace root the accepted seed edits are materialized into on `--apply`.
    pub seed_root: PathBuf,
    /// Optional memory file the promotion event chain is appended to on `--apply`.
    pub memory: Option<PathBuf>,
    /// Materialize the accepted seed edits. Requires `--confirm`.
    pub apply: bool,
    /// Optional full-memory backup written before applying to `--memory`.
    pub backup: Option<PathBuf>,
    /// Required acknowledgement when `--apply` is used.
    pub confirm: DestructiveConfirmation,
    /// Open the review as a draft pull request instead of printing the plan
    /// for a human to run (issue #1138 B7, plan 07 leaf 13).
    ///
    /// Opt-in and `false` by default, so #656's behaviour is unchanged when the
    /// flag is absent: the protocol still only prints the branch plan.
    pub open_draft_pr: bool,
}

/// Drive the promotion protocol: replay each proposal's benchmark ratchets,
/// print the resulting plan, and — only under `--apply --confirm` — materialize
/// the accepted seed edits onto a newly created local review branch. Never
/// pushes: committing and opening the draft PR are printed for human review.
pub fn run_improve(args: &ImproveArgs) -> Result<(), Box<dyn Error>> {
    if !args.promote {
        println!(
            "formal-ai improve — benchmark-gated promotion of self-improvement proposals (issue #656).\n\
             \n\
             Pass --promote to replay open proposals against their benchmark ratchets and print the\n\
             promotion plan (dry run; touches no files). Add --apply --confirm to materialize the\n\
             accepted `.lino` seed edits onto a local review branch in --seed-root. Promotion never\n\
             pushes: commit/draft-PR steps require human review, and rejected proposals are kept as\n\
             failure records."
        );
        return Ok(());
    }

    // Refuse destructive use before loading proposals or spending time on gate
    // replay. No command is run and no file is touched without acknowledgement.
    if args.apply {
        require_destructive_confirmation(
            args.confirm.is_confirmed(),
            "apply the promotion plan and materialize seed edits",
        )?;
    }

    let run = load_promotion_run(args.proposals.as_deref(), args.memory.as_deref())?;
    println!("{}", run.links_notation());

    let promoted = run.promoted().len();
    let rejected = run.rejected().len();
    eprintln!(
        "Promotion replay: {} considered, {promoted} promoted, {rejected} rejected.",
        run.records.len()
    );

    if !args.apply {
        eprintln!(
            "Dry run only; no files were changed. Rerun with `--apply --confirm` and preferably \
             `--seed-root <workspace>` to materialize the {promoted} accepted seed edit(s)."
        );
        return Ok(());
    }

    let outcome = apply_promotions(&run, &args.seed_root)?;
    for edit in &outcome.applied {
        eprintln!(
            "Materialized {} ({} byte(s)) at {}.",
            edit.seed_file,
            edit.bytes_written,
            edit.path.display()
        );
    }
    // These are FNV-1a digests of the recorded session JSON, not credentials:
    // the same value is committed as evidence under `docs/case-studies/`. Naming
    // them `session_id` made CodeQL's `rust/cleartext-logging` heuristic — which
    // treats any name matching `session.?(id|key)` as account information — read
    // this evidence line as leaking a session token.
    for digest in &outcome.agent_session_digests {
        eprintln!("Formal AI Agent session evidence: {digest}");
    }
    if !outcome.rejected.is_empty() {
        eprintln!(
            "Preserved {} rejected proposal(s) as failure record(s); their edits were NOT applied.",
            outcome.rejected.len()
        );
    }

    if let Some(memory_path) = args.memory.as_deref() {
        let mut store = load_memory_or_empty(memory_path)?;
        if let Some(backup_path) = args.backup.as_deref() {
            write_full_memory_backup(backup_path, &store)?;
        }
        let events = run.memory_events();
        let appended = store.import(&events);
        store.save_to_file(memory_path)?;
        eprintln!(
            "Appended {appended} promotion event(s) to {}; total now {}.",
            memory_path.display(),
            store.len()
        );
    }

    let plan = outcome.branch_plan;
    eprintln!(
        "Created local review branch {}. Remaining PR plan (never pushed; required CI and human review remain outer gates):",
        plan.branch
    );
    for command in plan
        .commands
        .iter()
        .filter(|command| !command.starts_with("git checkout -b "))
    {
        eprintln!("    {command}");
    }

    // Issue #1138 B7, plan 07 leaf 13. Without the flag nothing below runs and
    // the printed plan above is the whole of the protocol's last step, exactly
    // as #656 left it. With it, the draft is an artifact with a head, a base
    // and an append-only `promotion_published` event -- never the default
    // branch, never ready, never merged.
    if args.open_draft_pr {
        let draft = open_draft_pull_request(&run)?;
        println!("{}", draft.links_notation());
        if let Some(memory_path) = args.memory.as_deref() {
            let mut store = load_memory_or_empty(memory_path)?;
            let appended = store.import(&draft.memory_events());
            store.save_to_file(memory_path)?;
            eprintln!(
                "Appended {appended} promotion event(s) to {}; total now {}.",
                memory_path.display(),
                store.len()
            );
        }
    }

    Ok(())
}

fn load_promotion_run(
    proposals: Option<&std::path::Path>,
    memory: Option<&std::path::Path>,
) -> Result<PromotionRun, Box<dyn Error>> {
    let discovered;
    let path = if let Some(path) = proposals {
        path
    } else if let Some(memory_path) = memory {
        discovered = formal_ai::dreaming_runtime::learning_cycle_record_path(memory_path);
        discovered.as_path()
    } else {
        return Err(
            "no open proposal document supplied; pass --proposals <promotion_proposals.lino> \
             or --memory <memory.lino> to consume its latest dreaming-cycle proposals"
                .into(),
        );
    };
    let text = std::fs::read_to_string(path).map_err(|error| {
        formal_ai::repository_workspace::render_protocol_template(
            "promotion_proposals_read_error",
            &[
                ("path", &path.display().to_string()),
                ("error", &error.to_string()),
            ],
        )
        .unwrap_or_else(|| String::from("promotion_proposals_read_error"))
    })?;
    let parsed = parse_promotion_proposals(&text)
        .map_err(|error| format!("could not parse {}: {error}", path.display()))?;
    if parsed.is_empty() {
        return Err("proposal document contains no open proposals".into());
    }
    eprintln!(
        "Replaying coding-modification, industry, and unit-specification gates from canonical commands..."
    );
    let root = std::env::current_dir()?;
    let replayed = replay_promotion_gates(parsed, &root)
        .map_err(|error| format!("promotion gate replay failed: {error}"))?;
    Ok(PromotionRun::evaluate(replayed))
}

fn require_destructive_confirmation(confirm: bool, action: &str) -> Result<(), Box<dyn Error>> {
    if confirm {
        return Ok(());
    }
    Err(format!(
        "Refusing to {action} without --confirm. Rerun with --apply --confirm (and preferably --backup)."
    )
    .into())
}

fn write_full_memory_backup(
    path: &std::path::Path,
    store: &MemoryStore,
) -> Result<(), Box<dyn Error>> {
    let seed = formal_ai::seed_files();
    let info = BundleInfo {
        version: agent_info().get("version").cloned(),
        ..BundleInfo::default()
    };
    let text = export_memory_full(&seed, store.events(), &[], &info);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)?;
    eprintln!(
        "Wrote full-memory backup with {} event(s) to {}.",
        store.len(),
        path.display()
    );
    Ok(())
}
