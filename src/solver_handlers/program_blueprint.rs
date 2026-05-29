//! Issue #340: composite-program *blueprint* handler.
//!
//! When a `write_program` request names a language we support but a *task* the
//! verified catalog cannot resolve (e.g. "make an HTTP GET, parse the JSON,
//! compute the mean and median"), the formalizer produces
//! [`SelectedRule::UnsupportedWriteProgram`](crate::engine::SelectedRule) — a
//! dead end. This handler runs *after* the issue #324 context recovery and
//! before that dead end is rendered: it decomposes the prompt into recognized
//! capabilities and, when they match a curated [`blueprint`](crate::coding::blueprint)
//! recipe for the requested language, returns the full program with its
//! decomposition plan, library prerequisites, and an honest "not run" execution
//! report.
//!
//! The blueprint is deliberately kept out of the verified catalog: its programs
//! need external libraries and network access the offline sandbox cannot
//! execute, so they can never honestly claim "compiled and ran". Keeping them
//! here preserves the catalog's deterministic-verified invariant while still
//! answering the broad class of real-world coding requests the catalog cannot.

use crate::coding::blueprint::{render, select_blueprint};
use crate::coding::program_language_by_alias;
use crate::coding::WRITE_PROGRAM_INTENT;
use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;

use super::finalize_simple;

/// Try to answer an otherwise-unsupported `write_program` request with a
/// composite blueprint. `language_hint` is the program language the formalizer
/// already extracted (the `language` field of `UnsupportedWriteProgram`); when
/// it is `None` we fall back to alias matching on the prompt.
///
/// Returns `None` when no recipe matches — the caller then keeps the honest
/// `write_program_unsupported` answer.
pub fn try_program_blueprint(
    prompt: &str,
    normalized: &str,
    language_hint: Option<&str>,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let language_slug = language_hint.map(str::to_owned).or_else(|| {
        program_language_by_alias(normalized).map(|language| language.slug.to_owned())
    })?;

    let blueprint = select_blueprint(normalized, &language_slug)?;
    let response_language = detect_language(prompt);
    let body = render(&blueprint, response_language);

    // Evidence trail: record the resolved recipe, the decomposed capabilities,
    // the (language, task) parameters, and the honest execution status so the
    // diagnostic chips read the same way a catalog answer would — except the
    // status is explicitly "not run" rather than "compiled and ran".
    log.append("program_blueprint:recipe", blueprint.recipe.slug.to_owned());
    log.append("program_parameter:language", language_slug.clone());
    log.append(
        "program_parameter:task",
        format!("blueprint:{}", blueprint.recipe.slug),
    );
    for capability in &blueprint.capabilities {
        log.append("program_blueprint:capability", capability.slug.to_owned());
    }
    log.append(
        "execution_status",
        "not run — requires external libraries and network access".to_owned(),
    );
    log.append(
        "execution_environment",
        "offline sandbox cannot install libraries or reach the network".to_owned(),
    );

    let response_link = format!(
        "response:write_program:blueprint:{}:{language_slug}",
        blueprint.recipe.slug
    );
    Some(finalize_simple(
        prompt,
        log,
        WRITE_PROGRAM_INTENT,
        &response_link,
        &body,
        0.7,
    ))
}
