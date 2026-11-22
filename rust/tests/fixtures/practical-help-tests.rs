use super::*;
use crate::source_fetch::FetchError;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const TOILET_SOURCE: &str = "https://www.nhs.uk/conditions/constipation/";
const TOILET_HTML: &[u8] = include_bytes!("practical-help/toilet.html");
const GUIDANCE: &str = "Do not delay if you feel the urge to poo.";
static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Transport<'a> {
    body: &'a [u8],
}
impl SourceTransport for Transport<'_> {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        assert_eq!(url, TOILET_SOURCE);
        Ok(self.body.to_vec())
    }
}

fn temporary_cache_path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "formal-ai-practical-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ))
}

#[test]
fn practical_goals_render_observed_guidance_with_localized_attribution() {
    let root = temporary_cache_path();
    let client = CachedSourceClient::new(&root, Transport { body: TOILET_HTML }).with_online(true);
    for (prompt, label) in [
        (
            "I really need to poop, what should I do?",
            "Source guidance",
        ),
        ("Хочу какать, что делать?", "Рекомендации источника"),
        ("मुझे शौच करना है", "स्रोत के निर्देश"),
        ("我快要大便了，怎么办？", "来源指引"),
        (
            "Necesito ir al baño, ¿qué debo hacer?",
            "Consejos de la fuente",
        ),
    ] {
        let answer = with_client(
            prompt,
            &normalize_prompt(prompt),
            &mut EventLog::new(),
            &client,
        )
        .expect("recognized first-person need");
        assert_eq!(answer.intent, "procedural_how_to", "{prompt}");
        assert!(
            answer
                .answer
                .contains(&format!("[{label}]({TOILET_SOURCE})")),
            "{}",
            answer.answer
        );
        assert!(answer.answer.contains(GUIDANCE), "{}", answer.answer);
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn urgent_followups_resolve_the_latest_goal_with_the_same_transport() {
    let root = temporary_cache_path();
    let client = CachedSourceClient::new(&root, Transport { body: TOILET_HTML }).with_online(true);
    let mut log = EventLog::new();
    let prompt = "Сейчас обкакаюсь! Помоги!";
    let urgent = with_client(prompt, &normalize_prompt(prompt), &mut log, &client)
        .expect("recognized urgent need");
    assert!(urgent.answer.contains(GUIDANCE), "{}", urgent.answer);

    for (prior, question, locale) in [
        (
            "Хочу какать, что делать?",
            "Помоги!",
            "Рекомендации источника",
        ),
        (
            "Necesito ir al baño, ¿qué debo hacer?",
            "¡Ayúdame!",
            "Consejos de la fuente",
        ),
    ] {
        let mut log = EventLog::new();
        log.append("prior_turn:user", prior);
        log.append("prior_turn:assistant", "A previous answer.");
        let followup = crate::solver_handler_how::try_procedural_how_to_followup_with(
            question,
            &normalize_prompt(question),
            &mut log,
            |prior, normalized, log| with_client(prior, normalized, log, &client),
        )
        .expect("the previous practical need can be resolved again");
        assert!(followup.answer.contains(locale), "{}", followup.answer);
        assert!(followup.answer.contains(GUIDANCE), "{}", followup.answer);
    }

    log.append("prior_turn:user", "Tell me about stars");
    log.append("prior_turn:assistant", "Stars are astronomical objects.");
    assert!(
        crate::solver_handler_how::try_procedural_how_to_followup("help me", "help me", &mut log,)
            .is_none()
    );
    std::fs::remove_dir_all(root).unwrap();
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
        "I want to understand the toilet history",
        "I needed to poop yesterday",
    ] {
        assert!(
            goal(prompt, &normalize_prompt(prompt)).is_none(),
            "{prompt}"
        );
    }
}

#[test]
fn missing_evidence_prompts_an_authorized_choice_in_each_language() {
    let root = temporary_cache_path();
    let client = CachedSourceClient::new(&root, Transport { body: TOILET_HTML });
    for (prompt, expected) in [
        (
            "I need to poop",
            "It sounds urgent. Do you want help finding a toilet, or are you unable to go?",
        ),
        (
            "Хочу какать, что делать?",
            "Похоже, это срочно. Хотите, чтобы я помог найти туалет, или у вас не получается сходить?",
        ),
        (
            "मुझे शौच करना है",
            "यह ज़रूरी लग रहा है। क्या आप शौचालय ढूँढने में मदद चाहते हैं, या शौच नहीं हो पा रहा है?",
        ),
        (
            "我快要大便了，怎么办？",
            "听起来很着急。是否需要帮你找厕所，还是你排不出来？",
        ),
        (
            "Necesito ir al baño, ¿qué debo hacer?",
            "Suena urgente. ¿Quieres ayuda para encontrar un baño o no puedes evacuar?",
        ),
    ] {
        assert_eq!(
            crate::question_necessity::classify_question(expected).class,
            crate::question_necessity::QuestionClass::Requirement,
        );
        let mut log = EventLog::new();
        let answer = from_source(prompt, "toilet", &mut log, &client);
        assert_eq!(answer.answer, expected, "{prompt}");
        assert_eq!(answer.confidence, 0.0);
        assert!(log.last_of("source:http").is_none(), "{prompt}");
        assert!(
            log.last_of("question_necessity:authorized").is_some(),
            "{prompt}"
        );
    }
    assert!(
        !root.exists(),
        "a read-only miss must not create cache records"
    );
}

#[test]
fn matching_captures_supply_their_own_excerpt_and_record_the_digest() {
    for excerpt in [
        GUIDANCE,
        "Use the closest available toilet when the urge returns.",
    ] {
        let body = format!("<h3>Improve your toilet routine</h3><p>{excerpt}</p>");
        let root = temporary_cache_path();
        let transport = Transport {
            body: body.as_bytes(),
        };
        let client = CachedSourceClient::new(&root, transport).with_online(true);
        let mut log = EventLog::new();
        let answer = from_source("I need to poop", "toilet", &mut log, &client);
        assert!(answer.answer.contains(excerpt), "{}", answer.answer);
        let proof = &log.last_of("source:http").unwrap().payload;
        assert!(
            proof.starts_with(TOILET_SOURCE) && proof.ends_with(" cached=false"),
            "{proof}"
        );

        let cache_only = CachedSourceClient::new(&root, Transport { body: b"unused" });
        let mut log = EventLog::new();
        let replay = from_source("I need to poop", "toilet", &mut log, &cache_only);
        assert_eq!(answer.answer, replay.answer);
        assert_eq!(
            log.last_of("source:http").unwrap().payload,
            proof.replace("cached=false", "cached=true"),
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn unrelated_or_oversized_source_does_not_support_the_answer() {
    let outside_section =
        format!("<h3>Improve your toilet routine</h3><h3>Another topic</h3><p>{GUIDANCE}</p>");
    let oversized = vec![b'x'; MAX_CAPTURE_BYTES + 1];
    for body in [
        outside_section.as_bytes(),
        b"Unrelated festival text",
        &oversized,
    ] {
        let root = temporary_cache_path();
        let client = CachedSourceClient::new(&root, Transport { body }).with_online(true);
        let mut log = EventLog::new();
        let answer = from_source("I need to poop", "toilet", &mut log, &client);
        assert_eq!(answer.confidence, 0.0);
        assert!(log.last_of("source:http").is_none());
        assert_eq!(
            log.last_of("practical_help:source_mode").unwrap().payload,
            "unavailable"
        );
        assert!(answer.answer.contains('?'));
        let _ = std::fs::remove_dir_all(root);
    }
}
