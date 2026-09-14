//! Issue #144: the `When X then Y` grammar behavior rules are written in.
//!
//! A behavior rule is a sentence, and this file is about the sentence: the
//! forms it may be taught in across English, Russian and Hindi, and the
//! topic-grouped rendering the catalog answers with. The chat-surface tests it
//! was extracted from cover what the surface *does*; these cover the one
//! grammar it has to read, which is why they read as a set and split off as
//! one.

use formal_ai::{ConversationTurn, UniversalSolver};

use super::answer;

// ---------------------------------------------------------------------------
// Issue #144: `When X then Y` grammar, grouping by topic, and multilingual
// support for behavior rules and rule-update statements.
// ---------------------------------------------------------------------------

#[test]
fn behavior_rule_listing_renders_when_then_statements_grouped_by_topic() {
    let response = answer("List behavior rules");
    assert_eq!(response.intent, "behavior_rules_list");
    // The catalog must announce the When X then Y grouping in its preamble.
    assert!(
        response.answer.contains("`When X then Y`"),
        "listing preamble must announce When X then Y form, got: {}",
        response.answer
    );
    // Each catalog rule must be rendered as a `When ... then respond with ...` statement.
    let when_then_count = response.answer.matches("When ").count();
    assert!(
        when_then_count >= 6,
        "listing must include at least six `When X then Y` statements; got {when_then_count}: {}",
        response.answer
    );
    // Topics must appear as group headings.
    for topic in [
        "# Greetings",
        "# Farewells",
        "# Identity",
        "# Capabilities",
        "# Program templates",
        "# Unknown fallback",
    ] {
        assert!(
            response.answer.contains(topic),
            "listing must include topic heading {topic:?}, got: {}",
            response.answer
        );
    }
}

#[test]
fn behavior_rule_listing_invites_when_x_then_y_teaching_form() {
    let response = answer("List behavior rules");
    assert_eq!(response.intent, "behavior_rules_list");
    assert!(
        response
            .answer
            .contains("When `your prompt` then `your answer`"),
        "listing must invite the new When X then Y teach form, got: {}",
        response.answer
    );
}

#[test]
fn behavior_rule_detail_includes_topic_and_when_then_in_links() {
    let response = answer("Show behavior rule rule_greeting");
    assert_eq!(response.intent, "behavior_rule_detail");
    assert!(
        response.answer.contains("topic \"greetings\""),
        "detail body must include a topic line in the Links Notation block, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("when_then \""),
        "detail body must include a when_then line in the Links Notation block, got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("When `your prompt` then `your answer`"),
        "detail body must advertise the new When X then Y teach form, got: {}",
        response.answer
    );
}

#[test]
fn behavior_rule_teach_supports_english_when_x_then_y_grammar() {
    let solver = UniversalSolver::default();
    let update = solver.solve("When `tell me a joke` then `I do not have a joke pool yet.`");
    assert_eq!(update.intent, "behavior_rule_update");
    // The acknowledgement must surface the When X then Y rendering and Links Notation block.
    assert!(
        update
            .answer
            .contains("When the user says `tell me a joke` then respond with"),
        "acknowledgement must echo the canonical When X then Y form, got: {}",
        update.answer
    );
    assert!(
        update.answer.contains("when_then \""),
        "acknowledgement Links Notation must include a when_then field, got: {}",
        update.answer
    );

    let history = [ConversationTurn::user(
        "When `tell me a joke` then `I do not have a joke pool yet.`",
    )];
    let response = solver.solve_with_history("tell me a joke", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "I do not have a joke pool yet.");
}

#[test]
fn behavior_rule_teach_supports_english_when_x_do_y_grammar() {
    let solver = UniversalSolver::default();
    let update = solver.solve("When `say goodbye` do `Goodbye, friend.`");
    assert_eq!(update.intent, "behavior_rule_update");

    let history = [ConversationTurn::user(
        "When `say goodbye` do `Goodbye, friend.`",
    )];
    let response = solver.solve_with_history("say goodbye", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "Goodbye, friend.");
}

#[test]
fn behavior_rule_teach_supports_russian_kogda_togda_grammar() {
    let solver = UniversalSolver::default();
    let update =
        solver.solve("Когда `Какая у тебя модель личности?` тогда `Символьная модель личности.`");
    assert_eq!(update.intent, "behavior_rule_update");

    let history = [ConversationTurn::user(
        "Когда `Какая у тебя модель личности?` тогда `Символьная модель личности.`",
    )];
    let response = solver.solve_with_history("Какая у тебя модель личности?", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "Символьная модель личности.");
}

#[test]
fn behavior_rule_teach_supports_russian_kogda_delai_grammar() {
    let solver = UniversalSolver::default();
    let update = solver.solve("Когда `привет` делай `Здравствуй!`");
    assert_eq!(update.intent, "behavior_rule_update");

    let history = [ConversationTurn::user("Когда `привет` делай `Здравствуй!`")];
    let response = solver.solve_with_history("привет", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "Здравствуй!");
}

#[test]
fn behavior_rule_teach_supports_hindi_jab_tab_grammar() {
    let solver = UniversalSolver::default();
    let update = solver.solve("जब `नमस्ते` तब `नमस्ते, मैं formal-ai हूँ.`");
    assert_eq!(update.intent, "behavior_rule_update");

    let history = [ConversationTurn::user(
        "जब `नमस्ते` तब `नमस्ते, मैं formal-ai हूँ.`",
    )];
    let response = solver.solve_with_history("नमस्ते", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "नमस्ते, मैं formal-ai हूँ.");
}

#[test]
fn behavior_rule_teach_supports_chinese_dang_shi_grammar() {
    let solver = UniversalSolver::default();
    let update = solver.solve("当 `你好` 时 `你好,我是 formal-ai。`");
    assert_eq!(update.intent, "behavior_rule_update");

    let history = [ConversationTurn::user(
        "当 `你好` 时 `你好,我是 formal-ai。`",
    )];
    let response = solver.solve_with_history("你好", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "你好,我是 formal-ai。");
}

#[test]
fn behavior_rule_teach_supports_chinese_dang_ze_grammar() {
    let solver = UniversalSolver::default();
    let update = solver.solve("当 `天气` 则 `今天是晴天。`");
    assert_eq!(update.intent, "behavior_rule_update");

    let history = [ConversationTurn::user("当 `天气` 则 `今天是晴天。`")];
    let response = solver.solve_with_history("天气", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "今天是晴天。");
}

#[test]
fn behavior_rule_teach_recognized_for_capabilities_questions() {
    let solver = UniversalSolver::default();
    let update = solver.solve("When `Какая у тебя модель личности?` then `Символьная личность.`");
    assert_eq!(
        update.intent, "behavior_rule_update",
        "the When X then Y grammar must trigger the rule-update path, got: {}",
        update.intent
    );

    let history = [ConversationTurn::user(
        "When `Какая у тебя модель личности?` then `Символьная личность.`",
    )];
    let response = solver.solve_with_history("Какая у тебя модель личности?", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "Символьная личность.");
}

#[test]
fn behavior_rule_listing_shows_dialog_local_runtime_rule_as_when_then() {
    let solver = UniversalSolver::default();
    let history = [ConversationTurn::user(
        "When `synthetic-prompt` then `synthetic-answer`.",
    )];
    let response = solver.solve_with_history("List behavior rules", &history);
    assert_eq!(response.intent, "behavior_rules_list");
    assert!(
        response.answer.contains("Dialog-local rules"),
        "listing must surface a Dialog-local rules section when a rule was taught; got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("synthetic-prompt")
            && response.answer.contains("synthetic-answer"),
        "listing must include both trigger and answer for the runtime rule; got: {}",
        response.answer
    );
}

#[test]
fn behavior_rule_listing_does_not_match_when_x_then_y_in_running_text() {
    // A prompt that mentions `when` and `then` without two backtick spans must
    // not be misclassified as a rule update. The free-form text should fall
    // through to the standard handlers (greeting/unknown/etc.).
    let response = answer("When does the weather change then I take an umbrella");
    assert_ne!(response.intent, "behavior_rule_update");
}
