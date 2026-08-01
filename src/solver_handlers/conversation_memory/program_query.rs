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
use crate::memory_query_language::{
    compile_memory_query as compile_exact_memory_query,
    execute_memory_query as execute_exact_memory_query, QueryDialect,
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
    if let Some(dialect) = detect_exact_memory_query(prompt) {
        return Some(execute_exact_query(
            prompt,
            store,
            limits,
            authorization,
            dialect,
            &mut log,
        ));
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

pub fn is_exact_memory_query(prompt: &str) -> bool {
    detect_exact_memory_query(prompt).is_some()
}

fn detect_exact_memory_query(prompt: &str) -> Option<QueryDialect> {
    let normalized = prompt.trim().to_ascii_lowercase();
    let sql = normalized.starts_with("select ")
        || normalized.starts_with("insert into ")
        || normalized.starts_with("update ")
        || normalized.starts_with("delete from ");
    if sql {
        return Some(QueryDialect::SqlAnsi);
    }
    let graphql = normalized.contains('{')
        && (normalized.starts_with("query")
            || normalized.starts_with("mutation")
            || normalized.starts_with('{'))
        && (normalized.contains("memory")
            || normalized.contains("creatememory")
            || normalized.contains("updatememory")
            || normalized.contains("deletememory"));
    graphql.then_some(QueryDialect::GraphQl)
}

fn execute_exact_query(
    prompt: &str,
    store: &mut MemoryStore,
    limits: MemoryProgramLimits,
    authorization: MemoryProgramAuthorization,
    dialect: QueryDialect,
    log: &mut EventLog,
) -> MemoryQueryExecution {
    let query = match compile_exact_memory_query(prompt, dialect, limits) {
        Ok(query) => query,
        Err(error) => {
            log.append("memory_exact_query_rejected", error.to_string());
            let mut body = String::new();
            crate::links_format::push_lino_node(&mut body, 0, "memory_query_error", None);
            crate::links_format::push_lino_node(&mut body, 2, "dialect", Some(dialect.as_str()));
            crate::links_format::push_lino_node(&mut body, 2, "message", Some(&error.to_string()));
            return MemoryQueryExecution {
                answer: finalize_simple(
                    prompt,
                    log,
                    "memory_exact_query_rejected",
                    "response:memory_exact_query_rejected",
                    body.trim_end(),
                    1.0,
                ),
                changed: false,
            };
        }
    };
    log.append("memory_exact_query_compiled", query.links_notation());
    let outcome = execute_exact_memory_query(&query, store, authorization);
    let rendered = outcome.links_notation(&query);
    log.append("memory_exact_query_execution", rendered.clone());
    if matches!(
        outcome.halt,
        MemoryProgramHalt::PermissionDenied { ref required } if required == "destructive"
    ) {
        log.append(
            "policy:destructive_action_requires_confirmation",
            prompt.to_owned(),
        );
    }
    let changed = outcome.changed > 0
        || (!outcome.matched_ids.is_empty()
            && matches!(
                query.plan.operation,
                crate::memory_query_language::MemoryQueryOperation::Select
            ));
    let intent = if matches!(outcome.halt, MemoryProgramHalt::PermissionDenied { .. }) {
        "memory_exact_query_refused"
    } else {
        "memory_exact_query"
    };
    MemoryQueryExecution {
        answer: finalize_simple(
            prompt,
            log,
            intent,
            "response:memory_exact_query",
            &rendered,
            1.0,
        ),
        changed,
    }
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
