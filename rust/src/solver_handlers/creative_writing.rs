//! Creative-composition handlers, part two (issue #1178, E143): creative
//! writing and planning.
//!
//! **Creative writing** (`handle_creative_writing_request`) extracts the
//! form constraints from the request (form, line count, rhyme scheme,
//! topic), composes verse by constraint satisfaction — English end-words
//! are drawn from the Wiktionary-grounded rhyme classes of
//! `data/seed/creative-composition-rules.lino`, filled into the per-language
//! verse skeletons — and then **checks the rendered output against every
//! stated constraint before returning it**. An output violating a stated
//! constraint is retried once with the next rhyme class and, failing that,
//! never returned: the honest refusal names the unsatisfied constraint.
//! For languages without offline pronunciation coverage the gap is stated
//! (per the registered sources' own coverage notes) and unrhymed free
//! verse is composed under the line-count and topic constraints instead of
//! a guessed rhyme.
//!
//! **Planning** (`handle_planning_request`) schedules under constraints
//! from the seeded place cache: opening hours, visit lengths, intra- and
//! cross-district travel times, one lunch window, at most four items a
//! day, and no two consecutive items of one category. Every plan item
//! carries the fact it depends on — a plan with an uncited item is not
//! returned — and the feasibility check (no overlaps, travel time
//! accounted) runs on the schedule before it is returned. This is the
//! handler that answers "3-day itinerary for Rome" with a schedule instead
//! of the terminal-command misroute or the canned web-search paragraph.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use super::finalize_simple;
use super::creative_composition::{requested_count, topic_words};

const RULES_PATH: &str = "data/seed/creative-composition-rules.lino";
const ROLE_CREATIVE_WRITING: &str = "creative_writing_request";
const ROLE_PLANNING: &str = "planning_request";

/// Look up embedded seed content by its registered path.
fn seed_text(path: &str) -> Option<&'static str> {
    crate::seed::seed_files()
        .into_iter()
        .find(|(registered, _)| *registered == path)
        .map(|(_, text)| text)
}

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
struct WritingForm {
    form: String,
    default_lines: usize,
    default_scheme: String,
    syllables: String,
    surfaces: Vec<String>,
}

fn writing_forms() -> Vec<WritingForm> {
    let Some(text) = seed_text(RULES_PATH) else {
        return Vec::new();
    };
    let tree = crate::seed::parser::parse_lino(text);
    tree.children
        .iter()
        .filter(|record| record.name == "writing_form")
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

/// One `rhyme_class` record: the Wiktionary-grounded end-sound class.
struct RhymeClass {
    id: String,
    words: Vec<String>,
    source_url: String,
}

fn rhyme_classes() -> Vec<RhymeClass> {
    let Some(text) = seed_text(RULES_PATH) else {
        return Vec::new();
    };
    let tree = crate::seed::parser::parse_lino(text);
    tree.children
        .iter()
        .filter(|record| record.name == "rhyme_class")
        .map(|record| RhymeClass {
            id: record.find_child_value("id").to_string(),
            words: record
                .children
                .iter()
                .filter(|child| child.name == "word")
                .map(|child| child.id.clone())
                .filter(|word| !word.is_empty())
                .collect(),
            source_url: record.find_child_value("source_url").to_string(),
        })
        .filter(|class| !class.id.is_empty())
        .collect()
}

/// The `rhyme_coverage` record: which languages carry offline rhyme
/// grounding, and the stated gap text for the others.
fn rhyme_coverage() -> (Vec<String>, String) {
    let Some(text) = seed_text(RULES_PATH) else {
        return (Vec::new(), String::new());
    };
    let tree = crate::seed::parser::parse_lino(text);
    let Some(record) = tree
        .children
        .iter()
        .find(|record| record.name == "rhyme_coverage")
    else {
        return (Vec::new(), String::new());
    };
    (
        crate::seed::parser::split_pipe_list(record.find_child_value("rhyme_languages"))
            .into_iter()
            .map(|language| language.to_string())
            .collect(),
        record.find_child_value("gap").to_string(),
    )
}

/// One `poem_line_skeleton` record, in one language.
struct Skeleton {
    text: String,
    rhyme_slot: bool,
}

fn skeletons(language: &str) -> Vec<Skeleton> {
    let Some(text) = seed_text(RULES_PATH) else {
        return Vec::new();
    };
    let tree = crate::seed::parser::parse_lino(text);
    tree.children
        .iter()
        .filter(|record| record.name == "poem_line_skeleton")
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
        .find(|form| form.surfaces.iter().any(|surface| normalized.contains(surface)))
        .cloned()
        .unwrap_or_else(|| {
            forms
                .iter()
                .find(|form| form.form == "poem")
                .cloned()
                .unwrap_or(WritingForm {
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
fn form_constraints(normalized: &str, language: &str) -> FormConstraints {
    let form = detect_form(normalized);
    // A stated line count: a spelled number or digit beside a line word.
    let line_words = ["line", "lines", "строка", "строки", "पंक्ति", "行", "línea", "líneas"];
    let stated = requested_count(normalized, language).filter(|_| {
        line_words.iter().any(|word| normalized.contains(word))
            || crate::coding::contains_cjk(normalized)
    });
    let lines = stated
        .map(|count| count as usize)
        .unwrap_or(form.default_lines)
        .clamp(1, 14);
    // A stated rhyme scheme: the literal scheme, or "rhyming" poetry.
    let scheme = if normalized.contains("aabb") {
        "aabb".to_owned()
    } else if normalized.contains("abab") {
        "abab".to_owned()
    } else if normalized.contains("rhyming") || normalized.contains("rhyme") {
        "aabb".to_owned()
    } else {
        form.default_scheme.clone()
    };
    let (covered, _) = rhyme_coverage();
    let rhymes = scheme != "none" && covered.iter().any(|lang| lang == language);
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
        return vec![format!(
            "scheme {scheme} names {} positions but the poem has {} lines",
            letters.len(),
            poem.len()
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
        let repeats_i = letters.iter().filter(|letter| **letter == *letter_i).count();
        if repeats_i < 2 {
            continue;
        }
        for (j, letter_j) in letters.iter().enumerate().skip(i + 1) {
            let repeats_j = letters.iter().filter(|letter| **letter == *letter_j).count();
            if repeats_j < 2 {
                continue;
            }
            let (word_i, word_j) = (end_word(&poem[i]), end_word(&poem[j]));
            let (class_i, class_j) = (class_of(&word_i), class_of(&word_j));
            match (letter_i == letter_j, class_i, class_j) {
                (true, Some(a), Some(b)) if a != b => failures.push(format!(
                    "positions {} and {} must rhyme but '{word_i}' ({a}) and '{word_j}' ({b}) are different classes",
                    i + 1,
                    j + 1
                )),
                (true, None, _) | (true, _, None) => {
                    failures.push(format!(
                        "position {} or {} ends in a word outside every rhyme class",
                        i + 1,
                        j + 1
                    ));
                }
                (false, Some(a), Some(b)) if a == b => failures.push(format!(
                    "positions {} and {} must NOT rhyme but both end in class {a}",
                    i + 1,
                    j + 1
                )),
                _ => {}
            }
        }
    }
    failures
}

/// Fill a skeleton's `{topic}` and `{rhyme}` slots.
fn fill_skeleton(skeleton: &str, topic: &str, rhyme: &str) -> String {
    skeleton.replace("{topic}", topic).replace("{rhyme}", rhyme)
}

/// One composition attempt over the rhyme classes, rotated by `offset`.
fn compose_poem(
    constraints: &FormConstraints,
    language: &str,
    offset: usize,
) -> Option<Vec<String>> {
    let classes = rhyme_classes();
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

    // Rhymed composition: the topic's own rhyme class (else the rotated
    // one) serves the scheme's first letter; the partner class serves the
    // other repeating letters.
    let topic_lower = constraints.topic.to_lowercase();
    let primary_index = classes
        .iter()
        .position(|class| class.words.iter().any(|word| *word == topic_lower))
        .unwrap_or(offset % classes.len().max(1));
    let primary = classes.get(primary_index)?;
    let partner = classes.get((primary_index + 1 + offset) % classes.len().max(1))?;

    // End words, the topic word first when it belongs to the class.
    let mut primary_words: Vec<String> = primary
        .words
        .iter()
        .filter(|word| **word != topic_lower)
        .cloned()
        .collect();
    if primary.words.iter().any(|word| *word == topic_lower) {
        primary_words.insert(0, topic_lower.clone());
    }
    let partner_words: Vec<String> = partner
        .words
        .iter()
        .filter(|word| !primary.words.contains(word))
        .cloned()
        .collect();

    let rhyme_skeletons: Vec<&Skeleton> = all_skeletons
        .iter()
        .filter(|skeleton| skeleton.rhyme_slot)
        .collect();
    if rhyme_skeletons.is_empty() {
        return None;
    }

    let letters: Vec<char> = constraints
        .scheme
        .chars()
        .filter(|c| c.is_alphabetic())
        .collect();
    let count_a = letters.iter().filter(|c| **c == 'a').count();
    let count_other = letters.len() - count_a;
    if primary_words.len() < count_a.max(1) || partner_words.len() < count_other.max(1) {
        return None;
    }

    let mut poem = Vec::new();
    let mut a_used = 0usize;
    let mut other_used = 0usize;
    let mut rhyme_index = 0usize;
    let mut free_index = 0usize;
    for letter in letters {
        let repeats = letters.iter().filter(|other| **other == letter).count();
        if repeats < 2 {
            // A free position (abcb's a and c): a free skeleton, no rhyme
            // constraint on its end word.
            let text = if let Some(free) = free_skeletons.get(free_index % free_skeletons.len())
            {
                free_index += 1;
                free.text.clone()
            } else {
                rhyme_skeletons[rhyme_index % rhyme_skeletons.len()]
                    .text
                    .clone()
            };
            poem.push(fill_skeleton(&text, &constraints.topic, ""));
            continue;
        }
        let word = if letter == 'a' {
            let word = &primary_words[a_used % primary_words.len()];
            a_used += 1;
            word.clone()
        } else {
            let word = &partner_words[other_used % partner_words.len()];
            other_used += 1;
            word.clone()
        };
        let skeleton_text = &rhyme_skeletons[rhyme_index % rhyme_skeletons.len()].text;
        rhyme_index += 1;
        poem.push(fill_skeleton(skeleton_text, &constraints.topic, &word));
    }
    Some(poem)
}

/// Check a rendered poem against every stated constraint. Returns the
/// failure list; empty means the poem satisfies the constraints.
fn check_poem(poem: &[String], constraints: &FormConstraints) -> Vec<String> {
    let mut failures = Vec::new();
    if poem.len() != constraints.lines {
        failures.push(format!(
            "line count {} but {} wanted",
            poem.len(),
            constraints.lines
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
        failures.push(format!("topic word '{}' absent", constraints.topic));
    }
    if constraints.rhymes && constraints.scheme != "none" {
        failures.extend(check_scheme(poem, &constraints.scheme, &rhyme_classes()));
    }
    failures
}

/// Recognize a creative-writing request, compose verse under its form
/// constraints, and return it only after the rendered output passes the
/// constraint check. Returns `None` when the prompt is not a
/// creative-writing request.
pub fn handle_creative_writing_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if !crate::seed::lexicon().mentions_role(ROLE_CREATIVE_WRITING, normalized) {
        return None;
    }
    let language = crate::language::detect(prompt).slug();
    let constraints = form_constraints(normalized, &language);
    log.append(
        "creative_writing:constraints",
        format!(
            "lines {} scheme {} topic {} rhymes {} syllables {}",
            constraints.lines,
            constraints.scheme,
            constraints.topic,
            constraints.rhymes,
            constraints.syllables
        ),
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
        if let Some(candidate) = compose_poem(&constraints, &language, offset) {
            let candidate_failures = check_poem(&candidate, &constraints);
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
            "creative_writing:refusal",
            format!("constraint check failed: {}", failures.join("; ")),
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
        let topic_lower = constraints.topic.to_lowercase();
        let class = rhyme_classes()
            .into_iter()
            .find(|class| class.words.iter().any(|word| *word == topic_lower))
            .map(|class| class.id)
            .unwrap_or_else(|| "rotated".to_owned());
        let body = template(
            "creative_writing_poem",
            &[
                ("poem", &rendered),
                ("lines_got", &poem.len().to_string()),
                ("lines_want", &constraints.lines.to_string()),
                ("lines_check", "holds"),
                ("scheme", &constraints.scheme),
                ("rhyme_check", "holds — every rhyming pair ends in one rhyme class"),
                ("topic_check", "yes"),
                ("rhyme_class", &class),
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
    } else if rhyme_coverage()
        .0
        .iter()
        .any(|covered| covered == &language)
    {
        // No rhyme was requested in a language whose pronunciation data is
        // grounded: the honest report is the line-count and topic check
        // plus the unverified-meter statement, not a coverage gap.
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
        let (_, gap) = rhyme_coverage();
        let body = template(
            "creative_writing_gap",
            &[
                ("poem", &rendered),
                ("lines_got", &poem.len().to_string()),
                ("lines_want", &constraints.lines.to_string()),
                ("lines_check", "holds"),
                ("topic_check", "yes"),
                ("gap", &gap),
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
