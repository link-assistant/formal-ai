//! Issue #1557: an English work item must not be answered in Spanish or
//! Russian. Markers of a script several languages share match whole words
//! only, and a work item's language is decided by the proportion of its prose
//! (code, quotations, URLs and identifiers excluded), preferring the user's
//! own turn, then the issue body, then its title. Mirrored by
//! `rust/tests/web/issue-1557-prose-language.test.mjs`.

use formal_ai::agentic_coding::general_planner::{
    GeneralChangePlan, GeneralPlanMode, compose_general_change_plan,
};
use formal_ai::language::{Language, detect, detect_prose};
use serde_json::Value;

const WORK_ITEM: &str = "https://github.com/link-assistant/formal-ai/issues/904";

fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/issue-1557-attempted-issues.json"))
        .expect("the issue-1557 fixture is valid JSON")
}

fn text<'a>(case: &'a Value, field: &str) -> &'a str {
    case[field].as_str().expect("fixture fields are strings")
}

fn attempted(number: u64) -> Value {
    fixture()["attempted"]
        .as_array()
        .expect("attempted issues")
        .iter()
        .find(|case| case["number"].as_u64() == Some(number))
        .cloned()
        .unwrap_or_else(|| panic!("issue {number} is in the fixture"))
}

fn slug(language: Option<Language>) -> Option<&'static str> {
    language.map(Language::slug)
}

/// The work-item plan Hive Mind's objective field produces, with the fetched
/// issue standing in as its goal the way the execution layer substitutes it.
fn work_item_plan(goal: String) -> GeneralChangePlan {
    let mut plan = compose_general_change_plan(WORK_ITEM).expect("a repository work-item plan");
    assert_eq!(plan.mode, GeneralPlanMode::RepositoryWorkItem);
    assert_eq!(plan.request_language, None);
    plan.goal = goal;
    plan
}

#[test]
fn every_attempted_english_issue_is_detected_as_english_prose() {
    let fixture = fixture();
    let issues = fixture["attempted"].as_array().expect("attempted issues");
    assert_eq!(issues.len(), 19);
    for case in issues {
        let language =
            detect_prose(text(case, "body")).or_else(|| detect_prose(text(case, "title")));
        assert_eq!(slug(language), Some("en"), "issue {}", case["number"]);
    }
}

#[test]
fn issues_1235_1446_and_1548_no_longer_read_as_spanish() {
    for number in [1235, 1446, 1548] {
        let case = attempted(number);
        let body = text(&case, "body");
        assert_eq!(slug(detect_prose(body)), Some("en"), "issue {number} body");
        assert_ne!(detect(body).slug(), "es", "issue {number} detect(body)");
        let plan = work_item_plan(format!("{}\n\n{body}", text(&case, "title")));
        assert_eq!(plan.response_language(), "en", "issue {number} plan");
        assert!(
            plan.planned_not_executed_answer()
                .starts_with("Planned, not executed: "),
            "issue {number} answer"
        );
    }
}

#[test]
fn issues_1330_and_1396_are_english_and_a_russian_issue_is_russian() {
    for number in [1330, 1396] {
        let case = attempted(number);
        let plan = work_item_plan(format!(
            "{}\n\n{}",
            text(&case, "title"),
            text(&case, "body")
        ));
        assert_eq!(plan.response_language(), "en", "issue {number}");
    }
    // Issue 1396's title alone is Russian; the body decides before the title.
    assert_eq!(
        slug(detect_prose(text(&attempted(1396), "title"))),
        Some("ru")
    );
    let fixture = fixture();
    let russian = fixture["synthetic"]
        .as_array()
        .expect("synthetic cases")
        .iter()
        .find(|case| case["case"] == "a fully Russian issue")
        .expect("a fully Russian issue");
    let plan = work_item_plan(format!(
        "{}\n\n{}",
        text(russian, "title"),
        text(russian, "body")
    ));
    assert_eq!(plan.response_language(), "ru");
}

#[test]
fn each_synthetic_case_is_detected_as_its_expected_language() {
    let fixture = fixture();
    for case in fixture["synthetic"].as_array().expect("synthetic cases") {
        let expected = text(case, "expected");
        let name = text(case, "case");
        assert_eq!(
            slug(detect_prose(text(case, "body"))),
            Some(expected),
            "{name}"
        );
        let plan = work_item_plan(format!("{}\n\n{}", text(case, "title"), text(case, "body")));
        assert_eq!(plan.response_language(), expected, "{name}");
    }
}

#[test]
fn no_latin_script_marker_matches_as_the_prefix_of_a_longer_word() {
    for sentence in [
        "formalization described response",
        "The formalization is described in the response.",
        "Responsive formalizations get described",
    ] {
        assert_eq!(detect(sentence).slug(), "en", "{sentence}");
        assert_eq!(slug(detect_prose(sentence)), Some("en"), "{sentence}");
    }
    // A whole marker still counts.
    assert_eq!(detect("formaliza esto, por favor").slug(), "es");
    assert_eq!(detect("responde en español").slug(), "es");
}

#[test]
fn a_single_marker_or_a_quotation_does_not_outvote_the_prose() {
    assert_eq!(
        slug(detect_prose(
            "Please describe how the escribe command and the build pipeline should behave together."
        )),
        Some("en")
    );
    assert_eq!(
        slug(detect_prose(
            "Fix the greeting: the bot said «Привет, как дела?» and then stopped answering."
        )),
        Some("en")
    );
    assert_eq!(
        formal_ai::language::prose_of("See `код` and \"цитата\" at https://пример.рф/путь"),
        "See and at"
    );
}

#[test]
fn the_existing_es_ru_hi_and_zh_detections_are_unchanged() {
    for (sentence, expected) in [
        ("¿Qué es la formalización?", "es"),
        ("Привет, как дела?", "ru"),
        ("नमस्ते, आप कैसे हैं?", "hi"),
        ("你好，你是谁？", "zh"),
        ("介绍一下 julián andrés quiñones?", "zh"),
        ("Hello, how are you?", "en"),
    ] {
        assert_eq!(detect(sentence).slug(), expected, "{sentence}");
        assert_eq!(slug(detect_prose(sentence)), Some(expected), "{sentence}");
    }
}

#[test]
fn the_user_turn_outranks_the_body_and_the_body_outranks_the_title() {
    let body = text(&attempted(1548), "body").to_owned();
    let mut plan = work_item_plan(format!("Title\n\n{body}"));
    plan.request_language = Some(Language::Russian);
    assert_eq!(plan.response_language(), "ru");
    assert_eq!(
        work_item_plan(format!("Не помог...\n\n{body}")).response_language(),
        "en"
    );
    assert_eq!(
        work_item_plan("Не помог...\n\nhttps://example.com/only-a-link".to_owned())
            .response_language(),
        "ru"
    );
    let russian_turn = compose_general_change_plan(&format!("Пожалуйста, реши задачу {WORK_ITEM}"))
        .expect("a Russian turn naming a work item");
    assert_eq!(russian_turn.request_language, Some(Language::Russian));
}
