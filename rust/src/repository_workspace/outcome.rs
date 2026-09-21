//! What a surface may say about a command it ran (#1138, wave F defect).
//!
//! Wave F recorded two defects in three lines of one transcript. A Russian
//! prompt — *"Запусти это и скажи точно, что оно печатает: print(sum(range(1,
//! 11)))"* — had its leading verb stripped and the remainder handed to
//! `/bin/sh -c`; the shell answered `syntax error near unexpected token '('`,
//! and the reply called the command **выполнена** — completed.
//!
//! Two things had to be true for that to happen: prose had to be runnable, and
//! a non-zero exit had to be renderable as a completion. The first is closed by
//! the default-deny allowlist in [`super::allows`] — the first word of that
//! sentence names no program, so nothing reaches a shell. The second is closed
//! here: there is no variant of [`CommandOutcome`] that carries a non-zero exit
//! and renders as a completion, and the sentences are seed rows in five
//! languages rather than a `match` in Rust.

use crate::seed::parser::parse_lino;

/// The five-language sentences a surface may say about a command.
const COMMAND_OUTCOME_LINO: &str = include_str!("../../../data/seed/command-outcome.lino");

/// Record type of one localized outcome sentence.
const RECORD_OUTCOME: &str = "command_outcome";

/// What actually happened to a command, as the thing a surface reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandOutcome {
    /// The text named no allowlisted program. It was never run.
    NotACommand {
        /// The text, verbatim, as it arrived.
        text: String,
    },
    /// The program is not on this machine. Neither a pass nor a failure.
    DidNotRun {
        /// The program that is missing.
        program: String,
    },
    /// Exit status zero, with what it printed.
    Completed {
        /// The observed output.
        output: String,
    },
    /// A non-zero exit, with the status and what it printed.
    DidNotComplete {
        /// The observed exit status, when one was reported.
        exit: Option<i64>,
        /// The observed output.
        output: String,
    },
    /// The deadline was reached. Both numbers are reported.
    TimedOut {
        /// The bound it was measured against.
        deadline_seconds: u64,
        /// How long it actually ran.
        elapsed_seconds: u64,
        /// What it printed before the deadline.
        output: String,
    },
    /// Nothing could run the code at all (plan 06's honesty sentence).
    UnverifiedExecution,
}

impl CommandOutcome {
    /// Classify one observed exit status. There is no third answer: a status
    /// that is not zero did not complete, whatever the surface would prefer.
    #[must_use]
    pub fn of_exit(exit: Option<i64>, output: &str) -> Self {
        if exit == Some(0) {
            Self::Completed {
                output: output.to_owned(),
            }
        } else {
            Self::DidNotComplete {
                exit,
                output: output.to_owned(),
            }
        }
    }

    /// The seed row this outcome is rendered from.
    #[must_use]
    pub const fn seed_id(&self) -> &'static str {
        match self {
            Self::NotACommand { .. } => "prose_is_not_a_command",
            Self::DidNotRun { .. } => "command_did_not_run",
            Self::Completed { .. } => "command_completed",
            Self::DidNotComplete { .. } => "command_did_not_complete",
            Self::TimedOut { .. } => "command_timed_out",
            Self::UnverifiedExecution => "unverified_execution",
        }
    }

    /// Whether this outcome may be presented as the command having been done.
    #[must_use]
    pub const fn is_completion(&self) -> bool {
        matches!(self, Self::Completed { .. })
    }
}

/// The sentence for `outcome` in `language`, from seed.
///
/// Falls back to English only when seed carries no row for the language, and
/// says nothing at all when seed carries no row for the outcome — an empty
/// string is a visible gap, where a fabricated sentence would not be.
#[must_use]
pub fn describe_outcome(outcome: &CommandOutcome, language: &str) -> String {
    let template = seed_sentence(outcome.seed_id(), language);
    match outcome {
        CommandOutcome::NotACommand { .. } | CommandOutcome::UnverifiedExecution => template,
        CommandOutcome::DidNotRun { program } => {
            template.replace(concat!("{", "program", "}"), program)
        }
        CommandOutcome::Completed { output } => {
            template.replace(concat!("{", "output", "}"), output)
        }
        CommandOutcome::DidNotComplete { exit, output } => template
            .replace(
                concat!("{", "exit", "}"),
                &exit.map_or_else(|| String::from("none"), |code| code.to_string()),
            )
            .replace(concat!("{", "output", "}"), output),
        CommandOutcome::TimedOut {
            deadline_seconds,
            elapsed_seconds,
            output,
        } => template
            .replace(concat!("{", "deadline", "}"), &deadline_seconds.to_string())
            .replace(concat!("{", "elapsed", "}"), &elapsed_seconds.to_string())
            .replace(concat!("{", "output", "}"), output),
    }
}

/// One localized sentence from `data/seed/command-outcome.lino`.
#[must_use]
pub fn seed_sentence(id: &str, language: &str) -> String {
    let root = parse_lino(COMMAND_OUTCOME_LINO);
    let Some(record) = root.children.iter().find(|node| {
        node.find_child_value("record_type") == RECORD_OUTCOME && node.find_child_value("id") == id
    }) else {
        return String::new();
    };
    let localized = record.find_child_value(language);
    if localized.trim().is_empty() {
        record.find_child_value("en").to_owned()
    } else {
        localized.to_owned()
    }
}

/// The languages seed carries a sentence for, in declaration order.
#[must_use]
pub fn seeded_languages(id: &str) -> Vec<String> {
    parse_lino(COMMAND_OUTCOME_LINO)
        .children
        .iter()
        .find(|node| {
            node.find_child_value("record_type") == RECORD_OUTCOME
                && node.find_child_value("id") == id
        })
        .map(|record| {
            record
                .children
                .iter()
                .filter(|field| field.name != "record_type" && field.name != "id")
                .map(|field| field.name.clone())
                .collect()
        })
        .unwrap_or_default()
}
