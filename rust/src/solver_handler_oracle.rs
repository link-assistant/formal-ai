//! Composite fallback for program requests outside the local toolchain contract.

use crate::engine::{SymbolicAnswer, normalize_prompt};
use crate::event_log::EventLog;
use crate::solver::BlueprintComposition;
use crate::solver_handlers::try_program_blueprint;

/// Try a composite blueprint for a request whose task and language have no local implementation.
pub fn try_unsupported_write_program(
    prompt: &str,
    language: Option<&str>,
    composition: BlueprintComposition,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    try_program_blueprint(
        prompt,
        &normalize_prompt(prompt),
        language,
        composition,
        log,
    )
}
