use super::*;

/// Issue #324: a follow-up such as "Сделай так, чтобы программа принимала путь
/// как аргумент" ("make the program accept a path as an argument") routes to
/// `write_program` because it pairs a program noun with an imperative verb, yet
/// it names neither a concrete task nor a language — both came from the previous
/// turn. Without conversation context this surfaced the user-reported error
/// ("I do not have a template for language `missing` and task `missing`").
///
/// When the selected rule is [`SelectedRule::UnsupportedWriteProgram`] we recover
/// the missing task and language from the most recent prior turn that named them
/// and apply any data-defined modification modifier present in the follow-up.
/// If the recovered `(task, language)` pair has a template we upgrade the rule
/// to a concrete program; otherwise we return the rule with whatever we could
/// fill in so the unsupported message is still as specific as possible.
#[must_use]
pub(crate) fn recover_write_program_rule(
    rule: SelectedRule,
    follow_up: &str,
    history: &[ConversationTurn],
) -> WriteProgramRecovery {
    let SelectedRule::UnsupportedWriteProgram { task, language } = &rule else {
        return WriteProgramRecovery {
            rule,
            trace: None,
            plan: None,
        };
    };

    let mut recovered_task = task.clone();
    let mut recovered_language = language.clone();

    if recovered_task.is_none() || recovered_language.is_none() {
        for turn in history.iter().rev() {
            let normalized = normalize_prompt(&turn.content);
            let Some(parameters) = write_program_parameters(&normalized) else {
                continue;
            };
            if recovered_task.is_none() {
                recovered_task = parameters.get("task").cloned();
            }
            if recovered_language.is_none() {
                recovered_language = parameters.get("language").cloned();
            }
            if recovered_task.is_some() && recovered_language.is_some() {
                break;
            }
        }
    }

    // A modification follow-up lowers the recovered base task through the Links
    // Notation substitution pipeline, which rewrites e.g. `list_files ->
    // list_files_arg` or `list_files_arg -> list_files_arg_reverse_sort`. The
    // plan is captured as Links Notation for transparent tracing.
    let normalized_follow_up = normalize_prompt(follow_up);
    let modifiers = detected_program_modifiers(&normalized_follow_up);
    let mut plan = None;
    if !modifiers.is_empty() {
        if let Some(base) = recovered_task.as_deref() {
            let lowered = crate::program_plan::lower(base, &modifiers);
            if lowered.was_modified() {
                plan = Some(lowered.links_notation());
            }
            recovered_task = Some(lowered.resolved_task);
        }
    }

    if let (Some(task_slug), Some(language_slug)) =
        (recovered_task.as_deref(), recovered_language.as_deref())
    {
        if let Some(spec) = program_spec(task_slug, language_slug) {
            let trace = format!("write_program task={task_slug} language={language_slug}");
            return WriteProgramRecovery {
                rule: SelectedRule::WriteProgram(spec),
                trace: Some(trace),
                plan,
            };
        }
    }

    WriteProgramRecovery {
        rule: SelectedRule::UnsupportedWriteProgram {
            task: recovered_task,
            language: recovered_language,
        },
        trace: None,
        plan,
    }
}
