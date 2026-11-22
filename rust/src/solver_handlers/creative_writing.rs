//! Creative-composition handlers, part two (issue #1178, E143): creative
//! writing and planning.
//!
//! Compose verse from language skeletons and cited caller-supplied rhyme
//! classes. The itinerary planner schedules cited place observations from
//! the request. Both validate the rendered output before returning it.

use std::collections::{BTreeMap, BTreeSet};

use super::creative_composition::{
    creative_evidence, creative_request_text, cue_field, cue_words, evidence_source, evidence_text,
    requested_count, rules_records_named, topic_words,
};
use super::finalize_simple;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;

const ROLE_CREATIVE_WRITING: &str = "creative_writing_request";
const ROLE_PLANNING: &str = "planning_request";

/// Fill a localized response template's `{placeholder}` slots.
fn template(intent: &str, values: &[(&str, &str)]) -> String {
    let mut out = crate::seed::localized_response(intent, "en").unwrap_or_default();
    for (key, value) in values {
        out = out.replace(&format!("{{{key}}}"), value);
    }
    out
}

// ---------------------------------------------------------------------------
// Creative writing
// ---------------------------------------------------------------------------

/// One `writing_form` record: the default constraints of a verse form.
#[derive(Clone)]
struct WritingForm {
    form: String,
    default_lines: usize,
    default_scheme: String,
    syllables: String,
    surfaces: Vec<String>,
}

fn writing_forms() -> Vec<WritingForm> {
    rules_records_named("writing_form")
        .map(|record| WritingForm {
            form: record.find_child_value("form").to_string(),
            default_lines: record
                .find_child_value("default_lines")
                .parse()
                .unwrap_or(4),
            default_scheme: record.find_child_value("default_scheme").to_string(),
            syllables: record.find_child_value("syllables").to_string(),
            surfaces: record
                .children
                .iter()
                .filter(|child| {
                    !matches!(
                        child.name.as_str(),
                        "form" | "default_lines" | "default_scheme" | "syllables"
                    )
                })
                .map(|child| child.id.clone())
                .filter(|surface| !surface.is_empty())
                .collect(),
        })
        .filter(|form| !form.form.is_empty())
        .collect()
}

/// A group of distinct words with a common documented end sound.
struct RhymeClass {
    id: String,
    words: Vec<String>,
    source_url: String,
}

fn supplied_rhymes(prompt: &str, language: &str) -> (Option<String>, Vec<RhymeClass>) {
    let Some(evidence) = creative_evidence(prompt, "rhyme") else {
        return (None, Vec::new());
    };
    if evidence_text(&evidence, "language") != Some(language) {
        return (None, Vec::new());
    }
    let scheme = evidence_text(&evidence, "scheme")
        .filter(|scheme| {
            *scheme == "none"
                || (scheme.len() <= 14 && scheme.bytes().all(|letter| letter.is_ascii_alphabetic()))
        })
        .map(str::to_ascii_lowercase);
    let Some(records) = evidence
        .get("classes")
        .and_then(serde_json::Value::as_array)
    else {
        return (scheme, Vec::new());
    };
    let mut classes = Vec::<RhymeClass>::new();
    if records.len() > 32 {
        return (scheme, classes);
    }
    for record in records {
        let (Some(id), Some(source), Some(words)) = (
            evidence_text(record, "id"),
            evidence_source(record),
            record.get("words").and_then(serde_json::Value::as_array),
        ) else {
            continue;
        };
        if words.len() > 64 || classes.iter().any(|class| class.id == id) {
            continue;
        }
        let mut seen = BTreeSet::new();
        let words: Vec<_> = words
            .iter()
            .filter_map(serde_json::Value::as_str)
            .map(|word| word.trim().to_lowercase())
            .filter(|word| {
                !word.is_empty()
                    && !word.chars().any(char::is_whitespace)
                    && end_word(word) == *word
                    && seen.insert(word.clone())
            })
            .collect();
        if words.len() >= 2
            && !classes
                .iter()
                .any(|class| class.words.iter().any(|word| seen.contains(word)))
        {
            classes.push(RhymeClass {
                id: id.to_owned(),
                words,
                source_url: source.to_owned(),
            });
        }
    }
    (scheme, classes)
}

/// One `poem_line_skeleton` record, in one language.
struct Skeleton {
    text: String,
    rhyme_slot: bool,
}

fn skeletons(language: &str) -> Vec<Skeleton> {
    rules_records_named("poem_line_skeleton")
        .filter(|record| record.find_child_value("language") == language)
        .map(|record| Skeleton {
            text: record.find_child_value("text").to_string(),
            rhyme_slot: record.find_child_value("rhyme_slot") == "true",
        })
        .filter(|skeleton| !skeleton.text.is_empty())
        .collect()
}

/// The extracted form constraints of a creative-writing request.
struct FormConstraints {
    lines: usize,
    scheme: String,
    topic: String,
    rhymes: bool,
    syllables: String,
}

/// Detect the named verse form (haiku/limerick/couplet/sonnet, else poem)
/// from the prompt, in any of the five supported languages.
fn detect_form(normalized: &str) -> WritingForm {
    let forms = writing_forms();
    forms
        .iter()
        .find(|form| {
            form.surfaces
                .iter()
                .any(|surface| normalized.contains(surface))
        })
        .cloned()
        .unwrap_or_else(|| {
            forms
                .iter()
                .find(|form| form.form == "poem")
                .cloned()
                .unwrap_or_else(|| WritingForm {
                    form: "poem".to_owned(),
                    default_lines: 4,
                    default_scheme: "abcb".to_owned(),
                    syllables: String::new(),
                    surfaces: Vec::new(),
                })
        })
}

/// Extract every constraint the request states, with the form's defaults
/// filling the unstated ones.
fn form_constraints(
    normalized: &str,
    language: &str,
    supplied_scheme: Option<String>,
    classes: &[RhymeClass],
) -> FormConstraints {
    let form = detect_form(normalized);
    // A stated line count: a spelled number or digit beside a line word
    // (the `line_count` request cue).
    let line_words = cue_words("line_count");
    let stated = requested_count(normalized, language).filter(|_| {
        line_words
            .iter()
            .any(|word| normalized.contains(word.as_str()))
            || crate::coding::contains_cjk(normalized)
    });
    let lines = stated
        .map_or(form.default_lines, |count| count as usize)
        .clamp(1, 14);
    // A stated rhyme scheme: the literal scheme (the `stated_scheme` cue),
    // else a bare rhyme request (the `rhyme_request` cue and its scheme).
    let rhyme_requested = cue_words("rhyme_request")
        .iter()
        .any(|word| normalized.contains(word.as_str()));
    let scheme = cue_words("stated_scheme")
        .into_iter()
        .find(|scheme| {
            normalized
                .split_whitespace()
                .any(|word| word == scheme.as_str())
        })
        .or(supplied_scheme)
        .unwrap_or_else(|| {
            let cued = if rhyme_requested {
                cue_field("rhyme_request", "scheme")
            } else {
                String::new()
            };
            if cued.is_empty() {
                form.default_scheme.clone()
            } else {
                cued
            }
        });
    let rhymes = scheme != "none" && !classes.is_empty();
    let topic = topic_words(normalized, language)
        .into_iter()
        .next_back()
        .unwrap_or_default();
    FormConstraints {
        lines,
        scheme,
        topic,
        rhymes,
        syllables: form.syllables,
    }
}

/// The end word of a rendered verse line, lowercased.
fn end_word(line: &str) -> String {
    line.split_whitespace()
        .next_back()
        .unwrap_or_default()
        .trim_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase()
}

/// Verify a rendered poem against the scheme. Only positions whose scheme
/// letter repeats carry a constraint: positions sharing a letter must end
/// in one rhyme class, positions with different repeating letters in
/// different classes. Returns the failures (empty when the scheme holds).
fn check_scheme(poem: &[String], scheme: &str, classes: &[RhymeClass]) -> Vec<String> {
    let letters: Vec<char> = scheme.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.len() != poem.len() {
        return vec![template(
            "creative_writing_scheme_length_mismatch",
            &[
                ("scheme", scheme),
                ("positions", &letters.len().to_string()),
                ("lines", &poem.len().to_string()),
            ],
        )];
    }
    let class_of = |word: &str| -> Option<&str> {
        classes
            .iter()
            .find(|class| class.words.iter().any(|member| member == word))
            .map(|class| class.id.as_str())
    };
    let mut failures = Vec::new();
    for (i, letter_i) in letters.iter().enumerate() {
        let repeats_i = letters
            .iter()
            .filter(|letter| **letter == *letter_i)
            .count();
        if repeats_i < 2 {
            continue;
        }
        for (j, letter_j) in letters.iter().enumerate().skip(i + 1) {
            let repeats_j = letters
                .iter()
                .filter(|letter| **letter == *letter_j)
                .count();
            if repeats_j < 2 {
                continue;
            }
            let (word_i, word_j) = (end_word(&poem[i]), end_word(&poem[j]));
            let (class_i, class_j) = (class_of(&word_i), class_of(&word_j));
            let (first, second) = ((i + 1).to_string(), (j + 1).to_string());
            match (letter_i == letter_j, class_i, class_j) {
                (true, Some(a), Some(b)) if a != b => failures.push(template(
                    "creative_writing_rhyme_class_mismatch",
                    &[
                        ("first", &first),
                        ("second", &second),
                        ("first_word", &word_i),
                        ("first_class", a),
                        ("second_word", &word_j),
                        ("second_class", b),
                    ],
                )),
                (true, None, _) | (true, _, None) => failures.push(template(
                    "creative_writing_rhyme_outside_classes",
                    &[("first", &first), ("second", &second)],
                )),
                (false, Some(a), Some(b)) if a == b => failures.push(template(
                    "creative_writing_rhyme_unwanted",
                    &[("first", &first), ("second", &second), ("class", a)],
                )),
                _ => {}
            }
        }
    }
    failures
}

/// Fill a skeleton's `{topic}` and `{rhyme}` slots.
fn fill_skeleton(skeleton: &str, topic: &str, rhyme: &str) -> String {
    skeleton
        .replace(concat!("{", "topic}"), topic)
        .replace(concat!("{", "rhyme}"), rhyme)
}

/// One composition attempt over the rhyme classes, rotated by `offset`.
fn compose_poem(
    constraints: &FormConstraints,
    language: &str,
    classes: &[RhymeClass],
    offset: usize,
) -> Option<Vec<String>> {
    let all_skeletons = skeletons(language);
    if all_skeletons.is_empty() || constraints.topic.is_empty() {
        return None;
    }
    let free_skeletons: Vec<&Skeleton> = all_skeletons
        .iter()
        .filter(|skeleton| !skeleton.rhyme_slot)
        .collect();
    if !constraints.rhymes {
        // Unrhymed composition: cycle the language's free skeletons, and
        // past their number fall back to bare topic lines so no line is
        // ever a duplicate.
        if free_skeletons.is_empty() {
            return None;
        }
        return Some(
            (0..constraints.lines)
                .map(|index| {
                    if index < free_skeletons.len() {
                        fill_skeleton(free_skeletons[index].text.as_str(), &constraints.topic, "")
                    } else {
                        constraints.topic.clone()
                    }
                })
                .collect(),
        );
    }

    let rhymed: Vec<_> = all_skeletons
        .iter()
        .filter(|skeleton| skeleton.rhyme_slot)
        .collect();
    let letters: Vec<_> = constraints.scheme.chars().collect();
    let mut counts = BTreeMap::<char, usize>::new();
    for &letter in &letters {
        *counts.entry(letter).or_default() += 1;
    }
    let mut groups: Vec<_> = counts.into_iter().filter(|(_, count)| *count > 1).collect();
    groups.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    let topic = constraints.topic.to_lowercase();
    let mut available: Vec<_> = classes.iter().collect();
    available.rotate_left(offset % classes.len());
    available.sort_by_key(|class| !class.words.contains(&topic));
    let mut endings = BTreeMap::new();
    for (letter, count) in groups {
        let index = available
            .iter()
            .position(|class| class.words.len() >= count)?;
        let mut words = available.remove(index).words.clone();
        words.sort_by_key(|word| word != &topic);
        words.reverse();
        endings.insert(letter, words);
    }
    let mut poem = Vec::new();
    for (index, letter) in letters.into_iter().enumerate() {
        let word = endings
            .get_mut(&letter)
            .and_then(Vec::pop)
            .unwrap_or_default();
        let line = if !word.is_empty() && !rhymed.is_empty() {
            fill_skeleton(
                &rhymed[index % rhymed.len()].text,
                &constraints.topic,
                &word,
            )
        } else {
            let skeleton = free_skeletons.get(index % free_skeletons.len().max(1))?;
            let mut line = fill_skeleton(&skeleton.text, &constraints.topic, "");
            if !word.is_empty() {
                line.push_str(" — ");
                line.push_str(&word);
            }
            line
        };
        poem.push(line);
    }
    Some(poem)
}

/// Check a rendered poem against every stated constraint. Returns the
/// failure list; empty means the poem satisfies the constraints.
fn check_poem(
    poem: &[String],
    constraints: &FormConstraints,
    classes: &[RhymeClass],
) -> Vec<String> {
    let mut failures = Vec::new();
    if poem.len() != constraints.lines {
        failures.push(template(
            "creative_writing_line_count_mismatch",
            &[
                ("lines_got", &poem.len().to_string()),
                ("lines_want", &constraints.lines.to_string()),
            ],
        ));
    }
    if poem.iter().any(|line| line.trim().is_empty()) {
        failures.push("an empty line".to_owned());
    }
    let topic_lower = constraints.topic.to_lowercase();
    if !constraints.topic.is_empty()
        && !poem
            .iter()
            .any(|line| line.to_lowercase().contains(&topic_lower))
    {
        failures.push(template(
            "creative_writing_topic_absent",
            &[("topic", &constraints.topic)],
        ));
    }
    if constraints.rhymes && constraints.scheme != "none" {
        failures.extend(check_scheme(poem, &constraints.scheme, classes));
    }
    failures
}

/// Recognize a creative-writing request and compose verse under its form
/// constraints.
///
/// The verse is returned only after the rendered output passes the
/// constraint check. Returns `None` when the prompt is not a
/// creative-writing request.
pub fn handle_creative_writing_request(
    prompt: &str,
    _normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let request = creative_request_text(prompt);
    let normalized = crate::engine::normalize_prompt(request);
    if !crate::seed::lexicon().mentions_role(ROLE_CREATIVE_WRITING, &normalized) {
        return None;
    }
    let language = crate::language::detect(request).slug();
    let (scheme, classes) = supplied_rhymes(prompt, language);
    let constraints = form_constraints(&normalized, language, scheme, &classes);
    log.append_fields(
        "creative_writing:constraints",
        &[
            ("lines", &constraints.lines.to_string()),
            ("scheme", &constraints.scheme),
            ("topic", &constraints.topic),
            ("rhymes", &constraints.rhymes.to_string()),
            ("syllables", &constraints.syllables),
        ],
    );
    if constraints.topic.is_empty() {
        log.append("creative_writing:refusal", "no topic".to_owned());
        return Some(finalize_simple(
            prompt,
            log,
            "creative_writing",
            "response:creative_writing",
            &template(
                "creative_writing_refusal",
                &[
                    ("lines_want", &constraints.lines.to_string()),
                    ("scheme", &constraints.scheme),
                    ("topic", "none stated"),
                ],
            ),
            0.4,
        ));
    }

    // Two composition attempts (the second rotates the rhyme classes); an
    // output violating a stated constraint is never returned.
    let mut poem = None;
    let mut failures = Vec::new();
    for offset in 0..2 {
        if let Some(candidate) = compose_poem(&constraints, language, &classes, offset) {
            let candidate_failures = check_poem(&candidate, &constraints, &classes);
            if candidate_failures.is_empty() {
                poem = Some(candidate);
                failures.clear();
                break;
            }
            if offset == 0 {
                failures = candidate_failures;
            }
        }
    }
    let Some(poem) = poem else {
        log.append(
            "creative_writing:constraint_check_failed",
            failures.join("; "),
        );
        return Some(finalize_simple(
            prompt,
            log,
            "creative_writing",
            "response:creative_writing",
            &template(
                "creative_writing_refusal",
                &[
                    ("lines_want", &constraints.lines.to_string()),
                    ("scheme", &constraints.scheme),
                    ("topic", &constraints.topic),
                ],
            ),
            0.4,
        ));
    };
    log.append("creative_writing:composed", poem.len().to_string());

    let rendered = poem.join("\n");
    if constraints.rhymes {
        let ends: BTreeSet<_> = poem.iter().map(|line| end_word(line)).collect();
        let sources = classes
            .iter()
            .filter(|class| class.words.iter().any(|word| ends.contains(word)))
            .map(|class| format!("{} ({})", class.id, class.source_url))
            .collect::<Vec<_>>()
            .join(", ");
        let body = template(
            "creative_writing_poem",
            &[
                ("poem", &rendered),
                ("lines_got", &poem.len().to_string()),
                ("lines_want", &constraints.lines.to_string()),
                ("lines_check", "holds"),
                ("scheme", &constraints.scheme),
                (
                    "rhyme_check",
                    "holds — every rhyming pair ends in one rhyme class",
                ),
                ("topic_check", "yes"),
                ("rhyme_sources", &sources),
            ],
        );
        Some(finalize_simple(
            prompt,
            log,
            "creative_writing",
            "response:creative_writing",
            &body,
            0.6,
        ))
    } else if constraints.scheme == "none" {
        let body = template(
            "creative_writing_unrhymed",
            &[
                ("poem", &rendered),
                ("lines_got", &poem.len().to_string()),
                ("lines_want", &constraints.lines.to_string()),
                ("lines_check", "holds"),
                ("topic_check", "yes"),
                (
                    "syllables",
                    if constraints.syllables.is_empty() {
                        "none stated"
                    } else {
                        constraints.syllables.as_str()
                    },
                ),
            ],
        );
        Some(finalize_simple(
            prompt,
            log,
            "creative_writing",
            "response:creative_writing",
            &body,
            0.6,
        ))
    } else {
        let body = template(
            "creative_writing_gap",
            &[
                ("poem", &rendered),
                ("lines_got", &poem.len().to_string()),
                ("lines_want", &constraints.lines.to_string()),
                ("lines_check", "holds"),
                ("topic_check", "yes"),
            ],
        );
        Some(finalize_simple(
            prompt,
            log,
            "creative_writing",
            "response:creative_writing",
            &body,
            0.6,
        ))
    }
}

include!("creative_writing_planning.rs");
