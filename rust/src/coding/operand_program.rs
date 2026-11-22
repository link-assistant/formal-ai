//! A catalog task whose output is the request's own operand (issue #1173
//! R1173-3): "a program that prints hello".
//!
//! The `print_text` row of `data/seed/hello-world-programs.lino` stores no
//! program. It names the `procedure` it is composed from (the `hello_world`
//! print procedure), and its `output` is the `{operand}` slot. Publisher
//! examples are acquired and checked against that operand. The local stdout
//! operation can also render the same operand from language syntax alone.
//! The browser twin lives in `js/worker/formal_ai_worker_program_requests.js`.

use std::borrow::Cow;
use std::sync::OnceLock;

use crate::coding::{
    ProgramSpec, ProgramTask, ProgramTemplate, program_language_by_slug, program_task_by_slug,
};
use crate::discovery_production::{RunContract, catalog_run_commands, resolve_documented_program};
use crate::engine::SymbolicAnswer;
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
/// Owned quoted print data stays out of instruction classification; mirrors programTaskInstruction.
#[must_use]
pub fn program_instruction_view(prompt: &str) -> String {
    let Some(operand) = program_task_operand(prompt) else {
        return prompt.to_owned();
    };
    let words: Vec<&str> = prompt.split_whitespace().collect();
    let Some(print) = crate::seed::lexicon().meaning("print_stdout") else {
        return prompt.to_owned();
    };
    let Some(at) = words
        .iter()
        .position(|word| print.evidenced_in(&bare_word(word)))
    else {
        return prompt.to_owned();
    };
    let word = words[at];
    let mut start = 0;
    for preceding in &words[..at] {
        let Some(offset) = prompt[start..].find(*preceding) else {
            return prompt.to_owned();
        };
        start += offset + preceding.len();
    }
    let Some(offset) = prompt[start..].find(word) else {
        return prompt.to_owned();
    };
    start += offset;
    let surface = if crate::coding::contains_cjk(word) {
        print
            .words()
            .find(|candidate| crate::coding::contains_cjk(candidate) && word.contains(*candidate))
    } else {
        None
    };
    let end = start
        + surface.map_or(word.len(), |surface| {
            word.find(surface).unwrap_or(0) + surface.len()
        });
    let suffix = &prompt[end..];
    let after = suffix.trim_start();
    let Some(quoted) = quoted_segment_spans(after).into_iter().next() else {
        return prompt.to_owned();
    };
    if quoted.start != 0 || operand_after(after.trim()).as_deref() != Some(operand.as_str()) {
        return prompt.to_owned();
    }
    let start = end + suffix.len() - after.len();
    format!(
        "{}{}{}",
        &prompt[..start],
        " ".repeat(quoted.end),
        &prompt[start + quoted.end..]
    )
}

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
    let mut start = 0;
    for preceding in &words[..at] {
        start += prompt[start..].find(*preceding)? + preceding.len();
    }
    start += prompt[start..].find(word)?;
    let before = &prompt[..start + head.len()];
    let after = &prompt[start + word.len() - tail.len()..];
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
        let remainder = rest
            .trim()
            .trim_end_matches(['.', '!', '?', '。', '！', '？', '।'])
            .trim();
        let normalized = crate::engine::normalize_prompt(&remainder.to_lowercase());
        let lexicon = crate::seed::lexicon();
        let nouns = lexicon.words_for_role(crate::seed::ROLE_PROGRAM_GENUS);
        let mut accounted = nouns.clone();
        accounted.extend(lexicon.words_for_role(crate::seed::ROLE_REQUEST_FUNCTION_WORD));
        let nominal = nouns.iter().any(|noun| normalized.contains(noun.as_str()))
            && remainder.to_lowercase().split_whitespace().all(|word| {
                if accounted.iter().any(|candidate| candidate.as_str() == word) {
                    return true;
                }
                if !crate::coding::contains_cjk(word) {
                    return false;
                }
                let mut rest = word.to_owned();
                while let Some(surface) = accounted.iter().find(|surface| {
                    crate::coding::contains_cjk(surface) && rest.contains(surface.as_str())
                }) {
                    rest = rest.replace(surface.as_str(), "");
                }
                rest.is_empty()
            });
        let complete = remainder.is_empty()
            || nominal
            || lexicon
                .words_for_role("program-termination-clause")
                .contains(&remainder.to_lowercase());
        if complete && quoted_segment_spans(sentence).is_empty() {
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

/// The response format chosen by the calling handler.
pub(crate) enum ProgramReply {
    Program {
        language: Language,
        prior_code: bool,
    },
    Script,
}

impl ProgramReply {
    fn intent(&self, target: &str) -> Cow<'static, str> {
        match self {
            Self::Program { .. } => Cow::Borrowed(crate::coding::WRITE_PROGRAM_INTENT),
            Self::Script => Cow::Owned(format!("write_script_{target}")),
        }
    }

    fn render(&self, spec: ProgramSpec<'_>, output: &str, source: Option<&str>) -> String {
        match *self {
            Self::Program {
                language,
                prior_code,
            } => crate::engine::write_program_answer_with_output(
                spec, language, prior_code, output, source,
            ),
            Self::Script => format!(
                "{}\n\n```{}\n{}\n```\n\n{}",
                crate::coding::guidance::guidance(
                    "script-intro",
                    Language::English,
                    &[("language", spec.language.name)],
                ),
                spec.language.code_fence,
                spec.template.code,
                crate::engine::execution_report(
                    spec.language,
                    spec.run_command_line(),
                    output,
                    Language::English,
                    source,
                ),
            ),
        }
    }
}

/// Compose a program from a publisher page acquired under the request's policy.
pub(crate) fn try_write_documented_program(
    prompt: &str,
    task: &str,
    language: &str,
    reply: ProgramReply,
    documentation_access: Option<bool>,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let task = program_task_by_slug(task)?;
    let row = program_language_by_slug(language)?;
    let operand_procedure = operand_tasks()
        .iter()
        .find(|(candidate, _)| candidate.slug == task.slug)
        .map(|(_, procedure)| procedure.as_str());
    let procedure = operand_procedure.unwrap_or(task.slug);
    let output = match operand_procedure {
        Some(_) => program_task_operand(prompt)?,
        None if task.slug == "hello_world" => {
            crate::code_editing::inline_hello_world_replacement(prompt)
                .unwrap_or_else(|| task.output.to_owned())
        }
        None => task.output.to_owned(),
    };
    let commands = catalog_run_commands(row);
    let documented = documentation_access.and_then(|offline| {
        if crate::discovery_production::documentation_captures(language, procedure).is_empty() {
            return None;
        }
        match resolve_documented_program(
            row.slug,
            procedure,
            &output,
            RunContract {
                save_as: &row.save_as,
                commands: &commands,
            },
            offline,
        ) {
            Ok(program) => Some(program),
            Err(reason) => {
                crate::discovery_production::record_program_cache_miss(
                    log, row.slug, task.slug, &reason,
                );
                None
            }
        }
    });
    let Some(documented) = documented else {
        if operand_procedure.is_none() {
            return None;
        }
        let code = crate::coding::catalog::literal_output_program(language, &output)?;
        let template = ProgramTemplate {
            task_slug: task.slug,
            language_slug: row.slug,
            code: Cow::Owned(code),
        };
        let spec = ProgramSpec {
            language: row,
            task,
            template: &template,
        };
        let body = reply.render(spec, &output, None);
        log.append("program_parameter:language", row.slug.to_owned());
        log.append("program_parameter:task", task.slug.to_owned());
        log.append("program_parameters", spec.parameter_summary());
        log.append("legacy_intent", spec.legacy_intent());
        log.append(
            "program_operand",
            [
                format!("task={}", task.slug),
                format!("procedure={procedure}"),
            ]
            .join(" "),
        );
        log.append(
            "execution_status",
            crate::coding::ExecutionStatus::NotProbed.label().to_owned(),
        );
        record_unfulfilled_obligations(prompt, log);
        let mut answer = crate::solver_handlers::finalize_simple(
            prompt,
            log,
            &reply.intent(row.slug),
            &spec.response_link(),
            &body,
            0.9,
        );
        answer.execution_recipe = Some(Box::new(crate::engine::ExecutionRecipe {
            language: row.code_fence.to_owned(),
            source: spec.template.code.to_string(),
            path: row.save_as.to_string(),
            supporting_files: Vec::new(),
            commands: commands.into_iter().map(str::to_owned).collect(),
        }));
        return Some(answer);
    };
    let template = ProgramTemplate {
        task_slug: task.slug,
        language_slug: row.slug,
        code: Cow::Owned(documented.recipe.entry.clone()),
    };
    let language = crate::coding::bound_language(row, &documented.contract);
    let spec = ProgramSpec {
        language: &language,
        task,
        template: &template,
    };
    let source = documented.recipe.rediscovery_source.as_str();
    let body = reply.render(spec, &output, Some(source));
    let content_id = format!("0x{:016x}", documented.recipe.content_id);
    log.append(
        "execution_status",
        crate::coding::ExecutionStatus::Unavailable
            .label()
            .to_owned(),
    );
    log.append("execution_environment", source.to_owned());
    log.append("program_parameter:language", row.slug.to_owned());
    log.append("program_parameter:task", task.slug.to_owned());
    log.append("program_parameters", spec.parameter_summary());
    log.append("legacy_intent", spec.legacy_intent());
    for (kind, payload) in
        crate::discovery_production::documentation_events(row.slug, task.slug, &documented)
    {
        log.append(kind, payload);
    }
    if operand_procedure.is_some() {
        log.append_fields(
            "program_operand",
            &[
                ("task", task.slug),
                ("procedure", procedure),
                ("source", source),
                ("content_id", &content_id),
            ],
        );
    } else {
        let outcome = if documented.cached {
            "hit"
        } else {
            "discovered"
        };
        log.append_fields(
            "procedure_cache",
            &[
                ("outcome", outcome),
                ("language", row.slug),
                ("task", task.slug),
                ("rediscovery_source", source),
                ("content_id", &content_id),
            ],
        );
    }
    record_unfulfilled_obligations(prompt, log);
    Some(crate::solver_handlers::finalize_simple(
        prompt,
        log,
        &reply.intent(row.slug),
        &spec.response_link(),
        &body,
        0.9,
    ))
}

fn record_unfulfilled_obligations(prompt: &str, log: &mut EventLog) {
    if crate::intent_formalization::request_carries_work_obligations(prompt)
        || crate::intent_formalization::request_demands(
            prompt,
            crate::intent_formalization::ObligationKind::CiWorkflow,
        )
    {
        crate::intent_formalization::record_obligation_gaps(prompt, log);
    }
}

pub(crate) fn write_script_program(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    offline: bool,
) -> Option<SymbolicAnswer> {
    let (task, language) = print_script_request(prompt, normalized)?;
    try_write_documented_program(
        prompt,
        task.slug,
        language,
        ProgramReply::Script,
        Some(offline),
        log,
    )
}

pub(crate) fn print_script_request(
    prompt: &str,
    normalized: &str,
) -> Option<(&'static ProgramTask, &'static str)> {
    let task = operand_task(prompt)?;
    if !crate::solver_helpers::is_write_script_request(prompt, normalized) {
        return None;
    }
    let instructions = crate::engine::normalize_prompt(&program_instruction_view(prompt));
    let language = crate::coding::program_language_by_alias(&instructions)?;
    Some((task, language.slug))
}
