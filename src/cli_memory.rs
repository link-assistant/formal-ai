use std::error::Error;
use std::path::Path;

use crate::{read_input, MemoryAction};
use formal_ai::{
    agent_info, apply_dreaming_plan, execute_memory_query, export_memory_full,
    plan_memory_dreaming, render_dreaming_plan, seed_files, suggest_memory_migrations, BundleInfo,
    DreamingConfig, MemoryStore,
};

pub fn run_memory(action: MemoryAction) -> Result<(), Box<dyn Error>> {
    match action {
        MemoryAction::Export {
            path,
            from,
            events_only,
        } => {
            let source = match from {
                Some(explicit) => explicit,
                None if path.as_os_str() == "-" => std::env::var_os("FORMAL_AI_MEMORY_PATH")
                    .map_or_else(
                        || std::path::PathBuf::from("formal-ai-memory.lino"),
                        std::path::PathBuf::from,
                    ),
                None => path.clone(),
            };
            let store = load_memory_or_empty(&source)?;
            let (text, summary) = if events_only {
                let text = store.export_links_notation();
                let summary = format!("Wrote {} events (events-only)", store.len());
                (text, summary)
            } else {
                let seed = seed_files();
                let info = BundleInfo {
                    version: agent_info().get("version").cloned(),
                    ..BundleInfo::default()
                };
                let text = export_memory_full(&seed, store.events(), &[], &info);
                let summary = format!(
                    "Wrote full memory: {} event(s) + {} seed file(s)",
                    store.len(),
                    seed.len(),
                );
                (text, summary)
            };
            if path.as_os_str() == "-" {
                print!("{text}");
            } else {
                std::fs::write(&path, text)?;
                eprintln!("{summary} to {}", path.display());
            }
        }
        MemoryAction::Import { path, into } => {
            let inbound = read_input(&path)?;
            let parsed = formal_ai::import_memory_full(&inbound);
            let parsed_count = parsed.events.len();
            let mut store = load_memory_or_empty(&into)?;
            store.import(&parsed.events);
            store.save_to_file(&into)?;
            eprintln!(
                "Imported {parsed_count} event(s) into {}; total now {}.",
                into.display(),
                store.len()
            );
            let suggestions = suggest_memory_migrations(&parsed, &agent_info());
            for message in suggestions {
                eprintln!("Migration: {message}");
            }
        }
        MemoryAction::Show { path } => {
            let store = load_memory_or_empty(&path)?;
            if store.is_empty() {
                println!("(no events recorded at {})", path.display());
                return Ok(());
            }
            for (index, event) in store.events().iter().enumerate() {
                let role = event.role.as_deref().unwrap_or("?");
                let intent = event.intent.as_deref().unwrap_or("");
                let content = event.content.as_deref().unwrap_or("");
                let stamp = event.sent_at.as_deref().unwrap_or("");
                println!("{index:>3}. [{role}] {intent:<12} {stamp}  {content}");
            }
        }
        MemoryAction::Query { path, prompt } => {
            let mut store = load_memory_or_empty(&path)?;
            match execute_memory_query(&prompt, &mut store, None) {
                Some(execution) => {
                    if execution.changed {
                        store.save_to_file(&path)?;
                    }
                    println!("{}", execution.answer.answer);
                }
                None => println!("No natural-language memory query recognized."),
            }
        }
        MemoryAction::Dream {
            path,
            storage_capacity_bytes,
            free_bytes,
            incoming_bytes,
            target_free_ratio_percent,
            disable_daydreaming,
            apply,
            backup,
            confirm,
        } => {
            if apply && path.as_os_str() == "-" {
                return Err(
                    "Refusing to apply a dreaming plan to stdin/stdout memory path '-'.".into(),
                );
            }
            let mut store = load_memory_or_empty(&path)?;
            let config = DreamingConfig {
                daydreaming_enabled: !disable_daydreaming,
                target_free_ratio_percent,
                storage_capacity_bytes,
                free_bytes,
                incoming_bytes,
            };
            let plan = plan_memory_dreaming(store.events(), &config);
            println!("{}", render_dreaming_plan(&plan));

            if apply {
                require_destructive_confirmation(confirm, "apply dreaming memory plan")?;
                if let Some(backup_path) = backup.as_deref() {
                    write_full_memory_backup(backup_path, &store)?;
                } else {
                    eprintln!(
                        "Warning: no --backup path was provided; run `formal-ai memory export --from {} --path backup.lino` first if you need a copy.",
                        path.display()
                    );
                }
                let outcome = apply_dreaming_plan(&mut store, &plan);
                store.save_to_file(&path)?;
                eprintln!(
                    "Applied dreaming plan to {}; removed {} event(s), estimated {} byte(s) reclaimable, learned {} meta-algorithm amendment(s).",
                    path.display(),
                    outcome.removed_events,
                    outcome.estimated_reclaimed_bytes,
                    outcome.learned_amendments
                );
            } else if !plan.actions.is_empty() {
                eprintln!(
                    "Plan only; rerun with `--apply --confirm` and preferably `--backup` to mutate {}.",
                    path.display()
                );
            }
            if plan.requires_bigger_storage {
                eprintln!(
                    "Dreaming could not meet the requested free-space target from recomputable memory; migrate memory to larger storage or lower the target."
                );
            }
        }
        MemoryAction::PurgeDeleted {
            path,
            backup,
            confirm,
        } => {
            require_destructive_confirmation(confirm, "purge deleted conversations from memory")?;
            let mut store = load_memory_or_empty(&path)?;
            if let Some(backup_path) = backup.as_deref() {
                write_full_memory_backup(backup_path, &store)?;
            } else {
                eprintln!(
                    "Warning: no --backup path was provided; run `formal-ai memory export --from {} --path backup.lino` first if you need a copy.",
                    path.display()
                );
            }
            let removed = store.purge_deleted_conversations();
            store.save_to_file(&path)?;
            eprintln!(
                "Permanently deleted {removed} event(s) from deleted conversation(s) in {}.",
                path.display()
            );
        }
        MemoryAction::Reset {
            path,
            backup,
            confirm,
        } => {
            require_destructive_confirmation(confirm, "reset memory")?;
            let mut store = load_memory_or_empty(&path)?;
            if let Some(backup_path) = backup.as_deref() {
                write_full_memory_backup(backup_path, &store)?;
            } else {
                eprintln!(
                    "Warning: no --backup path was provided; run `formal-ai memory export --from {} --path backup.lino` first if you need a copy.",
                    path.display()
                );
            }
            let removed = store.reset();
            store.save_to_file(&path)?;
            eprintln!(
                "Reset memory at {}; permanently deleted {removed} event(s).",
                path.display()
            );
        }
    }
    Ok(())
}

pub fn load_memory_or_empty(path: &Path) -> Result<MemoryStore, Box<dyn Error>> {
    if path.as_os_str() == "-" {
        return Ok(MemoryStore::new());
    }
    Ok(MemoryStore::load_from_file(path)?)
}

fn require_destructive_confirmation(confirm: bool, action: &str) -> Result<(), Box<dyn Error>> {
    if confirm {
        return Ok(());
    }
    Err(format!(
        "Refusing to {action} because this operation is irreversible. Export memory first or pass --backup, then rerun with --confirm."
    )
    .into())
}

fn write_full_memory_backup(path: &Path, store: &MemoryStore) -> Result<(), Box<dyn Error>> {
    let seed = seed_files();
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
