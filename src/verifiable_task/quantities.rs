//! Seed-driven quantity and entity extraction, five languages (#1138 B8).
//!
//! Spelled-out numerals resolve through the arithmetic normalization tables the
//! seed lexicon already carries, not through an ASCII-digit scan, so `two`,
//! `два`, `दो`, `两` and `dos` are one code path. Multiplicity attaches to the
//! entity it modifies, which is what today's object-count table cannot express.
//!
//! Wave T lands the shapes only; wave I8 leaf 08-L4 fills the bodies in.

/// A quantity the prompt states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quantity {
    /// The value, as written or as normalized from a spelled-out numeral.
    pub value: String,
    /// The unit the prompt attaches, when it attaches one.
    pub unit: Option<String>,
    /// The label the prompt attaches, when it attaches one.
    pub label: Option<String>,
    /// Byte offset in the prompt, so the trace can point at the evidence.
    pub offset: usize,
}

/// An entity the prompt lists, for `Count` tasks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    /// The surface exactly as written.
    pub surface: String,
    /// Multiplicity the prompt states for this entity, default `1`.
    pub multiplicity: String,
    /// Byte offset in the prompt.
    pub offset: usize,
}

/// Every quantity the prompt states, in order of appearance.
#[must_use]
pub fn extract_quantities(_prompt: &str, _language: &str) -> Vec<Quantity> {
    todo!("plan 08 leaf L4")
}

/// Every entity the prompt lists, with the multiplicity it states.
#[must_use]
pub fn extract_entities(_prompt: &str, _language: &str) -> Vec<Entity> {
    todo!("plan 08 leaf L4")
}
