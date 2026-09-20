//! Issue #1138, plan 01 L11 — the universal loop stops claiming it cannot fetch.
//!
//! `src/solver.rs` today appends `policy:no_fetch_capability` at step 7 and
//! returns, so a word the seed does not contain has no route to a meaning. After
//! plan 01 L11 the loop asks the registry instead, in every registered language,
//! and the offline boundary stays explicit rather than becoming a silent
//! capability claim.
//!
//! The held-out prompts come from `data/benchmarks/concept-lookup-paraphrases.lino`
//! (family `lipogram`); none of them explains what the word means.

use std::fs;
use std::path::Path;

use formal_ai::concept_lookup::unknown_surfaces;
use formal_ai::solver::{SolverConfig, UniversalSolver};

const CORPUS: &str = "data/benchmarks/concept-lookup-paraphrases.lino";
const FORBIDDEN_POLICY: &str = "policy:no_fetch_capability";

struct Paraphrase {
    family: String,
    language: String,
    prompt: String,
}

fn corpus() -> Vec<Paraphrase> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let text = fs::read_to_string(root.join(CORPUS)).expect("concept-lookup corpus");
    let mut out = Vec::new();
    let mut current: Option<Paraphrase> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if line.starts_with("  paraphrase ") {
            if let Some(record) = current.take() {
                out.push(record);
            }
            current = Some(Paraphrase {
                family: String::new(),
                language: String::new(),
                prompt: String::new(),
            });
        } else if let Some(record) = &mut current {
            if let Some(value) = trimmed.strip_prefix("family ") {
                value.trim().clone_into(&mut record.family);
            } else if let Some(value) = trimmed.strip_prefix("language ") {
                value.trim().clone_into(&mut record.language);
            } else if let Some(value) = trimmed.strip_prefix("prompt ") {
                record.prompt = value.trim().trim_matches('"').replace("\"\"", "\"");
            }
        }
    }
    out.extend(current);
    out
}

fn loop_prompts() -> Vec<Paraphrase> {
    corpus()
        .into_iter()
        .filter(|case| case.family == "lipogram")
        .collect()
}

#[test]
fn the_loop_asks_about_an_unresolved_word_in_every_registered_language() {
    let cases = loop_prompts();
    assert_eq!(cases.len(), 5, "five languages, one held-out family");

    for case in cases {
        let surfaces = unknown_surfaces(&case.prompt, &case.language);
        assert!(
            surfaces
                .iter()
                .any(|surface| surface.to_lowercase().contains("lipogram")
                    || surface.contains("липограмм")
                    || surface.contains("लिपोग्राम")),
            "{}: the loop must ask about the word it does not know, got {surfaces:?}",
            case.language
        );
        assert!(
            !surfaces.contains(&String::from("quick")),
            "{}: a quoted example sentence is not an unresolved concept",
            case.language
        );
    }
}

#[test]
fn the_loop_no_longer_claims_a_missing_fetch_capability() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut offenders = Vec::new();
    let mut pending = vec![root.join("src")];
    while let Some(path) = pending.pop() {
        if path.is_dir() {
            pending.extend(
                fs::read_dir(&path)
                    .expect("src directory")
                    .filter_map(Result::ok)
                    .map(|entry| entry.path()),
            );
        } else if path.extension().is_some_and(|extension| extension == "rs")
            && fs::read_to_string(&path).is_ok_and(|text| text.contains(FORBIDDEN_POLICY))
        {
            offenders.push(path.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "the loop may not assert a capability limit that is no longer true: {offenders:?}"
    );

    for case in loop_prompts() {
        let answer = UniversalSolver::default().solve(&case.prompt);
        assert!(
            !answer
                .evidence_links
                .iter()
                .any(|link| link.contains("no_fetch_capability")),
            "{}: the event log still claims there is no retrieval",
            case.language
        );
    }
}

#[test]
fn an_offline_loop_keeps_the_explicit_no_network_boundary() {
    let solver = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    });
    for case in loop_prompts() {
        let answer = solver.solve(&case.prompt);
        assert!(
            answer
                .evidence_links
                .iter()
                .any(|link| link.starts_with("policy:offline")),
            "{}: an offline run states the boundary instead of inventing a meaning: {:?}",
            case.language,
            answer.evidence_links
        );
        assert!(
            !answer
                .evidence_links
                .iter()
                .any(|link| link.contains("no_fetch_capability")),
            "{}: offline is a boundary, not a missing capability",
            case.language
        );
    }
}
