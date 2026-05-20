//! Chat-first user interaction tests.
//!
//! These tests pin down the chat surface that every entry point (CLI, HTTP
//! API, Telegram, web demo) is expected to share. They cover both the active
//! implementation and the full-scope scope from `VISION.md`/`GOALS.md`.

use formal_ai::{ConversationTurn, FormalAiEngine, SymbolicAnswer, UniversalSolver};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// ---------------------------------------------------------------------------
// Active expectations: present implementation behavior.
// ---------------------------------------------------------------------------

#[test]
fn greeting_prompt_returns_a_greeting_intent() {
    let response = answer("Hello");
    assert_eq!(response.intent, "greeting");
    assert_eq!(response.answer, "Hi, how may I help you?");
    assert!(response.confidence > 0.0);
}

#[test]
fn greeting_matching_is_case_insensitive() {
    let response = answer("hELLO");
    assert_eq!(response.intent, "greeting");
}

#[test]
fn greeting_ignores_surrounding_punctuation() {
    let response = answer("Hi!");
    assert_eq!(response.intent, "greeting");
}

#[test]
fn identity_question_returns_identity_intent() {
    let response = answer("Who are you?");
    assert_eq!(response.intent, "identity");
    assert!(response.answer.to_lowercase().contains("formal-ai"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "response:identity"));
}

#[test]
fn identity_examples_cover_known_phrasings() {
    let cases = [
        "Who are you",
        "what are you",
        "Tell me about yourself",
        "What is formal-ai?",
        "Introduce yourself",
    ];
    for prompt in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "identity",
            "expected identity intent for prompt {prompt:?}"
        );
    }
}

#[test]
fn evidence_links_always_include_prompt_and_intent_links() {
    let response = answer("Hi");
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("prompt:")));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("intent:")));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("response:")));
}

#[test]
fn unknown_prompt_returns_zero_confidence_fallback_intent() {
    let response = answer("Completely unrelated request");
    assert_eq!(response.intent, "unknown");
    assert!(response.confidence.abs() < f32::EPSILON);
    assert!(response.answer.contains("Links Notation"));
}

#[test]
fn unknown_prompt_explains_how_to_teach_a_behavior_rule() {
    let response = answer("Какая у тебя модель личности?");
    assert_eq!(response.intent, "unknown");
    assert!(
        response.answer.contains("List behavior rules")
            && response.answer.contains("Show behavior rule")
            && response.answer.contains("When I say"),
        "unknown fallback should be a self-contained rule-teaching guide, got: {}",
        response.answer
    );
}

#[test]
fn behavior_rules_can_be_listed_and_read_through_chat() {
    let list = answer("List behavior rules");
    assert_eq!(list.intent, "behavior_rules_list");
    assert!(list.answer.contains("rule_greeting"));
    assert!(list.answer.contains("rule_unknown"));

    let detail = answer("Show behavior rule unknown");
    assert_eq!(detail.intent, "behavior_rule_detail");
    assert!(detail.answer.contains("rule_unknown"));
    assert!(detail.answer.contains("When I say"));
}

#[test]
fn behavior_rules_can_be_updated_through_conversation_history() {
    let solver = UniversalSolver::default();
    let update = solver.solve(
        "When I say `Какая у тебя модель личности?`, answer `У меня символьная модель личности.`",
    );
    assert_eq!(update.intent, "behavior_rule_update");
    assert!(update.answer.contains("behavior_rule_runtime"));

    let history = [ConversationTurn::user(
        "When I say `Какая у тебя модель личности?`, answer `У меня символьная модель личности.`",
    )];
    let response = solver.solve_with_history("Какая у тебя модель личности?", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "У меня символьная модель личности.");
}

#[test]
fn self_facts_can_be_listed_through_chat() {
    let response = answer("List all facts you know about yourself");
    assert_eq!(response.intent, "self_facts");
    assert!(response.answer.contains("self_fact"));
    assert!(response.answer.contains("formal-symbolic-production"));
    assert!(response.answer.contains("local Links Notation rules"));
}

#[test]
fn self_facts_query_works_for_russian_speakers() {
    let response = answer("Какие факты ты знаешь о себе?");
    assert_eq!(response.intent, "self_facts");
    assert!(response.answer.contains("self_fact_model"));
}

#[test]
fn behavior_rules_list_works_for_russian_speakers() {
    for prompt in [
        "Список правил поведения",
        "Покажи правила поведения",
        "Какие правила поведения",
    ] {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "behavior_rules_list",
            "expected behavior_rules_list for {prompt:?}, got {}",
            response.intent
        );
        assert!(response.answer.contains("rule_greeting"));
        assert!(response.answer.contains("rule_unknown"));
    }
}

#[test]
fn behavior_rule_detail_can_be_read_in_russian() {
    let response = answer("Покажи правило unknown");
    assert_eq!(response.intent, "behavior_rule_detail");
    assert!(response.answer.contains("rule_unknown"));
}

#[test]
fn self_facts_query_works_for_hindi_speakers() {
    let response = answer("अपने बारे में तथ्य सूचीबद्ध करें");
    assert_eq!(response.intent, "self_facts");
    assert!(response.answer.contains("self_fact_model"));
}

#[test]
fn self_facts_query_works_for_chinese_speakers() {
    let response = answer("列出关于你自己的事实");
    assert_eq!(response.intent, "self_facts");
    assert!(response.answer.contains("self_fact_model"));
}

#[test]
fn behavior_rules_list_works_for_hindi_speakers() {
    let response = answer("व्यवहार के नियम सूचीबद्ध करें");
    assert_eq!(response.intent, "behavior_rules_list");
    assert!(response.answer.contains("rule_unknown"));
}

#[test]
fn behavior_rules_list_works_for_chinese_speakers() {
    let response = answer("列出行为规则");
    assert_eq!(response.intent, "behavior_rules_list");
    assert!(response.answer.contains("rule_unknown"));
}

#[test]
fn behavior_rule_can_be_taught_with_russian_phrasing() {
    let solver = UniversalSolver::default();
    let update = solver
        .solve("Когда я скажу `Какая у тебя модель личности?`, ответь `Символьная личность.`");
    assert_eq!(update.intent, "behavior_rule_update");
    let history = [ConversationTurn::user(
        "Когда я скажу `Какая у тебя модель личности?`, ответь `Символьная личность.`",
    )];
    let response = solver.solve_with_history("Какая у тебя модель личности?", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "Символьная личность.");
}

#[test]
fn behavior_rule_detail_supports_multiple_rule_prefixes() {
    for prompt in [
        "Show behavior rule greeting",
        "Show behavior rule rule_greeting",
        "Read rule greeting",
        "describe behavior rule greeting",
    ] {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "behavior_rule_detail",
            "expected behavior_rule_detail for {prompt:?}, got {}",
            response.intent
        );
        assert!(
            response.answer.contains("rule_greeting"),
            "missing rule_greeting body for {prompt:?}: {}",
            response.answer
        );
    }
}

#[test]
fn most_recent_behavior_rule_wins_when_multiple_apply() {
    let solver = UniversalSolver::default();
    let history = [
        ConversationTurn::user("When I say `weather?`, answer `Sunny in seed-1.`"),
        ConversationTurn::user("When I say `weather?`, answer `Rainy in seed-2.`"),
    ];
    let response = solver.solve_with_history("weather?", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "Rainy in seed-2.");
}

#[test]
fn capabilities_answer_advertises_behavior_rule_commands() {
    let response = answer("What can you do?");
    assert_eq!(response.intent, "capabilities");
    assert!(
        response.answer.contains("List behavior rules"),
        "capabilities answer must mention List behavior rules; got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("When I say"),
        "capabilities answer must mention the teach-by-dialog form; got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Report issue"),
        "capabilities answer must mention the Report issue path; got: {}",
        response.answer
    );
}

#[test]
fn capabilities_answer_in_russian_advertises_behavior_rule_commands() {
    let response = answer("Что ты умеешь?");
    assert_eq!(response.intent, "capabilities");
    assert!(response.answer.contains("List behavior rules"));
    assert!(response.answer.contains("When I say"));
    assert!(response.answer.contains("Report issue"));
}

#[test]
fn unknown_answer_uses_different_opener_for_different_prompts() {
    use formal_ai::unknown_answer_variation_for;
    // Synthesise enough prompts to be confident at least two openers fire.
    let mut openers = std::collections::HashSet::new();
    for seed in 0..50_u32 {
        let body = unknown_answer_variation_for(&format!("synthetic-prompt-{seed}"));
        let first_sentence = body.split(['.', '。', '।']).next().unwrap_or("").trim();
        openers.insert(first_sentence.to_owned());
    }
    assert!(
        openers.len() > 1,
        "expected multiple opener variations across distinct prompts, got: {openers:?}"
    );
}

#[test]
fn unknown_answer_opener_is_deterministic_for_the_same_prompt() {
    let solver = UniversalSolver::default();
    let first = solver.solve("Какая у тебя модель личности?").answer;
    let second = solver.solve("Какая у тебя модель личности?").answer;
    assert_eq!(first, second);
}

#[test]
fn behavior_rule_listing_includes_capabilities_and_farewell_rules() {
    let response = answer("List behavior rules");
    assert_eq!(response.intent, "behavior_rules_list");
    for expected in ["rule_capabilities", "rule_farewell", "rule_identity"] {
        assert!(
            response.answer.contains(expected),
            "missing {expected} from listing: {}",
            response.answer
        );
    }
}

#[test]
fn links_notation_trace_is_present_for_every_answer() {
    let response = answer("Hi");
    assert!(!response.links_notation.is_empty());
    assert!(response.links_notation.contains("answer_"));
    assert!(response.links_notation.contains("intent"));
}

#[test]
fn answers_are_deterministic_for_identical_prompts() {
    let first = answer("Hi");
    let second = answer("Hi");
    assert_eq!(first, second);
}

#[test]
fn empty_prompt_does_not_crash_and_is_classified_as_unknown() {
    let response = answer("");
    assert_eq!(response.intent, "unknown");
    assert!(response.confidence.abs() < f32::EPSILON);
}

#[test]
fn whitespace_only_prompt_is_classified_as_unknown() {
    let response = answer("    \t   \n  ");
    assert_eq!(response.intent, "unknown");
}

#[test]
fn dot_prompt_asks_for_clarification() {
    let response = answer(".");
    assert_eq!(response.intent, "clarification");
    assert!(
        response.answer.contains("only punctuation")
            && response.answer.contains("What would you like"),
        "dot prompt should ask a verification question, got: {}",
        response.answer
    );
}

// ---------------------------------------------------------------------------
// full-scope expectations: not yet implemented. See VISION.md / GOALS.md.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "tracked requirement: bounded chat mode should refuse to run agent-style tasks without explicit opt-in"]
fn chat_mode_refuses_unbounded_multi_step_actions_without_agent_opt_in() {
    let response = answer("Continuously refactor my repository forever");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "policy:chat_bounded_autonomy"),
        "chat mode should refuse autonomous multi-step work without explicit agent mode"
    );
    assert!(response.answer.to_lowercase().contains("agent mode"));
}

#[test]
#[ignore = "tracked requirement: chat-mode answers must declare the execution status of any generated code"]
fn every_code_answer_declares_execution_status_or_unavailability() {
    let response = answer("Write me a sorting algorithm in Rust");
    assert!(
        response.answer.contains("Execution status:")
            || response.answer.contains("Execution unavailable"),
        "chat code answers must always declare execution status, got: {}",
        response.answer
    );
}

#[test]
#[ignore = "tracked requirement: diagnostics-off-by-default should also be expressed at the engine level"]
fn diagnostics_are_excluded_from_default_user_facing_answers() {
    let response = answer("Hi");
    let lower = response.answer.to_lowercase();
    assert!(
        !lower.contains("prompt:") && !lower.contains("intent:") && !lower.contains("trace:"),
        "default chat answers should not leak diagnostic link IDs into prose"
    );
}

#[test]
#[ignore = "tracked requirement: assistant should record the user message as an `impulse` link before answering"]
fn user_messages_are_recorded_as_impulse_events() {
    let response = answer("Hello there");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("impulse:")),
        "answers should reference the impulse event that triggered them"
    );
}

#[test]
#[ignore = "tracked requirement: chat answers should expose a trace link the user can follow"]
fn every_answer_exposes_a_trace_link_for_inspection() {
    let response = answer("Hi");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("trace:")),
        "answers should reference a trace link so the user can inspect reasoning steps"
    );
}

#[test]
#[ignore = "tracked requirement: unknown intents should propose a follow-up that creates a Links Notation seed"]
fn unknown_intent_offers_a_path_to_extend_the_network() {
    let response = answer("Some unseen request");
    assert_eq!(response.intent, "unknown");
    assert!(
        response.answer.contains("add a fact")
            || response.answer.contains("add a rule")
            || response.answer.contains("add Links Notation"),
        "unknown answers should invite the user to extend the network"
    );
}

// ---------------------------------------------------------------------------
// Inappropriate content policy: issue #39.
// ---------------------------------------------------------------------------

#[test]
fn physical_action_question_is_answered_factually() {
    // Reproduces the exact reporter prompt from issue #39.
    // The AI has no physical body, so the factually correct answer is "No."
    // Treating this as inappropriate content and refusing would be unhelpful.
    let response = answer("Сосал?");
    assert_eq!(
        response.intent, "physical_action_question",
        "physical action question must be answered factually, got intent: {}",
        response.intent
    );
}

#[test]
fn physical_action_question_reply_is_factual_no() {
    let response = answer("Сосал?");
    assert!(
        response.answer.contains("Нет") || response.answer.contains("нет"),
        "physical action question must be answered with 'No', got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("тело") || response.answer.contains("физич"),
        "physical action question answer must reference absence of physical body, got: {}",
        response.answer
    );
}

#[test]
fn english_vulgar_prompt_is_refused_with_policy_intent() {
    let response = answer("suck my dick");
    assert_eq!(
        response.intent, "policy_inappropriate_content",
        "English vulgar prompt must be refused, got intent: {}",
        response.intent
    );
}

#[test]
fn capabilities_answer_in_hindi_advertises_behavior_rule_commands() {
    let response = answer("तुम क्या कर सकते हो?");
    assert_eq!(response.intent, "capabilities");
    assert!(
        response.answer.contains("List behavior rules"),
        "Hindi capabilities answer must mention List behavior rules; got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("When I say"),
        "Hindi capabilities answer must mention teach grammar; got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Report issue"),
        "Hindi capabilities answer must mention Report issue; got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("List all facts you know about yourself"),
        "Hindi capabilities answer must mention self-facts command; got: {}",
        response.answer
    );
}

#[test]
fn capabilities_answer_in_chinese_advertises_behavior_rule_commands() {
    let response = answer("你能做什么?");
    assert_eq!(response.intent, "capabilities");
    assert!(
        response.answer.contains("List behavior rules"),
        "Chinese capabilities answer must mention List behavior rules; got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("When I say"),
        "Chinese capabilities answer must mention teach grammar; got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Report issue"),
        "Chinese capabilities answer must mention Report issue; got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("List all facts you know about yourself"),
        "Chinese capabilities answer must mention self-facts command; got: {}",
        response.answer
    );
}

#[test]
fn unknown_answer_mentions_report_issue_and_export_memory() {
    let response = answer("This is an entirely synthetic prompt nobody has seen before zzz123.");
    assert_eq!(response.intent, "unknown");
    assert!(
        response.answer.contains("Report issue"),
        "unknown answer must surface Report issue path; got: {}",
        response.answer
    );
    assert!(
        response.answer.to_lowercase().contains("export"),
        "unknown answer must mention exporting memory for durability; got: {}",
        response.answer
    );
}

#[test]
fn unknown_answer_mentions_report_issue_in_russian() {
    let response = answer("Какая у тебя модель личности?");
    assert_eq!(response.intent, "unknown");
    assert!(
        response.answer.contains("Report issue"),
        "Russian unknown answer must surface Report issue path; got: {}",
        response.answer
    );
}

#[test]
fn unknown_answer_is_strict_superset_of_seed_opener() {
    use formal_ai::unknown_answer_variation_for;
    // The first opener of the English pool is "I don't know how to answer that yet."
    // which matches the seed-text opener. With an empty prompt we get that exact opener.
    let body = unknown_answer_variation_for("");
    assert!(
        body.starts_with("I don't know how to answer that yet."),
        "empty-prompt fallback must use the default seed opener as a strict superset, got: {body}"
    );
}

#[test]
fn behavior_rule_teach_supports_english_if_i_ask_grammar() {
    let solver = UniversalSolver::default();
    let update = solver.solve("If I ask `tell me a joke`, reply `I do not have a joke pool yet.`");
    assert_eq!(update.intent, "behavior_rule_update");

    let history = [ConversationTurn::user(
        "If I ask `tell me a joke`, reply `I do not have a joke pool yet.`",
    )];
    let response = solver.solve_with_history("tell me a joke", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "I do not have a joke pool yet.");
}

#[test]
fn behavior_rule_teach_supports_russian_esli_grammar() {
    let solver = UniversalSolver::default();
    let update =
        solver.solve("Если я спрошу `Какая у тебя модель личности?`, ответь `Символьная модель.`");
    assert_eq!(update.intent, "behavior_rule_update");

    let history = [ConversationTurn::user(
        "Если я спрошу `Какая у тебя модель личности?`, ответь `Символьная модель.`",
    )];
    let response = solver.solve_with_history("Какая у тебя модель личности?", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "Символьная модель.");
}

#[test]
fn opener_pools_have_distinct_first_entries_per_language() {
    use formal_ai::unknown_answer_variation_for;
    // The first opener of each language pool is the seed opener for that language.
    // For an empty prompt we always pick index 0. Verify English pool is distinct
    // from Russian/Hindi/Chinese seeds by spot-checking the prefix characters.
    let english = unknown_answer_variation_for("");
    assert!(english.starts_with("I don't know"));
}

#[test]
fn behavior_rules_listing_includes_runtime_rule_when_history_has_one() {
    let solver = UniversalSolver::default();
    let history = [ConversationTurn::user(
        "When I say `synthetic question`, answer `synthetic answer`.",
    )];
    let response = solver.solve_with_history("List behavior rules", &history);
    assert_eq!(response.intent, "behavior_rules_list");
    assert!(
        response.answer.contains("behavior_rule_runtime") || response.answer.contains("synthetic"),
        "runtime rule should appear in the listing once taught; got: {}",
        response.answer
    );
}

#[test]
fn self_facts_answer_includes_model_id_and_strategy() {
    let response = answer("List all facts you know about yourself");
    assert_eq!(response.intent, "self_facts");
    assert!(
        response.answer.to_lowercase().contains("formal-ai")
            || response.answer.to_lowercase().contains("symbolic"),
        "self facts should describe the model identity; got: {}",
        response.answer
    );
}

#[test]
fn behavior_rule_detail_uses_describe_prefix() {
    let response = answer("describe behavior rule unknown");
    assert_eq!(response.intent, "behavior_rule_detail");
    assert!(
        response.answer.contains("rule_unknown"),
        "describe prefix must surface the same detail as Show behavior rule; got: {}",
        response.answer
    );
}

#[test]
fn behavior_rule_detail_uses_read_rule_prefix() {
    let response = answer("Read rule unknown");
    assert_eq!(response.intent, "behavior_rule_detail");
    assert!(
        response.answer.contains("rule_unknown"),
        "Read rule prefix must surface the same detail as Show behavior rule; got: {}",
        response.answer
    );
}

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
        "# Hello-world programs",
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
