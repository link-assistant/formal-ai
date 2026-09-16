//! A task whose answer can be checked, in any domain (#1138 B8).
//!
//! Plan 08 owns this module. Recognition is seed data in five languages, never a
//! suite id and never an English literal in Rust; `identity()` deliberately
//! excludes literal values so a renumbered paraphrase recalls the same
//! *procedure* and is recomputed rather than replayed.
//!
//! The benchmark grader must stay invisible here: nothing under
//! `src/verifiable_task*` may import `external_benchmarks`.
//!
//! Wave T lands the shapes only; wave I8 leaves 08-L3 through 08-L6 fill the
//! bodies in.

pub mod ledger;
pub mod quantities;

use crate::coding::task_spec::Example;
use quantities::{Entity, Quantity};

/// What kind of observation would settle this task — the shape the **answer**
/// must take.
///
/// This is *not* plan 05's `ObligationExpectation`, which declares what must be
/// **observed** before an obligation node may be called satisfied. The two are
/// bridged once, by [`TaskExpectation::to_obligation_expectation`], so there is
/// exactly one satisfaction rule in the tree (plan 00 §9 R15).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskExpectation {
    /// A single number is the answer. `unit` is `Some` when the prompt names one.
    Numeric {
        /// The unit the prompt names, when it names one.
        unit: Option<String>,
    },
    /// A cardinality over a described collection.
    Count {
        /// The category the prompt counts over.
        subject: String,
    },
    /// A transformed version of a supplied text.
    EditedText {
        /// The text the prompt supplied.
        source: String,
    },
    /// A value for a named unknown, as an equation states it.
    Unknown {
        /// The unknown's name.
        name: String,
    },
    /// A named callable checked by examples — today's `CodingTaskSpec`.
    Callable {
        /// The examples the prompt states.
        examples: Vec<Example>,
    },
    /// A process whose stdout is the observation.
    Stdout {
        /// The stdout the prompt states, when it states one.
        expected: Option<String>,
    },
    /// A yes/no claim.
    Boolean,
}

impl TaskExpectation {
    /// Bridge to plan 05's obligation vocabulary.
    ///
    /// `check_id` is `"<VerifiedAnswer::derivation_id>:<check slug>"`.
    #[must_use]
    pub fn to_obligation_expectation(
        &self,
        _check_id: &str,
    ) -> crate::obligation_ledger::ObligationExpectation {
        todo!("plan 08 leaf L3")
    }

    /// Stable slug used in the trace and the ledger.
    #[must_use]
    pub fn slug(&self) -> &'static str {
        todo!("plan 08 leaf L3")
    }
}

/// How an answer of this expectation must be presented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerShape {
    /// The number stands alone at the end of the answer.
    TrailingNumber,
    /// The value is delimited the way the prompt's own convention requires.
    DelimitedValue,
    /// The edited text is the whole answer body.
    WholeBody,
    /// A fenced program.
    CodeBlock,
}

/// A task whose answer can be checked, in any domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiableTask {
    /// The prompt as received, unmodified.
    pub prompt: String,
    /// Sentences the requirement decomposes into.
    pub requirement_sentences: Vec<String>,
    /// What kind of observation would settle it.
    pub expectation: TaskExpectation,
    /// How the answer must be presented.
    pub shape: AnswerShape,
    /// Named quantities the prompt states, in order of appearance.
    pub quantities: Vec<Quantity>,
    /// Entities the prompt lists, for `Count` tasks.
    pub entities: Vec<Entity>,
    /// Detected prose language slug (`en`, `ru`, `hi`, `zh`, `es`).
    pub prose_language: String,
}

impl VerifiableTask {
    /// Stable identity for ledger recall: normalized sentences + expectation
    /// kind + quantity shape. Deliberately excludes literal values.
    #[must_use]
    pub fn identity(&self) -> String {
        todo!("plan 08 leaf L3")
    }

    /// Links Notation projection of the task.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 08 leaf L3")
    }

    /// Present `value` the way `shape` requires, in `prose_language`.
    #[must_use]
    pub fn render(&self, _value: &str, _reasoning: &str) -> String {
        todo!("plan 08 leaf L6")
    }
}

/// Recognize any task carrying a checkable expectation.
///
/// Returns `None` for an open-ended request. Every cue is a seed meaning from
/// `data/seed/meanings-verifiable-task.lino`; no suite id, no benchmark name and
/// no hard-coded English phrase appears here.
#[must_use]
pub fn recognise_verifiable(_prompt: &str) -> Option<VerifiableTask> {
    todo!("plan 08 leaf L5")
}
