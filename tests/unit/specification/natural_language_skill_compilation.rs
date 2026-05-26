//! Natural-language skill compilation tests.
//!
//! Issue #259 requires that a prose skill can be compiled into a reusable,
//! reviewable associative package, replayed deterministically, and preferred
//! over step-by-step re-derivation with an explicit `cache_hit` trace.

use formal_ai::{compile_natural_language_skill, ConversationTurn, UniversalSolver};
use lino_objects_codec::format::parse_indented;

const SKILL: &str = "When the user says `checksum status`, answer `checksum cache is valid.`";
const TRIGGER: &str = "checksum status";
const RESPONSE: &str = "checksum cache is valid.";

#[test]
fn natural_language_skill_compiles_to_reusable_package() {
    let package = compile_natural_language_skill(SKILL).expect("skill should compile");

    assert!(package.id.starts_with("compiled_skill_"));
    assert_eq!(package.trigger, TRIGGER);
    assert_eq!(package.response, RESPONSE);
    assert!(package.rule_id.starts_with("compiled_skill_rule_"));
    assert!(package.handler_id.starts_with("compiled_skill_handler_"));
    assert!(package.links_notation().contains("compiled_handler"));
    assert!(package
        .link_records()
        .iter()
        .any(|record| record.record_type == "CompiledSkillPackage"));
}

#[test]
fn natural_language_skill_compiles_supported_language_shapes() {
    struct Case {
        language: &'static str,
        skill: &'static str,
        trigger: &'static str,
        response: &'static str,
    }

    let cases = [
        Case {
            language: "en",
            skill: "When `status check` then `all clear.`",
            trigger: "status check",
            response: "all clear.",
        },
        Case {
            language: "ru",
            skill: "Когда `статус` тогда `всё хорошо.`",
            trigger: "статус",
            response: "всё хорошо.",
        },
        Case {
            language: "hi",
            skill: "जब `स्थिति` तब `सब ठीक है।`",
            trigger: "स्थिति",
            response: "सब ठीक है।",
        },
        Case {
            language: "zh",
            skill: "当 `状态` 时 `一切正常。`",
            trigger: "状态",
            response: "一切正常。",
        },
    ];

    for case in cases {
        let package =
            compile_natural_language_skill(case.skill).expect("language skill should compile");
        assert_eq!(package.trigger, case.trigger, "{}", case.language);
        assert_eq!(package.response, case.response, "{}", case.language);
    }
}

#[test]
fn compiled_package_replays_deterministically_and_exports_links_notation() {
    let package = compile_natural_language_skill(SKILL).expect("skill should compile");
    let first = package.replay(TRIGGER).expect("trigger should replay");
    let second = package
        .replay("Checksum status")
        .expect("case-folded replay");

    assert_eq!(first, second);
    assert_eq!(first.answer, RESPONSE);
    assert_eq!(first.cache_hit, package.id);

    let notation = package.links_notation();
    parse_indented(&notation).expect("compiled skill package must be valid Links Notation");
    assert!(notation.contains("source_description"));
    assert!(notation.contains("replay_mode"));
}

#[test]
fn solver_prefers_compiled_skill_from_history_and_records_cache_hit() {
    let solver = UniversalSolver::default();
    let history = [ConversationTurn::user(SKILL)];

    let response = solver.solve_with_history(TRIGGER, &history);

    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, RESPONSE);
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("cache_hit:compiled_skill_")),
        "compiled skill replay must be visible as a cache hit: {:?}",
        response.evidence_links
    );
    assert!(
        response.links_notation.contains("compiled_skill:replay"),
        "trace should record compiled-skill replay, got: {}",
        response.links_notation
    );
    assert!(
        !response.links_notation.contains("behavior_rule:match"),
        "compiled replay should run before behavior-rule re-derivation"
    );
}
