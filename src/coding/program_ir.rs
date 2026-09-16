//! The language-neutral intermediate representation (issue #1138, plan 02
//! L2–L3, L12).
//!
//! Every node is representable as nested `.lino`, so the IR is
//! associative-stack data rather than a Rust-only structure, and its content id
//! is taken over that projection. Nothing in [`elaborate`] knows a task name:
//! a step contributes a node, consecutive nodes compose by type, and a step that
//! binds a name introduces [`IrNode::Bind`].

use crate::coding::fragment_catalog::FragmentCatalog;
use crate::coding::task_spec::CodingTaskSpec;
use crate::procedure_text::ProcedureStepRecord;

/// Whether a retrieved fragment's text may be emitted verbatim, or only its
/// abstract shape reused. Share-alike licensing is enforced structurally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReuseMode {
    /// The fragment's text may be emitted verbatim.
    Verbatim,
    /// Only the fragment's abstract shape may be used; text must be re-derived.
    ShapeOnly,
}

/// A value shape the IR can type-check without committing to a language.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrType {
    Integer,
    Float,
    Boolean,
    Text,
    Sequence(Box<IrType>),
    Pair(Box<IrType>, Box<IrType>),
    Mapping(Box<IrType>, Box<IrType>),
    /// Unconstrained; unifies with anything exactly once.
    Unknown(usize),
}

/// One node of a language-neutral program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrNode {
    Parameter {
        name: String,
        ty: IrType,
    },
    Literal {
        text: String,
        ty: IrType,
    },
    /// A named fragment applied to arguments. `fragment` is a
    /// [`FragmentCatalog`] id, never a language-specific symbol.
    Apply {
        fragment: String,
        arguments: Vec<IrNode>,
    },
    /// Comprehension over `items`, binding `item`, yielding `body`, optionally
    /// filtered by `predicate`.
    Each {
        item: String,
        items: Box<IrNode>,
        body: Box<IrNode>,
        predicate: Option<Box<IrNode>>,
    },
    /// Left fold with an initial value.
    Fold {
        item: String,
        accumulator: String,
        items: Box<IrNode>,
        initial: Box<IrNode>,
        body: Box<IrNode>,
    },
    /// Bounded iteration with an explicit termination condition.
    Repeat {
        counter: String,
        from: Box<IrNode>,
        to: Box<IrNode>,
        body: Box<IrNode>,
    },
    /// Named state, base cases and a transition — the recurrence shape OEIS and
    /// Wikifunctions both produce.
    Recurrence {
        state: Vec<String>,
        base: Vec<IrNode>,
        transition: Box<IrNode>,
        index: Box<IrNode>,
    },
    Condition {
        test: Box<IrNode>,
        then_branch: Box<IrNode>,
        else_branch: Box<IrNode>,
    },
    Bind {
        name: String,
        value: Box<IrNode>,
        body: Box<IrNode>,
    },
    Emit {
        value: Box<IrNode>,
    },
    Return {
        value: Box<IrNode>,
    },
}

/// A complete program plan: signature, body, provenance and cost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramIr {
    pub name: String,
    pub parameters: Vec<(String, IrType)>,
    pub result: IrType,
    pub body: IrNode,
    pub fragments: Vec<String>,
    pub source_urls: Vec<String>,
    pub source_licenses: Vec<String>,
    pub reuse: ReuseMode,
}

impl ProgramIr {
    /// Structural cost, the least-action key: node count plus fragment depth.
    #[must_use]
    pub fn action_cost(&self) -> usize {
        todo!("plan 02 leaf L2")
    }

    /// Unify parameter and result types through the fragment signatures.
    /// `Err` names the first node that cannot be typed.
    pub fn type_check(&self, _catalog: &FragmentCatalog) -> Result<(), String> {
        todo!("plan 02 leaf L3")
    }

    /// The canonical `.lino` projection; also the content-hash input.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 02 leaf L2")
    }

    /// Parse the canonical projection back into an IR.
    #[must_use]
    pub fn from_links_notation(_text: &str) -> Option<Self> {
        todo!("plan 02 leaf L2")
    }

    /// Stable content id over [`Self::to_links_notation`].
    #[must_use]
    pub fn content_id(&self) -> String {
        todo!("plan 02 leaf L2")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElaborationBounds {
    pub max_candidates: usize,
    pub max_depth: usize,
}

/// Elaborate an ordered step list into candidate IR bodies.
pub fn elaborate(
    _spec: &CodingTaskSpec,
    _steps: &[ProcedureStepRecord],
    _catalog: &FragmentCatalog,
    _bounds: ElaborationBounds,
) -> Vec<ProgramIr> {
    todo!("plan 02 leaf L12")
}
