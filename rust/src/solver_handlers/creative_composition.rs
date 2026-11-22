//! Creative-composition handlers, part one (issue #1178, E143):
//! brainstorming and advice.
//!
//! **Brainstorming** (`handle_brainstorm_request`) composes name candidates
//! from the formalized topic of the request itself: topic words are
//! extracted (every token minus the stop-word, cue, and number vocabulary),
//! resolved against the meaning lexicon where a meaning exists, and combined
//! through the morphology rules in `data/seed/creative-composition-rules.lino`
//! (compound, blend, suffix, definite). Every candidate then passes the
//! request's explicit constraints (length, pronounceability, named
//! exclusions) and the distinctness floor before it is returned, ranked by
//! the stated pairwise normalized Levenshtein metric.
//!
//! Advice ranks caller-supplied recommendations by the declared evidence grade.
//! Shared vocabulary and composition rules live in the creative seed files.

use serde_json::Value;
use std::sync::OnceLock;

use super::finalize_simple;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::seed::parser::{LinoNode, parse_lino};

const RULES_PATH: &str = "data/seed/creative-composition-rules.lino";
/// The root link every record of the rules seed sits under.
const RULES_ROOT: &str = "creative_composition_rules";
const ROLE_BRAINSTORMING: &str = "brainstorming_request";
const ROLE_ADVICE: &str = "advice_request";
const ROLE_CREATIVE_WRITING: &str = "creative_writing_request";
const ROLE_PLANNING: &str = "planning_request";
/// The four request classes whose cue meanings name the request, not the topic.
const COMPOSITION_ROLES: [&str; 4] = [
    ROLE_BRAINSTORMING,
    ROLE_ADVICE,
    ROLE_CREATIVE_WRITING,
    ROLE_PLANNING,
];
/// Name-length cap when no request cue states a shorter one.
const DEFAULT_MAX_LENGTH: usize = 18;

/// Look up embedded seed content by its registered path.
fn seed_text(path: &str) -> Option<&'static str> {
    crate::seed::seed_files()
        .into_iter()
        .find(|(registered, _)| *registered == path)
        .map(|(_, text)| text)
}

/// The records under the rules seed's `creative_composition_rules` root
/// (the top-level links when the file carries no such root).
fn rules_records() -> &'static [LinoNode] {
    static RECORDS: OnceLock<Vec<LinoNode>> = OnceLock::new();
    RECORDS.get_or_init(|| {
        let Some(text) = seed_text(RULES_PATH) else {
            return Vec::new();
        };
        let mut tree = parse_lino(text);
        if let Some(index) = tree
            .children
            .iter()
            .position(|child| child.name == RULES_ROOT)
        {
            return tree.children.swap_remove(index).children;
        }
        tree.children
    })
}

/// Every rules record named `name`, in seed order.
pub fn rules_records_named(name: &str) -> impl Iterator<Item = &'static LinoNode> {
    rules_records()
        .iter()
        .filter(move |record| record.name == name)
}

/// The non-empty values of a record's children named `name`.
pub fn child_values(record: &LinoNode, name: &str) -> Vec<String> {
    record
        .children
        .iter()
        .filter(|child| child.name == name)
        .map(|child| child.id.clone())
        .filter(|value| !value.is_empty())
        .collect()
}

/// The `request_cue` record named `cue`.
fn cue_record(cue: &str) -> Option<&'static LinoNode> {
    rules_records_named("request_cue").find(|record| record.find_child_value("cue") == cue)
}

/// The words of the `request_cue` record named `cue`.
pub fn cue_words(cue: &str) -> Vec<String> {
    cue_record(cue)
        .map(|record| child_values(record, "word"))
        .unwrap_or_default()
}

/// The value of child `name` on the `request_cue` record named `cue`.
pub fn cue_field(cue: &str, name: &str) -> String {
    cue_record(cue)
        .map(|record| record.find_child_value(name).to_owned())
        .unwrap_or_default()
}

/// Fill a localized response template's `{placeholder}` slots.
fn template(intent: &str, values: &[(&str, &str)]) -> String {
    let mut out = crate::seed::localized_response(intent, "en").unwrap_or_default();
    for (key, value) in values {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

/// One `topic_stop_word` list per language: words a topic extraction drops.
fn stop_words(language: &str) -> Vec<String> {
    rules_records_named("topic_stop_word")
        .filter(|record| record.find_child_value("language") == language)
        .flat_map(|record| child_values(record, "word"))
        .collect()
}

/// The `(word, value)` pairs of one language's `spelled_number` record. The
/// seed writes `number <word> <value>` (the value inline); a `value` child
/// is read too.
fn spelled_numbers(language: &str) -> Vec<(String, u32)> {
    let mut out = Vec::new();
    for record in rules_records_named("spelled_number")
        .filter(|record| record.find_child_value("language") == language)
    {
        for pair in record
            .children
            .iter()
            .filter(|child| child.name == "number")
        {
            let mut parts = pair.id.split_whitespace();
            let word = parts.next().unwrap_or_default();
            let inline = parts.next().unwrap_or_default();
            let child = pair.find_child_value("value");
            let raw = if child.is_empty() { inline } else { child };
            let value = raw.parse::<u32>().unwrap_or(0);
            if !word.is_empty() && value > 0 {
                out.push((word.to_owned(), value));
            }
        }
    }
    out
}

/// The CJK runs of `normalized` left once the request vocabulary is
/// removed: the cue surfaces of the four composition roles, plus `removed`
/// (the language's stop words and numerals), longest first.
fn cjk_remainder(normalized: &str, removed: &[String]) -> Vec<String> {
    let lexicon = crate::seed::lexicon();
    let mut vocabulary: Vec<&str> = removed.iter().map(String::as_str).collect();
    for role in COMPOSITION_ROLES {
        for meaning in lexicon.meanings_with_role(role) {
            vocabulary.extend(meaning.words());
        }
    }
    vocabulary.sort_by_key(|word| std::cmp::Reverse(word.chars().count()));
    let mut text = normalized.to_owned();
    for word in vocabulary {
        if !word.is_empty() && crate::coding::contains_cjk(word) {
            text = text.replace(word, " ");
        }
    }
    text.split_whitespace()
        .filter(|run| crate::coding::contains_cjk(run))
        .map(str::to_owned)
        .collect()
}

/// Extract the topic words of a composition request.
///
/// Space-delimited scripts keep every alphabetic token that is neither a
/// stop word nor a spelled number. CJK scripts have no token boundaries, so
/// the topic words there are the meaning-lexicon surfaces present in the
/// prompt, minus the same stop words and numerals, and when the lexicon
/// names none, the runs the request vocabulary leaves behind.
pub fn topic_words(normalized: &str, language: &str) -> Vec<String> {
    let stops = stop_words(language);
    let numbers: Vec<String> = spelled_numbers(language)
        .into_iter()
        .map(|(word, _)| word)
        .collect();
    let skipped = |word: &str| {
        stops.iter().any(|stop| stop == word) || numbers.iter().any(|number| number == word)
    };
    let mut words: Vec<String> = Vec::new();
    if crate::coding::contains_cjk(normalized) {
        for meaning in &crate::seed::lexicon().meanings {
            // Cue meanings name the request classes, not the topic.
            if COMPOSITION_ROLES.iter().any(|role| meaning.has_role(role)) {
                continue;
            }
            if let Some(found) = meaning
                .words()
                .find(|word| !word.is_empty() && !skipped(word) && normalized.contains(*word))
            {
                words.push(found.to_owned());
            }
        }
        if words.is_empty() {
            let mut removed = stops.clone();
            removed.extend(numbers.iter().cloned());
            words.extend(cjk_remainder(normalized, &removed));
        }
    } else {
        for token in normalized.split_whitespace() {
            let token = token.trim_matches(|c: char| !c.is_alphanumeric());
            if token.chars().count() < 2 || !token.chars().any(char::is_alphabetic) {
                continue;
            }
            if skipped(token) {
                continue;
            }
            words.push(token.to_owned());
        }
    }
    words.sort();
    words.dedup();
    words
}

/// The count a request states: a spelled number (CJK numerals by substring
/// once the language's stop words are removed, the longest numeral
/// winning; other scripts as whole tokens), else the first standalone
/// decimal number when it lies in 1..=12.
pub fn requested_count(normalized: &str, language: &str) -> Option<u32> {
    let mut stripped = normalized.to_owned();
    for stop in stop_words(language) {
        if crate::coding::contains_cjk(&stop) {
            stripped = stripped.replace(stop.as_str(), " ");
        }
    }
    let mut best: Option<(String, u32)> = None;
    for (word, value) in spelled_numbers(language) {
        // CJK scripts carry no inter-word spaces, so their numeral words
        // match by substring; space-delimited scripts match whole tokens.
        if crate::coding::contains_cjk(&word) {
            let longer = best
                .as_ref()
                .is_none_or(|(held, _)| word.chars().count() > held.chars().count());
            if longer && stripped.contains(word.as_str()) {
                best = Some((word, value));
            }
        } else if normalized.contains(&format!(" {word} "))
            || normalized.starts_with(&format!("{word} "))
        {
            return Some(value);
        }
    }
    if let Some((_, value)) = best {
        return Some(value);
    }
    normalized
        .split_whitespace()
        .find_map(|token| token.parse::<u32>().ok())
        .filter(|value| *value > 0 && *value <= 12)
}

/// One `brainstorm_rule` record: a composition operation on topic words.
struct BrainstormRule {
    kind: String,
    pieces: Vec<String>,
}

fn brainstorm_rules() -> Vec<BrainstormRule> {
    rules_records_named("brainstorm_rule")
        .map(|record| BrainstormRule {
            kind: record.find_child_value("kind").to_string(),
            pieces: record
                .children
                .iter()
                .filter(|child| child.name == "piece")
                .map(|child| child.id.clone())
                .filter(|piece| !piece.is_empty())
                .collect(),
        })
        .filter(|rule| !rule.kind.is_empty())
        .collect()
}

/// The stated distinctness metric of the rules seed.
struct DistinctnessMetric {
    floor: f32,
    statement: String,
    #[allow(dead_code)] // seed provenance, carried but not yet rendered
    share_rule: String,
}

fn distinctness_metric() -> DistinctnessMetric {
    let default = DistinctnessMetric {
        floor: 0.5,
        statement: "pairwise normalized levenshtein ratio".to_owned(),
        share_rule: String::new(),
    };
    let Some(record) = rules_records_named("distinctness_metric").next() else {
        return default;
    };
    DistinctnessMetric {
        floor: record.find_child_value("floor").parse().unwrap_or(0.5),
        statement: record.find_child_value("statement").to_string(),
        share_rule: record.find_child_value("share_rule").to_string(),
    }
}

/// Classic Levenshtein distance over characters.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut current = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            current.push(
                (previous[j] + cost)
                    .min(previous[j + 1] + 1)
                    .min(current[j] + 1),
            );
        }
        previous = current;
    }
    previous[b.len()]
}

/// True when the two strings are distinct enough for the metric: the
/// distance is at least `floor` times the longer length.
fn distinct_enough(a: &str, b: &str, floor: f32) -> bool {
    let longer = count_as_f32(a.chars().count().max(b.chars().count()).max(1));
    let distance = count_as_f32(levenshtein(a, b));
    distance / longer >= floor
}

/// A character count as `f32`, rounded to nearest like an `as` cast: the
/// high and low 16-bit halves convert exactly and the fused multiply-add
/// rounds once. Counts beyond `u32::MAX` saturate.
fn count_as_f32(count: usize) -> f32 {
    let count = u32::try_from(count).unwrap_or(u32::MAX);
    let high = f32::from(u16::try_from(count >> 16).unwrap_or(u16::MAX));
    let low = f32::from(u16::try_from(count & 0xFFFF).unwrap_or(u16::MAX));
    high.mul_add(65536.0, low)
}

/// True when a candidate is pronounceable: it carries a vowel (Latin or
/// Cyrillic; Devanagari and CJK syllabaries carry their own vowels and are
/// exempt) and no digits or separators.
fn pronounceable(candidate: &str) -> bool {
    if candidate.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }
    if crate::coding::contains_cjk(candidate)
        || candidate
            .chars()
            .any(|c| ('\u{0900}'..='\u{097F}').contains(&c))
    {
        return true;
    }
    candidate
        .chars()
        .any(|c| "aeiouyAEIOUYаеёиоуыэюяАЕЁИОУЫЭЮЯ".contains(c))
}

/// The words named after the `exclusion` request cue in the prompt:
/// candidates containing one are excluded before ranking.
fn exclusions(normalized: &str) -> Vec<String> {
    let markers = cue_words("exclusion");
    normalized
        .split_whitespace()
        .skip_while(|token| !markers.iter().any(|marker| marker.as_str() == *token))
        .skip(1)
        .map(|token| {
            token
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|token| token.chars().count() >= 2)
        .collect()
}

/// Capitalize the first character of a composed piece.
fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + chars.as_str()
    })
}

/// Compose the raw candidate pool from the topic words: every rule applied
/// to every ordered pair (and single words for the suffix rule).
fn compose_candidates(topics: &[String], rules: &[BrainstormRule]) -> Vec<String> {
    let mut out = Vec::new();
    for rule in rules {
        match rule.kind.as_str() {
            "compound" => {
                for a in topics {
                    for b in topics {
                        if a != b {
                            out.push(format!("{}{}", capitalize(a), capitalize(b)));
                        }
                    }
                }
            }
            "blend" => {
                for a in topics {
                    for b in topics {
                        if a == b {
                            continue;
                        }
                        let a_chars: Vec<char> = a.chars().collect();
                        let b_chars: Vec<char> = b.chars().collect();
                        let front = a_chars.len().div_ceil(2);
                        let back_start = b_chars.len() / 2;
                        if front >= 2 && b_chars.len() - back_start >= 2 {
                            let blend: String = a_chars[..front]
                                .iter()
                                .chain(&b_chars[back_start..])
                                .collect();
                            out.push(capitalize(&blend));
                        }
                    }
                }
            }
            "suffix" => {
                for a in topics {
                    for piece in &rule.pieces {
                        out.push(format!("{}{}", capitalize(a), piece));
                    }
                }
            }
            "definite" => {
                for a in topics {
                    for b in topics {
                        if a != b {
                            out.push(format!("The {} {}", capitalize(a), capitalize(b)));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// The content words of a candidate: which of the topic words occur in it.
fn content_words(candidate: &str, topics: &[String]) -> Vec<String> {
    let lower = candidate.to_lowercase();
    topics
        .iter()
        .map(|topic| topic.to_lowercase())
        .filter(|topic| lower.contains(topic.as_str()))
        .collect()
}

/// Filter the candidate pool by the explicit constraints, then enforce the
/// distinctness metric (pairwise ratio floor and the content-share rule).
fn select_candidates(
    pool: Vec<String>,
    topics: &[String],
    count: usize,
    max_length: usize,
    excluded: &[String],
    metric: &DistinctnessMetric,
) -> Vec<String> {
    let mut kept: Vec<String> = Vec::new();
    for candidate in pool {
        if kept.len() >= count {
            break;
        }
        if candidate.chars().count() > max_length {
            continue;
        }
        if !pronounceable(&candidate) {
            continue;
        }
        if excluded
            .iter()
            .any(|word| candidate.to_lowercase().contains(word.as_str()))
        {
            continue;
        }
        // The distinctness floor: keep the first of every too-similar pair.
        if kept
            .iter()
            .any(|other| !distinct_enough(&candidate, other, metric.floor))
        {
            continue;
        }
        // The content-share rule: no two candidates built from both of the
        // same topic words.
        let words = content_words(&candidate, topics);
        if words.len() >= 2 {
            let clash = kept.iter().any(|other| {
                let other_words = content_words(other, topics);
                other_words.len() >= 2 && words.iter().all(|word| other_words.contains(word))
            });
            if clash {
                continue;
            }
        }
        kept.push(candidate);
    }
    kept
}

/// Recognize a topic-bearing brainstorming request and compose ranked name
/// candidates from its formalized topic.
///
/// Returns `None` outside the naming request vocabulary.
pub fn handle_brainstorm_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !crate::seed::lexicon().mentions_role(ROLE_BRAINSTORMING, normalized) {
        return None;
    }
    Some(compose_brainstorm_names(
        prompt,
        normalized,
        normalized,
        "brainstorming",
        log,
    ))
}

pub(super) fn compose_brainstorm_names(
    prompt: &str,
    normalized: &str,
    topic: &str,
    intent: &str,
    log: &mut EventLog,
) -> SymbolicAnswer {
    let language = crate::language::detect(prompt).slug();
    let mut topics = topic_words(topic, language);
    topics.truncate(12);
    if topics.is_empty() {
        log.append("brainstorming:refusal", "missing_topic".to_owned());
        return finalize_simple(
            prompt,
            log,
            intent,
            "response:brainstorming",
            &template("brainstorming_refusal", &[]),
            0.4,
        );
    }
    for topic in &topics {
        log.append("brainstorming:topic", topic.clone());
    }
    let count = requested_count(normalized, language).unwrap_or(5);
    log.append("brainstorming:count", count.to_string());
    let short_cued = cue_words("short_names")
        .iter()
        .any(|word| normalized.contains(word.as_str()));
    let max_length = cue_field("short_names", "max_length")
        .parse::<usize>()
        .ok()
        .filter(|_| short_cued)
        .unwrap_or(DEFAULT_MAX_LENGTH);
    let excluded = exclusions(normalized);
    for word in &excluded {
        log.append("brainstorming:exclusion", word.clone());
    }
    let rules = brainstorm_rules();
    let metric = distinctness_metric();
    let pool = compose_candidates(&topics, &rules);
    log.append("brainstorming:pool", pool.len().to_string());
    let candidates = select_candidates(
        pool,
        &topics,
        count as usize,
        max_length,
        &excluded,
        &metric,
    );
    if candidates.is_empty() {
        log.append("brainstorming:refusal", "no candidate passed".to_owned());
        return finalize_simple(
            prompt,
            log,
            intent,
            "response:brainstorming",
            &template("brainstorming_refusal", &[]),
            0.4,
        );
    }
    log.append("brainstorming:candidates", candidates.join(", "));
    // Each kept candidate is an explored option of the reasoning loop's
    // candidate-generation step, linked on its own.
    for candidate in &candidates {
        log.append("candidate", format!("brainstorming:{candidate}"));
    }

    let listed = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| format!("{}. {}", index + 1, candidate))
        .collect::<Vec<_>>()
        .join("\n");
    let rules_used = rules
        .iter()
        .map(|rule| rule.kind.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let max_length_text = max_length.to_string();
    let constraints = if excluded.is_empty() {
        template(
            "brainstorming_constraints",
            &[("max_length", &max_length_text)],
        )
    } else {
        template(
            "brainstorming_constraints_excluding",
            &[
                ("max_length", &max_length_text),
                ("excluded", &excluded.join(", ")),
            ],
        )
    };
    let body = template(
        "brainstorming_candidates",
        &[
            ("count", &candidates.len().to_string()),
            ("concepts", &topics.join(", ")),
            ("candidates", &listed),
            ("metric", &metric.statement),
            ("constraints", &constraints),
            ("rules", &rules_used),
        ],
    );
    finalize_simple(prompt, log, intent, "response:brainstorming", &body, 0.6)
}

/// One `advice_evidence_grade` record: the weight scale of the ordinal
/// ranking.
struct EvidenceGrade {
    grade: String,
    weight: f32,
    label: String,
}

fn evidence_grades() -> Vec<EvidenceGrade> {
    rules_records_named("advice_evidence_grade")
        .map(|record| EvidenceGrade {
            grade: record.find_child_value("grade").to_string(),
            weight: record.find_child_value("weight").parse().unwrap_or(0.3),
            label: record.find_child_value("label").to_string(),
        })
        .filter(|grade| !grade.grade.is_empty())
        .collect()
}

/// Prompt prose before the first evidence block.
pub(super) fn creative_request_text(prompt: &str) -> &str {
    prompt.split("```").next().unwrap_or(prompt)
}

/// Read one bounded JSON object supplied as evidence with the request.
pub(super) fn creative_evidence(prompt: &str, kind: &str) -> Option<Value> {
    let block = crate::solver_helpers::extract_fenced_block(prompt, &["json"])?;
    if block.len() > 64 * 1024 {
        return None;
    }
    let value: Value = serde_json::from_str(&block).ok()?;
    let evidence = value.get(kind)?;
    evidence.is_object().then(|| evidence.clone())
}

pub(super) fn evidence_text<'a>(record: &'a Value, field: &str) -> Option<&'a str> {
    let value = record.get(field)?.as_str()?.trim();
    (!value.is_empty() && !value.chars().any(char::is_control)).then_some(value)
}

pub(super) fn evidence_source(record: &Value) -> Option<&str> {
    let source = evidence_text(record, "source_url")?;
    let (scheme, _) = source.split_once("://")?;
    let url = url::Url::parse(source).ok()?;
    (scheme == "https"
        && url.scheme() == scheme
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none())
    .then_some(source)
}

struct Recommendation {
    text: String,
    grade: String,
    source: String,
    source_url: String,
}

fn supplied_advice(
    prompt: &str,
    grades: &[EvidenceGrade],
) -> Option<(String, Vec<Recommendation>)> {
    let evidence = creative_evidence(prompt, "advice")?;
    let topic = evidence_text(&evidence, "topic")?.to_owned();
    let records = evidence.get("items")?.as_array()?;
    if records.len() > 64 {
        return None;
    }
    let items: Vec<_> = records
        .iter()
        .filter_map(|record| {
            let grade = evidence_text(record, "grade")?;
            if !grades.iter().any(|item| item.grade == grade) {
                return None;
            }
            Some(Recommendation {
                text: evidence_text(record, "text")?.to_owned(),
                grade: grade.to_owned(),
                source: evidence_text(record, "source")?.to_owned(),
                source_url: evidence_source(record)?.to_owned(),
            })
        })
        .collect();
    (!items.is_empty()).then_some((topic, items))
}

/// Claim evidence exists when the request supplies usable cited recommendations.
#[must_use]
pub fn has_advice_evidence(prompt: &str) -> bool {
    supplied_advice(prompt, &evidence_grades()).is_some()
}

/// Recognize an advice request and render its evidence-graded
/// recommendations, each with citation and strength.
///
/// Uncited items are
/// never returned; the ordinal weighting states its forward dependency on
/// issue #1179's relative meta logic.
pub fn handle_advice_request(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    // The canonical reading folds punctuation, as the browser twin does, so a
    // comma after the advice verb ("Посоветуй, как …") does not hide it.
    let request = creative_request_text(prompt);
    let normalized = crate::engine::normalize_prompt(request);
    if !crate::seed::lexicon().mentions_role(ROLE_ADVICE, &normalized) {
        return None;
    }
    let grades = evidence_grades();
    let Some((topic, records)) = supplied_advice(prompt, &grades) else {
        log.append("advice:refusal", "missing_recommendations".to_owned());
        return Some(finalize_simple(
            prompt,
            log,
            "advice",
            "response:advice",
            &template("advice_refusal", &[]),
            0.4,
        ));
    };
    log.append("advice:topic", topic.clone());
    let mut items: Vec<(f32, Recommendation)> = records
        .into_iter()
        .filter_map(|record| {
            // An item without grade, source, or citation is not returned.
            let grade = grades.iter().find(|grade| grade.grade == record.grade);
            grade.map(|grade| (grade.weight, record))
        })
        .collect();
    items.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    for (weight, item) in &items {
        log.append_fields(
            "advice:item",
            &[("weight", &format!("{weight:.2}")), ("grade", &item.grade)],
        );
    }
    let listed = items
        .iter()
        .enumerate()
        .map(|(index, (_weight, item))| {
            let label = grades
                .iter()
                .find(|grade| grade.grade == item.grade)
                .map_or("ungraded", |grade| grade.label.as_str());
            format!(
                "{}. {} [{} — {}] ({})",
                index + 1,
                item.text,
                label,
                item.source,
                item.source_url
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let weighting = grades
        .iter()
        .map(|grade| format!("{} {:.2}", grade.grade, grade.weight))
        .collect::<Vec<_>>()
        .join(" > ");
    let forward_note = template("advice_forward_note", &[]);
    let body = template(
        "advice_guidance",
        &[
            ("topic", &topic),
            ("items", &listed),
            ("weighting", &weighting),
            ("forward_note", &forward_note),
        ],
    );
    Some(finalize_simple(
        prompt,
        log,
        "advice",
        "response:advice",
        &body,
        0.6,
    ))
}
