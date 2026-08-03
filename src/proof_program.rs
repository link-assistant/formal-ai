//! Language-neutral formal proofs that can be projected into executable
//! programming-language presentations.
//!
//! The proof representation deliberately contains no presentation language.
//! A proof is constructed once from semantic bounds and can then be rendered by
//! the general code-translation pipeline into any supported target syntax.

mod core;

use crate::seed::proof_program_templates;

pub use core::{FormalProof, IntegerIntervalProof, ProofBound};

impl FormalProof {
    #[must_use]
    pub fn statement(&self) -> String {
        self.render_template(&proof_program_templates().statement)
    }

    /// Project this proof into an executable target-language program.
    #[must_use]
    pub fn render_program(&self, target: &str) -> Option<String> {
        let language = proof_program_templates().language(target)?;
        let template = if self.is_satisfiable() {
            &language.satisfiable
        } else {
            &language.unsatisfiable
        };
        Some(self.render_template(template))
    }
}
