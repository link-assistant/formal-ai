//! Requirement-to-part discovery for normalized coding-task specifications.

use std::collections::BTreeSet;

use crate::coding::function_catalog::python_docs::{StdlibIndex, StdlibPart};
use crate::coding::function_catalog::wikifunctions::{FunctionMatch, FunctionPart, Implementation};
use crate::coding::recurrence::Recurrence;
use crate::coding::task_spec::{CodingTaskSpec, Example};
use crate::concept_lookup::LookupOutcome;
use crate::links_format::push_lino_node;
use crate::needs::{NeedKind, NeedState};
use crate::source_walk::{LookupBounds, SourceLookup};

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

/// The state a need carries when every declared source was consulted about a
/// surface and none of them published a sense for it.
///
/// Plan 04 L3 merged the coding path's free-text status into the contract
/// enum, so this is now a name for the state rather than a second vocabulary
/// spelling the same word.
pub const UNSERVED: NeedState = NeedState::Unsatisfiable;

/// The candidate kind a retrieved gloss carries.
pub const CONCEPT_SENSE_KIND: &str = "concept_sense";

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

/// The one need record in the tree (issue #1138, plan 00 leaf C1, plan 04 L3).
///
/// Before this the coding path declared a second `ConceptNeed` struct with a
/// free-text `status`, so "the system lacks X" existed twice with two
/// vocabularies. It is now the contract record, and the coding-path extras --
/// the structures the sentence reduced to and the parts offered for them -- sit
/// *beside* it in [`ConceptRequirement`] rather than inside the ledger row,
/// which is what keeps `recipe_interpreter`'s event-for-event parity (R343)
/// untouched.
pub use crate::needs::Need as ConceptNeed;

/// One requirement sentence: the need it raised, and what the coding path found
/// for it.
#[derive(Debug, Clone, PartialEq)]
pub struct ConceptRequirement {
    /// The contract need row.
    pub need: ConceptNeed,
    /// Seeded structural meanings the sentence reduced to.
    pub structures: Vec<StructuralMeaning>,
    /// Executable parts offered for it, best first.
    pub candidates: Vec<CandidatePart>,
}

impl ConceptRequirement {
    /// A requirement row in the state the discovery pass observed.
    #[must_use]
    pub fn new(
        phrase: &str,
        language: &str,
        state: NeedState,
        structures: Vec<StructuralMeaning>,
        candidates: Vec<CandidatePart>,
    ) -> Self {
        let mut need = ConceptNeed::raised(NeedKind::Concept, phrase, language, "coding:discovery");
        need.state = state;
        Self {
            need,
            structures,
            candidates,
        }
    }

    /// The surface this requirement is about.
    #[must_use]
    pub fn phrase(&self) -> &str {
        &self.need.subject
    }

    /// The seed slug of the need's state.
    #[must_use]
    pub const fn status(&self) -> &'static str {
        self.need.state.slug()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConceptMap {
    pub needs: Vec<ConceptRequirement>,
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

    /// What this map says the program must compute, independent of the language
    /// the requirement was written in.
    ///
    /// **Retrieved senses are deliberately not part of it (issue #1138, plan 01
    /// L10).** A `concept_sense` candidate is named for the surface it
    /// answered — `concept_sense:<surface as that language writes it>` —
    /// because that surface is what was asked about and what the provenance
    /// attaches to. Folding those ids into the identity
    /// would make the identity a function of *which language asked* and of
    /// *which source happened to answer*, so five paraphrases of one task could
    /// never share one identity however well retrieval worked: the property
    /// plan 01 L10 exists to establish would be unreachable by construction.
    ///
    /// The identity is therefore over the structures the sentence reduced to
    /// and the executable parts offered for them. That retrieval *happened* is
    /// a separate claim, carried by [`ConceptMap::evidence`] and by the need
    /// rows, and the L10 case asserts both: one identity, and non-empty
    /// evidence in every language a source serves.
    #[must_use]
    pub fn identity(&self) -> String {
        let mut structures = self.structure_ids();
        structures.sort();
        let mut parts: Vec<String> = unique(
            self.needs
                .iter()
                .flat_map(|need| need.candidates.iter())
                .filter(|candidate| candidate.kind != CONCEPT_SENSE_KIND)
                .map(|candidate| candidate.id.clone()),
        );
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
            push_lino_node(&mut out, 4, "phrase", Some(need.phrase()));
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
            push_lino_node(&mut out, 4, "status", Some(need.status()));
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
pub fn discover(
    spec: &CodingTaskSpec,
    catalog: &DiscoveryCatalog,
    lookup: &mut dyn SourceLookup,
) -> ConceptMap {
    discover_with_lookup(spec, catalog, lookup, DiscoveryBounds::default())
}

#[must_use]
pub fn discover_with_lookup(
    spec: &CodingTaskSpec,
    catalog: &DiscoveryCatalog,
    lookup: &mut dyn SourceLookup,
    bounds: DiscoveryBounds,
) -> ConceptMap {
    let sentences = if spec.requirement_sentences.is_empty() {
        vec![spec.name.replace('_', " ")]
    } else {
        spec.requirement_sentences.clone()
    };
    let mut evidence = Vec::new();
    let mut unserved: Vec<ConceptRequirement> = Vec::new();
    let asked_language = spec.prose_language.clone();
    let lookup_bounds = LookupBounds {
        max_depth: bounds.max_depth,
        ..LookupBounds::default()
    };
    let mut needs: Vec<ConceptRequirement> = sentences
        .into_iter()
        .map(|phrase| {
            let normalized = crate::engine::normalize_prompt(&phrase);
            let structures = structures_for(&normalized);
            let query = discovery_query(spec, &structures);
            let recurrence_query = format!("{phrase} {}", spec.name.replace('_', " "));
            let mut candidates = candidates_for(&query, &recurrence_query, catalog);
            // Ask about every surface the sentence left unresolved, not only
            // about sentences that matched nothing at all: a sentence can be
            // four-fifths understood and still turn on the word nobody knows
            // (issue #1138, plan 01 L8). `max_pages` is the bound on how many
            // such words one specification may ask about.
            let unresolved = unresolved_surfaces(&normalized, &structures);
            let mut asked = 0_usize;
            let mut consulted_sources = false;
            for surface in &unresolved {
                if evidence.len() >= bounds.max_pages || bounds.max_depth == 0 {
                    break;
                }
                let need = ConceptNeed::raised(
                    NeedKind::Concept,
                    surface,
                    &asked_language,
                    "coding:discovery",
                );
                match lookup.lookup(&need, &lookup_bounds) {
                    LookupOutcome::Found(senses) => {
                        if let Some(sense) = senses.into_iter().next() {
                            consulted_sources = true;
                            let found = ConceptEvidence {
                                phrase: surface.clone(),
                                definition: sense.gloss,
                                source_url: sense.source_url,
                                depth: sense.depth,
                            };
                            candidates.push(concept_candidate(&found));
                            evidence.push(found);
                            asked += 1;
                        }
                    }
                    LookupOutcome::NotFound { consulted } if !consulted.is_empty() => {
                        consulted_sources = true;
                        // Every declared source was consulted about this surface
                        // and none published a sense for it. That is a need the
                        // map carries openly — an unserved language, measured
                        // rather than argued (issue #1138, plan 01 L10) — and not
                        // an absence the map is silent about.
                        unserved.push(ConceptRequirement::new(
                            &unserved_phrase(surface, &asked_language),
                            &asked_language,
                            UNSERVED,
                            Vec::new(),
                            Vec::new(),
                        ));
                    }
                    LookupOutcome::NotFound { .. } => {}
                }
            }
            candidates.sort_by(|left, right| {
                candidate_kind_rank(&left.kind).cmp(&candidate_kind_rank(&right.kind))
            });
            let state = if structures.is_empty() && candidates.is_empty() && asked == 0 {
                // `Open` is "nothing was tried"; `Unsatisfiable` is "every
                // declared source was asked and none of them serves this".
                // Collapsing the two would make an honest refusal by the
                // sources indistinguishable from a walk that never ran
                // (issue #1138, plan 01 L10).
                if consulted_sources && !unresolved.is_empty() {
                    UNSERVED
                } else {
                    NeedState::Open
                }
            } else {
                NeedState::Satisfied
            };
            ConceptRequirement::new(&phrase, &asked_language, state, structures, candidates)
        })
        .collect();
    needs.extend(unserved);
    ConceptMap { needs, evidence }
}

/// The phrase an unserved need carries: the surface, and the language nobody
/// served it in.
///
/// `"<surface>" in <language>` is not prose: it is the pair the need is about,
/// and it is what a reader of the map's Links Notation needs in order to go and
/// check the claim.
fn unserved_phrase(surface: &str, language: &str) -> String {
    if language.is_empty() {
        surface.to_owned()
    } else {
        format!("{surface}@{language}")
    }
}

#[must_use]
pub fn structural_meanings() -> Vec<StructuralMeaning> {
    let Some(text) =
        crate::coding::fragment_catalog::bootstrap_seed_text("meanings-coding-structure.lino")
    else {
        return Vec::new();
    };
    let root = crate::seed::parser::parse_lino(&text);
    root.children
        .iter()
        .find(|node| node.name == "meanings")
        .map(|container| {
            container
                .children
                .iter()
                .map(|node| StructuralMeaning {
                    id: if node.name == "meaning" {
                        node.id.clone()
                    } else {
                        node.name.clone()
                    },
                    idiom: node.find_child_value("idiom").to_owned(),
                    grounding: node.find_child_value("grounding").to_owned(),
                })
                .collect()
        })
        .unwrap_or_default()
}

#[must_use]
pub fn structures_for(normalized: &str) -> Vec<StructuralMeaning> {
    let mut matched = crate::seed::lexicon()
        .meanings_with_role(crate::seed::ROLE_CODING_STRUCTURE)
        .filter(|meaning| {
            meaning.evidenced_in(normalized)
                || meaning.words().any(|surface| {
                    let normalized_surface = crate::engine::normalize_prompt(surface);
                    normalized_surface_present(normalized, &normalized_surface)
                        || normalized_slotted_surface_present(normalized, surface)
                        || fuzzy_single_token_present(normalized, &normalized_surface)
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

/// Match a data-declared `before … after` surface with a short phrase in its
/// slot. This lets a language express relations such as “for every item”
/// without turning either half (for example, a bare “for”) into evidence on
/// its own. The bounded slot prevents two unrelated clauses from being joined.
fn normalized_slotted_surface_present(normalized: &str, surface: &str) -> bool {
    let Some((before, after)) = surface.split_once('…') else {
        return false;
    };
    let before = crate::engine::normalize_prompt(before);
    let after = crate::engine::normalize_prompt(after);
    let before = before.trim();
    let after = after.trim();
    if before.is_empty() || after.is_empty() {
        return false;
    }
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    let before = before.split_whitespace().collect::<Vec<_>>();
    let after = after.split_whitespace().collect::<Vec<_>>();
    tokens
        .windows(before.len())
        .enumerate()
        .any(|(start, words)| {
            if words != before {
                return false;
            }
            let slot_start = start + before.len();
            (1..=6).any(|slot_width| {
                let suffix_start = slot_start + slot_width;
                tokens
                    .get(suffix_start..suffix_start + after.len())
                    .is_some_and(|words| words == after)
            })
        })
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
        CONCEPT_SENSE_KIND => 4,
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

/// Unresolved surfaces of one requirement sentence, in sentence order — the
/// words the coding path must ask about even when the rest of the sentence was
/// understood (issue #1138, plan 01 L8).
///
/// Before this, the lookup was consulted only when a sentence matched *nothing
/// at all*. A requirement sentence whose verb, whose `true` and whose `word`
/// are all seeded, and which turns on one noun nobody has ever heard of,
/// matched enough to count as understood and therefore asked about nothing.
/// Understanding four words out of five is not understanding the sentence; the
/// fifth is the need.
#[must_use]
pub fn unresolved_surfaces(normalized: &str, structures: &[StructuralMeaning]) -> Vec<String> {
    let normalized = crate::engine::normalize_prompt(normalized);
    let lexicon = crate::seed::lexicon();
    let accounted: BTreeSet<String> = structures
        .iter()
        .filter_map(|structure| lexicon.meaning(&structure.id))
        .flat_map(|meaning| {
            meaning
                .words()
                .flat_map(|surface| {
                    surface
                        .split_whitespace()
                        .map(str::to_lowercase)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect();
    crate::concept_lookup::unknown_surfaces(&normalized, "")
        .into_iter()
        .filter(|surface| !accounted.contains(surface))
        .collect()
}

/// A retrieved sense, as a candidate the composer can read.
///
/// The candidate's kind is `concept_sense`, which ranks *after* `source_program`, so a
/// retrieved definition never outranks a retrieved implementation (issue #1138,
/// plan 01 L9).
///
/// `code` is `None` by construction. The Rosetta Code precedent
/// (`docs/meta-algorithm.md:897-901`) is binding: retrieved text under a
/// share-alike or non-commercial licence is shown and attributed, never
/// inserted into a generated program. A `concept_sense` exists to let the
/// composer reach *seeded structure* through a definition, not to supply
/// program text.
///
/// **Recorded deviation from plan 01 L9.** The plan adds `source_id`,
/// `sha256`, `fetched_at` and `license` fields to [`ConceptEvidence`].
/// `tests/unit/coding_discovery/concepts.rs` constructs the record with a
/// four-field struct literal and no `..Default::default()`, so adding a field
/// breaks a test written in wave T to pin this shape. The provenance is
/// therefore *derived* rather than duplicated: the licence comes from the
/// sources registry entry whose id the evidence URL names, which has the
/// further virtue that a licence can only ever be stated once, in the registry.
#[must_use]
pub fn concept_candidate(evidence: &ConceptEvidence) -> CandidatePart {
    let (source_id, license) = registry_licence(&evidence.source_url);
    CandidatePart {
        id: format!("concept_sense:{}", evidence.phrase),
        kind: CONCEPT_SENSE_KIND.to_owned(),
        label: evidence.definition.clone(),
        language: None,
        // A gloss is evidence, never a program.
        code: None,
        callable_name: None,
        source_tests: Vec::new(),
        license,
        source_url: evidence.source_url.clone(),
        // The digest of the quoted gloss, which is what this candidate carries.
        // The digest of the whole page lives on `ConceptSense` and in the sense
        // ledger; claiming it here would attribute a page to a sentence.
        sha256: crate::source_fetch::sha256_hex(evidence.definition.as_bytes()),
        fetched_at: String::new(),
        score: f64::from(u8::from(!source_id.is_empty())),
    }
}

/// The registry source an evidence URL belongs to, and the licence it declares.
///
/// Matched by registry id inside the URL's host rather than by a host list in
/// Rust, so a source that changes its API host keeps its licence and a source
/// nobody declared is reported as having no stated licence rather than being
/// given one.
fn registry_licence(source_url: &str) -> (String, String) {
    let host = source_url
        .split_once("://")
        .map_or(source_url, |(_, rest)| {
            rest.split('/').next().unwrap_or(rest)
        });
    crate::seed::source_registry()
        .into_iter()
        .find(|record| record.host() == host || host.contains(&record.id))
        .map_or_else(
            || (String::new(), String::from("license_unstated")),
            |record| (record.id, record.license_name),
        )
}
