//! A catalog task whose output is the request's own operand (issue #1173
//! R1173-3): "a program that prints hello".
//!
//! The `print_text` row of `data/seed/hello-world-programs.lino` stores no
//! program. It names the `procedure` it is composed from (the `hello_world`
//! print procedure the documentation route rediscovers from the captured
//! pages), and its `output` is the `{operand}` slot. The program is that
//! procedure with the request's text bound into its output literal
//! ([`crate::discovery_production::rediscover_documented_program`]), so no
//! program is memorized per text or per language. The JavaScript twins live in
//! `js/worker/formal_ai_worker_program_requests.js`.

use std::sync::OnceLock;

use crate::coding::{ProgramSpec, ProgramTask, program_language_by_slug, program_task_by_slug};
use crate::discovery_production::{
    RunContract, catalog_run_commands, rediscover_documented_program,
};
use crate::engine::{ExecutionRecipe, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::Language;
use crate::normal_markov::quoted_segment_spans;
use crate::seed::parser::parse_lino;

/// The marks that end the words an unquoted operand spans.
const OPERAND_ENDS: [char; 9] = [
    '.', '!', '?', ';', ':', '\u{3002}', '\u{ff01}', '\u{ff1f}', '\u{0964}',
];

/// The catalog tasks whose seed row names a `procedure`, each with it.
fn operand_tasks() -> &'static [(&'static ProgramTask, String)] {
    static CELL: OnceLock<Vec<(&'static ProgramTask, String)>> = OnceLock::new();
    CELL.get_or_init(|| {
        parse_lino(crate::seed::HELLO_WORLD_PROGRAMS_LINO)
            .children
            .iter()
            .filter(|node| node.name.starts_with("task_"))
            .filter_map(|node| {
                let procedure = node.find_child_value("procedure");
                let task = program_task_by_slug(node.find_child_value("task"))?;
                (!procedure.is_empty()).then(|| (task, procedure.to_owned()))
            })
            .collect()
    })
}

/// A word with its edge punctuation removed, lowercased.
///
/// The JavaScript twin is `programBareWord`.
fn bare_word(word: &str) -> String {
    word.trim_matches(|character: char| !character.is_alphanumeric())
        .to_lowercase()
}

/// The text a request asks its program to print.
///
/// Read around the request's first word that evidences the `print_stdout`
/// meaning. After it: a quoted literal that opens right there, else the
/// clause's words up to a seeded separator or a sentence end, a trailing
/// implementation-language span set aside, when they are an utterance
/// ([`operand_utterance`]). Before it (a verb-final request, "जो नमस्ते
/// प्रिंट करे"): a quoted literal or a greeting that ends right there. `None`
/// when the request names no such text. The JavaScript twin is
/// `programTaskOperand`.
#[must_use]
pub fn program_task_operand(prompt: &str) -> Option<String> {
    let lexicon = crate::seed::lexicon();
    let print = lexicon.meaning("print_stdout")?;
    let words: Vec<&str> = prompt.split_whitespace().collect();
    let at = words
        .iter()
        .position(|word| print.evidenced_in(&bare_word(word)))?;
    let word = words[at];
    // A Han token carries the whole clause: the print surface splits it.
    let surface = if crate::coding::contains_cjk(word) {
        print
            .words()
            .find(|candidate| crate::coding::contains_cjk(candidate) && word.contains(*candidate))
    } else {
        None
    };
    let (head, tail) = surface
        .and_then(|surface| {
            word.find(surface)
                .map(|cut| (&word[..cut], &word[cut + surface.len()..]))
        })
        .unwrap_or(("", ""));
    let before = words[..at]
        .iter()
        .copied()
        .chain(std::iter::once(head))
        .collect::<Vec<_>>()
        .join(" ");
    let after = std::iter::once(tail)
        .chain(words[at + 1..].iter().copied())
        .collect::<Vec<_>>()
        .join(" ");
    operand_after(after.trim()).or_else(|| operand_before(before.trim()))
}

/// A quoted literal is an operand when it is one non-empty line.
fn operand_literal(text: &str) -> Option<String> {
    (!text.is_empty() && !text.contains('\n')).then(|| text.to_owned())
}

/// The seeded greetings, longest first.
fn greetings() -> Vec<String> {
    let mut greetings = crate::seed::lexicon().words_for_role(crate::seed::ROLE_SOCIAL_GREETING);
    greetings.sort_by_key(|greeting| std::cmp::Reverse(greeting.chars().count()));
    greetings
}

/// The operand that opens `after`, the text after the print word.
///
/// The JavaScript twin is `programOperandAfter`.
fn operand_after(after: &str) -> Option<String> {
    if let Some(quoted) = quoted_segment_spans(after).first()
        && quoted.start == 0
    {
        // A second literal in the same sentence ("prints 'a' and 'b'") leaves
        // the operand open.
        let rest = &after[quoted.end..];
        let sentence = rest
            .split(['\n', '.', ';', '\u{3002}'])
            .next()
            .unwrap_or(rest);
        if quoted_segment_spans(sentence).is_empty() {
            return operand_literal(&quoted.text);
        }
        return None;
    }
    let greetings = greetings();
    if after
        .chars()
        .next()
        .is_some_and(|first| crate::coding::contains_cjk(&first.to_string()))
    {
        return greetings.into_iter().find(|greeting| {
            crate::coding::contains_cjk(greeting) && after.starts_with(greeting.as_str())
        });
    }
    let lexicon = crate::seed::lexicon();
    let separators = lexicon.words_for_role(crate::seed::ROLE_SKILL_PROCEDURE_CLAUSE_SEPARATOR);
    let mut tokens: Vec<&str> = Vec::new();
    for word in after.split_whitespace() {
        if separators.contains(&bare_word(word)) {
            break;
        }
        tokens.push(word);
        if word.ends_with(OPERAND_ENDS) {
            break;
        }
    }
    let span: Vec<String> = [
        crate::seed::ROLE_IMPLEMENTATION_LANGUAGE_PREPOSITION,
        crate::seed::ROLE_IMPLEMENTATION_LANGUAGE_NOUN,
        crate::seed::ROLE_PROGRAM_LANGUAGE_ALIAS,
    ]
    .into_iter()
    .flat_map(|role| lexicon.words_for_role(role))
    .collect();
    while tokens.len() > 1
        && tokens
            .last()
            .is_some_and(|word| span.contains(&bare_word(word)))
    {
        tokens.pop();
    }
    operand_utterance(&tokens, &greetings)
}

/// The operand that ends `before`, the text before a verb-final print word.
///
/// The JavaScript twin is `programOperandBefore`.
fn operand_before(before: &str) -> Option<String> {
    if let Some(last) = quoted_segment_spans(before).last()
        && last.end == before.len()
    {
        return operand_literal(&last.text);
    }
    let tokens: Vec<&str> = before.split_whitespace().collect();
    for greeting in greetings() {
        if crate::coding::contains_cjk(&greeting) {
            if before.ends_with(greeting.as_str()) {
                return Some(greeting);
            }
            continue;
        }
        let size = greeting.split(' ').count();
        let Some(tail) = tokens.len().checked_sub(size).map(|start| &tokens[start..]) else {
            continue;
        };
        if tail
            .iter()
            .map(|word| bare_word(word))
            .collect::<Vec<_>>()
            .join(" ")
            == greeting
        {
            let phrase = tail.join(" ");
            return Some(
                phrase
                    .trim_matches(|character: char| !character.is_alphanumeric())
                    .to_owned(),
            );
        }
    }
    None
}

/// Whether `tokens` are an utterance to print rather than a description of a
/// value: an unquoted utterance (it opens with a capital and no word
/// describes a value, as a work obligation's output is read), else a seeded
/// greeting (`social_greeting`).
///
/// The JavaScript twin is `programOperandUtterance`.
fn operand_utterance(tokens: &[&str], greetings: &[String]) -> Option<String> {
    if let Some(output) = crate::intent_formalization::unquoted_utterance(tokens.iter().copied()) {
        return Some(output);
    }
    let joined = tokens.join(" ");
    let bare = joined.trim_matches(|character: char| !character.is_alphanumeric());
    (!bare.is_empty() && greetings.contains(&bare.to_lowercase())).then(|| bare.to_owned())
}

/// The catalog task a request names through its operand: the first task
/// whose seed row names a `procedure`, when the request names text to print.
///
/// The JavaScript twin is `operandProgramTask`.
#[must_use]
pub fn operand_task(prompt: &str) -> Option<&'static ProgramTask> {
    program_task_operand(prompt)?;
    operand_tasks().first().map(|(task, _)| *task)
}

/// Answer a `write_program` request for an operand task (R1173-3).
///
/// The task's `procedure` is rediscovered from the documentation captures
/// with the operand bound into its output literal and verified by
/// decomposing it again; the answer runs it with the run contract the
/// procedure's catalog pair binds (the same names, since only the literal
/// changed), and states that it was rediscovered from the page rather than
/// borrowing a recorded run. `None` when the task takes no operand, the
/// request names none, or no captured page verifies a program, which leaves
/// the caller on its existing path. The JavaScript twin is the operand arm of
/// `tryWriteProgram` with `operandProgram`.
pub fn try_write_operand_program(
    prompt: &str,
    task: Option<&str>,
    language: Option<&str>,
    reply: Language,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let (task, procedure) = operand_tasks()
        .iter()
        .find(|(candidate, _)| Some(candidate.slug) == task)
        .map(|(task, procedure)| (*task, procedure.as_str()))?;
    let operand = program_task_operand(prompt)?;
    let row = program_language_by_slug(language?)?;
    let commands = catalog_run_commands(row);
    let documented = rediscover_documented_program(
        row.slug,
        procedure,
        &operand,
        RunContract {
            save_as: &row.save_as,
            commands: &commands,
        },
    )
    .ok()?;
    let pair = crate::coding::documented_pair(procedure, row.slug)?;
    if documented.contract.save_as != *pair.language.save_as {
        return None;
    }
    let spec = ProgramSpec {
        language: &pair.language,
        task,
        template: crate::coding::program_template(procedure, row.slug)?,
    };
    let source = documented.recipe.rediscovery_source.as_str();
    let body = format!(
        "{}\n\n```{}\n{}\n```\n\n{}\n\n{}\n\n{}",
        crate::engine::write_program_intro(spec.language.name, task.label, reply),
        spec.language.code_fence,
        documented.recipe.entry,
        crate::engine::execution_report(
            spec.language,
            &spec.run_command_line(),
            &operand,
            reply,
            Some(source),
        ),
        crate::coding::guidance::program_explanation_section(spec, reply),
        crate::coding::guidance::program_test_instructions(spec, reply, false),
    );
    let content_id = format!("0x{:016x}", documented.recipe.content_id);
    log.append("program_parameter:language", row.slug.to_owned());
    log.append("program_parameter:task", task.slug.to_owned());
    log.append("program_parameters", spec.parameter_summary());
    log.append_fields(
        "program_operand",
        &[
            ("task", task.slug),
            ("procedure", procedure),
            ("source", source),
            ("content_id", content_id.as_str()),
        ],
    );
    let mut answer = crate::solver_handlers::finalize_simple(
        prompt,
        log,
        crate::coding::WRITE_PROGRAM_INTENT,
        &spec.response_link(),
        &body,
        0.9,
    );
    answer.execution_recipe = Some(Box::new(ExecutionRecipe {
        language: spec.language.code_fence.to_owned(),
        source: documented.recipe.entry.clone(),
        path: spec.language.save_as.to_string(),
        supporting_files: Vec::new(),
        commands: catalog_run_commands(spec.language)
            .into_iter()
            .map(str::to_owned)
            .collect(),
    }));
    Some(answer)
}
