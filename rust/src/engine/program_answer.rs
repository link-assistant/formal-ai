//! Render generated programs and the evidence for their expected output.

use crate::coding::guidance::{guidance, program_explanation_section, program_test_instructions};
use crate::coding::{ProgramExecution, ProgramLanguage, ProgramSpec};
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
    write_program_answer_with_output(spec, language, prior_code_response, spec.task.output, None)
}

pub(crate) fn write_program_answer_with_output(
    spec: ProgramSpec<'_>,
    language: Language,
    prior_code_response: bool,
    expected_output: &str,
    rediscovered_from: Option<&str>,
) -> String {
    format!(
        "{}\n\n```{}\n{}\n```\n\n{}\n\n{}\n\n{}",
        write_program_intro(spec.language.name, spec.task.label, language),
        spec.language.code_fence,
        spec.template.code,
        execution_report(
            spec.language,
            spec.run_command_line(),
            expected_output,
            language,
            rediscovered_from,
        ),
        program_explanation_section(spec, language, expected_output),
        program_test_instructions(spec, language, prior_code_response, expected_output),
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
) -> String {
    let (status_line, output_key) = match rediscovered_from {
        Some(page) => (
            guidance("execution-rediscovered", language, &[("page", page)]),
            "execution-output-unavailable",
        ),
        None => (
            guidance(
                "execution-composed-line",
                language,
                &[(
                    "status",
                    &guidance("execution-status-not-probed", language, &[]),
                )],
            ),
            "execution-output-not-probed",
        ),
    };
    let commands = execution_command_lines(&program_language.execution, run_command);
    let mut report = format!("{status_line}\n{commands}");
    if !output.is_empty() {
        let label = guidance(output_key, language, &[]);
        report.push_str(&format!("\n{label}:\n```text\n{output}\n```"));
    }
    report
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
