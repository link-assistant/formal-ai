//! Export and import of portable seed-plus-memory bundles.

use std::error::Error;

use formal_ai::{
    MemoryStore, agent_info, export_memory_bundle, import_memory_full, merged_bundle, parse_bundle,
    seed_files, suggest_memory_migrations,
};

use crate::cli_memory::load_memory_or_empty;
use crate::{BundleAction, read_input};

pub fn run_bundle(action: BundleAction) -> Result<(), Box<dyn Error>> {
    match action {
        BundleAction::Export { path, memory } => {
            let store = match memory {
                Some(memory_path) => load_memory_or_empty(&memory_path)?,
                None => MemoryStore::new(),
            };
            let bundle = if store.is_empty() {
                merged_bundle()
            } else {
                export_memory_bundle(&seed_files(), store.events())
            };
            if path.as_os_str() == "-" {
                print!("{bundle}");
            } else {
                std::fs::write(&path, bundle)?;
                eprintln!(
                    "Wrote bundle with {} seed file(s) and {} event(s) to {}",
                    seed_files().len(),
                    store.len(),
                    path.display()
                );
            }
        }
        BundleAction::Import { path, into } => {
            let text = read_input(&path)?;
            let parsed = import_memory_full(&text);
            if parsed.events.is_empty() && parsed.seed_files.is_empty() {
                return Err(format!(
                    "{} does not appear to be a formal_ai_bundle Links Notation document",
                    path.display()
                )
                .into());
            }
            let parsed_seed = parse_bundle(&text);
            let mut store = load_memory_or_empty(&into)?;
            store.import(&parsed.events);
            // Seed files become recomputable `seed_cache` events so seed data
            // participates in usage/eviction accounting (issue #494).
            let known: std::collections::BTreeSet<String> = store
                .events()
                .iter()
                .map(|event| event.id.clone())
                .collect();
            let fresh_seed: Vec<_> = formal_ai::seed_cache_events(&parsed.seed_files)
                .into_iter()
                .filter(|event| !known.contains(&event.id))
                .collect();
            store.import(&fresh_seed);
            store.save_to_file(&into)?;
            eprintln!(
                "Imported {} event(s) and saw {} seed file(s); memory now has {} event(s) at {}.",
                parsed.events.len(),
                parsed_seed.len(),
                store.len(),
                into.display(),
            );
            for message in suggest_memory_migrations(&parsed, &agent_info()) {
                eprintln!("Migration: {message}");
            }
        }
    }
    Ok(())
}
