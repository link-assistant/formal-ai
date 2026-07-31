//! Record the *language* learning frontier (issue #706).
//!
//! Issue #701 gave the engine a general adoption cycle: a recorded frontier of
//! prompts it cannot route becomes a derived request frame, a held-out
//! validation, and a promotion proposal. That cycle is frontier-agnostic — it
//! takes a slug and a list of [`FrontierItem`]s and never mentions Google
//! Trends. This module supplies the *second* frontier: the prompts a newly
//! registered language brings with it.
//!
//! The pipeline a non-Rust language contributor follows is therefore closed:
//!
//! 1. They register the language in `data/seed/language-detection.lino` and
//!    describe it in `data/language-additions/<slug>.lino` — the same candidate
//!    file `scripts/language-protocol.mjs` already reads.
//! 2. The candidate file carries a **prompt corpus**: request frames in the new
//!    language, each over a query surface that is already committed in
//!    `data/seed/meanings-translation.lino`, so the corpus adds frames only.
//! 3. [`record_language_gap_frontier`] runs the *live engine* over that corpus
//!    and records what actually happens — the intent the engine returns and
//!    whether the answer is the explicit language-gap text. Nothing here is
//!    asserted by hand; a prompt the engine already routes never lands on the
//!    frontier.
//! 4. `formal-ai learn cycle --frontier language-gap` replays the record with
//!    the *same* cycle the Google Trends frontier uses, and every class that
//!    cannot be generalized is preserved as a blocked record naming the missing
//!    data (R425) rather than dropped.
//!
//! A language whose candidate file carries no prompts is not silently skipped:
//! it is recorded as a `frontier_gap` naming exactly what the contributor must
//! add next, which is the honest counterpart of the `language_gap` coverage
//! ledger.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::engine::FormalAiEngine;
use crate::engine_responses::unknown_language_fallback_answer;
use crate::seed::parser::parse_lino;

/// One prompt recorded from a language candidate file.
#[derive(Debug, Clone)]
pub struct CandidatePrompt {
    /// One-based rank inside its class, mirroring the trends frontier shape.
    pub rank: usize,
    /// The query surface the frame is built around.
    pub query: String,
    /// The stable class key (`concept_lookup`, `tell_me_about`, …).
    pub variation: String,
    /// The prompt text in the candidate language.
    pub prompt: String,
}

/// A parsed `data/language-additions/<slug>.lino` candidate file.
#[derive(Debug, Clone)]
pub struct LanguageCandidate {
    /// The language slug the file is named for.
    pub language: String,
    /// The candidate's English name, when declared.
    pub name: String,
    /// The prompt corpus the contributor supplied, possibly empty.
    pub prompts: Vec<CandidatePrompt>,
}

/// Parse one candidate document.
#[must_use]
pub fn parse_language_candidate(document: &str) -> Option<LanguageCandidate> {
    let tree = parse_lino(document);
    let root = tree
        .children
        .iter()
        .find(|node| node.name == "language_candidate")?;
    let mut candidate = LanguageCandidate {
        language: root.id.clone(),
        name: root.find_child_value("name").to_owned(),
        prompts: Vec::new(),
    };
    for node in &root.children {
        if node.name != "prompt" || node.children.is_empty() {
            continue;
        }
        candidate.prompts.push(CandidatePrompt {
            rank: node.find_child_value("rank").parse().unwrap_or_default(),
            query: node.find_child_value("query").to_owned(),
            variation: node.find_child_value("variation").to_owned(),
            prompt: node.find_child_value("prompt").to_owned(),
        });
    }
    Some(candidate)
}

/// Read every candidate file in `directory`, sorted by file name so the record
/// is deterministic.
///
/// # Errors
///
/// Returns an error when the directory cannot be listed or a file cannot be
/// read.
pub fn load_language_candidates(directory: &Path) -> Result<Vec<LanguageCandidate>, String> {
    let mut paths: Vec<_> = fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "lino"))
        .collect();
    paths.sort();

    let mut candidates = Vec::new();
    for path in paths {
        let document =
            fs::read_to_string(&path)
                .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        if let Some(candidate) = parse_language_candidate(&document) {
            candidates.push(candidate);
        }
    }
    Ok(candidates)
}

/// Escape a value for a quoted Links Notation field.
fn quote(value: &str) -> String {
    value.replace('"', "\"\"")
}

/// Run the live engine over every candidate prompt and render the recorded
/// `learning_frontier` document.
///
/// Only prompts the engine *fails* are recorded: a prompt that already routes
/// to a real intent and is answered in its own language is coverage, not a
/// frontier item.
#[must_use]
pub fn render_language_gap_frontier(candidates: &[LanguageCandidate]) -> String {
    let engine = FormalAiEngine;
    let gap_answer = unknown_language_fallback_answer();

    let mut items = Vec::new();
    let mut gaps = Vec::new();
    let mut probed = 0usize;
    for candidate in candidates {
        if candidate.prompts.is_empty() {
            gaps.push((
                candidate.language.clone(),
                String::from("no_prompt_corpus_in_language_addition_file"),
            ));
            continue;
        }
        let mut recorded = 0usize;
        for prompt in &candidate.prompts {
            probed += 1;
            let answer = engine.answer(&prompt.prompt);
            let is_gap = answer.answer.contains(gap_answer) || answer.intent == "unknown";
            if !is_gap {
                continue;
            }
            recorded += 1;
            items.push((candidate.language.clone(), prompt.clone(), answer.intent));
        }
        if recorded == 0 {
            gaps.push((
                candidate.language.clone(),
                String::from("every_recorded_prompt_already_routes"),
            ));
        }
    }

    let mut document = String::from("learning_frontier\n");
    let _ = writeln!(document, "  record_type \"learning_frontier_record\"");
    let _ = writeln!(document, "  frontier \"language-gap\"");
    let _ = writeln!(document, "  issue \"706\"");
    let _ = writeln!(
        document,
        "  recorded_from \"data/language-additions/*.lino via examples/issue_706_language_frontier.rs\""
    );
    let _ = writeln!(
        document,
        "  summary \"Every prompt a registered or candidate language supplied that the live engine still cannot answer in that language. Recorded by running the engine, not by hand: a prompt that already routes is coverage and never appears here. Replayed by 'formal-ai learn cycle --frontier language-gap', which derives the language's request frame from two prompts of a class and validates it on the rest.\""
    );
    let _ = writeln!(document, "  total_prompts \"{probed}\"");
    let _ = writeln!(document, "  learning_frontier \"{}\"", items.len());
    for (language, prompt, intent) in &items {
        let _ = writeln!(document, "  frontier_prompt");
        let _ = writeln!(document, "    rank \"{}\"", prompt.rank);
        let _ = writeln!(document, "    query \"{}\"", quote(&prompt.query));
        let _ = writeln!(document, "    language \"{language}\"");
        let _ = writeln!(document, "    variation \"{}\"", quote(&prompt.variation));
        let _ = writeln!(document, "    prompt \"{}\"", quote(&prompt.prompt));
        let _ = writeln!(document, "    engine_intent \"{}\"", quote(intent));
        let _ = writeln!(document, "    routed_to \"human_triage\"");
    }
    for (language, reason) in &gaps {
        let _ = writeln!(document, "  frontier_gap");
        let _ = writeln!(document, "    language \"{language}\"");
        let _ = writeln!(document, "    reason \"{reason}\"");
    }
    document
}

/// Record the language frontier from a candidate directory.
///
/// # Errors
///
/// Returns an error when the directory cannot be read.
pub fn record_language_gap_frontier(directory: &Path) -> Result<String, String> {
    Ok(render_language_gap_frontier(&load_language_candidates(
        directory,
    )?))
}
