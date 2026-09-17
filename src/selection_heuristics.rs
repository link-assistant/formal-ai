//! The heuristics the recursive core uses to choose among candidates, to choose
//! the next experiment, and to split a task it cannot solve directly
//! (#491, #901, #802, #453; plan 12).
//!
//! A heuristic is registry *data*, not a `sort_by` at each call site: the key
//! order, the roles and the precedence all live in
//! `data/meta/selection-heuristics.lino`, so reordering the dimensions is a
//! `.lino` edit. Correctness comes first and cost second, and no ranking key may
//! depend on a wall clock.

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::execution_evidence::Evidence;
use crate::links_format::format_lino_record;
use crate::relative_meta_logic::SourceTier;
use crate::seed::parser::parse_lino;

mod refutation;
mod splitting;
mod triz;

/// Which of the three selection jobs a heuristic performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeuristicRole {
    Rank,
    Experiment,
    Split,
}

impl HeuristicRole {
    /// Stable slug, exactly as `data/meta/selection-heuristics.lino` declares it.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Rank => "rank",
            Self::Experiment => "experiment",
            Self::Split => "split",
        }
    }

    /// The role a catalog row declares, or `None` when the slug is unknown.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "rank" => Some(Self::Rank),
            "experiment" => Some(Self::Experiment),
            "split" => Some(Self::Split),
            _ => None,
        }
    }
}

/// One heuristic as it appears in the registry: link data, not a branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeuristicMethod {
    pub name: String,
    pub role: HeuristicRole,
    /// Declaration order in `data/meta/selection-heuristics.lino` is precedence.
    pub order: usize,
    /// The situation slugs this heuristic applies to; empty means always.
    pub applies_when: Vec<String>,
    /// Role-specific parameters, read as data (e.g. the least-action key order).
    pub parameters: Vec<(String, String)>,
}

impl HeuristicMethod {
    /// The declared value of one parameter, when the row carries it.
    #[must_use]
    pub fn parameter(&self, key: &str) -> Option<&str> {
        self.parameters
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.as_str())
    }

    /// Whether this heuristic applies in `situation`. An empty `applies_when`
    /// means always, so an empty situation matches the unconditional rows only.
    #[must_use]
    pub fn applies_in(&self, situation: &str) -> bool {
        self.applies_when.is_empty() || self.applies_when.iter().any(|slug| slug == situation)
    }

    /// Links Notation projection, as the registry event renders it.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![
            ("record_type", String::from(HEURISTIC_RECORD_TYPE)),
            ("role", self.role.slug().to_string()),
            ("order", self.order.to_string()),
        ];
        for slug in &self.applies_when {
            pairs.push(("applies_when", slug.clone()));
        }
        for (key, value) in &self.parameters {
            pairs.push((key.as_str(), value.clone()));
        }
        format_lino_record(&self.name, &pairs)
    }
}

/// Field names the catalog reads structurally rather than as parameters.
const STRUCTURAL_FIELDS: [&str; 4] = ["record_type", "role", "order", "applies_when"];

/// The `record_type` a catalog row must carry to be a heuristic.
const HEURISTIC_RECORD_TYPE: &str = "selection_heuristic";

/// The shipped catalog, embedded so the browser build reads the same data.
pub const SELECTION_HEURISTICS_LINO: &str = include_str!("../data/meta/selection-heuristics.lino");

/// Load the catalog from `data/meta/selection-heuristics.lino`.
///
/// # Errors
/// Returns the offending record and field when a row declares an unknown role
/// or a non-numeric order.
pub fn catalog_from(text: &str) -> Result<Vec<HeuristicMethod>, String> {
    let tree = parse_lino(text);
    let mut catalog = Vec::new();
    for record in &tree.children {
        if record.find_child_value("record_type") != HEURISTIC_RECORD_TYPE {
            continue;
        }
        let role_slug = record.find_child_value("role");
        let Some(role) = HeuristicRole::from_slug(role_slug) else {
            return Err(format!("{}:role:{role_slug}", record.name));
        };
        let order = record
            .find_child_value("order")
            .parse::<usize>()
            .map_err(|_| format!("{}:order", record.name))?;
        let mut applies_when = Vec::new();
        let mut parameters = Vec::new();
        for field in &record.children {
            if field.name == "applies_when" {
                applies_when.push(field.id.clone());
            } else if !STRUCTURAL_FIELDS.contains(&field.name.as_str()) {
                parameters.push((field.name.clone(), field.id.clone()));
            }
        }
        catalog.push(HeuristicMethod {
            name: record.name.clone(),
            role,
            order,
            applies_when,
            parameters,
        });
    }
    Ok(catalog)
}

/// The shipped catalog, parsed. A catalog that does not parse is an empty one:
/// the caller then falls back to the deterministic identity ordering and emits
/// `heuristic:none`, rather than inventing a key.
#[must_use]
pub fn shipped_catalog() -> Vec<HeuristicMethod> {
    catalog_from(SELECTION_HEURISTICS_LINO).unwrap_or_default()
}

/// The deterministic cost of producing or running one candidate.
///
/// Every field is wall-clock independent, because a ranking key that depends on
/// machine speed is not reproducible. R491-C3's elapsed time is reported beside
/// the ranking, never inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActionCost {
    /// Elementary steps: evaluations, expansions, recorder invocations.
    pub steps: u32,
    /// Rendered size of the artifact, in characters.
    pub code_size: usize,
    /// Deterministic resource units: nodes visited, links touched, bytes read.
    pub resource_units: u64,
    /// Number of leaves the task was split into to reach this candidate (#453).
    pub leaf_count: u32,
}

/// Correctness first, cost second. Incomplete work is never a cheaper solution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateScore {
    pub candidate_id: String,
    /// Checks satisfied out of checks declared, counted from the `Evidence`
    /// rows plan 08's self-checks emit for this candidate's derivation id.
    /// `(0, 0)` is not a pass (plan 00 §9 R17, R15).
    pub checks: (usize, usize),
    pub cost: ActionCost,
}

impl CandidateScore {
    /// Whether every declared check passed and at least one was declared.
    ///
    /// `(0, 0)` is not a pass: nothing was declared and nothing was observed.
    #[must_use]
    pub const fn satisfies(&self) -> bool {
        self.checks.1 > 0 && self.checks.0 == self.checks.1
    }

    /// Links Notation projection, so a ranking is inspectable after the fact.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let pairs: Vec<(&str, String)> = vec![
            ("record_type", String::from("candidate_score")),
            ("checks_satisfied", self.checks.0.to_string()),
            ("checks_declared", self.checks.1.to_string()),
            ("satisfies", self.satisfies().to_string()),
            ("steps", self.cost.steps.to_string()),
            ("code_size", self.cost.code_size.to_string()),
            ("resource_units", self.cost.resource_units.to_string()),
            ("leaf_count", self.cost.leaf_count.to_string()),
        ];
        format_lino_record(&self.candidate_id, &pairs)
    }

    /// One dimension of the declared key order, as a comparable integer.
    ///
    /// `candidate_index` is the candidate's position in the scored set, which is
    /// the deterministic final tie-break `src/draft_portfolio.rs` already used.
    fn dimension(&self, name: &str, index: usize) -> u128 {
        match name {
            "steps" => u128::from(self.cost.steps),
            "code_size" => self.cost.code_size as u128,
            "resource_units" => u128::from(self.cost.resource_units),
            "leaf_count" => u128::from(self.cost.leaf_count),
            _ => index as u128,
        }
    }
}

/// The parameter whose value declares the ranking key order.
const KEY_ORDER_PARAMETER: &str = "key_order";

/// Read the declared key order out of a heuristic's parameters.
fn key_order(parameters: &[(String, String)]) -> Vec<&str> {
    parameters
        .iter()
        .find(|(key, _)| key == KEY_ORDER_PARAMETER)
        .map(|(_, value)| value.split(',').map(str::trim).collect())
        .unwrap_or_default()
}

/// The sort key of one candidate under `order`, smallest first.
fn ranking_key(score: &CandidateScore, index: usize, order: &[&str]) -> Vec<u128> {
    order
        .iter()
        .map(|dimension| score.dimension(dimension, index))
        .collect()
}

/// Rank a candidate set. The key order is read from the heuristic's parameters,
/// so reordering the dimensions is a `.lino` edit.
pub trait CandidateRanker {
    fn slug(&self) -> &'static str;
    /// Indices of `scores`, best first. Only candidates with `satisfies()` may
    /// be ranked; the rest are returned separately by the caller as refuted.
    fn rank(&self, scores: &[CandidateScore], parameters: &[(String, String)]) -> Vec<usize>;
}

/// The seeded default, byte-compatible with today's behaviour.
#[derive(Debug, Clone, Copy, Default)]
pub struct LeastActionRanker;

impl CandidateRanker for LeastActionRanker {
    fn slug(&self) -> &'static str {
        "least_action"
    }

    fn rank(&self, scores: &[CandidateScore], parameters: &[(String, String)]) -> Vec<usize> {
        let mut ranked: Vec<usize> = scores
            .iter()
            .enumerate()
            .filter(|(_, score)| score.satisfies())
            .map(|(index, _)| index)
            .collect();
        let order = key_order(parameters);
        if order.is_empty() {
            // No declared key order: the deterministic identity ordering, never
            // an invented one. The caller emits `heuristic:none` beside it.
            return ranked;
        }
        ranked.sort_by_key(|index| ranking_key(&scores[*index], *index, &order));
        ranked
    }
}

/// Split a task into exactly two children, preserving every segment.
pub trait TaskSplitter {
    fn slug(&self) -> &'static str;
    /// `None` when the task cannot be split at all -- which is the atomicity
    /// judgement, and is reported rather than silently returning the input.
    fn split(&self, task: &str, parameters: &[(String, String)]) -> Option<BinarySplit>;
}

/// Exactly two children, with the balance actually achieved recorded honestly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinarySplit {
    pub left: String,
    pub right: String,
    /// Checkable segments on each side, so an unbalanced split is visible.
    pub left_weight: usize,
    pub right_weight: usize,
    /// Byte spans in the original task; the two must be contiguous and cover it.
    pub left_span: (usize, usize),
    pub right_span: (usize, usize),
}

/// One candidate approach in discovery/history order.
///
/// `semantic_terms` are the grounded language-independent meanings of the
/// approach. When they are absent, the shared summarization deduplicator uses
/// its conservative lexical signature instead of guessing equivalence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApproachObservation {
    pub approach: String,
    pub source: String,
    pub tier: SourceTier,
    pub semantic_terms: Vec<String>,
}

/// One distinct approach after conservative merging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombinedApproach {
    pub approach_id: String,
    /// First wording observed for this meaning.
    pub approach: String,
    /// First source in the supplied history, as #453 requires.
    pub first_source: String,
    /// Every distinct source in first-seen order.
    pub sources: Vec<String>,
}

/// Combine different approaches, deduplicate equivalent meanings, and retain
/// the first historical source for every merged idea (#453 R453-M4).
///
/// The operation delegates equivalence and provenance to the same reversible
/// statement deduplicator used by multi-source synthesis. Input order is the
/// history order; no timestamp is invented and no later, higher-tier source is
/// allowed to rewrite who supplied an idea first.
#[must_use]
pub fn combine_approaches(history: &[ApproachObservation]) -> Vec<CombinedApproach> {
    let observations = history
        .iter()
        .map(|observation| {
            crate::summarization::SourcedStatement::from_sentence(
                &observation.approach,
                observation.source.clone(),
                observation.tier,
            )
            .with_semantic_terms(observation.semantic_terms.clone())
        })
        .collect::<Vec<_>>();
    crate::summarization::deduplicate(&observations)
        .statements
        .into_iter()
        .filter_map(|statement| {
            let first_source = statement.variants.first()?.source.clone();
            let mut sources = Vec::new();
            for variant in &statement.variants {
                if !sources.contains(&variant.source) {
                    sources.push(variant.source.clone());
                }
            }
            Some(CombinedApproach {
                approach_id: statement.id,
                approach: statement.representative.text,
                first_source,
                sources,
            })
        })
        .collect()
}

impl BinarySplit {
    /// `|left_weight - right_weight|`. Zero is a perfect halving; the value is
    /// reported, never optimized away by dropping a segment.
    #[must_use]
    pub const fn imbalance(&self) -> usize {
        self.left_weight.abs_diff(self.right_weight)
    }

    /// Links Notation projection, so the imbalance travels with the split.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let pairs: Vec<(&str, String)> = vec![
            ("record_type", String::from("binary_split")),
            ("left", self.left.clone()),
            ("right", self.right.clone()),
            ("left_weight", self.left_weight.to_string()),
            ("right_weight", self.right_weight.to_string()),
            ("imbalance", self.imbalance().to_string()),
            (
                "left_span",
                format!("{},{}", self.left_span.0, self.left_span.1),
            ),
            (
                "right_span",
                format!("{},{}", self.right_span.0, self.right_span.1),
            ),
        ];
        format_lino_record("binary_split", &pairs)
    }
}

/// Fold n-ary task decomposition into the declared balanced binary tree.
///
/// Cut the output of `task_decomposition::split_once_checkable` at the boundary
/// that minimizes `imbalance()`.
///
/// Every segment survives on exactly one side, so this is a regrouping, never a
/// discard (R710-R9). A task whose n-ary split has fewer than two checkable
/// segments yields `None`.
#[must_use]
pub fn balanced_split(task: &str) -> Option<BinarySplit> {
    splitting::balanced_split(task)
}

/// The seeded `balanced_split` heuristic, so the split reaches the core through
/// the registry rather than through a direct call.
#[derive(Debug, Clone, Copy, Default)]
pub struct BalancedSplitter;

impl TaskSplitter for BalancedSplitter {
    fn slug(&self) -> &'static str {
        "balanced_split"
    }

    fn split(&self, task: &str, _parameters: &[(String, String)]) -> Option<BinarySplit> {
        balanced_split(task)
    }
}

/// Why a split was refused, so `None` is never read as "the task is easy".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SplitRefusal {
    /// Fewer than two checkable segments: the honest atomicity judgement.
    SingleClause { reason: String },
    /// One clause, but not atomic: the decomposition approaches are a lookup
    /// that has not landed yet (plan 01).
    Underivable { blocker: String },
}

/// The refusal reason for a task `balanced_split` returned `None` for.
#[must_use]
pub fn split_refusal(task: &str) -> Option<SplitRefusal> {
    splitting::split_refusal(task)
}

/// A trade-off between two criteria, as a link with a value on the range (#901).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContradictionLink {
    /// `stable_id("contradiction", "<criterion_a>:<criterion_b>:<situation>")`.
    pub link_id: String,
    /// The two criteria in tension, as meaning slugs.
    pub criterion_a: String,
    pub criterion_b: String,
    /// Where on the range the requirement itself places the answer, in basis
    /// points of criterion B: `0` is fully A, `10_000` is fully B. Integer, so
    /// the value is exactly comparable and hashable -- never a float in an id.
    pub selection_basis_points: u16,
    /// How the value was derived; never guessed.
    pub derivation: ContradictionDerivation,
    pub resolution: TrizResolution,
    /// The requirement spans and source links the value was derived from.
    pub evidence: Vec<String>,
}

/// Where the selection value came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContradictionDerivation {
    /// Counted from the requirement's own clauses demanding each criterion.
    RequirementClauses { a_clauses: usize, b_clauses: usize },
    /// Read from a seeded or learned contradiction record for this situation.
    SeededRecord { record_id: String },
    /// Neither: the contradiction is named and left unresolved, honestly.
    Underivable { reason: String },
}

/// The resolution shapes TRIZ names, seeded as data, not as Rust branches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrizResolution {
    /// Pick the point on the range (`selection_basis_points`).
    Range,
    /// Satisfy both by separating them along an axis.
    Separation { axis: SeparationAxis },
    /// Apply a named inventive principle from `data/seed/triz-principles.lino`.
    Principle { principle_id: String },
    /// Named and unresolved. Reported; never silently defaulted to 50 %.
    Unresolved { reason: String },
}

/// The four classical separation principles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparationAxis {
    InTime,
    InSpace,
    OnCondition,
    BetweenPartAndWhole,
}

/// Detect the contradictions in a ranked candidate set: two candidates that each
/// win on a different cost dimension are a technical contradiction.
#[must_use]
pub fn contradictions_in(scores: &[CandidateScore], requirement: &str) -> Vec<ContradictionLink> {
    triz::contradictions_in(scores, requirement)
}

/// A ranker that resolves detected contradictions by their selection value
/// instead of by the arbitrary index tie-break.
#[derive(Debug, Clone, Copy, Default)]
pub struct TrizRanker;

impl CandidateRanker for TrizRanker {
    fn slug(&self) -> &'static str {
        "triz"
    }

    fn rank(&self, scores: &[CandidateScore], parameters: &[(String, String)]) -> Vec<usize> {
        triz::rank(scores, parameters)
    }
}

/// One rule under test. `SearchHypothesis`, not `Hypothesis`, because
/// `StepKind::Hypothesis` already owns the word in a different sense.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHypothesis {
    pub hypothesis_id: String,
    /// The rule in the meta language, e.g. `ascending_by_two`.
    pub rule: String,
    pub alive: bool,
    /// The experiment id that killed it, when it is dead.
    pub refuted_by: Option<String>,
}

/// One probe and what each hypothesis predicts for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Experiment {
    pub experiment_id: String,
    /// The probe as it will actually be run -- a command, a prompt, a fetch.
    pub probe: String,
    /// Hypotheses that predict the probe succeeds.
    pub predicts_yes: Vec<String>,
    /// Hypotheses that predict it fails.
    pub predicts_no: Vec<String>,
}

/// The live hypothesis set and the experiments run against it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HypothesisSpace {
    pub space_id: String,
    pub hypotheses: Vec<SearchHypothesis>,
    pub experiments: Vec<Experiment>,
    /// Every observation applied so far, so the elimination is replayable.
    pub observations: Vec<Evidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchVerdict {
    Concluded {
        hypothesis_id: String,
    },
    AllRefuted,
    NotConfirmedNotRefuted {
        survivors: Vec<String>,
        blocker: String,
    },
}

pub trait ExperimentChooser {
    fn slug(&self) -> &'static str;
    /// Pick the next probe: minimize `worst_case_survivors`, and among equals
    /// prefer one that attempts to refute the leading hypothesis.
    fn next_experiment<'a>(&self, space: &'a HypothesisSpace) -> Option<&'a Experiment>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RefutationSearch;

impl ExperimentChooser for RefutationSearch {
    fn slug(&self) -> &'static str {
        "refutation_search"
    }

    fn next_experiment<'a>(&self, space: &'a HypothesisSpace) -> Option<&'a Experiment> {
        refutation::next_experiment(space)
    }
}

/// How many obligation segments `task` carries.
#[must_use]
pub fn segment_count(task: &str) -> usize {
    splitting::segment_count(task)
}

/// Return the complete binary-tree layers over `task`'s segments.
///
/// This is how deep the recursion may descend before the bottom layer stops being a
/// complete one (`data/meta/task-decomposition-invariant.lino`).
#[must_use]
pub fn complete_layers(task: &str) -> u8 {
    splitting::complete_layers(task)
}
