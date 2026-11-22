use std::sync::OnceLock;

use super::finalize_simple;
use super::summarization_request::derived_topic_summary;

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::fact_live::{live_fact_question, record_fact_question};
use crate::language::detect as detect_language;
use crate::seed::{
    self, Antecedent, BrainstormSeeds, CoreferenceSeeds, FactRecord, Meaning, PersonaSeeds,
    Pronoun, SummaryTopicSeeds,
};
use crate::world_model_dialog::resolve_coreference_antecedent;

fn summary_topic_seed_data() -> &'static SummaryTopicSeeds {
    static CELL: OnceLock<SummaryTopicSeeds> = OnceLock::new();
    CELL.get_or_init(seed::summary_topic_seeds)
}

pub fn try_summarization_request(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_summarization_request_with_offline(prompt, log, true)
}

pub(crate) fn try_summarization_request_with_offline(
    prompt: &str,
    log: &mut EventLog,
    offline: bool,
) -> Option<SymbolicAnswer> {
    let seeds = summary_topic_seed_data();
    let lower = prompt.to_lowercase();
    if !seeds.matches_trigger(&lower) {
        return None;
    }

    let language = detect_language(prompt).slug();
    let derived = seeds
        .subject(&lower)
        .and_then(|subject| derived_topic_summary(subject, language, log, offline));
    if derived.is_none() {
        log.append("summarization:refusal", "unresolved_subject".to_owned());
    }
    let (topic, body, source) = derived.unwrap_or_else(|| {
        let label = prompt
            .trim_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
            .to_owned();
        let fallback = if seeds.fallback_response.is_empty() {
            "unknown"
        } else {
            seeds.fallback_response.as_str()
        };
        let body = seed::localized_response(fallback, language)
            .unwrap_or_else(|| crate::engine::unknown_answer().to_owned());
        (label, body, String::from("none"))
    });

    log.append("summarization:topic", topic);
    log.append("summarization:source", source);
    if let Some(label) = seeds.constraint_for(&lower) {
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
    let language = detect_language(prompt).slug();
    let topic = seeds
        .topic(normalized)
        .filter(|topic| !super::creative_composition::topic_words(topic, language).is_empty());
    let (body, confidence) = match topic {
        Some(topic) if category.operations.is_empty() => {
            log.append("brainstorm:category", category.slug.clone());
            return Some(super::creative_composition::compose_brainstorm_names(
                prompt,
                normalized,
                topic,
                &category.intent,
                log,
            ));
        }
        Some(topic) => {
            log.append("brainstorming:topic", topic.to_owned());
            let count = super::creative_composition::requested_count(normalized, language)
                .map_or_else(
                    || requested_brainstorm_count(seeds, normalized),
                    |count| count as usize,
                );
            (numbered(&category.operations, topic, count), 0.8)
        }
        None => {
            log.append("brainstorming:refusal", "missing_topic".to_owned());
            (
                seed::localized_response("brainstorming_refusal", language).unwrap_or_default(),
                0.4,
            )
        }
    };
    log.append("brainstorm:category", category.slug.clone());
    Some(finalize_simple(
        prompt,
        log,
        &category.intent,
        "response:brainstorm",
        &body,
        confidence,
    ))
}

/// The number of items the user asked for: the value of the seed's
/// `count_cardinal` meaning when the prompt evidences it (in any supported
/// language), else the seed's `default_count`.
///
/// The cardinal's integer value is read from its own numeral surface rather
/// than a literal, so the count stays language-independent and the policy
/// (which cardinal, which default) is the seed's, not this module's (issue
/// #918).
fn requested_brainstorm_count(seeds: &BrainstormSeeds, normalized: &str) -> usize {
    seed::lexicon()
        .meaning(&seeds.count_cardinal)
        .filter(|cardinal| cardinal.evidenced_in(normalized))
        .and_then(cardinal_value)
        .unwrap_or(seeds.default_count)
}

/// Read the integer value of a cardinal-number meaning from its own data.
///
/// Each cardinal carries a numeral word form (e.g. `"10"`) — the
/// script-independent surface that spells the value — so the count is derived
/// from the seed rather than restated as a literal in code.
fn cardinal_value(meaning: &Meaning) -> Option<usize> {
    meaning
        .words()
        .find(|word| !word.is_empty() && word.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|numeral| numeral.parse().ok())
}

fn numbered(operations: &[String], topic: &str, count: usize) -> String {
    operations
        .iter()
        .take(count)
        .enumerate()
        .map(|(index, item)| format!("{}. {}", index + 1, item.replace("{topic}", topic)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether the committed fact store has a record for this prompt.
///
/// The planner's open-web gate asks whether the symbolic engine can answer a
/// request before releasing it to the open web (issue #989). Concepts and
/// computations already answer there; a question the fact store's subject
/// gate admits -- the formalized subject's Q-id equals a record's
/// `subject_qid` (issue #1172 R2), with word-boundary alias matching only as
/// the last-resort hint for Q-id-less records, so an unseeded subject such as
/// Australia correctly falls through to `false`, exactly the dispatch the
/// solver's `fact_lookup` row runs -- is equally answerable, and planning a
/// web search for it would discard the answer the engine already owns
/// (issue #1138).
#[must_use]
pub fn fact_store_resolves(prompt: &str) -> bool {
    let normalized = crate::engine::normalize_prompt(prompt);
    super::factual_qa::gated_fact_record(prompt, &normalized).is_some()
        || super::project_origin::names_project_origin(prompt)
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
/// Both runtimes read the same grounded meanings, so they
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
///
/// Issue #1172: the alias must match at word boundaries via
/// [`FactRecord::contains_word_sequence`] — the same comparison
/// [`FactRecord::matches_normalized`] gates on — so the `fact_query:subject`
/// evidence reports an alias that actually appears in the prompt as a whole
/// word (e.g. "usa" for "capital of usa"), never the substring coincidence
/// that used to fire (alias "us" inside "australia").
fn detect_subject_alias<'a>(record: &'a FactRecord, normalized: &str) -> Option<&'a str> {
    record
        .subject_aliases
        .iter()
        .find(|alias| FactRecord::contains_word_sequence(normalized, alias))
        .map(String::as_str)
}

/// The `fact_lookup` row: a question over prompt-supplied text (issue #1172
/// R7) and a comparison of seeded subjects (R6) first, then the seeded record
/// the subject gate admits (R2).
pub fn try_fact_lookup(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if let Some(answer) = super::prompt_text_question::try_prompt_text_question(prompt, log) {
        return Some(answer);
    }
    let (record, gate) = super::factual_qa::gated_fact_record(prompt, normalized)?;
    if !record.publisher_heading.is_empty() || !record.release_timeline.is_empty() {
        return None;
    }

    log.append("fact_lookup:request", prompt.to_owned());
    log.append("fact_lookup:hit", record.slug.clone());
    log.append("fact_lookup:subject_gate", gate);

    // Structured fact_query trace events (Issue #127). When the matched record
    // declares a `relation`, surface the parsed (relation, subject) tuple so
    // memory consumers can render the structured reasoning trace identically
    // to the browser worker. Records without a `relation` still emit the
    // legacy `fact_lookup:*` events for backward compatibility.
    if !record.relation.is_empty() {
        let parsed_relation = detect_relation(normalized).unwrap_or(record.relation.as_str());
        let parsed_subject =
            detect_subject_alias(record, normalized).unwrap_or(record.subject_label.as_str());
        record_fact_question(log, prompt, parsed_relation, parsed_subject);
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
    let source = record.source_for(language);
    if !source.is_empty() {
        log.append("source", source.to_owned());
    }

    Some(finalize_simple(
        prompt,
        log,
        "fact_lookup",
        "response:fact_lookup",
        record.summary_for(language),
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
    try_coreference_with(prompt, normalized, log, |_, _, _| None)
}

/// Resolve a conversation reference against a caller-supplied fact source.
pub fn try_coreference_with_client<T: crate::source_fetch::SourceTransport>(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    client: &crate::source_fetch::CachedSourceClient<T>,
) -> Option<SymbolicAnswer> {
    try_coreference_with(prompt, normalized, log, |original, rewritten, log| {
        super::fact_live_answer::try_rewritten_fact_answer(original, rewritten, log, client)
    })
}

fn try_coreference_with(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    resolve_fact: impl FnOnce(&str, &str, &mut EventLog) -> Option<SymbolicAnswer>,
) -> Option<SymbolicAnswer> {
    let seeds = coreference_seed_data();
    let pronoun = seeds.matching_pronoun(&crate::engine::normalize_prompt(
        super::text_rewrite::command_head(prompt),
    ))?;

    let (antecedent, resolution) = resolve_coreference_antecedent(seeds, log)?;
    log.append("context_resolution", resolution.links_notation());

    log.append(
        "coreference:resolved",
        format!("{}={}", pronoun.token, antecedent.display_name),
    );
    if !antecedent.wikidata.is_empty() {
        log.append("wikidata", antecedent.wikidata.clone());
    }

    if let Some(rewritten) = rewrite_coreference_prompt(normalized, antecedent, pronoun) {
        log.append("coreference:rewrite", rewritten.clone());
        if let Some(answer) = try_fact_lookup(prompt, &rewritten, log) {
            return Some(answer);
        }
        if let Some(question) = live_fact_question(&rewritten)
            && question.relation != "physical_constant"
        {
            return resolve_fact(prompt, &rewritten, log);
        }
    }

    let resolved_reference;
    let body = if antecedent.body.is_empty() {
        resolved_reference = seed::localized_response(
            "coreference_resolved_reference",
            detect_language(prompt).slug(),
        )?
        .replace("{referent}", &antecedent.display_name);
        resolved_reference.as_str()
    } else {
        &antecedent.body
    };
    Some(finalize_simple(
        prompt,
        log,
        &antecedent.intent,
        "response:coreference",
        body,
        0.85,
    ))
}

/// Whether an earlier user turn names an antecedent a pronoun resolves to.
///
/// The `coreference_antecedent` claim evidence of issue #1175 R3.
#[must_use]
pub fn names_coreference_antecedent(log: &EventLog) -> bool {
    resolve_coreference_antecedent(coreference_seed_data(), log).is_some()
}

/// A seeded command-head reference with a real earlier user antecedent.
#[must_use]
pub fn resolves_coreference_request(prompt: &str, log: &EventLog) -> bool {
    let head = crate::engine::normalize_prompt(super::text_rewrite::command_head(prompt));
    coreference_seed_data().matching_pronoun(&head).is_some() && names_coreference_antecedent(log)
}

fn rewrite_coreference_prompt(
    normalized: &str,
    antecedent: &Antecedent,
    pronoun: &Pronoun,
) -> Option<String> {
    let replacement = antecedent
        .aliases
        .first()
        .filter(|alias| !alias.is_empty())
        .map_or_else(|| antecedent.display_name.to_lowercase(), Clone::clone);

    replace_token(normalized, pronoun.token.trim(), &replacement)
}

fn replace_token(text: &str, token: &str, replacement: &str) -> Option<String> {
    if token.is_empty() || replacement.is_empty() {
        return None;
    }

    let mut out = String::with_capacity(text.len() + replacement.len());
    let mut cursor = 0;
    let mut changed = false;

    while let Some(offset) = text[cursor..].find(token) {
        let start = cursor + offset;
        let end = start + token.len();
        let previous = text[..start].chars().next_back();
        let next = text[end..].chars().next();

        if token_boundary(previous) && token_boundary(next) {
            out.push_str(&text[cursor..start]);
            out.push_str(replacement);
            changed = true;
        } else {
            out.push_str(&text[cursor..end]);
        }
        cursor = end;
    }

    if !changed {
        return None;
    }
    out.push_str(&text[cursor..]);
    Some(out)
}

fn token_boundary(ch: Option<char>) -> bool {
    ch.is_none_or(|ch| !ch.is_alphanumeric() && ch != '_')
}

fn persona_seed_data() -> &'static PersonaSeeds {
    static CELL: OnceLock<PersonaSeeds> = OnceLock::new();
    CELL.get_or_init(seed::persona_seeds)
}

fn roleplay_facts(request: &str) -> Option<(&str, &str)> {
    let (role, facts) = request.split_once([':', '：'])?;
    let (role, facts) = (role.trim(), facts.trim());
    (!role.is_empty() && !facts.is_empty()).then_some((role, facts))
}

pub(crate) fn roleplay_has_facts(prompt: &str) -> bool {
    persona_seed_data()
        .request(prompt)
        .and_then(roleplay_facts)
        .is_some()
}

pub fn try_roleplay_request(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let seeds = persona_seed_data();
    let request = seeds.request(prompt)?;
    let language = detect_language(prompt).slug();
    let (body, confidence) = if let Some((role, facts)) = roleplay_facts(request) {
        let display_name = seeds.pick_persona(role).map_or(role, |persona| {
            if !persona.wikidata.is_empty() {
                log.append("wikidata", persona.wikidata.clone());
            }
            &persona.display_name
        });
        log.append("roleplay:persona", display_name.to_owned());
        log.append("roleplay:supplied_facts", String::new());
        let template = seed::localized_response("roleplay_supplied_facts", language)?;
        let (prefix, suffix) = template.split_once("{role}")?;
        (
            format!("{prefix}{display_name}{}", suffix.replace("{facts}", facts)),
            0.8,
        )
    } else {
        log.append("roleplay:refusal", "missing role or facts".to_owned());
        (
            seed::localized_response("roleplay_facts_needed", language)?,
            0.5,
        )
    };

    Some(finalize_simple(
        prompt,
        log,
        "roleplay_explanation",
        "response:roleplay",
        &body,
        confidence,
    ))
}
