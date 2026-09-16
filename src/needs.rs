//! The `Need` record — the one "I lack X" contract every step connects through.
//!
//! Plan 00 §4.1 of the issue #1138 plan set (leaf C1) owns this module: the
//! registry's selection axis ([`NeedKind`]), the need lifecycle ([`NeedState`])
//! and the record itself ([`Need`]) have exactly one Rust home, so plan 01's
//! retrieval kernel and plan 04's formalizer consume one type rather than
//! declaring two.
//!
//! The record is also a link in the associative store, projected by
//! [`Need::to_links_notation`], so a need can be remembered, forgotten and
//! rediscovered:
//!
//! ```text
//! need <id>
//!   kind      concept | procedure | part | prerequisite | evidence | decision
//!   subject   "<surface text as written by the user or the failing tool>"
//!   language  en | ru | hi | zh | es | ...
//!   raised_by <obligation id | need id | tool result id>
//!   state     open | planned | satisfied | unsatisfiable
//!   satisfied_by <evidence id>
//! ```

/// What a need lacks, and therefore which sources may answer it.
///
/// One enum serves the registry (`SourceRecord::need_kinds`), the need record
/// and the lookup.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum NeedKind {
    Concept,
    Procedure,
    Part,
    Prerequisite,
    Evidence,
    Decision,
    #[default]
    None,
}

impl NeedKind {
    /// The seed vocabulary slug for this kind.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        panic!("plan 00 leaf C1")
    }

    /// The kind a seed record names, or [`NeedKind::None`].
    #[must_use]
    pub fn from_seed(_value: &str) -> Self {
        todo!("plan 00 leaf C1")
    }
}

/// The need lifecycle. This is the single need-status vocabulary in the tree;
/// `meta_frame::NeedStatus` maps onto it (Pending->Open, Planned->Planned,
/// Satisfied->Satisfied, Blocked->Unsatisfiable).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeedState {
    Open,
    Planned,
    Satisfied,
    Unsatisfiable,
}

/// One thing the system does not know.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Need {
    pub need_id: String,
    pub kind: NeedKind,
    pub subject: String,
    pub language: String,
    pub raised_by: String,
    /// `"<doc_id>@<start>:<end>"`, exact byte span in the source text.
    pub source_span: String,
    pub depth: usize,
    pub state: NeedState,
    /// `Evidence::evidence_id` (plan 00 §4.3). Set only by an observation.
    pub satisfied_by: Option<String>,
}

impl Need {
    /// The Links Notation projection of this record.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 00 leaf C1")
    }

    /// Parse the projection back into a record; `None` when the text is not a
    /// `need` record.
    #[must_use]
    pub fn from_links_notation(_text: &str) -> Option<Self> {
        todo!("plan 00 leaf C1")
    }
}
