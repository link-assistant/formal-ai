//! Issue #330: novice-first guidance that accompanies every generated program.
//!
//! A code answer must *teach* a novice, so each `write_program` response is
//! followed by a plain-language "How it works" explanation and step-by-step
//! "How to test it yourself" instructions. Both are localized for every
//! supported response language. These builders live alongside the coding
//! [`catalog`](crate::coding::catalog) and are kept out of `engine.rs` so that
//! module stays under the repository's per-file line-count limit.

use crate::coding::catalog::ProgramSpec;
use crate::language::Language;
use crate::seed::fill_template_once;
use crate::seed::parser::parse_lino;
use crate::solver::{ConversationRole, ConversationTurn};

/// Every guidance sentence, in every response language (R379): the
/// `coding_guidance` records of `data/seed/coding-guidance.lino`, which the
/// browser worker reads too.
const CODING_GUIDANCE_LINO: &str = include_str!("../../embedded/data/seed/coding-guidance.lino");

/// The guidance sentence `id` in `language`, falling back to English, with
/// each `{name}` slot filled from `values` in one pass. A record seed does not
/// carry renders as the empty string, a visible gap rather than invented
/// prose.
fn guidance(id: &str, language: Language, values: &[(&str, &str)]) -> String {
    let tree = parse_lino(CODING_GUIDANCE_LINO);
    let Some(record) = tree.children.iter().find(|node| {
        node.find_child_value("record_type") == "coding_guidance"
            && node.find_child_value("id") == id
    }) else {
        return String::new();
    };
    // A language slug never reads the record's own `id` or `record_type`.
    let localized = match language.slug() {
        "id" | "record_type" => "",
        slug => record.find_child_value(slug),
    };
    let template = if localized.is_empty() {
        record.find_child_value("en")
    } else {
        localized
    };
    fill_template_once(template, values)
}

/// Issue #330: did an earlier assistant turn already present a fenced code
/// block? When it did, follow-up code edits omit the verbose setup steps and
/// show a concise "test it the same way" note instead. Detected from the dialog
/// rather than hard-coded to a turn number.
pub fn history_has_prior_code(history: &[ConversationTurn]) -> bool {
    history.iter().any(|turn| {
        matches!(turn.role, ConversationRole::Assistant) && turn.content.contains("```")
    })
}

/// Issue #330: a "How it works" paragraph so a novice understands the program
/// instead of receiving an unexplained snippet. The explanation is localized
/// for every supported response language.
pub fn program_explanation_section(spec: ProgramSpec, language: Language) -> String {
    format!(
        "{}\n{}",
        guidance("how_it_works_heading", language, &[]),
        program_explanation(spec.task.slug, language)
    )
}

/// Plain-language description of the algorithm for each supported task. Kept
/// language-agnostic in the *programming* sense (every template implements the
/// same algorithm) and localized in the *response* sense (issue #330). A task
/// without a bespoke explanation gets the neutral `default_explanation`, which
/// avoids claiming behaviour the program may not have.
fn program_explanation(task_slug: &str, language: Language) -> String {
    let explanation = guidance(task_slug, language, &[]);
    if explanation.is_empty() {
        guidance("default_explanation", language, &[])
    } else {
        explanation
    }
}

/// Issue #330: step-by-step, novice-friendly instructions for testing the
/// program. When the dialog already walked the user through running code
/// (`prior_code_response`), the verbose setup steps are replaced by a short
/// "test it the same way" note so follow-up edits stay concise.
pub fn program_test_instructions(
    spec: ProgramSpec,
    language: Language,
    prior_code_response: bool,
) -> String {
    let execution = &spec.language.execution;
    let save_as: &str = &spec.language.save_as;
    // Issue #863: a task defined against standard input is not run by naming
    // the command alone — without the pipe the reader gets a program waiting on
    // a terminal, not the expected output. `run_command_line` carries the
    // fixture the task was verified against, and is the plain run command for
    // every task that reads no input.
    let run_command = spec.run_command_line();

    let values = [("save_as", save_as), ("run_command", run_command.as_str())];
    if prior_code_response {
        return guidance("prior_code_note", language, &values);
    }

    let heading = guidance("how_to_test_heading", language, &[]);
    let setup_hint = spec.language.setup_hint();
    let mut steps = vec![
        guidance("step_install", language, &[("setup_hint", &setup_hint)]),
        guidance("step_save", language, &values),
    ];
    if let Some(check_command) = execution.check_command.as_deref() {
        steps.push(guidance(
            "step_check",
            language,
            &[("check_command", check_command)],
        ));
    }
    steps.push(guidance("step_run", language, &values));
    steps.push(guidance("step_compare", language, &[]));

    let numbered = steps
        .iter()
        .enumerate()
        .map(|(index, step)| format!("{}. {step}", index + 1))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{heading}\n{numbered}")
}
