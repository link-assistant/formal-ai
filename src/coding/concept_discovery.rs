//! Requirement-to-part discovery for normalized coding-task specifications.

use std::collections::BTreeSet;

use crate::coding::function_catalog::python_docs::{StdlibIndex, StdlibPart};
use crate::coding::function_catalog::wikifunctions::{FunctionMatch, FunctionPart, Implementation};
use crate::coding::recurrence::Recurrence;
use crate::coding::task_spec::{CodingTaskSpec, Example};
use crate::links_format::push_lino_node;

const STRUCTURES: &str = include_str!("../../data/seed/meanings-coding-structure.lino");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuralMeaning {
    pub id: String,
    pub idiom: String,
    pub grounding: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogFunction {
    pub definition: FunctionPart,
    pub implementations: Vec<Implementation>,
    pub recurrence: Option<Recurrence>,
    pub source_tests: Vec<Example>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DiscoveryCatalog {
    pub stdlib: StdlibIndex,
    pub function_matches: Vec<FunctionMatch>,
    pub functions: Vec<CatalogFunction>,
    pub source_candidates: Vec<CandidatePart>,
}

impl DiscoveryCatalog {
    #[must_use]
    pub const fn new(
        stdlib: StdlibIndex,
        function_matches: Vec<FunctionMatch>,
        functions: Vec<CatalogFunction>,
    ) -> Self {
        Self {
            stdlib,
            function_matches,
            functions,
            source_candidates: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_source_candidates(mut self, source_candidates: Vec<CandidatePart>) -> Self {
        self.source_candidates = source_candidates;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiscoveryBounds {
    pub max_depth: usize,
    pub max_pages: usize,
}

impl Default for DiscoveryBounds {
    fn default() -> Self {
        Self {
            max_depth: 2,
            max_pages: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptEvidence {
    pub phrase: String,
    pub definition: String,
    pub source_url: String,
    pub depth: usize,
}

pub trait UnknownConceptLookup {
    fn lookup(&mut self, phrase: &str, depth: usize) -> Option<ConceptEvidence>;
}

struct NoLookup;

impl UnknownConceptLookup for NoLookup {
    fn lookup(&mut self, _phrase: &str, _depth: usize) -> Option<ConceptEvidence> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CandidatePart {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub language: Option<String>,
    pub code: Option<String>,
    pub callable_name: Option<String>,
    pub source_tests: Vec<Example>,
    pub license: String,
    pub source_url: String,
    pub sha256: String,
    pub fetched_at: String,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConceptNeed {
    pub phrase: String,
    pub structures: Vec<StructuralMeaning>,
    pub candidates: Vec<CandidatePart>,
    pub status: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConceptMap {
    pub needs: Vec<ConceptNeed>,
    pub evidence: Vec<ConceptEvidence>,
}

impl ConceptMap {
    #[must_use]
    pub fn structure_ids(&self) -> Vec<String> {
        unique(
            self.needs
                .iter()
                .flat_map(|need| need.structures.iter().map(|structure| structure.id.clone())),
        )
    }

    #[must_use]
    pub fn candidate_ids(&self) -> Vec<String> {
        unique(
            self.needs
                .iter()
                .flat_map(|need| need.candidates.iter().map(|candidate| candidate.id.clone())),
        )
    }

    #[must_use]
    pub fn identity(&self) -> String {
        let mut structures = self.structure_ids();
        structures.sort();
        let mut parts = self.candidate_ids();
        parts.sort();
        format!(
            "structures={};parts={}",
            structures.join(","),
            parts.join(",")
        )
    }

    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut out = String::new();
        push_lino_node(&mut out, 0, "concept_map", None);
        for (index, need) in self.needs.iter().enumerate() {
            let need_id = format!("need_{}", index + 1);
            push_lino_node(&mut out, 2, "need", Some(&need_id));
            push_lino_node(&mut out, 4, "phrase", Some(&need.phrase));
            for structure in &need.structures {
                push_lino_node(&mut out, 4, "structure", Some(&structure.id));
                push_lino_node(&mut out, 6, "idiom", Some(&structure.idiom));
                push_lino_node(&mut out, 6, "grounding", Some(&structure.grounding));
            }
            for candidate in &need.candidates {
                push_lino_node(&mut out, 4, "candidate", Some(&candidate.id));
                push_lino_node(&mut out, 6, "kind", Some(&candidate.kind));
                push_lino_node(&mut out, 6, "license", Some(&candidate.license));
                push_lino_node(&mut out, 6, "source_url", Some(&candidate.source_url));
                push_lino_node(&mut out, 6, "sha256", Some(&candidate.sha256));
            }
            push_lino_node(&mut out, 2, "need_ledger", Some(&need_id));
            push_lino_node(&mut out, 4, "status", Some(&need.status));
        }
        for evidence in &self.evidence {
            push_lino_node(&mut out, 2, "concept_evidence", Some(&evidence.phrase));
            push_lino_node(&mut out, 4, "definition", Some(&evidence.definition));
            push_lino_node(&mut out, 4, "source_url", Some(&evidence.source_url));
            push_lino_node(&mut out, 4, "depth", Some(&evidence.depth.to_string()));
        }
        out.trim_end().to_owned()
    }
}

#[must_use]
pub fn discover(spec: &CodingTaskSpec, catalog: &DiscoveryCatalog) -> ConceptMap {
    discover_with_lookup(spec, catalog, &mut NoLookup, DiscoveryBounds::default())
}

#[must_use]
pub fn discover_with_lookup<L: UnknownConceptLookup>(
    spec: &CodingTaskSpec,
    catalog: &DiscoveryCatalog,
    lookup: &mut L,
    bounds: DiscoveryBounds,
) -> ConceptMap {
    let sentences = if spec.requirement_sentences.is_empty() {
        vec![spec.name.replace('_', " ")]
    } else {
        spec.requirement_sentences.clone()
    };
    let mut evidence = Vec::new();
    let needs = sentences
        .into_iter()
        .map(|phrase| {
            let normalized = crate::engine::normalize_prompt(&phrase);
            let structures = structures_for(&normalized);
            let query = discovery_query(spec, &structures);
            let recurrence_query = format!("{phrase} {}", spec.name.replace('_', " "));
            let candidates = candidates_for(&query, &recurrence_query, catalog);
            if structures.is_empty()
                && candidates.is_empty()
                && evidence.len() < bounds.max_pages
                && bounds.max_depth > 0
                && let Some(found) = lookup.lookup(&phrase, 1)
            {
                evidence.push(found);
            }
            let status = if structures.is_empty()
                && candidates.is_empty()
                && !evidence.iter().any(|item| item.phrase == phrase)
            {
                "blocked"
            } else {
                "satisfied"
            };
            ConceptNeed {
                phrase,
                structures,
                candidates,
                status: status.to_owned(),
            }
        })
        .collect();
    ConceptMap { needs, evidence }
}

#[must_use]
pub fn structural_meanings() -> Vec<StructuralMeaning> {
    let root = crate::seed::parser::parse_lino(STRUCTURES);
    root.children
        .iter()
        .find(|node| node.name == "meanings")
        .map(|container| {
            container
                .children
                .iter()
                .map(|node| StructuralMeaning {
                    id: node.name.clone(),
                    idiom: node.find_child_value("idiom").to_owned(),
                    grounding: node.find_child_value("grounding").to_owned(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn structures_for(normalized: &str) -> Vec<StructuralMeaning> {
    let mut matched = crate::seed::lexicon()
        .meanings_with_role(crate::seed::ROLE_CODING_STRUCTURE)
        .filter(|meaning| {
            meaning.evidenced_in(normalized)
                || meaning.words().any(|surface| {
                    let surface = crate::engine::normalize_prompt(surface);
                    normalized_surface_present(normalized, &surface)
                        || fuzzy_single_token_present(normalized, &surface)
                })
        })
        .map(|meaning| meaning.slug.as_str())
        .collect::<BTreeSet<_>>();
    // "Distinct pair" describes how pairs are enumerated, not a request to
    // deduplicate the input collection before enumerating them.
    if matched.contains("pairwise_distinct") {
        matched.remove("distinct_elements");
    }
    // An existential cue such as "at least one" is the governing quantifier
    // when a nested phrase such as "each pair" also contributes an
    // incidental universal cue.
    if matched.contains("quantifier_any") {
        matched.remove("quantifier_all");
    }
    // In an inclusive range request, "count to N" denotes enumeration. It
    // is not the cardinality reduction used by "count distinct items".
    if matched.contains("range_inclusive") {
        matched.remove("reduce_count");
        matched.remove("membership");
    }
    structural_meanings()
        .into_iter()
        .filter(|structure| matched.contains(structure.id.as_str()))
        .collect()
}

fn normalized_surface_present(normalized: &str, surface: &str) -> bool {
    !surface.is_empty() && format!(" {normalized} ").contains(&format!(" {surface} "))
}

fn fuzzy_single_token_present(normalized: &str, surface: &str) -> bool {
    surface.len() >= 8
        && surface.bytes().all(|byte| byte.is_ascii_alphabetic())
        && normalized
            .split_whitespace()
            .filter(|token| token.bytes().all(|byte| byte.is_ascii_alphabetic()))
            .any(|token| edit_distance_at_most_one(token, surface))
}

fn edit_distance_at_most_one(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    let left = left.as_bytes();
    let right = right.as_bytes();
    if left.len().abs_diff(right.len()) > 1 {
        return false;
    }
    let (shorter, longer) = if left.len() <= right.len() {
        (left, right)
    } else {
        (right, left)
    };
    let mut short_index = 0;
    let mut long_index = 0;
    let mut edits = 0;
    while short_index < shorter.len() && long_index < longer.len() {
        if shorter[short_index] == longer[long_index] {
            short_index += 1;
            long_index += 1;
            continue;
        }
        edits += 1;
        if edits > 1 {
            return false;
        }
        if shorter.len() == longer.len() {
            short_index += 1;
        }
        long_index += 1;
    }
    edits + usize::from(long_index < longer.len()) <= 1
}

fn discovery_query(spec: &CodingTaskSpec, structures: &[StructuralMeaning]) -> String {
    let mut terms = vec![spec.name.replace('_', " ")];
    let lexicon = crate::seed::lexicon();
    terms.extend(structures.iter().filter_map(|structure| {
        lexicon
            .meaning(&structure.id)
            .and_then(|meaning| meaning.word_in("en"))
            .map(str::to_owned)
    }));
    terms.join(" ")
}

fn candidates_for(
    query: &str,
    recurrence_query: &str,
    catalog: &DiscoveryCatalog,
) -> Vec<CandidatePart> {
    let mut out = Vec::new();
    out.extend(catalog.source_candidates.iter().cloned());
    for part in catalog.stdlib.parts_for_phrase(query) {
        let score = part.match_score(query);
        if score >= 3.0 {
            out.push(stdlib_candidate(part, score));
        }
    }
    for matched in &catalog.function_matches {
        let score = overlap_score(query, &matched.label);
        if score >= 1.0 {
            out.push(CandidatePart {
                id: matched.page_title.clone(),
                kind: "wikifunctions_definition".to_owned(),
                label: matched.label.clone(),
                language: None,
                code: None,
                callable_name: None,
                source_tests: Vec::new(),
                license: "CC0-1.0".to_owned(),
                source_url: matched.source_url.clone(),
                sha256: matched.sha256.clone(),
                fetched_at: matched.fetched_at.clone(),
                score,
            });
        }
    }
    for function in &catalog.functions {
        let labels = function
            .definition
            .labels
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let score = labels
            .iter()
            .map(|label| overlap_score(query, label))
            .max_by(f64::total_cmp)
            .unwrap_or(0.0);
        if score >= 1.0 {
            for implementation in &function.implementations {
                if implementation.language != "python" {
                    continue;
                }
                out.push(CandidatePart {
                    id: implementation.zid.clone(),
                    kind: "wikifunctions_implementation".to_owned(),
                    label: labels.first().cloned().unwrap_or_default(),
                    language: Some(implementation.language.clone()),
                    code: Some(implementation.code.clone()),
                    callable_name: None,
                    source_tests: Vec::new(),
                    license: implementation.license.clone(),
                    source_url: implementation.source_url.clone(),
                    sha256: implementation.sha256.clone(),
                    fetched_at: implementation.fetched_at.clone(),
                    score,
                });
            }
        }
        if let Some(recurrence) = &function.recurrence {
            let recurrence_score = labels
                .iter()
                .map(|label| overlap_score(recurrence_query, label))
                .max_by(f64::total_cmp)
                .unwrap_or(0.0);
            if recurrence_score < 1.0 {
                continue;
            }
            let callable_name = recurrence.suggested_identifier();
            out.push(CandidatePart {
                id: recurrence.source_implementation_zid.clone(),
                kind: "wikifunctions_recurrence".to_owned(),
                label: recurrence.source_label.clone(),
                language: Some("python".to_owned()),
                code: Some(recurrence.render_python(&callable_name)),
                callable_name: Some(callable_name),
                source_tests: function.source_tests.clone(),
                license: recurrence.license.clone(),
                source_url: recurrence.source_url.clone(),
                sha256: recurrence.source_sha256.clone(),
                fetched_at: recurrence.fetched_at.clone(),
                score: recurrence_score,
            });
        }
    }
    out.sort_by(|left, right| {
        candidate_kind_rank(&left.kind)
            .cmp(&candidate_kind_rank(&right.kind))
            .then_with(|| left.id.cmp(&right.id))
    });
    out.dedup_by(|left, right| left.id == right.id);
    out
}

fn overlap_score(left: &str, right: &str) -> f64 {
    u32::try_from(token_overlap(left, right)).map_or_else(|_| f64::from(u32::MAX), f64::from)
}

fn candidate_kind_rank(kind: &str) -> usize {
    match kind {
        "stdlib" => 0,
        "wikifunctions_definition" => 1,
        "source_program" => 2,
        _ => 3,
    }
}

fn stdlib_candidate(part: &StdlibPart, score: f64) -> CandidatePart {
    CandidatePart {
        id: part.symbol.clone(),
        kind: "stdlib".to_owned(),
        label: part.description.clone(),
        language: Some("python".to_owned()),
        code: None,
        callable_name: None,
        source_tests: Vec::new(),
        license: part.license.clone(),
        source_url: part.source_url.clone(),
        sha256: part.sha256.clone(),
        fetched_at: part.fetched_at.clone(),
        score,
    }
}

fn token_overlap(left: &str, right: &str) -> usize {
    let left = tokens(left);
    let right = tokens(right);
    left.intersection(&right).count()
}

fn tokens(value: &str) -> BTreeSet<String> {
    crate::engine::normalize_prompt(value)
        .split_whitespace()
        .filter(|token| token.len() > 2)
        .map(|token| token.trim_end_matches('s').to_owned())
        .collect()
}

fn unique(values: impl Iterator<Item = String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values.filter(|value| seen.insert(value.clone())).collect()
}
