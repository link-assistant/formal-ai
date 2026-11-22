//! Render generated programs and the evidence for their expected output.

use crate::coding::guidance::{guidance, program_explanation_section, program_test_instructions};
use crate::coding::{ExecutionStatus, ProgramExecution, ProgramLanguage, ProgramSpec};
use crate::language::Language;

pub(super) fn write_program_answer(
    spec: ProgramSpec<'_>,
    language: Language,
    prior_code_response: bool,
) -> String {
    // Issue #324: the natural-language framing around the generated program is
    // localized to the detected (or preferred) response language so a Russian,
    // Hindi, or Chinese request no longer receives an all-English reply. The
    // code itself and the literal shell commands stay in their canonical form
    // because they are the requested artefact, not prose.
    //
    // Issue #330: a code answer must teach a novice — so the program is always
    // accompanied by a plain-language explanation of *how it works* and
    // step-by-step instructions for testing it. When the dialog already walked
    // the user through running code (`prior_code_response`), the verbose setup
    // steps are omitted and replaced by a short "test it the same way" note so
    // follow-up edits stay concise.
    write_program_answer_with_output(
        spec,
        language,
        prior_code_response,
        &spec.expected_output(),
        None,
        spec.template.composed,
    )
}

pub(crate) fn write_program_answer_with_output(
    spec: ProgramSpec<'_>,
    language: Language,
    prior_code_response: bool,
    expected_output: &str,
    rediscovered_from: Option<&str>,
    composed: bool,
) -> String {
    format!(
        "{}\n\n```{}\n{}\n```\n\n{}\n\n{}\n\n{}",
        write_program_intro(spec.language.name, spec.task.label, language),
        spec.language.code_fence,
        spec.template.code,
        execution_report(
            spec.language,
            &spec.run_command_line(),
            expected_output,
            language,
            rediscovered_from,
            composed,
        ),
        program_explanation_section(spec, language, expected_output),
        program_test_instructions(spec, language, prior_code_response),
    )
}

fn write_program_intro(language_name: &str, task_label: &str, language: Language) -> String {
    guidance(
        "program-intro",
        language,
        &[("language", language_name), ("task", task_label)],
    )
}

pub(crate) fn execution_report(
    program_language: &ProgramLanguage,
    run_command: &str,
    output: &str,
    language: Language,
    rediscovered_from: Option<&str>,
    composed: bool,
) -> String {
    let (status, status_line) = if let Some(page) = rediscovered_from {
        (
            ExecutionStatus::Unavailable,
            guidance("execution-rediscovered", language, &[("page", page)]),
        )
    } else if composed {
        (
            ExecutionStatus::NotProbed,
            guidance(
                "execution-composed-line",
                language,
                &[(
                    "status",
                    &guidance("execution-status-not-probed", language, &[]),
                )],
            ),
        )
    } else {
        let status = program_language.execution_status();
        let phrase = guidance(
            &format!("execution-status-{}", status_slug(status)),
            language,
            &[],
        );
        (
            status,
            guidance(
                "execution-status-line",
                language,
                &[
                    ("status", &phrase),
                    ("environment", &program_language.environment()),
                ],
            ),
        )
    };
    let commands = execution_command_lines(&program_language.execution, run_command);
    let output_label = guidance(
        &format!("execution-output-{}", status_slug(status)),
        language,
        &[],
    );
    let mut report = format!("{status_line}\n{commands}\n{output_label}:\n```text\n{output}\n```");
    if rediscovered_from.is_none() && !composed {
        report.push('\n');
        report.push_str(&program_language.execution_notes());
    }
    report
}

fn status_slug(status: ExecutionStatus) -> &'static str {
    match status {
        ExecutionStatus::Verified => "verified",
        ExecutionStatus::Unavailable => "unavailable",
        ExecutionStatus::NotProbed => "not-probed",
    }
}

fn execution_command_lines(execution: &ProgramExecution, run_command: &str) -> String {
    let run = guidance(
        "execution-run-command",
        Language::English,
        &[("command", run_command)],
    );
    match execution.check_command.as_deref() {
        Some(command) => {
            let check = guidance(
                "execution-check-command",
                Language::English,
                &[("command", command)],
            );
            format!("{check}\n{run}")
        }
        None => run,
    }
}
