//! The recursive meta reasoner, ported by hand from JavaScript (R1012 of
//! docs/requirements/doctrine-standing-doctrine-recursive-meta-algorithm-2026-10-06.md).
//!
//! Originals: `js/worker/formal_ai_worker_meta_reasoner.js` (grounding, the
//! core loop, answers), `js/worker/formal_ai_worker_meta_synthesis.js` (typed
//! enumeration, inference, rendering, verification) and
//! `js/worker/formal_ai_worker_meta_composite.js` (decomposition, memory). Every
//! function names the JavaScript function it mirrors.
//!
//! Nothing here knows a task. Every word of a request starts as an unknown,
//! grounded from a definition the request gives, a learned chunk, the
//! documentation of the instruction set (data/seed/meta-reasoning.lino) and
//! dictionary captures. The goal state is the request's own examples:
//! programs are enumerated over the typed instruction set, shortest first,
//! and each candidate's difference from the goal is its failing examples. The
//! program an answer shows is rendered from the seed's JavaScript `code`; the
//! verification runs the same program through [`interpreter`], which
//! implements every seeded operation by id. Every decision is a trace event;
//! the derivation serialises to links notation.
//!
//! Dictionary knowledge is injected: [`meta_reason`] takes a lookup callback,
//! and the solver passes the registry concept lookup, cache-only
//! ([`integration`]).

pub mod answer;
pub mod catalog;
pub mod grounding;
pub mod integration;
pub mod interpreter;
pub mod phrases;
pub mod reasoner;
pub mod seed;
pub mod synthesis;
pub mod text;
pub mod value;

pub use answer::{MetaAnswer, meta_answer, meta_reason, meta_sub_requests};
pub use grounding::{Knowledge, Sense};
pub use integration::{meta_is_impasse_intent, resolve_impasse, try_meta_answer};
pub use reasoner::{
    MetaResult, ProgramInfo, Unknown, forget_learned, import_learned, meta_derivation_lino,
    meta_learned_lino, meta_reason_core, take_learned,
};
pub use value::Value;

/// Search and grounding bounds.
///
/// Mirrors `META_BOUNDS` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    /// How deep a gloss is grounded.
    pub ground_depth: usize,
    /// Glosses read per word.
    pub glosses_per_word: usize,
    /// The score above which a gloss word's hypothesis is specific.
    pub specific_gloss: f64,
    /// The share of a gloss's words that must support an operation.
    pub gloss_support: f64,
    /// The longest program enumerated.
    pub program_length: usize,
    /// Candidates evaluated against examples.
    pub candidate_budget: usize,
    /// Lookup rounds of the loop.
    pub lookup_rounds: usize,
    /// Lookups per round.
    pub lookups_per_round: usize,
    /// Counterexamples written to the trace.
    pub rejections_traced: usize,
    /// The longest measure composed.
    pub measure_length: usize,
    /// The largest tie that still requires an operation.
    pub required_group_size: usize,
}

/// The bounds in force.
pub const BOUNDS: Bounds = Bounds {
    ground_depth: 2,
    glosses_per_word: 3,
    specific_gloss: 0.5,
    gloss_support: 0.2,
    program_length: 4,
    candidate_budget: 20_000,
    lookup_rounds: 3,
    lookups_per_round: 4,
    rejections_traced: 5,
    measure_length: 3,
    required_group_size: 2,
};

/// One numbered decision of the reasoner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEvent {
    /// 1-based position.
    pub seq: usize,
    /// Event kind (`impulse`, `hypothesis`, `search`, ...).
    pub kind: String,
    /// What was decided.
    pub detail: String,
}

/// The trace recorder: numbered events in the order decisions were taken.
///
/// Mirrors `metaTrace` in `js/worker/formal_ai_worker_meta_reasoner.js`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Trace {
    /// The events, in order.
    pub events: Vec<TraceEvent>,
}

impl Trace {
    /// Record one event.
    ///
    /// Mirrors `trace.emit` of `metaTrace` in `js/worker/formal_ai_worker_meta_reasoner.js`.
    pub fn emit(&mut self, kind: &str, detail: impl Into<String>) {
        self.events.push(TraceEvent {
            seq: self.events.len() + 1,
            kind: kind.to_owned(),
            detail: detail.into(),
        });
    }
}
