//! Issue #702: the current→target difference is askable from chat.
//!
//! [`crate::world_model_dialog`](formal_ai::world_model_dialog) maintains the
//! world model from the dialogue; these tests pin the *chat surface* of it:
//!
//! * the handler is inert until the knob is opted in (R13);
//! * once opted in, "what is left to do?" is answered from the recomputed
//!   difference, in the language the question was asked in;
//! * when every target statement already holds, the answer says nothing is left.

use formal_ai::solver::{ConversationTurn, SolverConfig, UniversalSolver};
use formal_ai::world_model_dialog::WorldModelMode;

/// One scripted dialogue per supported language: the fact, the wish, and the
/// state question. The English row is the reference dialogue; the Russian,
/// Hindi and Chinese rows ask the same three things in their own wording, so
/// every supported language exercises the handler rather than only English.
const DIALOGUES: &[(&str, &str, &str, &str, &str)] = &[
    (
        "en",
        "the door is closed",
        "I want the door to be open",
        "what is left to do?",
        "door -> open",
    ),
    (
        "ru",
        "дверь закрыта",
        "я хочу чтобы дверь была открыта",
        "что осталось сделать?",
        "дверь -> открыта",
    ),
    (
        "hi",
        "दरवाज़ा बंद है",
        "मुझे चाहिए दरवाज़ा खुला",
        "क्या बाकी है?",
        "दरवाज़ा -> खुला",
    ),
    (
        "zh",
        "门是关着的",
        "我想要门是开着的",
        "还剩什么?",
        // The declared filler `的` is dropped, so both phrasings reduce to the
        // same atom term and the difference lines up.
        "门 -> 开着",
    ),
];

fn tracking_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        world_model_mode: WorldModelMode::Track,
        ..SolverConfig::default()
    })
}

fn history(fact: &str, wish: &str) -> Vec<ConversationTurn> {
    vec![
        ConversationTurn::user(fact),
        ConversationTurn::assistant("noted"),
        ConversationTurn::user(wish),
        ConversationTurn::assistant("noted"),
    ]
}

#[test]
fn the_world_model_handler_is_inert_until_the_knob_is_opted_in() {
    let solver = UniversalSolver::default();
    assert_eq!(
        SolverConfig::default().world_model_mode,
        WorldModelMode::Off,
        "the world model must be off by default (R13)"
    );
    for (language, fact, wish, question, _) in DIALOGUES {
        let answer = solver.solve_with_history(question, &history(fact, wish));
        assert_ne!(
            answer.intent, "world_state_remaining",
            "[{language}] the state answer must not appear while the knob is off: {}",
            answer.answer
        );
        assert!(
            answer
                .evidence_links
                .iter()
                .all(|link| !link.contains("world_state:")),
            "[{language}] no world-model trace links may be emitted while off: {:?}",
            answer.evidence_links
        );
    }
}

#[test]
fn asking_what_is_left_answers_from_the_difference_in_every_language() {
    let solver = tracking_solver();
    for (language, fact, wish, question, expected) in DIALOGUES {
        let answer = solver.solve_with_history(question, &history(fact, wish));
        assert_eq!(
            answer.intent, "world_state_remaining",
            "[{language}] the state question must reach the world-model handler: {}",
            answer.answer
        );
        assert!(
            answer.answer.contains(expected),
            "[{language}] the answer must name the open target `{expected}`: {}",
            answer.answer
        );
        assert!(
            answer
                .evidence_links
                .iter()
                .any(|link| link.contains("world_state:remaining")),
            "[{language}] the open target must be evidence, not prose: {:?}",
            answer.evidence_links
        );
    }
}

#[test]
fn once_the_goal_holds_the_answer_says_nothing_is_left() {
    let solver = tracking_solver();
    let mut turns = history("the door is closed", "I want the door to be open");
    turns.push(ConversationTurn::user("the door is open"));
    turns.push(ConversationTurn::assistant("noted"));

    let answer = solver.solve_with_history("what is left to do?", &turns);
    assert_eq!(
        answer.intent, "world_state_reached",
        "a satisfied target must be reported as reached: {}",
        answer.answer
    );
}

#[test]
fn the_answer_is_recomputed_from_the_dialogue_and_never_mentions_embeddings() {
    let solver = tracking_solver();
    let turns = history("the door is closed", "I want the door to be open");
    let first = solver.solve_with_history("what is left to do?", &turns);
    let second = solver.solve_with_history("what is left to do?", &turns);
    assert_eq!(
        first.answer, second.answer,
        "the same dialogue must produce the same answer on replay"
    );
    let lowered = first.answer.to_lowercase();
    for forbidden in ["embedding", "vector", "vertex", "graph"] {
        assert!(
            !lowered.contains(forbidden),
            "the answer must stay in links vocabulary, found `{forbidden}`: {}",
            first.answer
        );
    }
}
