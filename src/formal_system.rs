//! Named formal-system scope for relative statement probabilities.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::engine::stable_id;

/// A named formal system that gives truth values their interpretation.
///
/// Probability is never absolute: a statement is assessed relative to this
/// system's universe, interpretation, and axioms. All fields are symbolic and
/// caller supplied; the type does not embed a preferred logic or ontology.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FormalSystem {
    /// Stable human/machine name of the system.
    pub name: String,
    /// Universe over which statements are interpreted.
    pub universe: String,
    /// Model-theoretic interpretation used in this system.
    pub interpretation: String,
    /// Axioms available to proof and refutation, kept sorted for replay.
    pub axioms: BTreeSet<String>,
}

impl FormalSystem {
    /// Create a named system with an unspecified universe and interpretation.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            universe: String::new(),
            interpretation: String::new(),
            axioms: BTreeSet::new(),
        }
    }

    /// Set the universe, returning `self` for declarative construction.
    #[must_use]
    pub fn with_universe(mut self, universe: impl Into<String>) -> Self {
        self.universe = universe.into();
        self
    }

    /// Set the interpretation, returning `self` for declarative construction.
    #[must_use]
    pub fn with_interpretation(mut self, interpretation: impl Into<String>) -> Self {
        self.interpretation = interpretation.into();
        self
    }

    /// Add an axiom, returning `self` for declarative construction.
    #[must_use]
    pub fn with_axiom(mut self, axiom: impl Into<String>) -> Self {
        self.axioms.insert(axiom.into());
        self
    }

    /// Content-addressed identifier for the complete system definition.
    #[must_use]
    pub fn id(&self) -> String {
        let mut canonical = format!(
            "name:{};universe:{};interpretation:{};",
            self.name, self.universe, self.interpretation
        );
        for axiom in &self.axioms {
            let _ = write!(canonical, "axiom:{axiom};");
        }
        stable_id("formal_system", &canonical)
    }
}
