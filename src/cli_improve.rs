use std::error::Error;
use std::path::PathBuf;

use crate::load_memory_or_empty;
use formal_ai::{
    agent_info, apply_promotions, export_memory_full, parse_promotion_proposals,
    replay_promotion_gates, BundleInfo, MemoryStore, PromotionRun,
};

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
    pub confirm: bool,
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
            args.confirm,
            "apply the promotion plan and materialize seed edits",
        )?;
    }

    let run = load_promotion_run(args.proposals.as_deref())?;
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
    for session_id in &outcome.agent_session_ids {
        eprintln!("Formal AI Agent session evidence: {session_id}");
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

    Ok(())
}

fn load_promotion_run(proposals: Option<&std::path::Path>) -> Result<PromotionRun, Box<dyn Error>> {
    let path = proposals
        .ok_or("no open proposal document supplied; pass --proposals <promotion_proposals.lino>")?;
    let text = std::fs::read_to_string(path)?;
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
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(path, text)?;
    eprintln!(
        "Wrote full-memory backup with {} event(s) to {}.",
        store.len(),
        path.display()
    );
    Ok(())
}
