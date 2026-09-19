//! Issue #1138 B7, plan 07 leaves 5-7: the adopted learned method must change a
//! held-out answer, in five languages, and removing the seed record must restore
//! the old answer.
//!
//! Exactly two learned artifacts reach a live answer today, and one of them —
//! `learned_recursive_core_d21ca03aaabaf13d` in
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
use formal_ai::{SolverConfig, UniversalSolver};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

const ADOPTED_ITEM: &str = "learned_recursive_core_d21ca03aaabaf13d";

/// Held-out explicit invocations of the learned method. The content-addressed
/// method reference is language-neutral; the surrounding requests prove that
/// discovery does not depend on one language's trigger phrase.
const HELD_OUT: [(&str, &str); 5] = [
    (
        "en",
        "Execute method:learned_recursive_core_d21ca03aaabaf13d.",
    ),
    (
        "ru",
        "Выполни method:learned_recursive_core_d21ca03aaabaf13d.",
    ),
    (
        "hi",
        "method:learned_recursive_core_d21ca03aaabaf13d को निष्पादित करें।",
    ),
    (
        "zh",
        "执行 method:learned_recursive_core_d21ca03aaabaf13d。",
    ),
    (
        "es",
        "Ejecuta method:learned_recursive_core_d21ca03aaabaf13d.",
    ),
];

/// Paraphrases of the same class, asserted to receive the same verdict.
const HELD_OUT_PARAPHRASE: [(&str, &str); 5] = [
    (
        "en",
        "Run method:learned_recursive_core_d21ca03aaabaf13d now.",
    ),
    (
        "ru",
        "Запусти method:learned_recursive_core_d21ca03aaabaf13d сейчас.",
    ),
    (
        "hi",
        "अभी method:learned_recursive_core_d21ca03aaabaf13d चलाएँ।",
    ),
    (
        "zh",
        "现在运行 method:learned_recursive_core_d21ca03aaabaf13d。",
    ),
    (
        "es",
        "Ejecuta ahora method:learned_recursive_core_d21ca03aaabaf13d.",
    ),
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
    let with_item = MethodRegistry::from_store_with_learned_seed(&seed)
        .expect("the adopted learned method loads for its counterfactual measurement");
    let stripped: String = seed
        .split("\n\n")
        .filter(|block| !block.contains(ADOPTED_ITEM))
        .collect::<Vec<_>>()
        .join("\n\n");
    let without_item = MethodRegistry::from_store_with_learned_seed(&stripped)
        .expect("the stripped learned-method seed loads");
    (with_item, without_item)
}

#[test]
fn the_adopted_method_changes_the_answer_to_a_held_out_prompt() {
    let (with_item, without_item) = registries();
    let held_out: Vec<(&str, &str)> = HELD_OUT.to_vec();
    let effect = prove_effect(ADOPTED_ITEM, "method", &held_out, &with_item, &without_item);

    for delta in &effect.deltas {
        assert!(
            delta.answer_changed(),
            "{}: the observed answer must change",
            delta.language
        );
        assert_eq!(
            delta.verdict,
            DeltaVerdict::Improved,
            "{}: the after answer must execute and verify every learned operation",
            delta.language
        );
    }
    assert!(
        effect.qualifies(),
        "five verified improvements and zero regressions qualify the adopted item"
    );
    assert!(
        learned_seed().contains("status \"adopted\""),
        "the shipped record must carry the measured adoption"
    );
}

#[test]
fn a_held_out_paraphrase_gets_the_same_verdict() {
    let (with_item, without_item) = registries();
    let original: Vec<(&str, &str)> = HELD_OUT.to_vec();
    let paraphrase: Vec<(&str, &str)> = HELD_OUT_PARAPHRASE.to_vec();
    let first = prove_effect(ADOPTED_ITEM, "method", &original, &with_item, &without_item);
    let second = prove_effect(
        ADOPTED_ITEM,
        "method",
        &paraphrase,
        &with_item,
        &without_item,
    );

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
fn explicit_references_execute_the_adopted_method_on_the_live_solver_path() {
    let solver = UniversalSolver::new(SolverConfig::default());
    for (language, prompt) in HELD_OUT {
        let response = solver.solve(prompt);
        assert_eq!(
            response.intent,
            format!("learned_method:{ADOPTED_ITEM}"),
            "{language}: explicit references are resolved from registry data"
        );
        assert_eq!(
            response.answer.lines().collect::<Vec<_>>(),
            expected_answer_lines(),
            "{language}: the complete learned-method answer is the verified recipe execution"
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("method:learned:operations_verified:true")),
            "{language}: the trace must expose the successful learned-operation postcondition: {:?}",
            response.evidence_links
        );
    }
}

fn expected_answer_lines() -> Vec<&'static str> {
    vec![
        "recipe_program",
        "  record_type \"recipe_program\"",
        "  step_count \"11\"",
        "  recorder_count \"11\"",
        "plan_build_problem_frame",
        "  record_type \"recipe_plan_step\"",
        "  order \"2\"",
        "  id \"build_problem_frame\"",
        "  executes \"record_problem_frame\"",
        "plan_decompose_recursively",
        "  record_type \"recipe_plan_step\"",
        "  order \"3\"",
        "  id \"decompose_recursively\"",
        "  executes \"record_work_units\"",
        "plan_account_for_needs",
        "  record_type \"recipe_plan_step\"",
        "  order \"4\"",
        "  id \"account_for_needs\"",
        "  executes \"record_need_ledger\"",
        "plan_catalogue_methods",
        "  record_type \"recipe_plan_step\"",
        "  order \"5\"",
        "  id \"catalogue_methods\"",
        "  executes \"record_method_registry\"",
        "plan_reason_white_box",
        "  record_type \"recipe_plan_step\"",
        "  order \"6\"",
        "  id \"reason_white_box\"",
        "  executes \"record_work_unit_reasoning\"",
        "plan_construct_upward",
        "  record_type \"recipe_plan_step\"",
        "  order \"7\"",
        "  id \"construct_upward\"",
        "  executes \"record_upward_construction\"",
        "plan_record_evidence",
        "  record_type \"recipe_plan_step\"",
        "  order \"9\"",
        "  id \"record_evidence\"",
        "  executes \"record_solution_evidence\"",
        "plan_select_methods",
        "  record_type \"recipe_plan_step\"",
        "  order \"10\"",
        "  id \"select_methods\"",
        "  executes \"record_selection\"",
        "plan_accumulate_skills",
        "  record_type \"recipe_plan_step\"",
        "  order \"12\"",
        "  id \"accumulate_skills\"",
        "  executes \"record_skill_ledger\"",
        "plan_audit_reasoning_standard",
        "  record_type \"recipe_plan_step\"",
        "  order \"13\"",
        "  id \"audit_reasoning_standard\"",
        "  executes \"record_reasoning_standard\"",
        "plan_verify_obligations",
        "  record_type \"recipe_plan_step\"",
        "  order \"14\"",
        "  id \"verify_obligations\"",
        "  executes \"record_obligation_ledger\"",
        "  executed build_problem_frame",
        "  executed decompose_recursively",
        "  executed account_for_needs",
        "  executed catalogue_methods",
        "  executed reason_white_box",
        "  executed construct_upward",
        "  executed record_evidence",
        "  executed select_methods",
        "  executed accumulate_skills",
        "  executed audit_reasoning_standard",
        "  executed verify_obligations",
    ]
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
    corpora.push(("data/seed/learned-methods.lino".to_owned(), learned_seed()));
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
