//! The heuristics the recursive core uses to choose among candidates, to choose
//! the next experiment, and to split a task it cannot solve directly
//! (#491, #901, #802, #453; plan 12).
//!
//! Wave T skeleton: the shapes the tests name exist, the behaviour does not.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::execution_evidence::Evidence;

/// Which of the three selection jobs a heuristic performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeuristicRole {
    Rank,
    Experiment,
    Split,
}

impl HeuristicRole {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        panic!("plan 12 leaf 1 -- HeuristicRole slugs")
    }

    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        let _ = slug;
        todo!("plan 12 leaf 1 -- HeuristicRole slugs")
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
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 12 leaf 1 -- HeuristicMethod links notation")
    }
}

/// Load the catalog from `data/meta/selection-heuristics.lino`.
///
/// # Errors
/// Returns the parse failure when the document is not a heuristic catalog.
pub fn catalog_from(text: &str) -> Result<Vec<HeuristicMethod>, String> {
    let _ = text;
    todo!("plan 12 leaf 2 -- read data/meta/selection-heuristics.lino")
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
    #[must_use]
    pub const fn satisfies(&self) -> bool {
        panic!("plan 12 leaf 3 -- correctness before cost")
    }

    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 12 leaf 3 -- CandidateScore links notation")
    }
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
        let _ = (scores, parameters);
        todo!("plan 12 leaf 3 -- least-action ranking over the declared key order")
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

impl BinarySplit {
    /// `|left_weight - right_weight|`. Zero is a perfect halving; the value is
    /// reported, never optimized away by dropping a segment.
    #[must_use]
    pub const fn imbalance(&self) -> usize {
        panic!("plan 12 leaf 7 -- report the imbalance, never hide it")
    }

    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 12 leaf 7 -- BinarySplit links notation")
    }
}

/// Fold the n-ary output of `task_decomposition::split_once_checkable` into the
/// balanced binary tree `data/meta/task-decomposition-invariant.lino` declares,
/// by cutting at the boundary that minimizes `imbalance()`.
///
/// Every segment survives on exactly one side, so this is a regrouping, never a
/// discard (R710-R9). A task whose n-ary split has fewer than two checkable
/// segments yields `None`.
#[must_use]
pub fn balanced_split(task: &str) -> Option<BinarySplit> {
    let _ = task;
    todo!("plan 12 leaf 7 -- balanced binary split")
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
    let _ = task;
    todo!("plan 12 leaf 7 -- an unsplittable task reports its reason")
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
    let _ = (scores, requirement);
    todo!("plan 12 leaf 15 -- detect technical contradictions")
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
        let _ = (scores, parameters);
        todo!("plan 12 leaf 15 -- break a least-action tie by the selection value")
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

impl Experiment {
    /// Live hypotheses surviving the worse of the two outcomes. Lower is better:
    /// this is the halving criterion (#802).
    #[must_use]
    pub fn worst_case_survivors(&self) -> usize {
        todo!("plan 12 leaf 12 -- the halving criterion")
    }

    /// Whether this probe is a *disproof attempt* of the leading hypothesis.
    #[must_use]
    pub fn attempts_refutation_of(&self, leading: &str) -> bool {
        let _ = leading;
        todo!("plan 12 leaf 12 -- disproof first")
    }
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

impl HypothesisSpace {
    #[must_use]
    pub fn alive(&self) -> Vec<&SearchHypothesis> {
        todo!("plan 12 leaf 12 -- live hypotheses")
    }

    /// Apply one observation: kill every hypothesis whose prediction it
    /// contradicts, recording which experiment did it.
    pub fn observe(&mut self, experiment_id: &str, record: Evidence) -> usize {
        let _ = (experiment_id, record);
        todo!("plan 12 leaf 12 -- every elimination is backed by an Evidence record")
    }

    /// The honest verdict when the space stops shrinking.
    #[must_use]
    pub fn verdict(&self) -> SearchVerdict {
        todo!("plan 12 leaf 12 -- not_confirmed_not_refuted names its survivors")
    }
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
        let _ = space;
        todo!("plan 12 leaf 12 -- minimize worst-case survivors, prefer refutation")
    }
}
