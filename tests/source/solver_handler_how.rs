//! "How it works" follow-up handler extracted from `solver_handlers` to keep
//! that module under the 1000-line cap enforced by `scripts/check-file-size.rs`.

use crate::concepts::{extract_concept_query, lookup_concept_query};
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::{detect as detect_language, Language};
use crate::seed;
use crate::solver_handlers::{finalize_simple, try_concept_lookup};
use crate::solver_helpers::last_assistant_turn;
use crate::web_search_core::{WEB_SEARCH_PROVIDERS, WEB_SEARCH_RRF_K};

struct HowItWorksQuery {
    subject: Option<String>,
}

struct ProceduralHowToTask {
    task: String,
    action: String,
    object: String,
    corrections: Vec<SpellingCorrection>,
}

#[derive(Debug, Clone)]
struct SpellingCorrection {
    from: String,
    to: String,
}

/// Handles source-backed procedural requests such as "How to make tea?" and
/// "How can I prepare fried potatoes?". The offline Rust engine records the
/// same discovery plan that the browser worker can execute: local
/// decomposition first, Wikimedia/wikiHow candidates next, then web search
/// and recursive fetch checks only as the fallback path.
pub fn try_how_to_procedure(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let task = extract_procedural_how_to_task(normalized)?;
    for correction in &task.corrections {
        log.append(
            "spelling_correction",
            format!("{}->{}", correction.from, correction.to),
        );
    }
    let query = format!("how to {}", task.task);
    let wikihow_candidate = wikihow_page_title(&task.task);
    let wikihow_api_url = format!(
        "https://www.wikihow.com/api.php?action=parse&page={wikihow_candidate}\
         &prop=text%7Csections%7Cdisplaytitle&format=json&origin=*"
    );

    log.append("procedural_how_to:request", task.task.clone());
    log.append("procedural_how_to:action", task.action.clone());
    if !task.object.is_empty() {
        log.append("procedural_how_to:object", task.object.clone());
    }
    log.append("procedural_how_to:stage", "wikipedia".to_owned());
    log.append("procedural_how_to:stage", "wikidata".to_owned());
    log.append("procedural_how_to:stage", "wikihow_api".to_owned());
    log.append(
        "procedural_how_to:wikihow_candidate",
        wikihow_candidate.clone(),
    );
    log.append("http_fetch:request", wikihow_api_url.clone());
    log.append("procedural_how_to:stage", "web_search".to_owned());
    log.append("web_search:request", query.clone());
    for provider in WEB_SEARCH_PROVIDERS {
        log.append("web_search:provider", (*provider).to_owned());
    }
    log.append("web_search:combined", format!("rrf:k={WEB_SEARCH_RRF_K}"));
    log.append(
        "procedural_how_to:stage",
        "recursive_fetch_check".to_owned(),
    );
    log.append(
        "procedural_how_to:source_gate",
        "explicit_steps_only".to_owned(),
    );

    let provider_summary = WEB_SEARCH_PROVIDERS.join(", ");
    let body = render_procedural_how_to_body(
        &task,
        &wikihow_candidate,
        &wikihow_api_url,
        &query,
        &provider_summary,
        detect_language(prompt),
    );

    Some(finalize_simple(
        prompt,
        log,
        "procedural_how_to",
        "response:procedural_how_to",
        &body,
        0.78,
    ))
}

/// Handles follow-up elaboration prompts such as "how it works?", "how does
/// it work?", or multilingual "how X works?" variants. When the prior
/// assistant turn mentioned a named concept the solver re-runs a concept lookup
/// for that topic. Explicit unknown subjects get a source-backed discovery plan
/// instead of falling through to the unknown fallback.
pub fn try_how_it_works(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let query = extract_how_it_works_query(prompt, normalized)?;
    log.append("followup:how_it_works", normalized.to_owned());

    // When a subject was explicit in the prompt, do a direct concept lookup.
    if let Some(ref term) = query.subject {
        log.append(
            "followup:subject",
            format!("inline:{}", term.to_lowercase()),
        );
        let concept_prompt = format!("what is {term}");
        if let Some(concept_query) = extract_concept_query(&concept_prompt) {
            if lookup_concept_query(&concept_query).is_some() {
                // Delegate to try_concept_lookup by synthesising a standard prompt.
                return try_concept_lookup(&concept_prompt, log);
            }
        }
        record_mechanism_query(log, term);
        let body = render_mechanism_discovery_answer(term, detect_language(prompt));
        return Some(finalize_simple(
            prompt,
            log,
            "how_it_works",
            "response:how_it_works",
            &body,
            0.68,
        ));
    }

    // No inline subject — look for the topic in the prior assistant reply.
    if let Some(prior) = last_assistant_turn(log).map(str::to_owned) {
        log.append("followup:prior_turn", "assistant".to_owned());
        // Extract the first capitalised noun phrase from the prior reply
        // (typically the term in "Term (category): …" format).
        if let Some(term) = extract_topic_from_prior_reply(&prior) {
            use crate::concepts::{extract_concept_query, lookup_concept_query};
            if let Some(query) = extract_concept_query(&format!("what is {term}")) {
                if lookup_concept_query(&query).is_some() {
                    log.append("followup:subject", format!("prior_reply:{term}"));
                    return try_concept_lookup(&format!("what is {term}"), log);
                }
            }
            // Topic is known from history but not in the concept corpus —
            // return a helpful explanation that names the topic.
            let body = format!(
                "To explain how {term} works: I know the term from the prior conversation \
                 but do not have a detailed symbolic rule for it yet. Add a Links Notation \
                 fact with the mechanism description, then ask again."
            );
            log.append("followup:subject", format!("prior_reply_no_record:{term}"));
            return Some(finalize_simple(
                prompt,
                log,
                "concept_elaboration_missing",
                "response:concept_elaboration_missing",
                &body,
                0.3,
            ));
        }
    }

    // No context at all — route to meta_explanation.
    let body = String::from(
        "I answered that way because the prompt matched a deterministic Links Notation rule. \
         To ask about a specific topic, try \"how does X work?\" where X is a concept I know \
         (e.g. \"how does Wikipedia work?\"). The evidence and trace events are appended to \
         the log; see the trace link for the full chain.",
    );
    Some(finalize_simple(
        prompt,
        log,
        "meta_explanation",
        "response:meta_explanation",
        &body,
        0.5,
    ))
}

fn extract_how_it_works_query(prompt: &str, _normalized: &str) -> Option<HowItWorksQuery> {
    let original = clean_mechanism_fragment(prompt);
    if original.is_empty() {
        return None;
    }
    let lower = original.to_lowercase();
    if is_bare_how_it_works(&lower) {
        return Some(HowItWorksQuery { subject: None });
    }
    extract_how_it_works_subject(&original, &lower).map(|subject| HowItWorksQuery {
        subject: Some(subject),
    })
}

/// Is `lower` a bare "how it works" form — a fixed phrase carrying no subject?
///
/// The bare phrases are the [`seed::Slot::Bare`] word forms of the
/// `mechanism_inquiry` meaning (issue #386): no hardcoded per-language list
/// lives here, only the language-independent concept. A prompt is the bare form
/// when it equals such a phrase or is that phrase followed by a space.
fn is_bare_how_it_works(lower: &str) -> bool {
    seed::lexicon()
        .role_word_forms(seed::ROLE_MECHANISM_INQUIRY)
        .into_iter()
        .filter(|form| form.slot() == seed::Slot::Bare)
        .any(|form| {
            let phrase = form.text.as_str();
            lower == phrase
                || lower
                    .strip_prefix(phrase)
                    .is_some_and(|rest| rest.starts_with(' '))
        })
}

/// Extract the explicit subject from multilingual "how X works?" prompts.
/// Returns `None` when the prompt is the bare "how it works?" form.
///
/// The affixes are the slot-marked surface forms of the `mechanism_inquiry`
/// meaning (issue #386): the position of the `…` (U+2026) marker classifies
/// each form, so the matching strategy is derived from the data, not from a
/// hardcoded per-language list. The mechanism order — prefix surfaces, then
/// circumfix, then suffix — and the within-bucket declaration order are
/// preserved so behaviour is identical to the prior inline arrays:
/// * [`seed::Slot::Prefix`] (`how does …`): the literal precedes the subject;
///   `strip_mechanism_tail` then trims a trailing verb, and we return its
///   result even when it is `None` (an early commit, as before).
/// * [`seed::Slot::Circumfix`] (`how … works`): the subject sits between the
///   two literals.
/// * [`seed::Slot::Suffix`] (`… как работает`): the subject precedes the
///   literal. Suffix surfaces are end-anchored and script-disjoint across
///   languages, so their relative order across languages does not change which
///   one matches a given prompt.
fn extract_how_it_works_subject(original: &str, lower: &str) -> Option<String> {
    let forms = seed::lexicon().role_word_forms(seed::ROLE_MECHANISM_INQUIRY);

    for form in forms.iter().filter(|f| f.slot() == seed::Slot::Prefix) {
        if let Some(subject) = subject_after_prefix(original, lower, form.before_slot()) {
            return strip_mechanism_tail(&subject);
        }
    }

    for form in forms.iter().filter(|f| f.slot() == seed::Slot::Circumfix) {
        if let Some(subject) =
            subject_between(original, lower, form.before_slot(), &[form.after_slot()])
        {
            return Some(subject);
        }
    }

    for form in forms.iter().filter(|f| f.slot() == seed::Slot::Suffix) {
        if let Some(subject) = subject_before_suffix(original, lower, form.after_slot()) {
            return Some(subject);
        }
    }

    None
}

fn subject_after_prefix(original: &str, lower: &str, prefix: &str) -> Option<String> {
    lower.strip_prefix(prefix)?;
    let rest = original.get(prefix.len()..)?;
    clean_mechanism_subject(rest)
}

fn subject_before_suffix(original: &str, lower: &str, suffix: &str) -> Option<String> {
    lower.strip_suffix(suffix)?;
    let end = original.len().checked_sub(suffix.len())?;
    clean_mechanism_subject(original.get(..end)?)
}

fn subject_between(original: &str, lower: &str, prefix: &str, suffixes: &[&str]) -> Option<String> {
    if !lower.starts_with(prefix) {
        return None;
    }
    for suffix in suffixes {
        if lower.ends_with(suffix) {
            let end = original.len().checked_sub(suffix.len())?;
            if end <= prefix.len() {
                return None;
            }
            return clean_mechanism_subject(original.get(prefix.len()..end)?);
        }
    }
    None
}

/// Strip a trailing mechanism predicate ("work", "works", "structured", …) so a
/// prefix match such as "how does X work" yields the bare subject "X".
///
/// The predicate tails carry the [`seed::ROLE_MECHANISM_PREDICATE`] role; each
/// is a [`seed::Slot::Suffix`] whose text after the `…` slot is the literal to
/// trim. They are tried in declaration order and the first match wins, mirroring
/// the original return-on-first-match loop. No per-language tail list lives here.
fn strip_mechanism_tail(subject: &str) -> Option<String> {
    let mut clean = clean_mechanism_subject(subject)?;
    let lower = clean.to_lowercase();
    for form in &seed::lexicon().role_word_forms(seed::ROLE_MECHANISM_PREDICATE) {
        let suffix = form.after_slot();
        if lower.ends_with(suffix) {
            let end = clean.len().checked_sub(suffix.len())?;
            clean.truncate(end);
            return clean_mechanism_subject(&clean);
        }
    }
    Some(clean)
}

fn clean_mechanism_fragment(value: &str) -> String {
    value
        .trim()
        .trim_matches(|character: char| {
            matches!(
                character,
                '`' | '"' | '\'' | '«' | '»' | '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}'
            )
        })
        .trim_end_matches(['?', '？', '。', '.', '!', ',', ';', ':'])
        .trim()
        .to_owned()
}

/// Trim optional detail/politeness modifiers from a candidate subject and reject
/// it outright when it is a non-referential subject (a pronoun or bare function
/// word that names no real topic).
///
/// The modifier tails carry the [`seed::ROLE_DETAIL_MODIFIER`] role; each is a
/// [`seed::Slot::Suffix`] stripped in declaration order, re-trimming after each,
/// so several modifiers can fall away in one pass. The rejection set carries the
/// [`seed::ROLE_NON_REFERENTIAL_SUBJECT`] role: [`seed::Slot::Bare`] surfaces
/// match the whole candidate exactly and [`seed::Slot::Prefix`] surfaces match a
/// candidate that begins with the literal before the `…` slot. No per-language
/// modifier or pronoun list lives here.
fn clean_mechanism_subject(value: &str) -> Option<String> {
    let mut clean = clean_mechanism_fragment(value);
    for form in &seed::lexicon().role_word_forms(seed::ROLE_DETAIL_MODIFIER) {
        let suffix = form.after_slot();
        let lower = clean.to_lowercase();
        if lower.ends_with(suffix) {
            let end = clean.len().checked_sub(suffix.len())?;
            clean.truncate(end);
            clean = clean_mechanism_fragment(&clean);
        }
    }
    let lower = clean.to_lowercase();
    let non_referential = seed::lexicon()
        .role_word_forms(seed::ROLE_NON_REFERENTIAL_SUBJECT)
        .iter()
        .any(|form| match form.slot() {
            seed::Slot::Bare => lower == form.text,
            seed::Slot::Prefix => lower.starts_with(form.before_slot()),
            _ => false,
        });
    if clean.is_empty() || non_referential {
        return None;
    }
    Some(clean)
}

fn record_mechanism_query(log: &mut EventLog, subject: &str) {
    log.append("mechanism_query:request", subject.to_owned());
    for stage in ["wikipedia", "wikidata", "web_search"] {
        log.append("mechanism_query:stage", stage.to_owned());
    }
    log.append("web_search:request", format!("how {subject} works"));
    for provider in WEB_SEARCH_PROVIDERS {
        log.append("web_search:provider", (*provider).to_owned());
    }
    log.append("web_search:combined", format!("rrf:k={WEB_SEARCH_RRF_K}"));
    log.append(
        "mechanism_query:source_gate",
        "source_backed_mechanism_only".to_owned(),
    );
}

fn render_mechanism_discovery_answer(subject: &str, language: Language) -> String {
    let provider_summary = WEB_SEARCH_PROVIDERS.join(", ");
    match language {
        Language::Russian => format!(
            "План поиска механизма для `{subject}`.\n\n\
             Я не отвечаю на это из зашитого факта. Решатель трактует запрос \
             как вопрос о том, как устроен или работает `{subject}`: сначала \
             проверяет Wikipedia для обзорного источника, затем Wikidata для \
             связей сущности, затем веб-поиск через {provider_summary}. Если \
             источники не объясняют механизм, ответ должен попросить источник \
             или более узкий термин, а не выдумывать детали."
        ),
        Language::Hindi => format!(
            "`{subject}` के लिए mechanism discovery plan.\n\n\
             I do not answer this from a memoized fact. The solver treats the \
             prompt as a question about how `{subject}` works, checks Wikipedia \
             for a source-backed overview, Wikidata for entity relationships, \
             then web search across {provider_summary}. If no source explains \
             the mechanism, it should ask for a source or a narrower term."
        ),
        Language::Chinese => format!(
            "`{subject}` 的机制发现计划。\n\n\
             I do not answer this from a memoized fact. The solver treats the \
             prompt as a question about how `{subject}` works, checks Wikipedia \
             for a source-backed overview, Wikidata for entity relationships, \
             then web search across {provider_summary}. If no source explains \
             the mechanism, it should ask for a source or a narrower term."
        ),
        Language::English | Language::Unknown => format!(
            "Mechanism discovery plan for `{subject}`.\n\n\
             I do not answer this from a memoized fact. The solver treats the \
             prompt as a question about how `{subject}` works, checks Wikipedia \
             for a source-backed overview, Wikidata for entity relationships, \
             then web search across {provider_summary}. If no source explains \
             the mechanism, it should ask for a source or a narrower term \
             instead of inventing details."
        ),
    }
}

/// Detect a procedural "how to X" request and split it into task/action/object.
///
/// The prefixes are the slot-marked surface forms of the `procedural_request`
/// meaning (issue #386): every form is a [`seed::Slot::Prefix`] whose literal
/// (the text before the `…` marker) is the matchable prefix, in declaration
/// order so "how to do " still precedes "how to ". A form may name the
/// canonical operation in its `action` field (do, perform, implement, create,
/// write); an empty action means the operation is taken from the task's first
/// word. No per-language prefix list lives here — only the concept.
fn extract_procedural_how_to_task(normalized: &str) -> Option<ProceduralHowToTask> {
    let clean_prompt = clean_procedural_fragment(normalized);
    for form in seed::lexicon().role_word_forms(seed::ROLE_PROCEDURAL_REQUEST) {
        if let Some(rest) = clean_prompt.strip_prefix(form.before_slot()) {
            let action_override = (!form.action.is_empty()).then_some(form.action.as_str());
            return build_procedural_task(rest, action_override);
        }
    }
    None
}

fn build_procedural_task(
    raw_task: &str,
    action_override: Option<&str>,
) -> Option<ProceduralHowToTask> {
    let task = clean_procedural_fragment(raw_task);
    if task.is_empty() {
        return None;
    }
    let (task, corrections) = correct_common_procedural_typos(&task);

    let (action, object) = if let Some(action) = action_override {
        (action.to_owned(), task.clone())
    } else {
        let mut parts = task.splitn(2, char::is_whitespace);
        let action = parts.next()?.trim();
        if action.is_empty() {
            return None;
        }
        let object = parts.next().unwrap_or("").trim();
        (action.to_owned(), object.to_owned())
    };

    Some(ProceduralHowToTask {
        task,
        action,
        object,
        corrections,
    })
}

fn clean_procedural_fragment(value: &str) -> String {
    let mut clean = value
        .trim()
        .trim_matches(|character: char| matches!(character, '`' | '"' | '\'' | ' '))
        .trim_end_matches(['?', '!', '.', ',', ';', ':'])
        .trim()
        .to_owned();

    // The trailing "step by step" / politeness modifiers are the slot-marked
    // surface forms of the `procedural_task_modifier` meaning
    // (data/seed/meanings-how.lino): every form is a suffix whose text after the
    // … marker (`after_slot`) is the tail to strip, scanned in declaration order
    // so the longer Russian "напиши по шагам" still precedes its "по шагам" tail.
    // No per-language modifier list lives here — only the concept.
    for form in &seed::lexicon().role_word_forms(seed::ROLE_PROCEDURAL_TASK_MODIFIER) {
        if let Some(stripped) = clean.strip_suffix(form.after_slot()) {
            clean = stripped.trim().to_owned();
            break;
        }
    }
    clean
}

fn correct_common_procedural_typos(task: &str) -> (String, Vec<SpellingCorrection>) {
    // The misspelling -> correction pairs are the `common_typo` meaning's bare
    // surface forms (data/seed/meanings-how.lino): each form's text is the
    // misspelled token and its `action` child names the correct spelling. No
    // per-language typo table lives here — only the concept.
    let lexicon = seed::lexicon();
    let typos = lexicon.role_word_forms(seed::ROLE_COMMON_TYPO);

    let mut corrections: Vec<SpellingCorrection> = Vec::new();
    let corrected = task
        .split_whitespace()
        .map(|token| {
            for form in &typos {
                if token == form.text {
                    if !corrections.iter().any(|seen| seen.from == form.text) {
                        corrections.push(SpellingCorrection {
                            from: form.text.clone(),
                            to: form.action.clone(),
                        });
                    }
                    return form.action.clone();
                }
            }
            token.to_owned()
        })
        .collect::<Vec<_>>()
        .join(" ");

    (corrected, corrections)
}

fn wikihow_page_title(task: &str) -> String {
    task.split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(capitalize_word)
        .collect::<Vec<_>>()
        .join("-")
}

fn render_procedural_how_to_body(
    task: &ProceduralHowToTask,
    wikihow_candidate: &str,
    wikihow_api_url: &str,
    query: &str,
    provider_summary: &str,
    language: Language,
) -> String {
    if language == Language::Russian {
        return format!(
            "План поиска процедуры для `{}` (действие `{}`, объект `{}`).\n\n\
             Я не отвечаю на это как на заученный рецепт. Сначала solver проверяет \
             Wikipedia для контекста темы и Wikidata для подсказок сущности, действия \
             и объекта. Затем он пробует CORS-readable MediaWiki parse API wikiHow \
             для кандидата `{}` через `{}`. Если эти источники не дают пригодные шаги, \
             fallback запускает web search по `{}` через {} и объединяет верхние \
             результаты reciprocal rank fusion (k = {}). Финальная recursive fetch \
             check принимает только страницы с явными упорядоченными или \
             инструкционными шагами для `{}`.",
            task.task,
            task.action,
            task.object,
            wikihow_candidate,
            wikihow_api_url,
            query,
            provider_summary,
            WEB_SEARCH_RRF_K,
            task.task,
        );
    }

    format!(
        "Procedural discovery plan for `{}` (action `{}`, object `{}`).\n\n\
         I do not answer this from a memoized recipe. The solver first checks \
         Wikipedia for topic context and Wikidata for entity/action/object hints. \
         It then tries wikiHow's CORS-readable MediaWiki parse API candidate \
         `{}` via `{}`. If those sources do not expose usable steps, the fallback \
         path runs web search for `{}` across {} and merges the top results with \
         reciprocal rank fusion (k = {}). The final recursive fetch check only \
         accepts pages that actually contain explicit ordered or instructional \
         steps for `{}`.",
        task.task,
        task.action,
        task.object,
        wikihow_candidate,
        wikihow_api_url,
        query,
        provider_summary,
        WEB_SEARCH_RRF_K,
        task.task,
    )
}

fn capitalize_word(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    format!("{}{rest}", first.to_uppercase(), rest = chars.as_str())
}

/// Extract the first meaningful topic word/phrase from a prior assistant reply.
/// Looks for "Term (category):" patterns first, then the first capitalised token.
fn extract_topic_from_prior_reply(reply: &str) -> Option<String> {
    // Match "Term (category): description" — common in concept_lookup answers.
    let first_line = reply.lines().next().unwrap_or("").trim();
    if let Some(paren_pos) = first_line.find('(') {
        let candidate = first_line[..paren_pos].trim();
        if !candidate.is_empty() {
            return Some(candidate.to_lowercase());
        }
    }
    // Fallback: first capitalised token that is not a function word. The skip
    // list is the `topic_scan_stop_word` meaning's bare surface forms
    // (data/seed/meanings-how.lino): closed-class words (articles, prepositions,
    // conjunctions, pronouns) and the 'source' citation heading that name no
    // subject, compared case-insensitively. No per-language word list lives here
    // — only the concept "a function word that names no topic".
    let stop_words = seed::lexicon().role_word_forms(seed::ROLE_TOPIC_SCAN_STOP_WORD);
    for word in reply.split_whitespace() {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
        if clean.len() >= 2 && clean.chars().next().is_some_and(char::is_uppercase) {
            let lowered = clean.to_lowercase();
            if !stop_words.iter().any(|form| form.text == lowered) {
                return Some(lowered);
            }
        }
    }
    None
}
