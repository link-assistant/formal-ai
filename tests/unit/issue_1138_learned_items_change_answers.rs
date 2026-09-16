//! Issue #1138 B7, plan 07 leaves 5-7: the adopted learned method must change a
//! held-out answer, in five languages, and removing the seed record must restore
//! the old answer.
//!
//! Exactly two learned artifacts reach a live answer today, and one of them —
//! `learned_recursive_core_e17957243eaaf6db` in
//! `data/seed/learned-methods.lino` — has zero production read path. This suite
//! is what turns that from a claim into an observation.
//!
//! The prompts are held out from the inference corpus: they occur in no
//! `data/meta/learning-frontier-*.lino` and in neither of the two support traces
//! the method was learned from.
//!
//! Written before the leaves that make it pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::behavior_delta::{DeltaVerdict, prove_effect};
use formal_ai::method_registry::MethodRegistry;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

const ADOPTED_ITEM: &str = "learned_recursive_core_e17957243eaaf6db";

/// The class under test: a request that needs a counted-scan answer, which the
/// compiled table routes to `unknown` today.
const HELD_OUT: [(&str, &str); 5] = [
    ("en", "In the word alphabet, how many times does the letter a appear?"),
    ("ru", "Сколько раз буква а встречается в слове алфавит?"),
    ("hi", "शब्द वर्णमाला में अक्षर व कितनी बार आता है?"),
    ("zh", "在「字母表」这个词里，字母表这两个字出现了几次？"),
    ("es", "¿Cuántas veces aparece la letra a en la palabra alfabeto?"),
];

/// Paraphrases of the same class, asserted to receive the same verdict.
const HELD_OUT_PARAPHRASE: [(&str, &str); 5] = [
    ("en", "Count the occurrences of a inside alphabet and tell me the number."),
    ("ru", "Посчитай, сколько букв а внутри слова алфавит, и назови число."),
    ("hi", "वर्णमाला के अंदर व की गिनती करो और संख्या बताओ।"),
    ("zh", "数一数「字母表」里面有几个「字」，把数目告诉我。"),
    ("es", "Cuenta cuántas veces está la a dentro de alfabeto y dime el número."),
];

fn learned_seed() -> String {
    let path = repo_root().join("data/seed/learned-methods.lino");
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("learned-methods.lino readable: {error}"))
}

/// The registry with the adopted record present, and the same registry with the
/// record removed. The delta must be caused by the seed edit and nothing else.
fn registries() -> (MethodRegistry, MethodRegistry) {
    let seed = learned_seed();
    let with_item = MethodRegistry::from_dispatch_with_learned_seed(&seed)
        .expect("the shipped learned-method seed loads");
    let stripped: String = seed
        .split("\n\n")
        .filter(|block| !block.contains(ADOPTED_ITEM))
        .collect::<Vec<_>>()
        .join("\n\n");
    let without_item = MethodRegistry::from_dispatch_with_learned_seed(&stripped)
        .expect("the stripped learned-method seed loads");
    (with_item, without_item)
}

#[test]
fn the_adopted_method_changes_the_answer_to_a_held_out_prompt() {
    let (with_item, without_item) = registries();
    let held_out: Vec<(&str, &str)> = HELD_OUT.to_vec();
    let effect = prove_effect(ADOPTED_ITEM, "method", &held_out, &with_item, &without_item);

    let mut failures: Vec<String> = Vec::new();
    for delta in &effect.deltas {
        if delta.before.observed_output_sha256 == delta.after.observed_output_sha256 {
            failures.push(format!("{}: before and after are byte-identical", delta.language));
        }
        if delta.verdict != DeltaVerdict::Improved {
            failures.push(format!("{}: verdict {:?}", delta.language, delta.verdict));
        }
    }
    assert!(
        failures.is_empty(),
        "an adopted learned item must demonstrably change the next answer in each of \
         en, ru, hi, zh and es. If it does not, the honest outcome is to record \
         `status \"adopted_not_effective\"` rather than to force a delta: {failures:?}"
    );
    assert!(
        effect.qualifies(),
        "the adoption contract needs one Improved delta per language and no regressions"
    );
}

#[test]
fn a_held_out_paraphrase_gets_the_same_verdict() {
    let (with_item, without_item) = registries();
    let original: Vec<(&str, &str)> = HELD_OUT.to_vec();
    let paraphrase: Vec<(&str, &str)> = HELD_OUT_PARAPHRASE.to_vec();
    let first = prove_effect(ADOPTED_ITEM, "method", &original, &with_item, &without_item);
    let second = prove_effect(ADOPTED_ITEM, "method", &paraphrase, &with_item, &without_item);

    assert_eq!(first.deltas.len(), 5);
    assert_eq!(second.deltas.len(), 5);
    for (left, right) in first.deltas.iter().zip(second.deltas.iter()) {
        assert_eq!(
            left.verdict, right.verdict,
            "{}: the paraphrase of the same class must receive the same verdict; a \
             different verdict means the item matched the wording, not the class",
            left.language
        );
    }
}

#[test]
fn removing_the_seed_record_restores_the_old_answer() {
    let (with_item, without_item) = registries();
    assert!(
        with_item
            .learned_methods
            .iter()
            .any(|method| method.name == ADOPTED_ITEM),
        "the shipped seed carries the adopted record"
    );
    assert!(
        !without_item
            .learned_methods
            .iter()
            .any(|method| method.name == ADOPTED_ITEM),
        "the stripped seed does not"
    );

    let held_out: Vec<(&str, &str)> = HELD_OUT.to_vec();
    let effect = prove_effect(ADOPTED_ITEM, "method", &held_out, &with_item, &without_item);
    // The "before" side is the answer with the item removed; proving the effect
    // twice against the *same* stripped registry must reproduce that side byte
    // for byte, which is what makes the seed edit the only cause.
    let control = prove_effect(
        ADOPTED_ITEM,
        "method",
        &held_out,
        &without_item,
        &without_item,
    );
    for (measured, restored) in effect.deltas.iter().zip(control.deltas.iter()) {
        assert_eq!(
            measured.before.observed_output_sha256, restored.after.observed_output_sha256,
            "{}: removing the seed record must restore the old answer exactly",
            measured.language
        );
        assert_eq!(
            restored.verdict,
            DeltaVerdict::Unchanged,
            "{}: with the item absent on both sides nothing may change",
            restored.language
        );
    }
}

#[test]
fn no_prompt_in_the_delta_set_appears_in_the_inference_corpus() {
    // The anti-memorization check: a prompt the method was inferred from would
    // prove nothing about the next answer.
    let mut corpora: Vec<(String, String)> = Vec::new();
    for entry in walkdir::WalkDir::new(repo_root().join("data/meta"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with("learning-frontier-") && name != "learning-ledger.lino" {
            continue;
        }
        corpora.push((name, fs::read_to_string(entry.path()).unwrap_or_default()));
    }
    corpora.push((
        "data/seed/learned-methods.lino".to_owned(),
        learned_seed(),
    ));
    assert!(
        !corpora.is_empty(),
        "the inference corpus must be readable for this guard to mean anything"
    );

    let mut memorized: Vec<String> = Vec::new();
    for (language, prompt) in HELD_OUT.iter().chain(HELD_OUT_PARAPHRASE.iter()) {
        for (origin, text) in &corpora {
            if text.contains(prompt) {
                memorized.push(format!("{origin}: {language} prompt"));
            }
        }
    }
    assert!(
        memorized.is_empty(),
        "a held-out prompt occurs in the corpus the item was inferred from: {memorized:?}"
    );
}
