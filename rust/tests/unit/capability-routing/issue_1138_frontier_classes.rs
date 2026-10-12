//! Issue #1138 B10, plan 10 leaves 12-17 and 22: one test per frontier class.
//!
//! Each #1087 frontier prompt is a *class*, not a prompt. The class test asserts
//! the generalized behaviour and the honest-failure text when the capability is
//! unavailable, and `reported_frontier_prompts_are_not_in_the_seed` asserts the
//! seven reported strings occur in no seed file — which is the test that deletes
//! `data/seed/intent-routing.lino:400-402`.
//!
//! Written before the leaves that make them pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::capability_routing::{RoutingOutcome, route};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

/// Everything the table's rows can name, so a class failure is a routing
/// failure rather than an unadvertised tool.
const ADVERTISED: &[&str] = &[
    "web_fetch",
    "web_search",
    "read_file",
    "write_file",
    "list_dir",
    "grep",
    "shell",
    "calendar_create_event",
    "response_language_demonstration",
    "concept_measurement_lookup",
    "compose_from_sources",
    "explain_previous_turn",
    "report_issue",
    "ask_user",
];

fn resolved(outcome: &RoutingOutcome) -> Option<String> {
    match outcome {
        RoutingOutcome::Routed { capability } | RoutingOutcome::Lowered { capability, .. } => {
            Some(capability.clone())
        }
        RoutingOutcome::HonestGap { .. } | RoutingOutcome::Ask { .. } => None,
    }
}

fn assert_class(class: &str, capability: &str, prompts: [&str; 5]) {
    let mut failures: Vec<String> = Vec::new();
    for (language, prompt) in ["en", "ru", "hi", "zh", "es"].iter().zip(prompts) {
        let outcome = route(prompt, ADVERTISED);
        if resolved(&outcome).as_deref() != Some(capability) {
            failures.push(format!("{language}: {prompt:?} -> {outcome:?}"));
        }
    }
    assert!(
        failures.is_empty(),
        "class `{class}` must reach `{capability}` in every language: {failures:?}"
    );
}

/// With the class's capability withheld, the answer names what was needed and
/// what was missing — never a capability menu, never invented prose.
fn assert_honest_gap(class: &str, capability: &str, prompt: &str) {
    let withheld: Vec<&str> = ADVERTISED
        .iter()
        .copied()
        .filter(|tool| *tool != capability)
        .collect();
    match route(prompt, &withheld) {
        RoutingOutcome::HonestGap { needed, missing } => {
            assert_eq!(
                needed, capability,
                "class `{class}`: the gap must name the capability that was needed"
            );
            assert!(
                !missing.trim().is_empty(),
                "class `{class}`: the gap must name what was missing"
            );
        }
        RoutingOutcome::Lowered { preferred, .. } => assert_eq!(
            preferred, capability,
            "class `{class}`: a lowering must log the capability it lowered from"
        ),
        other => panic!(
            "class `{class}`: withholding `{capability}` must produce an honest gap or a \
             declared lowering, got {other:?}"
        ),
    }
}

#[test]
fn news_class_routes_to_a_live_search() {
    assert_class(
        "news",
        "web_search",
        [
            "Catch me up on what has happened since this morning.",
            "Введи меня в курс того, что случилось с утра.",
            "सुबह से अब तक क्या हुआ, मुझे बताइए।",
            "把今天早上到现在发生的事讲给我听。",
            "Ponme al día de lo que ha ocurrido desde esta mañana.",
        ],
    );
    assert_honest_gap("news", "web_search", "Anything breaking right now?");
}

#[test]
fn non_understanding_class_re_renders_the_previous_turn() {
    assert_class(
        "non_understanding",
        "explain_previous_turn",
        [
            "I did not follow that at all.",
            "Я совсем не уловил, о чём речь.",
            "मुझे कुछ भी पल्ले नहीं पड़ा।",
            "我完全没听明白。",
            "No he seguido nada de eso.",
        ],
    );
    assert_honest_gap(
        "non_understanding",
        "explain_previous_turn",
        "Could you unpack that a bit?",
    );
}

#[test]
fn compose_class_routes_to_the_composition_procedure() {
    assert_class(
        "compose",
        "compose_from_sources",
        [
            "Write me an extended piece on how the uncertainty principle came about.",
            "Напиши развёрнутый текст о том, как появился принцип неопределённости.",
            "अनिश्चितता सिद्धांत कैसे आया, इस पर विस्तृत लेख लिखिए।",
            "写一篇较长的文章，讲不确定性原理是怎么来的。",
            "Escríbeme un texto amplio sobre cómo surgió el principio de incertidumbre.",
        ],
    );
    assert_honest_gap(
        "compose",
        "compose_from_sources",
        "Produce a long-form explanation of the double-slit experiment.",
    );
}

#[test]
fn demonstrate_class_forces_the_response_language() {
    assert_class(
        "demonstrate",
        "response_language_demonstration",
        [
            "Say something to me in Russian.",
            "Скажи мне что-нибудь по-русски.",
            "मुझसे हिंदी में कुछ कहिए।",
            "用中文跟我说点什么。",
            "Dime algo en español.",
        ],
    );
    assert_honest_gap(
        "demonstrate",
        "response_language_demonstration",
        "Switch to Hindi for a moment.",
    );
}

#[test]
fn schedule_class_routes_to_the_calendar() {
    assert_class(
        "schedule",
        "calendar_create_event",
        [
            "Pencil in a review with Dmitri on Tuesday at 09:30, Berlin time.",
            "Поставь разбор с Дмитрием во вторник на 09:30 по Берлину.",
            "मंगलवार 09:30 बर्लिन समय दिमित्री के साथ समीक्षा रख दीजिए।",
            "周二 09:30 柏林时间，和德米特里安排一次评审。",
            "Agenda una revisión con Dmitri el martes a las 09:30, hora de Berlín.",
        ],
    );
    assert_honest_gap(
        "schedule",
        "calendar_create_event",
        "Hold Wednesday 20:00 for a call with the partners.",
    );
}

#[test]
fn measurement_class_routes_to_a_concept_measurement_lookup() {
    assert_class(
        "measurement",
        "concept_measurement_lookup",
        [
            "How tall does a mature birch normally grow?",
            "До какой высоты обычно вырастает взрослая берёза?",
            "पूर्ण विकसित भोजपत्र कितना ऊँचा होता है?",
            "成年白桦一般能长到多高？",
            "¿Hasta qué altura crece normalmente un abedul adulto?",
        ],
    );
    assert_honest_gap(
        "measurement",
        "concept_measurement_lookup",
        "How deep does the root of a dandelion go?",
    );
}

#[test]
fn ui_complaint_class_routes_to_a_structured_report() {
    assert_class(
        "ui_complaint",
        "report_issue",
        [
            "The panel divider you drew is impossible to drag with a mouse.",
            "Разделитель панелей, который ты нарисовал, невозможно перетащить мышью.",
            "आपने जो पैनल विभाजक बनाया है उसे माउस से खींचना नामुमकिन है।",
            "你画的那个面板分隔条用鼠标根本拖不动。",
            "El divisor de paneles que dibujaste no hay quien lo arrastre con el ratón.",
        ],
    );
    assert_honest_gap(
        "ui_complaint",
        "report_issue",
        "The panel divider you drew is impossible to drag with a mouse.",
    );
}

#[test]
fn reported_frontier_prompts_are_not_in_the_seed() {
    // The seven reported strings must be answered by a class, never memorized
    // as a phrase row. This is the test that deletes
    // `data/seed/intent-routing.lino:400-402` and the matching `lexeme zh`
    // surfaces at `data/seed/meanings-intent.lino:444-450` (plan 10 leaf 13).
    let reported = [
        "я не понимаю",
        "我不明白",
        "не понял",
        "Какого размера средний корень яблони",
        "Назначь мне встречу с Александром",
        "напиши мне эссе по квантовой механике",
        "скажи что-нибудь на хинди",
    ];
    let mut memorized: Vec<String> = Vec::new();
    for entry in walkdir::WalkDir::new(repo_root().join("data/seed"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("lino") {
            continue;
        }
        let text = fs::read_to_string(entry.path()).unwrap_or_default();
        let lowered = text.to_lowercase();
        for prompt in reported {
            if lowered.contains(&prompt.to_lowercase()) {
                memorized.push(format!(
                    "{}: {prompt:?}",
                    entry
                        .path()
                        .strip_prefix(repo_root())
                        .unwrap_or_else(|_| entry.path())
                        .display()
                ));
            }
        }
    }
    assert!(
        memorized.is_empty(),
        "a reported frontier prompt is stored as seed text, so a pass would prove \
         memorization rather than routing: {memorized:?}"
    );
}
