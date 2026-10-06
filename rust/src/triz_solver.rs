//! User-facing TRIZ/contradiction solver (issue #901).
//!
//! Issue #1138 seeded the forty inventive principles and four separation
//! principles; rust/src/selection_heuristics.rs consumes them as
//! contradiction links with a basis-point value counted from the
//! requirement's clauses. Issue #901 asks for the rest of the ask: the
//! *general* ways paradoxes and contradictions dissolve — other
//! dimensions, alternation, a chosen point on the range — collected as
//! data, replayed against a top-20 benchmark corpus of commonly
//! discussed invention tasks, and reachable from a prompt.
//!
//! All content is seed data (`data/seed/triz-principles.lino`):
//! `triz_resolution_family` records (twelve general families — range
//! selection, dimension change, separations in time/space/condition,
//! supersystem, subsystem split, phase transition, accumulate-then-
//! release, inversion, bypass, partial spectrum), `triz_benchmark_task`
//! records (the top-20 corpus with each task's contradiction and solving
//! methods), and `triz_cues` phrases that mark a prompt as a
//! contradiction question. This module adds no principle names in code —
//! it reads identifiers out of the seed, exactly like the selection
//! heuristic does, so the catalogue stays forgettable and
//! rediscoverable.
//!
//! The answer renders through `data/seed/multilingual-responses-triz.lino`
//! and names the range-selection link explicitly (value in 0-1 from the
//! requirement's clauses, no default 50 %). The dispatch entry is the
//! maintainer's lift.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::parser::parse_lino;
use crate::solver_handlers::finalize_simple;

const SEED_PATH: &str = "data/seed/triz-principles.lino";
const INTENT: &str = "triz_resolution";

/// One `triz_resolution_family` record: a general paradox-resolution
/// method beyond the forty principles.
#[derive(Clone, Debug)]
pub struct ResolutionFamily {
    pub method_id: String,
    pub name: String,
    pub applies_to: String,
    pub mechanism: String,
    pub example: String,
}

/// One `triz_benchmark_task` record: a commonly discussed invention
/// challenge with its contradiction and solving methods.
#[derive(Clone, Debug)]
pub struct BenchmarkTask {
    pub task_id: String,
    pub domain: String,
    pub statement: String,
    pub contradiction: String,
    pub methods: Vec<String>,
    pub notes: String,
}

/// Look up embedded seed content by its registered path.
fn seed_text(path: &str) -> Option<&'static str> {
    crate::seed::seed_files()
        .into_iter()
        .find(|(registered, _)| *registered == path)
        .map(|(_, text)| text)
}

/// Iterate the top-level records of the seed of one type (this seed's
/// records are siblings of the `triz_principles` root, not nested under
/// it). The seed names its records individually
/// (`triz_family_range_selection`, `triz_task_umbrella_crowd`) and types
/// them with a `record_type` field, so the type is read from that field;
/// a record literally named `name` is accepted as well — mirroring
/// `trizRecordsOfType` in js/worker/formal_ai_worker_triz.js.
fn records_of_type(record_type: &str, name: &str) -> Vec<crate::seed::parser::LinoNode> {
    let Some(text) = seed_text(SEED_PATH) else {
        return Vec::new();
    };
    parse_lino(text)
        .children
        .into_iter()
        .filter(|record| {
            record.name == name || record.find_child_value("record_type") == record_type
        })
        .collect()
}

/// The seeded principle identifiers (forty inventive + four separation),
/// for validating that benchmark tasks cite only real methods.
#[cfg(test)]
fn principle_ids() -> Vec<String> {
    ["triz_inventive_principle", "triz_separation_principle"]
        .iter()
        .flat_map(|record_type| records_named_by_type(record_type))
        .collect()
}

/// Top-level records whose `record_type` field matches (this seed types
/// its records by field, and names the records `triz_inventive_01` …).
#[cfg(test)]
fn records_named_by_type(record_type: &str) -> Vec<String> {
    let Some(text) = seed_text(SEED_PATH) else {
        return Vec::new();
    };
    parse_lino(text)
        .children
        .into_iter()
        .filter(|record| record.find_child_value("record_type") == record_type)
        .map(|record| record.find_child_value("principle_id").to_string())
        .filter(|id| !id.is_empty())
        .collect()
}

/// The general resolution families, in seed order.
pub fn triz_families() -> Vec<ResolutionFamily> {
    records_of_type("triz_resolution_family", "triz_resolution_family")
        .iter()
        .filter_map(|record| {
            let method_id = record.find_child_value("method_id").trim().to_owned();
            if method_id.is_empty() {
                return None;
            }
            Some(ResolutionFamily {
                method_id,
                name: record.find_child_value("name").to_string(),
                applies_to: record.find_child_value("applies_to").to_string(),
                mechanism: record.find_child_value("mechanism").to_string(),
                example: record.find_child_value("example").to_string(),
            })
        })
        .collect()
}

/// The top-20 benchmark corpus, in seed order.
pub fn triz_benchmark_tasks() -> Vec<BenchmarkTask> {
    records_of_type("triz_benchmark_task", "triz_benchmark_task")
        .iter()
        .filter_map(|record| {
            let task_id = record.find_child_value("task_id").trim().to_owned();
            if task_id.is_empty() {
                return None;
            }
            Some(BenchmarkTask {
                task_id,
                domain: record.find_child_value("domain").to_string(),
                statement: record.find_child_value("statement").to_string(),
                contradiction: record.find_child_value("contradiction").to_string(),
                methods: record
                    .find_child_value("methods")
                    .split_whitespace()
                    .map(|method| method.to_owned())
                    .collect(),
                notes: record.find_child_value("notes").to_string(),
            })
        })
        .collect()
}

/// The `triz_cues` phrases that mark a prompt as a contradiction
/// question (stemmed fragments match by substring: противоречи-, triz).
fn triz_cues() -> Vec<String> {
    records_of_type("triz_intent_cues", "triz_cues")
        .iter()
        .flat_map(|record| {
            record
                .children
                .iter()
                .filter(|child| child.name == "phrase")
        })
        .filter_map(|child| {
            let phrase = child.id.trim().to_lowercase();
            (!phrase.is_empty()).then_some(phrase)
        })
        .collect()
}

/// Every benchmark task whose statement or domain shares a word with the
/// prompt, best match first (words of five or more characters, so
/// stopwords do not carry the match). Empty when nothing overlaps.
fn relevant_tasks<'a>(prompt: &str, tasks: &'a [BenchmarkTask]) -> Vec<&'a BenchmarkTask> {
    let words: Vec<&str> = prompt
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|word| word.chars().count() >= 5)
        .collect();
    let mut scored: Vec<(usize, &BenchmarkTask)> = tasks
        .iter()
        .filter_map(|task| {
            let haystack = format!("{} {}", task.domain, task.statement).to_lowercase();
            let score = words
                .iter()
                .filter(|word| haystack.contains(word.to_lowercase().as_str()))
                .count();
            (score > 0).then_some((score, task))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.task_id.cmp(&b.1.task_id)));
    scored.into_iter().map(|(_, task)| task).collect()
}

/// Render the families block: `Name — mechanism` one per line.
fn families_text(families: &[ResolutionFamily]) -> String {
    families
        .iter()
        .map(|family| format!("- {}: {}", family.name, family.mechanism))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Render the precedent block: statement, contradiction, methods.
fn precedents_text(tasks: &[&BenchmarkTask]) -> String {
    tasks
        .iter()
        .map(|task| {
            format!(
                "- {} ({}: {}) — methods: {}",
                task.statement,
                task.domain,
                task.contradiction,
                task.methods.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The range-selection note, tying this answer to what
/// rust/src/selection_heuristics.rs computes for candidate sets.
const LINK_NOTE: &str = "A contradiction is a link whose value (0-1) is chosen from the requirement's own clauses — basis points in the selection heuristic, with no default 50 %. Say which side the requirements favor and the point on the range follows.";

/// Try to recognize a contradiction/invention question and answer with
/// the seeded resolution families and relevant benchmark precedents.
/// Returns `None` when no cue matches.
pub fn handle_triz(prompt: &str, normalized: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let families = triz_families();
    let tasks = triz_benchmark_tasks();
    let lower = prompt.to_lowercase();
    let cued = triz_cues()
        .into_iter()
        .any(|cue| normalized.contains(cue.as_str()) || lower.contains(cue.as_str()));
    if !cued {
        return None;
    }
    let relevant = relevant_tasks(prompt, &tasks);
    log.append("triz_solver:cued", "contradiction question");
    for task in &relevant {
        log.append("triz_solver:precedent", task.task_id.clone());
    }
    // With no overlapping task, cite the first three corpus entries as
    // canonical shape examples rather than nothing.
    let cited: Vec<&BenchmarkTask> = if relevant.is_empty() {
        tasks.iter().take(3).collect()
    } else {
        relevant.iter().take(3).copied().collect()
    };
    let body = {
        let mut out =
            crate::seed::localized_response("triz_resolution_map", "en").unwrap_or_default();
        for (key, value) in [
            ("families", families_text(&families)),
            ("precedents", precedents_text(&cited)),
            ("link_note", LINK_NOTE.to_owned()),
        ] {
            out = out.replace(&format!("{{{key}}}"), &value);
        }
        out
    };
    Some(finalize_simple(
        prompt,
        log,
        INTENT,
        "response:triz_resolution_map",
        &body,
        0.7,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn families_and_benchmark_load_from_the_seed() {
        let families = triz_families();
        assert!(
            families.len() >= 12,
            "twelve general families, got {}",
            families.len()
        );
        assert!(
            families
                .iter()
                .any(|family| family.method_id == "family_range_selection")
        );
        let tasks = triz_benchmark_tasks();
        assert_eq!(tasks.len(), 20, "the top-20 corpus");
        for task in &tasks {
            assert!(
                !task.methods.is_empty(),
                "{} must name its methods",
                task.task_id
            );
        }
        // Every method a task names must exist as a family or a seeded
        // principle identifier — the corpus cannot cite ghosts.
        let known: Vec<String> = families
            .iter()
            .map(|family| family.method_id.clone())
            .chain(principle_ids())
            .collect();
        for task in &tasks {
            for method in &task.methods {
                assert!(
                    known.iter().any(|id| id == method),
                    "{} cites unknown method {method}",
                    task.task_id
                );
            }
        }
    }

    #[test]
    fn umbrella_prompt_finds_the_umbrella_precedent_first() {
        let tasks = triz_benchmark_tasks();
        let relevant = relevant_tasks(
            "How do I design an umbrella that is big enough in rain but small in a crowded bus?",
            &tasks,
        );
        assert_eq!(
            relevant.first().map(|task| task.task_id.as_str()),
            Some("umbrella_crowd")
        );
    }

    #[test]
    fn stems_match_russian_and_english_cues() {
        let cues = triz_cues();
        assert!(cues.iter().any(|cue| cue == "противоречи"));
        assert!(cues.iter().any(|cue| cue == "contradiction"));
    }
}
