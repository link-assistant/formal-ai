//! Issue #1138, plan 14 **wave F** — self-use observations for plan 01.
//!
//! Every case in this file was produced by giving Formal AI a held-out prompt
//! through the real `@link-assistant/agent` CLI and through `formal-ai chat`,
//! and recording what came back. The raw transcripts are committed under
//! `docs/case-studies/issue-1138/self-use/`; the prompts live in
//! `data/benchmarks/self-use-concept-lookup.lino` and nowhere under `src/` or
//! `data/seed/`.
//!
//! The binding rule of wave F
//! (`docs/case-studies/issue-710/plans/07-prerequisite-discovery-bridge.md:80-81`)
//! applies to this file: nothing here was repaired by hand and counted as the
//! system's work. Each test asserts the behaviour plan 01 says the system must
//! have, and each one was observed **failing** on `formal-ai 0.350.0` at commit
//! `dc9b0574607a26f3e1c8bdb8ce93c0c7f786f197` before it was committed. A test
//! here is not silently weakened to make it pass: when full language parity is
//! acknowledged as incremental debt, the replacement contract must derive the
//! exact shortfall, date it, reject growth, and reject stale declarations.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::solver::solve;

/// The wave F corpus this file reads its prompts back from.
const CORPUS: &str = "data/benchmarks/self-use-concept-lookup.lino";

/// The seed file whose per-language rows decide which languages a surface can
/// actually answer in.
const RESPONSES_SEED: &str = "data/seed/multilingual-responses.lino";

/// Explicit, strictly bounded response-template debt. Lexeme parity has its own
/// structural gate because response templates and lexemes are different data
/// shapes and neither may silently stand in for the other.
const RESPONSE_PARITY_DEBT: &str = "data/meta/response-language-parity-debt.lino";

/// The five languages every held-out corpus in this plan set is written in.
const LANGUAGES: &[&str] = &["en", "ru", "hi", "zh", "es"];

/// One recorded self-use case.
#[derive(Debug, Clone)]
struct Case {
    family: String,
    language: String,
    prompt: String,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// Strip the surrounding quotes and undo Links Notation's doubled `""`.
fn unquote(raw: &str) -> String {
    let trimmed = raw.trim();
    let inner = trimmed
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(trimmed);
    inner.replace("\"\"", "\"")
}

fn corpus() -> Vec<Case> {
    let path = repo_root().join(CORPUS);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{CORPUS} should be readable: {error}"));

    let mut out = Vec::new();
    let mut family = String::new();
    let mut language = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !line.starts_with(char::is_whitespace) {
            family = trimmed.to_owned();
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("id ") {
            family = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("language ") {
            language = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("prompt ") {
            out.push(Case {
                family: family.clone(),
                language: language.clone(),
                prompt: unquote(value),
            });
        }
    }
    out
}

fn family(id: &str) -> Vec<Case> {
    let cases: Vec<Case> = corpus().into_iter().filter(|c| c.family == id).collect();
    assert!(
        !cases.is_empty(),
        "corpus family `{id}` must exist in {CORPUS}"
    );
    cases
}

/// Every `(intent, language)` pair the multilingual response seed declares.
fn seeded_response_languages() -> BTreeMap<String, BTreeSet<String>> {
    let path = repo_root().join(RESPONSES_SEED);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{RESPONSES_SEED} should be readable: {error}"));

    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut intent = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("intent ") {
            intent = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("language ") {
            if !intent.is_empty() {
                out.entry(intent.clone())
                    .or_default()
                    .insert(unquote(value));
            }
        }
    }
    out
}

#[derive(Debug)]
struct ResponseParityDebt {
    ceiling: usize,
    languages: BTreeSet<String>,
    gaps: BTreeMap<String, BTreeSet<String>>,
}

fn csv_set(raw: &str) -> BTreeSet<String> {
    if raw.is_empty() {
        BTreeSet::new()
    } else {
        raw.split(',').map(str::to_owned).collect()
    }
}

fn is_iso_date(value: &str) -> bool {
    value.len() == 10
        && value
            .chars()
            .enumerate()
            .all(|(index, character)| match index {
                4 | 7 => character == '-',
                _ => character.is_ascii_digit(),
            })
}

fn response_parity_debt() -> ResponseParityDebt {
    let path = repo_root().join(RESPONSE_PARITY_DEBT);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{RESPONSE_PARITY_DEBT} should be readable: {error}"));
    let mut measured_on = String::new();
    let mut ceiling = None;
    let mut languages = BTreeSet::new();
    let mut source = String::new();
    let mut gaps = BTreeMap::new();
    let mut saw_root = false;
    let mut record_type = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !saw_root {
            assert_eq!(
                trimmed, "response_language_parity_debt",
                "{RESPONSE_PARITY_DEBT}: unexpected root"
            );
            saw_root = true;
        } else if let Some(value) = trimmed.strip_prefix("record_type ") {
            record_type = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("measured_on ") {
            measured_on = unquote(value);
        } else if let Some(value) = trimmed.strip_prefix("ceiling ") {
            ceiling = Some(
                value
                    .parse::<usize>()
                    .unwrap_or_else(|error| panic!("invalid response parity ceiling: {error}")),
            );
        } else if let Some(value) = trimmed.strip_prefix("target_languages ") {
            languages = csv_set(&unquote(value));
        } else if let Some(value) = trimmed.strip_prefix("source ") {
            source = unquote(value);
        } else if trimmed.starts_with("uncovered_behavior ") {
            assert!(
                !measured_on.is_empty(),
                "measurement date must precede response debt rows"
            );
            let fields: Vec<&str> = trimmed.split('"').collect();
            assert_eq!(
                fields.len(),
                7,
                "{RESPONSE_PARITY_DEBT}: malformed uncovered_behavior row: {trimmed}"
            );
            assert_eq!(fields[0], "uncovered_behavior missing_languages ");
            assert_eq!(fields[2], " observed_on ");
            assert_eq!(fields[4], " response_intents ");
            assert!(fields[6].is_empty());
            assert_eq!(
                fields[3], measured_on,
                "every response parity row must carry the ledger measurement date"
            );
            for intent in csv_set(fields[5]) {
                assert!(
                    gaps.insert(intent.clone(), csv_set(fields[1])).is_none(),
                    "{RESPONSE_PARITY_DEBT}: duplicate debt declaration for `{intent}`"
                );
            }
        } else {
            panic!("{RESPONSE_PARITY_DEBT}: unexpected row `{trimmed}`");
        }
    }

    assert!(saw_root, "response parity debt must declare its root");
    assert_eq!(
        record_type, "response_language_parity_debt",
        "response parity debt must declare its record type"
    );
    assert!(is_iso_date(&measured_on), "debt must have an ISO date");
    assert_eq!(source, RESPONSES_SEED, "debt must name its measured seed");
    ResponseParityDebt {
        ceiling: ceiling.expect("response parity debt must declare a ceiling"),
        languages,
        gaps,
    }
}

/// **Wave F observation, plan 01.** The held-out word `lipogram` occurs nowhere
/// under `src/` or `data/seed/`, so a prompt that turns on its meaning cannot be
/// answered from memory: plan 01 requires the loop to ask about the unresolved
/// word and answer from a licensed source, or refuse naming every source it
/// consulted.
///
/// Observed instead, in `en`, `ru`, `hi` and `zh`: the prompt routes to
/// `web_search` and the reply is a **description of the search machinery** — the
/// provider list and the reciprocal-rank-fusion formula — with no gloss of the
/// word and no verdict on the question. Describing the retrieval capability is
/// not retrieval, and it is the same class of defect as counting a read-back
/// plan as executed semantics.
///
/// Transcripts: `docs/case-studies/issue-1138/self-use/lipogram/*/`.
#[test]
fn an_unresolved_word_is_looked_up_rather_than_answered_with_the_provider_description() {
    let mut offenders = Vec::new();
    for case in family("provider_description_instead_of_lookup") {
        let answer = solve(&case.prompt).answer;
        if case.language == "en" {
            assert_eq!(
                answer,
                "No consulted source defined \"Is the sentence \"quick brown fox\" a lipogram\". Consulted: github unbound_template wikidata unbound_template wiktionary offline_cache_miss wordnet offline_cache_miss wikipedia offline_cache_miss stackexchange offline_cache_miss; github unbound_template wikidata unbound_template wiktionary offline_cache_miss wordnet offline_cache_miss wikipedia offline_cache_miss stackexchange offline_cache_miss; github unbound_template wikidata unbound_template wiktionary offline_cache_miss wordnet offline_cache_miss wikipedia offline_cache_miss stackexchange offline_cache_miss."
            );
        }
        if answer.contains("Providers considered") || answer.contains("reciprocal rank fusion") {
            offenders.push(format!(
                "{}: answered with the web-search capability description instead of a lookup",
                case.language
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "plan 01: a prompt turning on an unresolved word must reach a source, not a \
         description of the search machinery. Offending languages:\n{}",
        offenders.join("\n")
    );
}

/// **Wave F observation, plan 01 / plan 10.** Spanish is one of the five
/// registered languages of every held-out corpus in this plan set. The Spanish
/// paraphrase is answered with the `language unknown` seed row — *"I detected an
/// unsupported language and am falling back to English"* — even though the
/// coding route answers the very same language correctly, so this is a route
/// defect, not a missing locale.
///
/// Transcript: `docs/case-studies/issue-1138/self-use/lipogram/es/`.
#[test]
fn a_spanish_prompt_is_not_reported_as_an_unsupported_language() {
    let case = family("spanish_reported_unsupported")
        .into_iter()
        .find(|c| c.language == "es")
        .expect("the corpus family must carry the Spanish paraphrase");
    let answer = solve(&case.prompt).answer;
    assert_eq!(
        answer,
        "Ninguna fuente consultada definió «¿La frase \"quick brown fox\" es un lipograma en e». Fuentes consultadas: github unbound_template wikidata unbound_template wiktionary offline_cache_miss wordnet unbound_template wikipedia offline_cache_miss stackexchange offline_cache_miss; github unbound_template wikidata unbound_template wiktionary offline_cache_miss wordnet unbound_template wikipedia offline_cache_miss stackexchange offline_cache_miss; github unbound_template wikidata unbound_template wiktionary offline_cache_miss wordnet unbound_template wikipedia offline_cache_miss stackexchange offline_cache_miss."
    );
    assert!(
        !answer.contains("unsupported language"),
        "plan 01/10: a Spanish prompt must be answered in Spanish, never demoted to the \
         unknown-language fallback. Observed answer:\n{answer}"
    );
}

/// **Wave F observation, plan 01.** The synthesis refusal for the held-out word
/// `isogram` is honest about its outcome and is localized into all five
/// languages, but its research trail is empty — `parts=;failed_examples=;attempts=`.
/// The reply claims every synthesis route was tried while naming no source that
/// was consulted and no attempt that was made, so a reader cannot tell a real
/// exhausted search from a search that never ran. Plan 01 requires a lookup that
/// finds nothing to report **every consulted source**.
///
/// Transcripts: `docs/case-studies/issue-1138/self-use/isogram/*/`.
#[test]
fn an_honest_refusal_names_every_source_it_consulted() {
    let mut offenders = Vec::new();
    for case in family("refusal_names_no_consulted_source") {
        let answer = solve(&case.prompt).answer;
        if case.language == "en" {
            assert_eq!(
                answer.lines().next(),
                Some(
                    "I cannot write this program: no synthesis route reaches task \"is_isogram\" in language \"python\"."
                ),
                "the exact stable answer opener documents the named gap"
            );
        }
        let Some(trail) = answer.split("attempts=").nth(1) else {
            continue;
        };
        if trail.trim().is_empty() {
            offenders.push(format!(
                "{}: refusal carries a research trail whose `attempts=` names nothing",
                case.language
            ));
        }
    }
    assert!(
        offenders.is_empty(),
        "plan 01: a lookup that finds nothing must report every consulted source. \
         Offending languages:\n{}",
        offenders.join("\n")
    );
}

/// **Wave F root cause, plan 01 / plan 11-L75 (#949).** Response templates are
/// not lexemes, so plan 11's structural lexeme-parity gate cannot discover
/// their language coverage. This companion census derives every response
/// shortfall from the seed and requires exact, dated ledger coverage: a new or
/// changed gap cannot disappear into an assertion rewrite, while real
/// translations reduce the bounded debt.
///
/// This assertion needs no binary and no network — it is a property of the seed
/// and stays true or false independently of how any route behaves.
#[test]
fn every_seeded_response_intent_serves_all_five_languages_or_has_exact_debt() {
    let seeded = seeded_response_languages();
    let debt = response_parity_debt();
    let expected_languages: BTreeSet<String> = LANGUAGES
        .iter()
        .map(|language| (*language).to_owned())
        .collect();
    assert_eq!(
        debt.languages, expected_languages,
        "response parity debt and the registered language contract must agree"
    );

    let mut gaps = BTreeMap::new();
    for (intent, languages) in &seeded {
        let missing: BTreeSet<String> = LANGUAGES
            .iter()
            .map(|language| (*language).to_owned())
            .filter(|language| !languages.contains(language))
            .collect();
        if !missing.is_empty() {
            gaps.insert(intent.clone(), missing);
        }
    }
    assert_eq!(
        gaps.len(),
        debt.ceiling,
        "plan 11-L75 (#949): measured response debt must equal its reviewed ceiling; lower the \
         ceiling with real translation improvements, and never raise it for new gaps"
    );
    assert_eq!(
        gaps,
        debt.gaps,
        "plan 11-L75 (#949): every response-language shortfall must have one exact dated \
         debt declaration, with no untracked, stale, or duplicate intent. {} of {} intents \
         currently need translations",
        gaps.len(),
        seeded.len()
    );
}
