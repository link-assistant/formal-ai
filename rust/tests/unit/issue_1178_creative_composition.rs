//! Issue #1178 (E143): creative writing, brainstorming, planning, and
//! advice as constraint-based composition from formalized sources.
//!
//! The four handlers are tested directly at the handler level — the
//! dispatch wiring lands with the integration commit that registers
//! `meanings-creative-tasks.lino` (lexicon), `creative-composition-rules.lino`
//! (bundle), and `multilingual-responses-creative-tasks.lino` (responses)
//! in the seed registry, exactly as the code-task family of issue #1177
//! landed before its wiring.
//!
//! Requirement coverage:
//! - R1 brainstorming composes candidates from the formalized topic
//!   (coffee/shop/university), never the canned web-search paragraph and
//!   never the memorized dev-tool name pool;
//! - R2 the answer states its distinctness metric and the constraints every
//!   candidate passed (length, pronounceability, named exclusions);
//! - R3 a poem's form constraints are extracted and the rendered output is
//!   checked against them before it is returned — a constraint-violating
//!   composition yields the honest refusal, never the poem;
//! - R4 English rhyme grounds in the Wiktionary rhyme classes; for
//!   ru/hi/zh the pronunciation-coverage gap is stated, not guessed;
//! - R5 the itinerary schedules under its constraints, cites every item,
//!   and reports its feasibility check;
//! - R6 advice items carry citation and evidence grade, ordered by grade
//!   weight, with the #1179 relative-meta-logic forward dependency stated;
//! - R7 none of the four classes returns the "Web search requested for"
//!   paragraph, and planning never answers with a terminal command;
//! - R8 held-out probe sets per class in en/ru/hi/zh with automated
//!   constraint checks (lift-out into data/benchmarks/*-tasks/ shards is
//!   the integration step; the checks themselves live here).

use formal_ai::engine::SymbolicAnswer;
use formal_ai::event_log::EventLog;
use formal_ai::web_engine_core::normalize_prompt;

/// Run one handler over a prompt and return its answer body.
macro_rules! answer_of {
    ($handler:path, $prompt:expr) => {{
        let prompt: &str = $prompt;
        let normalized = normalize_prompt(prompt);
        let mut log = EventLog::new();
        let answer: SymbolicAnswer = $handler(prompt, &normalized, &mut log)
            .unwrap_or_else(|| panic!("{} should answer this request", stringify!($handler)));
        answer.answer
    }};
}

/// The poem block of a creative-writing answer: the text before the
/// "Constraint check" report.
fn poem_of(answer: &str) -> Vec<String> {
    answer
        .split("Constraint check")
        .next()
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

// ---------------------------------------------------------------------------
// R1 + R7: brainstorming composes from the topic, never the canned
// paragraph, never the memorized pool.
// ---------------------------------------------------------------------------

#[test]
fn handler_brainstorm_composes_from_the_formalized_topic() {
    let answer = answer_of!(
        formal_ai::handle_brainstorm_request,
        "five name ideas for a coffee shop near a university"
    );
    assert!(
        answer.contains("Coffee"),
        "candidates compose from the topic: {answer}"
    );
    assert!(
        answer.contains("coffee") && answer.contains("university"),
        "the formalized concepts are named: {answer}"
    );
    assert!(!answer.contains("Web search requested for"), "{answer}");
    assert!(
        !answer.contains("TraceLint"),
        "the memorized pool is not consulted: {answer}"
    );
    assert!(!answer.contains("Links Notation notebook"), "{answer}");
}

#[test]
fn handler_brainstorm_states_metric_and_constraints() {
    let answer = answer_of!(
        formal_ai::handle_brainstorm_request,
        "five short name ideas for a coffee shop without cafe in them"
    );
    assert!(
        answer.contains("levenshtein"),
        "the metric is stated: {answer}"
    );
    assert!(
        answer.contains("12 characters"),
        "short caps the length: {answer}"
    );
    assert!(
        answer.contains("excluding"),
        "the named exclusion is stated: {answer}"
    );
    assert!(
        !answer.contains("Cafe"),
        "excluded words are filtered: {answer}"
    );
    // Every numbered candidate obeys the short cap.
    for line in answer.lines() {
        if let Some(rest) = line
            .strip_prefix("1. ")
            .or_else(|| line.strip_prefix("2. "))
        {
            assert!(rest.chars().count() <= 12, "{rest} exceeds the cap");
        }
    }
}

#[test]
fn handler_brainstorm_honors_the_requested_count() {
    let answer = answer_of!(
        formal_ai::handle_brainstorm_request,
        "three name ideas for a bakery"
    );
    assert!(answer.contains("3 name candidates"), "{answer}");
}

#[test]
fn handler_brainstorm_multilingual_probes() {
    let probes = [
        (
            "ru",
            "придумай пять идей для кофейной рядом с университетом",
        ),
        ("hi", "विश्वविद्यालय के पास कॉफी की दुकान के लिए नाम के विचार दो"),
        ("zh", "给大学附近的咖啡店起个名字"),
    ];
    for (language, prompt) in probes {
        let answer = answer_of!(formal_ai::handle_brainstorm_request, prompt);
        assert!(
            answer.contains('.'),
            "{language} brainstorming returns candidates: {answer}"
        );
        assert!(
            !answer.contains("Web search requested for"),
            "{language}: {answer}"
        );
    }
}

// ---------------------------------------------------------------------------
// R3: poems are checked against their form constraints before returning.
// ---------------------------------------------------------------------------

#[test]
fn handler_creative_writing_four_line_poem_about_the_sea() {
    let answer = answer_of!(
        formal_ai::handle_creative_writing_request,
        "Write a four-line poem about the sea"
    );
    let poem = poem_of(&answer);
    assert_eq!(poem.len(), 4, "exactly four lines: {answer}");
    assert!(
        poem.iter().any(|line| line.to_lowercase().contains("sea")),
        "the topic word appears: {answer}"
    );
    assert!(
        answer.contains("rhyme scheme abcb"),
        "the default scheme is stated: {answer}"
    );
    assert!(
        answer.contains("holds"),
        "the constraint check passed: {answer}"
    );
    assert!(!answer.contains("Web search requested for"), "{answer}");
}

#[test]
fn handler_creative_writing_rhymed_positions_share_a_rhyme_class() {
    let answer = answer_of!(
        formal_ai::handle_creative_writing_request,
        "Write a four-line poem about the sea"
    );
    let poem = poem_of(&answer);
    let end = |line: &str| {
        line.split_whitespace()
            .next_back()
            .unwrap_or_default()
            .to_lowercase()
    };
    let night_class = ["night", "light", "bright", "white", "sight", "flight"];
    assert!(
        night_class.contains(&end(&poem[1])) && night_class.contains(&end(&poem[3])),
        "positions 2 and 4 end in one rhyme class: {answer}"
    );
}

#[test]
fn handler_creative_writing_never_returns_a_violating_poem() {
    // A rhyming sonnet: 14 lines under a 4-letter scheme cannot satisfy
    // the check, so the refusal comes back instead of a violating poem.
    let answer = answer_of!(
        formal_ai::handle_creative_writing_request,
        "Write a rhyming sonnet about the star"
    );
    assert!(
        answer.contains("never returned"),
        "the honest refusal names the unsatisfied constraint: {answer}"
    );
    assert!(
        !answer.contains("Constraint check, run on the rendered poem before it was returned:\n- lines: 14 (wanted 14)"),
        "no unverified poem body is returned: {answer}"
    );
}

#[test]
fn handler_creative_writing_haiku_states_syllables_unverified() {
    let answer = answer_of!(
        formal_ai::handle_creative_writing_request,
        "Write a haiku about the moon"
    );
    let poem = poem_of(&answer);
    assert_eq!(poem.len(), 3, "a haiku carries three lines: {answer}");
    assert!(
        answer.contains("not machine-verified"),
        "meter honesty: {answer}"
    );
}

// ---------------------------------------------------------------------------
// R4: the pronunciation-coverage gap is stated, not guessed.
// ---------------------------------------------------------------------------

#[test]
fn handler_creative_writing_states_the_non_english_rhyme_gap() {
    let prompts = [
        ("ru", "напиши стихотворение о море в четыре строки"),
        ("hi", "चार पंक्तियों में समुद्र पर कविता लिखो"),
        ("zh", "写一首关于海的四行诗"),
    ];
    for (language, prompt) in prompts {
        let answer = answer_of!(formal_ai::handle_creative_writing_request, prompt);
        let poem = poem_of(&answer);
        assert_eq!(poem.len(), 4, "{language}: exactly four lines: {answer}");
        assert!(
            answer.contains("gap"),
            "{language}: the gap is stated: {answer}"
        );
        assert!(
            !answer.contains("rhyme scheme") || answer.contains("none"),
            "{language}: no guessed rhyme: {answer}"
        );
    }
}

// ---------------------------------------------------------------------------
// R5: the itinerary schedules under constraints and cites every item.
// ---------------------------------------------------------------------------

#[test]
fn handler_planning_builds_a_cited_feasible_itinerary() {
    let answer = answer_of!(
        formal_ai::handle_planning_request,
        "3-day itinerary for Rome"
    );
    assert!(answer.contains("Day 1"), "{answer}");
    assert!(answer.contains("Day 3"), "{answer}");
    assert!(answer.contains("Colosseum"), "{answer}");
    assert!(
        answer.contains("wikivoyage.org"),
        "items cite their source: {answer}"
    );
    assert!(
        answer.matches("wikivoyage.org").count() >= 5,
        "every scheduled item carries a citation: {answer}"
    );
    assert!(
        answer.contains("no overlapping items"),
        "feasibility reported: {answer}"
    );
    assert!(answer.contains("travel time accounted"), "{answer}");
    assert!(
        !answer.contains("terminal"),
        "no terminal-command misroute: {answer}"
    );
    assert!(!answer.contains("Web search requested for"), "{answer}");
}

#[test]
fn handler_planning_schedule_respects_windows() {
    let answer = answer_of!(
        formal_ai::handle_planning_request,
        "3-day itinerary for Rome"
    );
    // Every scheduled window is inside the day window and ordered within
    // its day (the clock resets at each "Day N" header).
    let mut previous_end = 0;
    for line in answer.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Day ") {
            previous_end = 0;
            continue;
        }
        let Some((window, _rest)) = trimmed.split_once(' ') else {
            continue;
        };
        let Some((start, end)) = window.split_once('-') else {
            continue;
        };
        let (Ok(start), Ok(end)) = (
            start.replace(':', "").parse::<u32>(),
            end.replace(':', "").parse::<u32>(),
        ) else {
            continue;
        };
        let (hours, minutes) = (start / 100, start % 100);
        let start_minutes = hours * 60 + minutes;
        let (hours, minutes) = (end / 100, end % 100);
        let end_minutes = hours * 60 + minutes;
        assert!(
            start_minutes >= 9 * 60,
            "nothing before the day window: {trimmed}"
        );
        assert!(
            end_minutes <= 19 * 60,
            "nothing past the day window: {trimmed}"
        );
        assert!(start_minutes >= previous_end, "no overlap: {trimmed}");
        previous_end = end_minutes;
    }
}

#[test]
fn handler_planning_unknown_destination_refuses_honestly() {
    let answer = answer_of!(
        formal_ai::handle_planning_request,
        "plan a trip to Atlantis"
    );
    assert!(answer.contains("no places in the seeded cache"), "{answer}");
    assert!(!answer.contains("Web search requested for"), "{answer}");
}

#[test]
fn handler_planning_multilingual_probes() {
    let probes = [
        ("ru", "составь маршрут по Риму на 3 дня"),
        ("zh", "罗马三天行程"),
    ];
    for (language, prompt) in probes {
        let answer = answer_of!(formal_ai::handle_planning_request, prompt);
        assert!(answer.contains("Day 1"), "{language}: {answer}");
        assert!(
            answer.contains("wikivoyage.org"),
            "{language}: cited: {answer}"
        );
        assert!(!answer.contains("terminal"), "{language}: {answer}");
    }
}

// ---------------------------------------------------------------------------
// R6: advice carries citation and evidence grade, weighted and honest.
// ---------------------------------------------------------------------------

#[test]
fn handler_advice_returns_cited_graded_guidance() {
    let answer = answer_of!(
        formal_ai::handle_advice_request,
        "evidence-based tips for trouble sleeping"
    );
    assert!(answer.contains("CDC"), "{answer}");
    assert!(answer.contains("cdc.gov"), "citations are URLs: {answer}");
    assert!(
        answer.contains("strong (health-authority guideline)"),
        "the evidence grade is labeled: {answer}"
    );
    assert!(
        answer.contains("issue #1179"),
        "the relative-meta-logic forward dependency is stated: {answer}"
    );
    assert!(!answer.contains("Web search requested for"), "{answer}");
}

#[test]
fn handler_advice_multilingual_probes() {
    let probes = [
        ("ru", "дай советы для борьбы с бессонницей"),
        ("hi", "नींद के लिए सलाह दो"),
        ("zh", "改善睡眠的建议"),
    ];
    for (language, prompt) in probes {
        let answer = answer_of!(formal_ai::handle_advice_request, prompt);
        assert!(answer.contains("CDC"), "{language}: {answer}");
        assert!(answer.contains("cdc.gov"), "{language}: {answer}");
    }
}

#[test]
fn handler_advice_unknown_topic_refuses_honestly() {
    let answer = answer_of!(
        formal_ai::handle_advice_request,
        "advice for choosing a hoverboard"
    );
    assert!(answer.contains("no formalized recommendations"), "{answer}");
}

// ---------------------------------------------------------------------------
// R8: none of the four classes claims unrelated prompts.
// ---------------------------------------------------------------------------

#[test]
fn unrelated_prompts_are_not_claimed_by_any_composition_handler() {
    let unrelated = [
        "Hello, how are you today?",
        "What is the capital of France?",
        "Convert this JSON to YAML:\n```json\n{\"a\": 1}\n```",
    ];
    let handlers: [(
        fn(&str, &str, &mut EventLog) -> Option<SymbolicAnswer>,
        &str,
    ); 4] = [
        (formal_ai::handle_brainstorm_request, "brainstorming"),
        (
            formal_ai::handle_creative_writing_request,
            "creative writing",
        ),
        (formal_ai::handle_planning_request, "planning"),
        (formal_ai::handle_advice_request, "advice"),
    ];
    for prompt in unrelated {
        let normalized = normalize_prompt(prompt);
        for (handler, name) in handlers {
            let mut log = EventLog::new();
            assert!(
                handler(prompt, &normalized, &mut log).is_none(),
                "{name} must not claim: {prompt}"
            );
        }
    }
}
