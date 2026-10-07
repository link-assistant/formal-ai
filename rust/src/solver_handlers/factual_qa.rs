//! Subject-verified factual Q&A (issue #1172 R2 and R6).
//!
//! R2: a seeded fact may answer only when the question's formalized subject
//! resolves to the record's `subject_qid` (and the question's relation to the
//! record's relation). The subject comes from the question formalizer first —
//! a [`FormalizationRole::Subject`] slot whose anchor is already a Wikidata
//! item — and otherwise from the clause of the prompt that carries the asked
//! relation, resolved through the fact store's own label index (aliases and
//! localized subject labels mapped to their Q-id). The longest whole-word
//! label wins; a tie between different Q-ids resolves to nothing, so an
//! ambiguous subject never picks a record by declaration order. Alias plus
//! keyword matching ([`FactRecord::matches_normalized`]) survives only as the
//! last-resort surface hint, for records that carry no Q-id at all, and the
//! gate that admitted a record is logged as `fact_lookup:subject_gate`.
//!
//! R6: "Compare X and Y" over two seeded subjects aligns both subjects' facts
//! by relation and states each difference; when one side is seeded and the
//! other is not, the answer is an honest gap naming the missing side. Two
//! unseeded sides leave the prompt to the research route.
//!
//! The comparison cues and joiners are the `fact_comparison_cue` and
//! `fact_comparison_joiner` meanings of `data/seed/meanings-facts.lino`; the
//! wording is `data/seed/multilingual-responses-entities.lino`. The browser
//! twin is `js/worker/formal_ai_worker_factual_qa.js`.

use std::sync::OnceLock;

use super::finalize_simple;
use crate::coding::contains_cjk;
use crate::engine::{SymbolicAnswer, normalize_prompt};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{self, FactRecord, Meaning};
use crate::translation::{FormalizationAnchorKind, FormalizationRole, formalize_prompt_candidates};

/// Semantic role: a word asking to compare two subjects.
const ROLE_FACT_COMPARISON_CUE: &str = "fact_comparison_cue";
/// Semantic role: the word joining the two compared subjects.
const ROLE_FACT_COMPARISON_JOINER: &str = "fact_comparison_joiner";
/// The prefix a formalization anchor id carries for a Wikidata item.
const WIKIDATA_ANCHOR_PREFIX: &str = "wikidata:";
/// Punctuation that ends a clause, and with it a subject slot.
const CLAUSE_BOUNDARIES: [char; 13] = [
    ',', ';', ':', '(', ')', '—', '!', '?', '，', '；', '：', '。', '？',
];

/// A subject slot resolved to one Q-id, with the label that resolved it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSubject {
    pub qid: String,
    pub surface: String,
}

/// One entry of the fact store's label index.
struct SubjectLabel {
    qid: String,
    surface: String,
}

fn fact_records() -> &'static [FactRecord] {
    static CELL: OnceLock<Vec<FactRecord>> = OnceLock::new();
    CELL.get_or_init(seed::facts).as_slice()
}

/// Every alias and localized subject label of a record that carries a Q-id.
fn subject_label_index() -> &'static [SubjectLabel] {
    static INDEX: OnceLock<Vec<SubjectLabel>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut index = Vec::new();
        for record in fact_records()
            .iter()
            .filter(|record| !record.subject_qid.is_empty())
        {
            let labels = record
                .subject_aliases
                .iter()
                .map(String::as_str)
                .chain(std::iter::once(record.subject_label.as_str()))
                .chain(
                    record
                        .localized
                        .iter()
                        .map(|localized| localized.subject_label.as_str()),
                );
            for label in labels {
                let surface = normalize_prompt(label);
                if !surface.is_empty() {
                    index.push(SubjectLabel {
                        qid: record.subject_qid.clone(),
                        surface,
                    });
                }
            }
        }
        index
    })
}

/// Resolve a subject slot to one Q-id: the longest whole-word label wins, and
/// a tie between labels of different Q-ids (or no label at all) is `None`.
#[must_use]
pub fn resolve_fact_subject(slot: &str) -> Option<ResolvedSubject> {
    let normalized = normalize_prompt(slot);
    if normalized.is_empty() {
        return None;
    }
    let mut best_length = 0;
    let mut best: Vec<&SubjectLabel> = Vec::new();
    for entry in subject_label_index() {
        if !FactRecord::contains_word_sequence(&normalized, &entry.surface) {
            continue;
        }
        let length = entry.surface.chars().count();
        if length > best_length {
            best_length = length;
            best.clear();
        }
        if length == best_length {
            best.push(entry);
        }
    }
    let first = best.first()?;
    best.iter()
        .all(|entry| entry.qid == first.qid)
        .then(|| ResolvedSubject {
            qid: first.qid.clone(),
            surface: first.surface.clone(),
        })
}

/// The relation the question asks about: the first `fact_relation` meaning
/// (declaration order) whose surface appears as a whole word.
pub(super) fn question_relation(normalized: &str) -> Option<&'static Meaning> {
    seed::lexicon()
        .meanings_with_role(seed::ROLE_FACT_RELATION)
        .find(|meaning| {
            meaning
                .words()
                .any(|word| FactRecord::contains_word_sequence(normalized, word))
        })
}

/// The clause of `text` that carries the relation (the whole text when no
/// clause does), so "the capital of Canada, not the USA" keeps "Canada".
pub(super) fn relation_clause<'a>(text: &'a str, relation: Option<&Meaning>) -> &'a str {
    let Some(relation) = relation else {
        return text;
    };
    text.split(CLAUSE_BOUNDARIES)
        .find(|clause| {
            let clause = normalize_prompt(clause);
            relation
                .words()
                .any(|word| FactRecord::contains_word_sequence(&clause, word))
        })
        .unwrap_or(text)
}

/// The formalized question: the asked relation and the subject's Q-id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectGate {
    pub relation: Option<String>,
    pub qid: Option<String>,
}

/// Formalize `text` into its relation and subject Q-id. A formalizer subject
/// slot already anchored on a Wikidata item wins; otherwise the relation's
/// clause is resolved through the fact store's label index.
#[must_use]
pub fn fact_subject_gate(text: &str) -> SubjectGate {
    let normalized = normalize_prompt(text);
    let relation = question_relation(&normalized);
    let language = detect_language(text).slug();
    let anchored = formalize_prompt_candidates(text, language)
        .into_iter()
        .next()
        .and_then(|candidate| {
            candidate
                .slot(FormalizationRole::Subject)
                .filter(|slot| slot.anchor.kind == FormalizationAnchorKind::WikidataItem)
                .and_then(|slot| slot.anchor.id.strip_prefix(WIKIDATA_ANCHOR_PREFIX))
                .map(str::to_owned)
        });
    let qid = anchored.or_else(|| {
        resolve_fact_subject(relation_clause(text, relation)).map(|subject| subject.qid)
    });
    SubjectGate {
        relation: relation.map(|meaning| meaning.slug.clone()),
        qid,
    }
}

/// The seeded record allowed to answer, and the gate that admitted it.
///
/// The gate is `subject_qid:<Q>` when the formalized subject matched the
/// record's Q-id, `surface_hint` for a Q-id-less record matched by alias and
/// keyword. The question is formalized from `prompt` unless `normalized` is a rewrite
/// of it (a resolved coreference), which is then the text to formalize.
#[must_use]
pub fn gated_fact_record(prompt: &str, normalized: &str) -> Option<(&'static FactRecord, String)> {
    let rewritten = normalize_prompt(prompt) != normalize_prompt(normalized);
    let gate = fact_subject_gate(if rewritten { normalized } else { prompt });
    let keyword_hint = |record: &FactRecord| {
        record
            .question_keywords
            .iter()
            .any(|keyword| FactRecord::contains_word_sequence(normalized, keyword))
    };
    if let Some(qid) = gate.qid.as_deref()
        && let Some(record) = fact_records().iter().find(|&record| {
            record.subject_qid == qid
                && gate.relation.as_deref().map_or_else(
                    || keyword_hint(record),
                    |relation| record.relation == relation,
                )
        })
    {
        return Some((record, format!("subject_qid:{qid}")));
    }
    fact_records()
        .iter()
        .find(|record| record.subject_qid.is_empty() && record.matches_normalized(normalized))
        .map(|record| (record, String::from("surface_hint")))
}

/// A seed response template with its `{name}` slots filled.
pub(super) fn render_template(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    let mut rendered = seed::localized_response(intent, language).unwrap_or_default();
    for (name, value) in values {
        rendered = rendered.replace(&format!("{{{name}}}"), value);
    }
    rendered
}

/// The normalized surfaces of every meaning carrying `role`, longest first.
pub(super) fn role_surfaces(role: &str) -> Vec<String> {
    let mut surfaces: Vec<String> = Vec::new();
    for meaning in seed::lexicon().meanings_with_role(role) {
        for word in meaning.words() {
            let surface = if contains_cjk(word) {
                word.to_lowercase()
                    .chars()
                    .filter(|character| *character != '…' && !character.is_whitespace())
                    .collect()
            } else {
                normalize_prompt(word)
            };
            if !surface.is_empty() && !surfaces.contains(&surface) {
                surfaces.push(surface);
            }
        }
    }
    surfaces.sort_by_key(|surface| std::cmp::Reverse(surface.chars().count()));
    surfaces
}

/// Where `surface` starts in `text` as a whole word (CJK: as a substring).
pub(super) fn locate(text: &str, surface: &str) -> Option<usize> {
    if surface.is_empty() {
        return None;
    }
    if contains_cjk(surface) {
        return text.find(surface);
    }
    format!(" {text} ").find(&format!(" {surface} "))
}

/// The two sides of a comparison: the prompt minus its cue, split at the
/// first joiner.
fn comparison_sides(normalized: &str) -> Option<(String, String)> {
    let (at, cue) = role_surfaces(ROLE_FACT_COMPARISON_CUE)
        .into_iter()
        .find_map(|cue| locate(normalized, &cue).map(|at| (at, cue)))?;
    let rest = format!("{} {}", &normalized[..at], &normalized[at + cue.len()..]);
    let rest = rest.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut split: Option<(usize, String)> = None;
    for joiner in role_surfaces(ROLE_FACT_COMPARISON_JOINER) {
        if let Some(index) = locate(&rest, &joiner)
            && index > 0
            && split.as_ref().is_none_or(|(best, _)| index < *best)
        {
            split = Some((index, joiner));
        }
    }
    let (index, joiner) = split?;
    let left = rest[..index].trim();
    let right = rest[index + joiner.len()..].trim();
    (!left.is_empty() && !right.is_empty()).then(|| (left.to_owned(), right.to_owned()))
}

/// One subject's seeded facts, one per relation (first record wins).
struct SubjectFacts {
    label: String,
    values: Vec<(String, String)>,
}

impl SubjectFacts {
    fn of(subject: &ResolvedSubject, language: &str) -> Self {
        let mut label = String::new();
        let mut values: Vec<(String, String)> = Vec::new();
        for record in fact_records()
            .iter()
            .filter(|record| record.subject_qid == subject.qid && !record.relation.is_empty())
        {
            if values
                .iter()
                .any(|(relation, _)| *relation == record.relation)
            {
                continue;
            }
            if label.is_empty() {
                record.subject_label_for(language).clone_into(&mut label);
            }
            values.push((
                record.relation.clone(),
                record.value_label_for(language).to_owned(),
            ));
        }
        if label.is_empty() {
            label.clone_from(&subject.surface);
        }
        Self { label, values }
    }

    fn value(&self, relation: &str) -> Option<&str> {
        self.values
            .iter()
            .find(|(candidate, _)| candidate == relation)
            .map(|(_, value)| value.as_str())
            .filter(|value| !value.is_empty())
    }
}

/// `value`, or the missing-value marker when the subject has none.
const fn or_missing<'a>(value: &'a str, missing: &'a str) -> &'a str {
    if value.is_empty() { missing } else { value }
}

/// The relation's word in `language` (the `fact_relation` meaning's surface).
pub(super) fn relation_label(relation: &str, language: &str) -> String {
    seed::lexicon()
        .meaning(relation)
        .and_then(|meaning| meaning.word_in(language).or_else(|| meaning.word_in("en")))
        .unwrap_or(relation)
        .to_owned()
}

/// "Compare X and Y": both seeded subjects aligned by relation, with the
/// differences stated; one seeded side is an honest gap naming the other.
pub fn try_fact_comparison(prompt: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let normalized = normalize_prompt(prompt);
    let (left_side, right_side) = comparison_sides(&normalized)?;
    let language = detect_language(prompt).slug();
    match (
        resolve_fact_subject(&left_side),
        resolve_fact_subject(&right_side),
    ) {
        (Some(left), Some(right)) if left.qid != right.qid => {
            compare_subjects(prompt, &left, &right, language, log)
        }
        (Some(known), None) => comparison_gap(prompt, &known, &right_side, language, log),
        (None, Some(known)) => comparison_gap(prompt, &known, &left_side, language, log),
        _ => None,
    }
}

fn compare_subjects(
    prompt: &str,
    left: &ResolvedSubject,
    right: &ResolvedSubject,
    language: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let left_facts = SubjectFacts::of(left, language);
    let right_facts = SubjectFacts::of(right, language);
    if left_facts.values.is_empty() || right_facts.values.is_empty() {
        return None;
    }
    log.append("fact_comparison:left", left.qid.clone());
    log.append("fact_comparison:right", right.qid.clone());
    let missing = render_template("fact_comparison_missing_value", language, &[]);
    let mut relations: Vec<&str> = Vec::new();
    for (relation, _) in left_facts.values.iter().chain(&right_facts.values) {
        if !relations.contains(&relation.as_str()) {
            relations.push(relation.as_str());
        }
    }
    let (left_label, right_label) = (left_facts.label.as_str(), right_facts.label.as_str());
    let mut rows = Vec::new();
    let mut differences = Vec::new();
    for relation in relations {
        let label = relation_label(relation, language);
        let left_value = left_facts.value(relation).unwrap_or_default();
        let right_value = right_facts.value(relation).unwrap_or_default();
        rows.push(render_template(
            "fact_comparison_row",
            language,
            &[
                ("relation", label.as_str()),
                ("left", left_label),
                ("right", right_label),
                ("left_value", or_missing(left_value, &missing)),
                ("right_value", or_missing(right_value, &missing)),
            ],
        ));
        let (kind, event) = match (left_value.is_empty(), right_value.is_empty()) {
            (false, false) if left_value == right_value => ("same", "fact_comparison:same"),
            (false, false) => ("difference", "fact_comparison:difference"),
            _ => ("unaligned", "fact_comparison:unaligned"),
        };
        let subject = if left_value.is_empty() {
            right_label
        } else {
            left_label
        };
        differences.push(render_template(
            &format!("fact_comparison_{kind}"),
            language,
            &[
                ("relation", label.as_str()),
                ("left", left_label),
                ("right", right_label),
                ("left_value", left_value),
                ("right_value", right_value),
                ("value", left_value),
                ("subject", subject),
            ],
        ));
        log.append("fact_comparison:relation", relation.to_owned());
        log.append(event, relation.to_owned());
    }
    log.append("wikidata", left.qid.clone());
    log.append("wikidata", right.qid.clone());
    let (rows, differences) = (rows.join("\n"), differences.join("\n"));
    let body = render_template(
        "fact_comparison",
        language,
        &[
            ("left", left_label),
            ("right", right_label),
            ("rows", rows.as_str()),
            ("differences", differences.as_str()),
        ],
    );
    Some(finalize_simple(
        prompt,
        log,
        "fact_comparison",
        "response:fact_comparison",
        &body,
        0.9,
    ))
}

fn comparison_gap(
    prompt: &str,
    known: &ResolvedSubject,
    missing_side: &str,
    language: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let facts = SubjectFacts::of(known, language);
    if facts.values.is_empty() {
        return None;
    }
    // Name the missing side as the user spelled it when the prompt carries it verbatim.
    let missing = prompt
        .to_lowercase()
        .find(missing_side)
        .and_then(|at| prompt.get(at..at + missing_side.len()))
        .filter(|original| original.to_lowercase() == missing_side)
        .unwrap_or(missing_side);
    let rows = facts
        .values
        .iter()
        .map(|(relation, value)| {
            let label = relation_label(relation, language);
            render_template(
                "fact_comparison_known_row",
                language,
                &[("relation", label.as_str()), ("value", value.as_str())],
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    log.append("fact_comparison:known", known.qid.clone());
    log.append("fact_comparison:missing", missing.to_owned());
    let body = render_template(
        "fact_comparison_gap",
        language,
        &[
            ("known", facts.label.as_str()),
            ("missing", missing),
            ("rows", rows.as_str()),
        ],
    );
    Some(finalize_simple(
        prompt,
        log,
        "fact_comparison_gap",
        "response:fact_comparison_gap",
        &body,
        0.6,
    ))
}
