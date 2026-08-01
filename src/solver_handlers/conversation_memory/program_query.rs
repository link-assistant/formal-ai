//! Bounded memory-program integration for natural-language memory queries.

use super::{
    answer_memory_recall, recalled_event_indices, try_link_substitution_query, try_memory_write,
    MemoryQueryExecution,
};
use crate::engine::normalize_prompt;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::memory::MemoryStore;
use crate::memory_program::{
    compile_memory_program, execute_memory_program, MemoryProgramAuthorization,
    MemoryProgramCompileError, MemoryProgramHalt, MemoryProgramLimits, MemoryProgramOutcome,
};
use crate::seed;
use crate::solver_handlers::finalize_simple;

#[must_use]
pub fn execute_memory_query(
    prompt: &str,
    store: &mut MemoryStore,
    current_conversation_id: Option<&str>,
) -> Option<MemoryQueryExecution> {
    execute_memory_query_with_options(
        prompt,
        store,
        current_conversation_id,
        MemoryProgramLimits::default(),
        MemoryProgramAuthorization::Write,
    )
}

/// Execute with explicit solver-derived bounds and effect authorization.
#[must_use]
pub fn execute_memory_query_with_options(
    prompt: &str,
    store: &mut MemoryStore,
    current_conversation_id: Option<&str>,
    limits: MemoryProgramLimits,
    authorization: MemoryProgramAuthorization,
) -> Option<MemoryQueryExecution> {
    let normalized = normalize_prompt(prompt);
    let mut log = EventLog::new();
    log.append("impulse", prompt.to_owned());
    // Link queries are an explicit meta-language and retain precedence over
    // natural-language compilation. Reads never persist store changes.
    if let Some(answer) = try_link_substitution_query(prompt, store, &mut log) {
        return Some(MemoryQueryExecution {
            answer,
            changed: false,
        });
    }
    let program_compilation = compile_memory_program(prompt, limits);
    if let Ok(program) = &program_compilation {
        log.append("memory_program_compiled", program.links_notation());
        let outcome = execute_memory_program(program, store, authorization);
        log.append("memory_program_execution", outcome.links_notation());
        if matches!(
            outcome.halt,
            MemoryProgramHalt::PermissionDenied { ref required } if required == "destructive"
        ) {
            log.append(
                "policy:destructive_action_requires_confirmation",
                prompt.to_owned(),
            );
        }
        let changed = outcome.changed > 0;
        let body = render_memory_program_outcome(&outcome, detect_language(prompt).slug());
        let intent = if matches!(outcome.halt, MemoryProgramHalt::PermissionDenied { .. }) {
            "memory_program_refused"
        } else {
            "memory_program"
        };
        return Some(MemoryQueryExecution {
            answer: finalize_simple(
                prompt,
                &mut log,
                intent,
                "response:memory_program",
                &body,
                0.9,
            ),
            changed,
        });
    }
    if let Some(answer) = try_memory_write(
        prompt,
        &normalized,
        store,
        current_conversation_id,
        &mut log,
    ) {
        return Some(MemoryQueryExecution {
            answer,
            changed: true,
        });
    }
    if let Err(MemoryProgramCompileError::ProgramGap { gap, .. }) = program_compilation {
        log.append("program_gap", gap.clone());
        return Some(MemoryQueryExecution {
            answer: finalize_simple(
                prompt,
                &mut log,
                "memory_program_gap",
                "response:memory_program_gap",
                &memory_program_response(
                    "memory_program_compilation_gap",
                    detect_language(prompt).slug(),
                    &[("gap", gap.as_str())],
                ),
                0.4,
            ),
            changed: false,
        });
    }
    answer_memory_recall(prompt, store.events(), current_conversation_id).map(|answer| {
        let accessed =
            recalled_event_indices(&normalized, store.events(), current_conversation_id, prompt);
        let changed = store.record_access(&accessed) > 0;
        MemoryQueryExecution { answer, changed }
    })
}

fn render_memory_program_outcome(outcome: &MemoryProgramOutcome, language: &str) -> String {
    let matched = outcome.matched.to_string();
    let changed = outcome.changed.to_string();
    let iterations = outcome.iterations.to_string();
    match &outcome.halt {
        MemoryProgramHalt::Complete | MemoryProgramHalt::Fixpoint => memory_program_response(
            "memory_program_complete",
            language,
            &[
                ("program", outcome.program_id.as_str()),
                ("matched", matched.as_str()),
                ("changed", changed.as_str()),
                (
                    "halt",
                    if matches!(outcome.halt, MemoryProgramHalt::Fixpoint) {
                        "fixpoint"
                    } else {
                        "complete"
                    },
                ),
                ("iterations", iterations.as_str()),
            ],
        ),
        MemoryProgramHalt::MatchLimit {
            matched,
            max_matches,
        } => memory_program_response(
            "memory_program_match_limit",
            language,
            &[
                ("matched", matched.to_string().as_str()),
                ("max_matches", max_matches.to_string().as_str()),
            ],
        ),
        MemoryProgramHalt::IterationLimit { max_iterations } => memory_program_response(
            "memory_program_iteration_limit",
            language,
            &[("max_iterations", max_iterations.to_string().as_str())],
        ),
        MemoryProgramHalt::PermissionDenied { required } if required == "destructive" => {
            memory_program_response("memory_program_destructive_refused", language, &[])
        }
        MemoryProgramHalt::PermissionDenied { required } => memory_program_response(
            "memory_program_permission_refused",
            language,
            &[("required", required)],
        ),
        MemoryProgramHalt::ProgramGap { primitive } => memory_program_response(
            "memory_program_interpreter_gap",
            language,
            &[("primitive", primitive)],
        ),
    }
}

fn memory_program_response(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    let mut response = seed::localized_response(intent, language).unwrap_or_default();
    for (name, value) in values {
        response = response.replace(&format!("{{{name}}}"), value);
    }
    response
}
