//! Sense → concept / predicate / entity links with provenance (issue #1138,
//! plan 04 L6).
//!
//! A gloss is split into genus and differentiae using the cues declared in
//! `data/seed/formalization-relations.lino`; the relation seed declares how a
//! gloss is read, never what a word means. No substring of a grounded gloss may
//! reach generated source — that is the gloss-not-code gate plan 01 and this
//! plan share.
//!
//! **Where the genus comes from when no cue fires.** A dictionary gloss states
//! its genus by opening with it: *"a word in which no letter is repeated"* is a
//! kind of word, and no copula says so. So the reader first looks for a seeded
//! `is_a` cue, and falls back to the head of the opening noun phrase with the
//! seeded determiner removed. Both halves are data: the determiners are a
//! `determiners` block in the same file, because "which words open a noun
//! phrase" is a fact about a language and not about this module.
//!
//! **What this module refuses to do.** `relations_in` returns a relation only
//! when a seeded cue stands between two terms the graph has already grounded.
//! A cue with nothing grounded on one side of it is a sentence the formalizer
//! read and did not understand, and saying so is the honest result; guessing
//! the missing side from position would manufacture exactly the relations the
//! two issue #710 probes found the formalizer could not produce.

use std::sync::OnceLock;

use crate::concept_lookup::ConceptSense;
use crate::formalization::concept_links::ConceptGraph;
use crate::formalization::segment::Segment;
use crate::seed::FORMALIZATION_RELATIONS_LINO;
use crate::seed::parser::{LinoNode, parse_lino};

/// A concept the formalizer grounded, and the exact bytes that ground it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedConcept {
    /// `"concept:<slug of lemma>"`.
    pub id: String,
    pub label: String,
    pub language: String,
    pub gloss: String,
    /// The "is a" term the gloss names, when it names one.
    pub genus: Option<String>,
    pub differentiae: Vec<String>,
    /// Seeded structural meaning ids the gloss evidences.
    pub structures: Vec<String>,
    pub source_id: String,
    pub source_url: String,
    pub sha256: String,
    pub license_name: String,
    pub depth: usize,
}

/// A relation evidenced between two grounded terms in one clause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedRelation {
    pub id: String,
    /// One of the eight seeded relation kinds.
    pub kind: String,
    pub subject: String,
    pub object: String,
    /// The seeded cue that evidenced the relation.
    pub cue: String,
    /// `"<doc_id>@<start>:<end>"`, the clause span the cue was read from.
    pub source_span: String,
    pub language: String,
}

/// A named individual the formalizer recognised in the text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedEntity {
    pub id: String,
    pub label: String,
    pub language: String,
    /// `"<doc_id>@<start>:<end>"`, the exact span of the mention.
    pub source_span: String,
}

/// One seeded relation: its slug, the kind it belongs to, and every cue that
/// evidences it in any language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationDefinition {
    pub slug: String,
    pub kind: String,
    pub inverse: String,
    pub grounding: String,
    /// `(language, cue)` pairs, longest cue first so a longer cue is never
    /// shadowed by a prefix of itself.
    pub cues: Vec<(String, String)>,
}

/// The eight relations `data/seed/formalization-relations.lino` declares.
#[must_use]
pub fn relations() -> &'static [RelationDefinition] {
    static RELATIONS: OnceLock<Vec<RelationDefinition>> = OnceLock::new();
    RELATIONS.get_or_init(|| {
        let root = parse_lino(FORMALIZATION_RELATIONS_LINO);
        let mut out: Vec<RelationDefinition> = root
            .children
            .iter()
            .find(|node| node.name == "relations")
            .map(|container| {
                container
                    .children
                    .iter()
                    .map(|node| RelationDefinition {
                        slug: node.name.clone(),
                        kind: node.find_child_value("kind").to_owned(),
                        inverse: node.find_child_value("inverse").to_owned(),
                        grounding: node
                            .find_child_value("grounding")
                            .trim_matches('"')
                            .to_owned(),
                        cues: surfaces_of(node),
                    })
                    .collect()
            })
            .unwrap_or_default();
        for definition in &mut out {
            definition
                .cues
                .sort_by(|left, right| right.1.chars().count().cmp(&left.1.chars().count()));
        }
        out
    })
}

/// The determiners the same seed file declares, longest first.
fn determiners() -> &'static [String] {
    static DETERMINERS: OnceLock<Vec<String>> = OnceLock::new();
    DETERMINERS.get_or_init(|| {
        let root = parse_lino(FORMALIZATION_RELATIONS_LINO);
        let mut out: Vec<String> = root
            .children
            .iter()
            .find(|node| node.name == "determiners")
            .map(|container| {
                surfaces_of(container)
                    .into_iter()
                    .map(|(_, surface)| surface)
                    .collect()
            })
            .unwrap_or_default();
        out.sort_by(|left, right| right.chars().count().cmp(&left.chars().count()));
        out
    })
}

/// Every `lexeme <language> / surface / text` pair under one node.
fn surfaces_of(node: &LinoNode) -> Vec<(String, String)> {
    node.children
        .iter()
        .filter(|child| child.name == "lexeme")
        .flat_map(|lexeme| {
            let language = lexeme.id.clone();
            lexeme
                .children
                .iter()
                .filter(|child| child.name == "surface")
                .filter_map(move |surface| {
                    let text = surface
                        .find_child_value("text")
                        .trim_matches('"')
                        .to_owned();
                    (!text.is_empty()).then(|| (language.clone(), text))
                })
        })
        .collect()
}

/// Ground one retrieved sense as a concept: split the gloss into genus and
/// differentiae using the seeded relation cues, then re-run the differentiae
/// through the structural lexicon so the concept reaches seeded idioms.
#[must_use]
pub fn concept_from_sense(sense: &ConceptSense) -> Option<ExtractedConcept> {
    let lemma = sense.lemma.trim();
    let gloss = sense.gloss.trim();
    if lemma.is_empty() || gloss.is_empty() {
        return None;
    }
    let (genus, differentiae) = read_gloss(gloss);
    let structures =
        crate::coding::concept_discovery::structures_for(&crate::engine::normalize_prompt(gloss))
            .into_iter()
            .map(|structure| structure.id)
            .collect();
    Some(ExtractedConcept {
        id: concept_id(lemma),
        label: lemma.to_owned(),
        language: sense.language.clone(),
        gloss: gloss.to_owned(),
        genus,
        differentiae,
        structures,
        source_id: sense.source_id.clone(),
        source_url: sense.source_url.clone(),
        sha256: sense.sha256.clone(),
        license_name: sense.license_name.clone(),
        depth: sense.depth,
    })
}

/// `"concept:<slug of lemma>"`.
#[must_use]
pub fn concept_id(lemma: &str) -> String {
    let mut id = String::from("concept:");
    let mut previous_dash = false;
    for character in lemma.trim().chars() {
        if character.is_alphanumeric() {
            for lowered in character.to_lowercase() {
                id.push(lowered);
            }
            previous_dash = false;
        } else if !previous_dash {
            id.push('-');
            previous_dash = true;
        }
    }
    id.trim_end_matches('-').to_owned()
}

/// The genus a gloss names and the differentiae it adds.
fn read_gloss(gloss: &str) -> (Option<String>, Vec<String>) {
    for definition in relations() {
        if definition.slug != "is_a" {
            continue;
        }
        for (_, cue) in &definition.cues {
            if let Some(position) = find_cue(gloss, cue) {
                let after = gloss[position + cue.len()..].trim();
                if let Some(head) = head_term(after) {
                    return (Some(head.clone()), differentiae_of(gloss, &head));
                }
            }
        }
    }
    head_term(gloss).map_or_else(
        || (None, Vec::new()),
        |head| {
            let differentiae = differentiae_of(gloss, &head);
            (Some(head), differentiae)
        },
    )
}

/// The head of the opening noun phrase, with any seeded determiner removed.
fn head_term(text: &str) -> Option<String> {
    let mut rest = text.trim();
    for determiner in determiners() {
        let lowered = rest.to_lowercase();
        let prefix = format!("{determiner} ");
        if lowered.starts_with(&prefix) {
            rest = rest[prefix.len()..].trim_start();
            break;
        }
    }
    rest.split(crate::source_walk::is_word_boundary)
        .find(|token| !token.is_empty())
        .map(str::to_lowercase)
}

/// The clauses of a gloss that follow its genus, each one a differentia.
fn differentiae_of(gloss: &str, genus: &str) -> Vec<String> {
    let lowered = gloss.to_lowercase();
    let after = lowered
        .find(genus)
        .map_or(gloss, |position| &gloss[position + genus.len()..]);
    let mut out: Vec<String> = Vec::new();
    for definition in relations() {
        if definition.slug == "is_a" {
            continue;
        }
        for (_, cue) in &definition.cues {
            if let Some(position) = find_cue(after, cue) {
                let tail = after[position + cue.len()..].trim();
                if !tail.is_empty() && !out.contains(&tail.to_owned()) {
                    out.push(tail.to_owned());
                }
            }
        }
    }
    if out.is_empty() {
        let tail = after.trim();
        if !tail.is_empty() {
            out.push(tail.to_owned());
        }
    }
    out
}

/// The byte offset of `cue` in `text`, matched case-insensitively and only on
/// word boundaries, so `"a"` never matches inside `"that"`.
fn find_cue(text: &str, cue: &str) -> Option<usize> {
    let lowered = text.to_lowercase();
    let needle = cue.to_lowercase();
    let mut from = 0_usize;
    while let Some(relative) = lowered[from..].find(&needle) {
        let start = from + relative;
        let end = start + needle.len();
        let before_ok = start == 0
            || lowered[..start]
                .chars()
                .next_back()
                .is_some_and(crate::source_walk::is_word_boundary);
        let after_ok = end == lowered.len()
            || lowered[end..]
                .chars()
                .next()
                .is_some_and(crate::source_walk::is_word_boundary);
        if before_ok && after_ok {
            return Some(start);
        }
        from = end;
    }
    None
}

/// Relations evidenced between two grounded terms in one clause, from the cues
/// declared in `data/seed/formalization-relations.lino`. Never invents a
/// relation the cues do not evidence.
#[must_use]
pub fn relations_in(clause: &Segment, graph: &ConceptGraph) -> Vec<ExtractedRelation> {
    let mut out: Vec<ExtractedRelation> = Vec::new();
    for definition in relations() {
        for (language, cue) in &definition.cues {
            let Some(position) = find_cue(&clause.text, cue) else {
                continue;
            };
            let before = &clause.text[..position];
            let after = &clause.text[position + cue.len()..];
            let (Some(subject), Some(object)) =
                (grounded_last(graph, before), grounded_first(graph, after))
            else {
                continue;
            };
            let id = crate::engine::stable_id(
                "relation",
                &[definition.slug.as_str(), subject.as_str(), object.as_str()].join("\u{1f}"),
            );
            if out.iter().any(|relation| relation.id == id) {
                continue;
            }
            out.push(ExtractedRelation {
                id,
                kind: definition.slug.clone(),
                subject,
                object,
                cue: cue.clone(),
                source_span: crate::formalization::needs::span(
                    &graph.doc_id,
                    clause.start,
                    clause.end,
                ),
                language: language.clone(),
            });
        }
    }
    out.sort_by(|left, right| left.id.cmp(&right.id));
    out
}

/// The last grounded term mentioned in `text`, or nothing.
fn grounded_last(graph: &ConceptGraph, text: &str) -> Option<String> {
    grounded_terms(graph, text).last().cloned()
}

/// The first grounded term mentioned in `text`, or nothing.
fn grounded_first(graph: &ConceptGraph, text: &str) -> Option<String> {
    grounded_terms(graph, text).into_iter().next()
}

/// Every id the graph grounded that `text` mentions, in order of mention.
fn grounded_terms(graph: &ConceptGraph, text: &str) -> Vec<String> {
    let lowered = text.to_lowercase();
    let mut found: Vec<(usize, String)> = Vec::new();
    for concept in &graph.concepts {
        if let Some(position) = find_cue(&lowered, &concept.label.to_lowercase()) {
            found.push((position, concept.id.clone()));
        }
    }
    for entity in &graph.entities {
        if let Some(position) = find_cue(&lowered, &entity.label.to_lowercase()) {
            found.push((position, entity.id.clone()));
        }
    }
    found.sort_by_key(|(position, _)| *position);
    found.into_iter().map(|(_, id)| id).collect()
}
