//! Issue #412: coding-oracle fallback for `write_program` requests.
//!
//! When a `write_program` request names a *language* the verified catalog does
//! not template (Kotlin, Swift, PHP, Bash, Lua, Haskell, …), the formalizer
//! produces [`SelectedRule::UnsupportedWriteProgram`](crate::engine::SelectedRule)
//! and — once the issue #340 blueprint handler also declines — the request dead
//! ends on the honest `write_program_unsupported` answer or, worse, the unknown
//! opener. That is the wrong outcome for a canonical request like "write a hello
//! world program in Kotlin": the answer is public knowledge.
//!
//! This handler treats the external knowledge bases (Rosetta Code, the Hello
//! World Collection, Wikifunctions, Stack Overflow) as cached APIs via
//! [`crate::knowledge::CodingOracle`]. When the oracle has a reviewed snippet
//! for the requested `(task, language)` it returns that snippet plus its
//! deterministic output and source attribution, exactly the "code + result"
//! shape the catalog produces — but sourced from the cached external corpus
//! rather than a hand-written template. The data is plain Rust so it compiles
//! into both the native binary and the WASM worker, and is mirrored in
//! `src/web/formal_ai_worker.js` for cross-runtime parity.
//!
//! Runs *after* the blueprint handler (which owns supported-language composite
//! tasks) and only ever supplies an answer the caller would otherwise not have,
//! so it is purely additive.

use crate::coding::program_task_by_alias;
use crate::engine::{normalize_prompt, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::knowledge::CodingOracle;
use crate::solver::BlueprintComposition;

use crate::solver_handlers::{finalize_simple, try_program_blueprint};

/// Render an otherwise-unsupported `write_program` request, trying the composite
/// blueprint handler first (issue #340, supported-language composite tasks) and
/// the cached coding oracle second (issue #412, languages the catalog does not
/// template at all). Returns `None` when neither can answer, leaving the caller
/// on its existing path.
///
/// Extracted from `solver.rs` so that module stays under the repository line
/// limit; `task`/`language` are the `UnsupportedWriteProgram` fields.
pub fn try_unsupported_write_program(
    prompt: &str,
    task: Option<&str>,
    language: Option<&str>,
    composition: BlueprintComposition,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let normalized = normalize_prompt(prompt);
    try_program_blueprint(prompt, &normalized, language, composition, log)
        .or_else(|| try_write_program_from_oracle(prompt, &normalized, task, language, log))
}

/// Try to answer an otherwise-unsupported `write_program` request from the
/// coding oracle's cached external snippets.
///
/// `task_hint`/`language_hint` are the parameters the formalizer already
/// extracted (the `task`/`language` fields of `UnsupportedWriteProgram`). The
/// task falls back to alias matching on the prompt when the formalizer left it
/// unset, so a bare "hello world in kotlin" still resolves. Returns `None` when
/// the oracle has no cached answer — the caller then keeps its existing path.
pub fn try_write_program_from_oracle(
    prompt: &str,
    normalized: &str,
    task_hint: Option<&str>,
    language_hint: Option<&str>,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let language = language_hint?.trim();
    if language.is_empty() {
        return None;
    }
    let task_slug = task_hint
        .map(str::to_owned)
        .or_else(|| program_task_by_alias(normalized).map(|task| task.slug.to_owned()))?;

    let snippet = CodingOracle::lookup(&task_slug, language)?;

    let body = format!(
        "Here is a minimal {} program ({}):\n\n```{}\n{}\n```\n\nOutput:\n```text\n{}\n```\n\
         Source: {} ({}), cached locally as a popular example.",
        snippet.language_label,
        snippet.task_slug.replace('_', " "),
        snippet.language_slug,
        snippet.code,
        snippet.expected_output,
        snippet.source.display_name(),
        snippet.source_url,
    );

    // Evidence trail: the answer is sourced from an external knowledge base, not
    // the verified catalog, so record the provenance and an honest "not run"
    // execution status — the snippet is reviewed and cached, not sandbox-run.
    log.append("knowledge_source", snippet.source.slug().to_owned());
    log.append("knowledge_source_url", snippet.source_url.to_owned());
    log.append(
        "execution_status",
        "not run (cached external snippet)".to_owned(),
    );
    log.append(
        "execution_environment",
        "no compile/run sandbox configured for cached external snippets".to_owned(),
    );
    log.append(
        "program_parameter:language",
        snippet.language_slug.to_owned(),
    );
    log.append("program_parameter:task", snippet.task_slug.to_owned());

    let intent = format!(
        "write_program_oracle_{}_{}",
        snippet.task_slug, snippet.language_slug
    );
    let response_link = format!(
        "response:write_program:{}:{}:{}",
        snippet.task_slug,
        snippet.language_slug,
        snippet.source.slug()
    );
    Some(finalize_simple(
        prompt,
        log,
        &intent,
        &response_link,
        &body,
        1.0,
    ))
}

#[path = "source_tests/solver_handler_oracle/tests.rs"]
mod tests;
