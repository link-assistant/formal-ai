use super::*;
use crate::source_fetch::FetchError;
use crate::{ConversationTurn, FormalAiEngine, UniversalSolver};
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Transport {
    body: Vec<u8>,
}
impl SourceTransport for Transport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        assert_eq!(url, policy("toilet-source"));
        Ok(self.body.clone())
    }
}
#[test]
fn practical_goals_reach_the_native_solver_in_all_registered_languages() {
    for (prompt, expected) in [
        (
            "I really need to poop, what should I do?",
            "Use a toilet as soon as you can rather than holding it. If you cannot go or have pain, tell me what is happening. [NHS guidance](https://www.nhs.uk/conditions/constipation/)",
        ),
        (
            "Хочу какать, что делать?",
            "Идите в туалет при первой возможности, не терпите. Если не получается сходить или есть боль, уточните, что происходит. [Рекомендации NHS](https://www.nhs.uk/conditions/constipation/)",
        ),
        (
            "मुझे शौच करना है",
            "जितनी जल्दी हो सके शौचालय जाएँ, रोककर न रखें। यदि शौच नहीं हो रहा है या दर्द है, तो बताएँ कि क्या हो रहा है। [NHS की सलाह](https://www.nhs.uk/conditions/constipation/)",
        ),
        (
            "我快要大便了，怎么办？",
            "尽快去厕所，不要憋着。如果排不出来或疼痛，请说明具体情况。[NHS 指引](https://www.nhs.uk/conditions/constipation/)",
        ),
        (
            "Necesito ir al baño, ¿qué debo hacer?",
            "Ve al baño en cuanto puedas y no te aguantes. Si no puedes evacuar o tienes dolor, explica qué ocurre. [Consejos del NHS](https://www.nhs.uk/conditions/constipation/)",
        ),
    ] {
        let answer = FormalAiEngine.answer(prompt);
        assert_eq!(
            answer.intent, "procedural_how_to",
            "{prompt}: {}",
            answer.answer
        );
        assert_eq!(answer.answer, expected, "{prompt}");
    }
}
#[test]
fn urgent_and_contextual_help_keep_the_last_owned_goal() {
    let solver = UniversalSolver::default();
    let urgent = solver.solve("Сейчас обкакаюсь! Помоги!");
    assert!(
        urgent.answer.contains("Идите в туалет"),
        "{}",
        urgent.answer
    );
    let history = [
        ConversationTurn::user("Хочу какать, что делать?"),
        ConversationTurn::assistant("Уточните, что происходит."),
    ];
    let answer = solver.solve_with_history("Помоги!", &history);
    assert!(
        answer.answer.contains("Идите в туалет"),
        "{}",
        answer.answer
    );
    let mut log = EventLog::new();
    log.append("prior_turn:user", "I need to poop");
    log.append("prior_turn:assistant", "Use a toilet.");
    log.append("prior_turn:user", "Tell me about stars");
    log.append("prior_turn:assistant", "Stars are astronomical objects.");
    assert!(followup("help me", "help me", &mut log).is_none());
}
#[test]
fn quoted_negated_and_unrelated_goals_are_not_claimed() {
    for prompt in [
        "\"I need to poop\" is a quotation",
        "I need to not poop",
        "Мне нужно не какать",
        "मुझे शौच नहीं करना है",
        "我不想大便",
        "Necesito no ir al baño",
        "No necesito ir al baño",
        "I need to write about poop",
        "I want to understand toilet history",
        "I needed to poop yesterday",
    ] {
        assert!(
            goal(prompt, &normalize_prompt(prompt)).is_none(),
            "{prompt}"
        );
    }
}
#[test]
fn spanish_followups_and_unavailable_evidence_keep_the_registered_goal() {
    let prompt = "Necesito ir al baño, ¿qué debo hacer?";
    let history = [
        ConversationTurn::user(prompt),
        ConversationTurn::assistant(
            "Ve al baño en cuanto puedas y no te aguantes. Si no puedes evacuar o tienes dolor, explica qué ocurre. [Consejos del NHS](https://www.nhs.uk/conditions/constipation/)",
        ),
    ];
    let answer = UniversalSolver::default().solve_with_history("¡Ayúdame!", &history);
    assert_eq!(
        answer.answer,
        "Ve al baño en cuanto puedas y no te aguantes. Si no puedes evacuar o tienes dolor, explica qué ocurre. [Consejos del NHS](https://www.nhs.uk/conditions/constipation/)"
    );
    let root = std::env::temp_dir().join(format!(
        "formal-ai-practical-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    let client = CachedSourceClient::new(
        &root,
        Transport {
            body: b"Unrelated festival text".to_vec(),
        },
    )
    .with_online(true);
    let mut log = EventLog::new();
    let unavailable = from_source(prompt, "toilet", &mut log, &client);
    assert_eq!(
        unavailable.answer,
        "Suena urgente. ¿Necesitas encontrar un baño o no puedes evacuar?"
    );
    assert_eq!(unavailable.confidence, 0.0);
    let _ = std::fs::remove_dir_all(root);
}
#[test]
fn an_empty_capture_cache_fetches_or_asks_without_claiming_unsupported_evidence() {
    for (body, supported) in [
        (policy("toilet-proof").into_bytes(), true),
        (b"Unrelated festival text".to_vec(), false),
        (vec![b'x'; 2_097_153], false),
    ] {
        let root = std::env::temp_dir().join(format!(
            "formal-ai-practical-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let client = CachedSourceClient::new(&root, Transport { body }).with_online(true);
        let mut log = EventLog::new();
        let answer = from_source("I need to poop", "toilet", &mut log, &client);
        assert_eq!(
            answer.answer.contains(&policy("toilet-source")),
            supported,
            "{}",
            answer.answer
        );
        assert_eq!(
            log.last_of("practical_help:source_mode").unwrap().payload,
            if supported {
                "fetched-source"
            } else {
                "unavailable"
            }
        );
        if !supported {
            assert!(answer.answer.contains('?'));
        }
        let _ = std::fs::remove_dir_all(root);
    }
}
