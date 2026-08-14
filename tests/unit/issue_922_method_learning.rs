//! Issue #922: learn reusable methods from real recursive-core event logs, then
//! adopt them only through the existing benchmark-gated promotion protocol.

use std::cell::Cell;
use std::fs;
use std::path::Path;

use formal_ai::intent_formalization::formalize_intent;
use formal_ai::meta_construction::RecursionMode;
use formal_ai::method_learning::{learn_methods_from_event_logs, LEARNED_METHODS_SEED_FILE};
use formal_ai::method_registry::MethodRegistry;
use formal_ai::promotion::{
    parse_promotion_proposals, replay_promotion_gates_with, GateCommandOutput, PromotionOutcome,
    PromotionRun,
};
use formal_ai::recipe_interpreter::RecipeProgram;
use formal_ai::selection::SelectionMode;
use formal_ai::skill_ledger::SkillMode;
use formal_ai::translation::formalize_prompt;
use formal_ai::EventLog;

fn formalize(prompt: &str) -> formal_ai::intent_formalization::IntentFormalization {
    let candidate = formalize_prompt(prompt, "en");
    formalize_intent(prompt, "en", Some(&candidate))
}

fn solve_trace(prompt: &str) -> EventLog {
    RecipeProgram::from_repo()
        .execute(
            &formalize(prompt),
            4,
            RecursionMode::Both,
            SelectionMode::Record,
            SkillMode::Accumulate,
        )
        .expect("the production recursive recipe should produce an event log")
        .log
}

fn real_observations() -> Vec<(String, EventLog)> {
    [
        ("solve-translation", "translate apple to Russian"),
        (
            "solve-composed",
            "translate apple to Russian and write a hello world program in Python",
        ),
        ("solve-unknown", "zzqqx unfathomable gibberish token"),
    ]
    .into_iter()
    .map(|(id, prompt)| (id.to_owned(), solve_trace(prompt)))
    .collect()
}

/// Localized payloads for every language in the seed registry. Method learning
/// must abstract the stable control-flow kinds, never language-specific event
/// text; the registry comparison makes a newly supported language extend this
/// test instead of silently losing coverage.
const LOCALIZED_EVENT_PAYLOADS: [(&str, &str); 5] = [
    ("en", "English solved request"),
    ("ru", "Русский решённый запрос"),
    ("hi", "हिंदी में हल किया गया अनुरोध"),
    ("zh", "中文已解决请求"),
    ("es", "Solicitud resuelta en español"),
];

#[test]
fn event_payload_language_does_not_change_the_learned_method() {
    let mut registered = formal_ai::language::registered_languages()
        .into_iter()
        .map(formal_ai::Language::slug)
        .collect::<Vec<_>>();
    let mut covered = LOCALIZED_EVENT_PAYLOADS
        .iter()
        .map(|(language, _)| *language)
        .collect::<Vec<_>>();
    registered.sort_unstable();
    covered.sort_unstable();
    assert_eq!(
        covered, registered,
        "every registered language needs a case"
    );

    let observations = LOCALIZED_EVENT_PAYLOADS
        .iter()
        .map(|(language, payload)| {
            let mut log = EventLog::new();
            log.append("formalize", format!("language:{language}:{payload}"));
            log.append("select", format!("language:{language}:{payload}"));
            log.append("execute", format!("language:{language}:{payload}"));
            log.append("verify", format!("language:{language}:{payload}"));
            (format!("language-{language}"), log)
        })
        .collect::<Vec<_>>();
    let borrowed = observations
        .iter()
        .map(|(id, log)| (id.as_str(), log))
        .collect::<Vec<_>>();
    let learning = learn_methods_from_event_logs(&borrowed);
    let proposal = learning
        .validated_proposals()
        .into_iter()
        .next()
        .expect("localized payloads should produce one shared control-flow method");

    assert_eq!(
        proposal.operations,
        ["formalize", "select", "execute", "verify"]
    );
    assert_eq!(proposal.support_trace_ids, ["language-en", "language-ru"]);
    assert_eq!(
        proposal.held_out_trace_ids,
        ["language-hi", "language-zh", "language-es"]
    );
}

#[test]
fn real_event_logs_propose_an_inert_method_that_is_adopted_only_after_promotion() {
    let observations = real_observations();
    let borrowed = observations
        .iter()
        .map(|(id, log)| (id.as_str(), log))
        .collect::<Vec<_>>();
    let learning = learn_methods_from_event_logs(&borrowed);
    let proposal = learning
        .validated_proposals()
        .into_iter()
        .next()
        .expect("three real recursive-core traces should propose a held-out method");

    assert_eq!(proposal.mode(), "proposal_only");
    assert_eq!(proposal.support_trace_ids.len(), 2);
    assert_eq!(proposal.held_out_trace_ids, ["solve-unknown"]);
    assert!(proposal.operations.len() >= 2);

    // Discovery itself cannot alter the active catalogue. Even parsing the
    // proposal document is side-effect-free; only the promotion's seed edit is
    // an adopted registry record.
    let empty_registry = MethodRegistry::from_dispatch_with_learned_seed("")
        .expect("an empty learned-method seed is valid");
    assert!(empty_registry.learned_method(&proposal.name).is_none());
    assert!(empty_registry.method_for_route(&proposal.name).is_none());

    let promotions = learning.promotion_proposals();
    assert!(!promotions.is_empty());
    assert_eq!(promotions[0].edit.seed_file, LEARNED_METHODS_SEED_FILE);
    let reviewed_document = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples/issue-922-method-learning/open-proposals.lino"),
    )
    .expect("the reviewed promotion input should be readable");
    let reviewed = parse_promotion_proposals(&reviewed_document)
        .expect("the reviewed promotion input should remain valid");
    assert_eq!(reviewed.len(), 1);
    assert_eq!(reviewed[0].source, promotions[0].source);
    assert_eq!(reviewed[0].edit, promotions[0].edit);
    let promotion_count = promotions.len();

    let replayed = replay_promotion_gates_with(promotions, |_runner| {
        Ok(GateCommandOutput::success("passed=64 failed=0"))
    })
    .expect("canonical benchmark evidence should replay");
    let promotion = PromotionRun::evaluate(replayed);
    assert_eq!(promotion.promoted().len(), promotion_count);
    assert_eq!(promotion.records[0].outcome, PromotionOutcome::Promoted);

    let adopted = MethodRegistry::from_dispatch_with_learned_seed(
        &promotion.promoted()[0].proposal.edit.lino,
    )
    .expect("a promoted method seed should parse");
    let learned = adopted
        .learned_method(&proposal.name)
        .expect("materializing the promoted seed should adopt the method");
    assert_eq!(learned.algorithm_id, proposal.algorithm_id);
    assert_eq!(learned.operations, proposal.operations);
    assert!(
        adopted.method_for_route(&proposal.name).is_none(),
        "learned link data must not pretend a compiled Rust handler exists"
    );

    // The checked-in seed is the byte-equivalent adoption of this real-trace
    // proposal, proving the production registry consumes the promoted result.
    let production = MethodRegistry::from_dispatch();
    assert_eq!(
        production.learned_method(&proposal.name),
        Some(learned),
        "the checked-in promoted seed must contain:\n{}",
        promotion.promoted()[0].proposal.edit.lino
    );
}

#[test]
fn rejected_benchmark_proposals_are_durable_and_include_the_reason() {
    let observations = real_observations();
    let borrowed = observations
        .iter()
        .map(|(id, log)| (id.as_str(), log))
        .collect::<Vec<_>>();
    let learning = learn_methods_from_event_logs(&borrowed);
    let command_index = Cell::new(0usize);
    let replayed = replay_promotion_gates_with(learning.promotion_proposals(), |_runner| {
        let index = command_index.get();
        command_index.set(index + 1);
        if index == 0 {
            Ok(GateCommandOutput::failure(
                "passed=0 failed=1",
                "candidate regressed the coding benchmark",
            ))
        } else {
            Ok(GateCommandOutput::success("passed=64 failed=0"))
        }
    })
    .expect("failed commands are blocking benchmark evidence, not replay errors");
    let promotion = PromotionRun::evaluate(replayed);

    assert_eq!(
        promotion.rejected().len(),
        learning.promotion_proposals().len()
    );
    assert!(promotion.promoted().is_empty());
    let rejection = promotion
        .memory_events()
        .into_iter()
        .find(|event| event.kind.as_deref() == Some("promotion_rejection"))
        .expect("the rejected edit and its reasons must remain append-only evidence");
    assert!(rejection
        .content
        .as_deref()
        .is_some_and(|reason| reason.contains("issue_362_multilingual_coding_modification")));
    assert!(rejection
        .evidence
        .iter()
        .any(|link| link.contains("benchmark:issue_362_multilingual_coding_modification:blocked")));
}
