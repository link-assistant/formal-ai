use formal_ai::engine::SymbolicAnswer;
use formal_ai::event_log::EventLog;
use formal_ai::normalize_prompt_text;
use formal_ai::triz_solver::{handle_triz, triz_demands, triz_families};

fn handled(prompt: &str) -> Option<SymbolicAnswer> {
    handle_triz(prompt, &normalize_prompt_text(prompt), &mut EventLog::new())
}

#[test]
fn a_russian_contradiction_prompt_gets_the_localized_worksheet() {
    let prompt = "Как разрешить противоречие: деталь должна быть жёсткой и одновременно гибкой?";
    let answer = handled(prompt).expect("explicit contradiction");
    assert_eq!(answer.intent, "triz_resolution");
    assert_eq!(
        answer.answer.split_once('.').map(|(demands, _)| demands),
        Some("Конфликтующие требования: деталь должна быть жёсткой / гибкой")
    );
    assert!(answer.answer.contains("Range selection"));
    assert!(answer.answer.contains("Dimension change"));
}

#[test]
fn unseen_demands_produce_their_own_worksheet_without_recalled_cases() {
    for (prompt, left, right) in [
        (
            "Apply TRIZ to latency vs acoustic damping",
            "latency",
            "acoustic damping",
        ),
        (
            "Un dilema, contradicción: latencia frente a fidelidad",
            "latencia",
            "fidelidad",
        ),
        ("TRIZ: कम विलंब बनाम अधिक सटीकता", "कम विलंब", "अधिक सटीकता"),
        ("矛盾：延迟与隔音", "延迟", "隔音"),
        ("TRIZ: A {families} vs B {left}", "A {families}", "B {left}"),
        (
            "TRIZ: ratio 1:2 vs equal allocation",
            "ratio 1:2",
            "equal allocation",
        ),
    ] {
        assert_eq!(triz_demands(prompt), Some((left, right)), "{prompt}");
        let answer = handled(prompt).expect("explicit contradiction").answer;
        let demands = answer
            .split_once([':', '：'])
            .and_then(|(_, tail)| tail.split_once(['.', '。', '।']))
            .map(|(demands, _)| demands.trim());
        assert_eq!(
            demands,
            Some(format!("{left} / {right}").as_str()),
            "{answer}"
        );
        for old_case in ["pills", "umbrella", "wheat grains"] {
            assert!(!answer.contains(old_case), "{answer}");
        }
    }
}

#[test]
fn conflicting_demands_are_recorded_and_missing_demands_are_requested() {
    let mut log = EventLog::new();
    let prompt = "How do we resolve this paradox: transparency vs strength?";
    handle_triz(prompt, &normalize_prompt_text(prompt), &mut log).expect("explicit contradiction");
    assert!(log.first_of("triz_solver:demands").is_some());

    for prompt in ["Explain a paradox: zoetrope", "TRIZ: speed vs speed"] {
        let mut log = EventLog::new();
        let answer =
            handle_triz(prompt, &normalize_prompt_text(prompt), &mut log).expect("missing demands");
        assert!(log.first_of("triz_resolution:refusal").is_some());
        assert_eq!(
            answer.answer,
            "State the two conflicting demands and the conditions under which each must hold."
        );
    }
}

#[test]
fn ordinary_prompts_decline_and_the_family_catalog_is_present() {
    assert!(handled("What is the capital of France?").is_none());
    assert!(handled("Напомни мне встречу в 20:00").is_none());
    assert!(triz_families().len() >= 12);
}
