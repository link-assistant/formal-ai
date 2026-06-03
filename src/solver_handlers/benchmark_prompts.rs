use std::sync::OnceLock;

use super::finalize_simple;

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{
    self, BrainstormSeeds, CoreferenceSeeds, FactRecord, PersonaSeeds, SummaryTopicSeeds,
};
use crate::solver_helpers::last_user_turn;

fn summary_topic_seed_data() -> &'static SummaryTopicSeeds {
    static CELL: OnceLock<SummaryTopicSeeds> = OnceLock::new();
    CELL.get_or_init(seed::summary_topic_seeds)
}

pub fn try_summarization_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let seeds = summary_topic_seed_data();
    if !seeds.matches_trigger(normalized) {
        return None;
    }

    let (topic, body) = seeds.pick_topic(normalized).map_or_else(
        || {
            let label = prompt
                .trim_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
                .to_owned();
            let body = seeds.render_fallback(&label);
            (label, body)
        },
        |topic| (topic.display_name.clone(), topic.body.clone()),
    );

    log.append("summarization:topic", topic);
    if let Some(label) = seeds.constraint_for(normalized) {
        log.append("summarization:constraint", label.to_owned());
    }

    Some(finalize_simple(
        prompt,
        log,
        "summarize_topic",
        "response:summarize_topic",
        &body,
        0.85,
    ))
}

fn brainstorm_seed_data() -> &'static BrainstormSeeds {
    static CELL: OnceLock<BrainstormSeeds> = OnceLock::new();
    CELL.get_or_init(seed::brainstorm_seeds)
}

pub fn try_brainstorming_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let seeds = brainstorm_seed_data();
    if !seeds.matches_trigger(normalized) {
        return None;
    }
    let category = seeds.pick_category(normalized)?;
    let requested_count = requested_brainstorm_count(normalized);
    let body = numbered(&category.items, requested_count);
    log.append("brainstorm:category", category.slug.clone());
    Some(finalize_simple(
        prompt,
        log,
        &category.intent,
        "response:brainstorm",
        &body,
        0.8,
    ))
}

pub fn try_conversation_topic_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let topic = conversation_topic(prompt, normalized)?;
    let language = detect_language(prompt).slug();
    let body = match language {
        "ru" => format!(
            "Можем. Тема: {topic}. Я могу начать с краткого определения, \
             контекста или конкретного вопроса; если веб-поиск доступен, \
             публичные факты можно уточнить через внешний источник."
        ),
        "hi" => format!(
            "हम बात कर सकते हैं. विषय: {topic}. मैं छोटी परिभाषा, संदर्भ, \
             या किसी конкрет प्रश्न से शुरू कर सकता हूँ; web search उपलब्ध हो \
             तो public facts बाहरी स्रोत से जाँचे जा सकते हैं."
        ),
        "zh" => format!(
            "可以聊。主题: {topic}。我可以从简短定义、上下文或具体问题开始; \
             如果 web search 可用, 公开事实可以通过外部来源核对。"
        ),
        _ => format!(
            "We can talk about {topic}. I can start with a short definition, \
             context, or a specific question; when web search is available, \
             public facts can be checked against an external source."
        ),
    };

    log.append("conversation_topic", topic);
    Some(finalize_simple(
        prompt,
        log,
        "conversation_topic",
        "response:conversation_topic",
        &body,
        0.75,
    ))
}

/// Extract the topic the user proposed to discuss from a conversational opener.
///
/// The recognized surfaces — the let-us-talk-about-X phrasings in every
/// supported language — carry the [`seed::ROLE_CONVERSATION_TOPIC_OPENER`] role;
/// each is a prefix whose text before the `…` slot is the matchable opener, in
/// declaration order. A form whose `action` field is `scan` is also matched
/// anywhere in the prompt, not only at the start, so an opener that follows a
/// greeting is still found. No per-language opener list lives here — only the
/// concept; the surfaces come from `data/seed/meanings-conversation.lino`.
fn conversation_topic(prompt: &str, normalized: &str) -> Option<String> {
    let forms = seed::lexicon().role_word_forms(seed::ROLE_CONVERSATION_TOPIC_OPENER);
    for form in &forms {
        if let Some(topic) = normalized.strip_prefix(form.before_slot()) {
            return clean_conversation_topic(topic);
        }
    }

    let lower = prompt.to_lowercase();
    for form in &forms {
        if form.action == "scan" {
            if let Some((_, topic)) = lower.split_once(form.before_slot()) {
                return clean_conversation_topic(topic);
            }
        }
    }
    None
}

fn clean_conversation_topic(raw: &str) -> Option<String> {
    let topic = raw
        .trim()
        .trim_matches(|ch: char| {
            ch.is_whitespace()
                || matches!(
                    ch,
                    '`' | '"' | '\'' | ':' | '-' | '_' | '.' | ',' | '?' | '!'
                )
        })
        .to_owned();
    (!topic.is_empty()).then_some(topic)
}

/// Parse the number of items the user asked for. Defaults to 5 when no
/// explicit count is present. Recognises numeric and word forms in every
/// supported language so the algorithm doesn't depend on English-only
/// spelling.
fn requested_brainstorm_count(normalized: &str) -> usize {
    const TEN_HINTS: &[&str] = &[
        " 10 ",
        "10.",
        "10 ",
        " 10",
        "ten ",
        "десять",
        "10 идей",
        "10 имён",
        "दस ",
        "10 ",
        "十个",
        "10 个",
    ];
    if TEN_HINTS.iter().any(|hint| normalized.contains(hint)) {
        10
    } else {
        5
    }
}

fn numbered(items: &[String], count: usize) -> String {
    items
        .iter()
        .take(count)
        .enumerate()
        .map(|(index, item)| format!("{}. {item}", index + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn fact_records() -> &'static [FactRecord] {
    static CELL: OnceLock<Vec<FactRecord>> = OnceLock::new();
    CELL.get_or_init(seed::facts).as_slice()
}

/// Detect which knowledge-base relation a prompt asks about.
///
/// Issue #386: the relations are no longer a hardcoded per-language keyword
/// table. They are self-describing `fact_relation` meanings in
/// `data/seed/meanings-facts.lino` — each `defined_by` the `knowledge_relation`
/// concept and lexicalised in every supported language. We walk every meaning
/// carrying [`seed::ROLE_FACT_RELATION`] in declaration order and return the
/// first whose surface words appear in the prompt. Declaration order is
/// load-bearing: "написал"/"кто написал" evidence both `author_of_book` and
/// `painter_of_painting`, and `author_of_book` is declared first so it wins —
/// exactly as the former pattern table did. Matching is raw substring (not a
/// whitespace-token test) because several relations carry multi-word phrases
/// ("how many people", "official language", "who wrote", "speed of light", …).
/// The slugs match the JS worker's `FACT_RELATIONS` table so the two stacks
/// emit identical `fact_query:relation:*` evidence and route to the same
/// Wikidata property (e.g. P36 for `capital`).
fn detect_relation(normalized: &str) -> Option<&'static str> {
    seed::lexicon()
        .meanings_with_role(seed::ROLE_FACT_RELATION)
        .find(|meaning| {
            meaning
                .words()
                .any(|word| !word.is_empty() && normalized.contains(word))
        })
        .map(|meaning| meaning.slug.as_str())
}

/// Find the subject alias the prompt mentions, if any. Returns the alias
/// substring as it appears in `subject_aliases`.
fn detect_subject_alias<'a>(record: &'a FactRecord, normalized: &str) -> Option<&'a str> {
    record
        .subject_aliases
        .iter()
        .find(|alias| !alias.is_empty() && normalized.contains(alias.as_str()))
        .map(String::as_str)
}

pub fn try_fact_lookup(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let record = fact_records()
        .iter()
        .find(|record| record.matches_normalized(normalized))?;

    log.append("fact_lookup:request", prompt.to_owned());
    log.append("fact_lookup:hit", record.slug.clone());

    // Structured fact_query trace events (Issue #127). When the matched record
    // declares a `relation`, surface the parsed (relation, subject) tuple so
    // memory consumers can render the structured reasoning trace identically
    // to the browser worker. Records without a `relation` still emit the
    // legacy `fact_lookup:*` events for backward compatibility.
    if !record.relation.is_empty() {
        let parsed_relation = detect_relation(normalized).unwrap_or(record.relation.as_str());
        let parsed_subject =
            detect_subject_alias(record, normalized).unwrap_or(record.subject_label.as_str());
        log.append("fact_query:request", prompt.to_owned());
        log.append("fact_query:relation", parsed_relation.to_owned());
        log.append("fact_query:subject", parsed_subject.to_owned());
        // Treat the seed entry as a pre-warmed cache hit, mirroring the JS
        // worker's `fact_query:cache:hit:seed` event.
        log.append("fact_query:cache:hit", "seed".to_owned());
        if !record.subject_qid.is_empty() {
            log.append("fact_query:subject_qid", record.subject_qid.clone());
        }
        if !record.value_qid.is_empty() {
            log.append("fact_query:value_qid", record.value_qid.clone());
        }
    }

    for qid in &record.wikidata {
        if !qid.is_empty() {
            log.append("wikidata", qid.clone());
        }
    }

    let language = detect_language(prompt).slug();
    let summary = record.summary_for(language);
    let source = record.source_for(language);
    if !source.is_empty() {
        log.append("source", source.to_owned());
    }

    Some(finalize_simple(
        prompt,
        log,
        "fact_lookup",
        "response:fact_lookup",
        summary,
        0.9,
    ))
}

fn coreference_seed_data() -> &'static CoreferenceSeeds {
    static CELL: OnceLock<CoreferenceSeeds> = OnceLock::new();
    CELL.get_or_init(seed::coreference_seeds)
}

pub fn try_coreference_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let seeds = coreference_seed_data();
    if !seeds.matches_pronoun(normalized) {
        return None;
    }

    let previous = last_user_turn(log)?;
    let antecedent = seeds.pick_antecedent(&previous.to_lowercase())?;

    log.append(
        "coreference:resolved",
        format!("it={}", antecedent.display_name),
    );
    if !antecedent.wikidata.is_empty() {
        log.append("wikidata", antecedent.wikidata.clone());
    }
    Some(finalize_simple(
        prompt,
        log,
        &antecedent.intent,
        "response:coreference",
        &antecedent.body,
        0.85,
    ))
}

fn persona_seed_data() -> &'static PersonaSeeds {
    static CELL: OnceLock<PersonaSeeds> = OnceLock::new();
    CELL.get_or_init(seed::persona_seeds)
}

pub fn try_roleplay_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let seeds = persona_seed_data();
    if !seeds.matches_trigger(normalized) {
        return None;
    }

    let persona_display = seeds.pick_persona(normalized).map_or_else(
        || seeds.default_persona.as_str(),
        |persona| {
            if !persona.wikidata.is_empty() {
                log.append("wikidata", persona.wikidata.clone());
            }
            persona.display_name.as_str()
        },
    );
    log.append("roleplay:persona", persona_display.to_owned());

    let topic_body = seeds
        .pick_topic(normalized)
        .map_or(seeds.fallback_body.as_str(), |topic| topic.body.as_str());
    let body = seeds.render_body(persona_display, topic_body);

    Some(finalize_simple(
        prompt,
        log,
        "roleplay_explanation",
        "response:roleplay",
        &body,
        0.8,
    ))
}

#[cfg(test)]
mod tests {
    use super::detect_relation;

    /// Issue #386: `detect_relation` no longer reads a hardcoded keyword table —
    /// it queries the `fact_relation` meanings in `data/seed/meanings-facts.lino`.
    /// This pins that every relation is still recognised from a representative
    /// prompt in each supported language (the words now live in the lexicon), so
    /// the data-driven rewrite cannot silently drop a relation or a language.
    #[test]
    fn every_relation_is_detected_in_every_language() {
        for (slug, prompts) in [
            (
                "capital",
                [
                    "what is the capital of france",
                    "столица россии",
                    "भारत की राजधानी क्या है",
                    "法国的首都是什么",
                ],
            ),
            (
                "population",
                [
                    "what is the population of india",
                    "какое население москвы",
                    "जापान की जनसंख्या कितनी है",
                    "中国的人口是多少",
                ],
            ),
            (
                "currency",
                [
                    "what is the currency of japan",
                    "какая валюта в индии",
                    "ब्राज़ील की मुद्रा क्या है",
                    "美国的货币是什么",
                ],
            ),
            (
                "official_language",
                [
                    "what is the official language of brazil",
                    "государственный язык швейцарии",
                    "स्विट्ज़रलैंड की राजभाषा क्या है",
                    "瑞士的官方语言是什么",
                ],
            ),
            (
                "continent",
                [
                    "which continent is egypt in",
                    "на каком континенте египет",
                    "मिस्र किस महाद्वीप में है",
                    "埃及在哪个大洲",
                ],
            ),
            (
                "author_of_book",
                [
                    "who wrote war and peace",
                    "кто автор войны и мира",
                    "महाभारत के लेखक कौन हैं",
                    "战争与和平的作者是谁",
                ],
            ),
            (
                "painter_of_painting",
                [
                    "who painted the mona lisa",
                    "кто художник этой картины",
                    "इस चित्र का चित्रकार कौन है",
                    "蒙娜丽莎是谁画的",
                ],
            ),
            (
                "built_year",
                [
                    "when was the eiffel tower built",
                    "когда построена эйфелева башня",
                    "ताज महल कब बनी थी",
                    "长城建于何时",
                ],
            ),
            (
                "physical_constant",
                [
                    "what is the speed of light",
                    "чему равна скорость света",
                    "प्रकाश की गति कितनी है",
                    "光速是多少",
                ],
            ),
        ] {
            for prompt in prompts {
                assert_eq!(
                    detect_relation(prompt),
                    Some(slug),
                    "prompt `{prompt}` should resolve to relation `{slug}`",
                );
            }
        }
    }

    /// The bare Russian verb "написал" (and "кто написал") lexicalises *both*
    /// `author_of_book` and `painter_of_painting`. Declaration order in
    /// `meanings-facts.lino` is load-bearing: `author_of_book` is declared first,
    /// so it must win — exactly as the former first-match-wins pattern table did.
    #[test]
    fn ambiguous_napisal_prefers_author_over_painter() {
        assert_eq!(
            detect_relation("кто написал войну и мир"),
            Some("author_of_book")
        );
        assert_eq!(
            detect_relation("кто написал эту картину"),
            Some("author_of_book")
        );
    }

    /// A prompt that mentions no relation surface word resolves to nothing, so
    /// the caller falls back to the matched record's declared relation.
    #[test]
    fn unrelated_prompt_detects_no_relation() {
        assert_eq!(detect_relation("hello there how are you"), None);
        assert_eq!(detect_relation("привет как дела"), None);
    }
}
