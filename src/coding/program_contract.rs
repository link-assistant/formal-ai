//! Compose a process from explicit output requirements and source-backed operations.
//!
//! The output, implementation language, and artifact are independent operands.
//! A source path cannot become a string literal merely because it is quoted.

use crate::engine::{ExecutionRecipe, ExecutionRecipeFile, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::normal_markov::quoted_segment_spans;
use crate::seed::{self, parser::parse_lino};

const CONTRACTS: &str = include_str!("../../data/meta/stdout-program-contracts.lino");

pub fn runtime_steps(language: &str) -> Option<String> {
    let root = parse_lino(CONTRACTS);
    let contract = root
        .children
        .first()?
        .children
        .iter()
        .find(|node| node.name == "language" && node.id == language)?;
    Some(contract.find_child_value("ci_setup").to_owned())
}

/// Read an explicitly quoted output operand in its own clause.
fn explicit_stdout(prompt: &str) -> Option<String> {
    let mut previous_end = 0;
    let mut outputs = Vec::new();
    for literal in quoted_segment_spans(prompt) {
        let prefix = &prompt[previous_end..literal.start];
        previous_end = literal.end;
        let clause = prefix
            .rsplit(['\n', '.', ';', '。'])
            .next()
            .unwrap_or(prefix);
        if seed::lexicon()
            .meaning("print_stdout")
            .is_some_and(|meaning| meaning.evidenced_in(&clause.to_lowercase()))
        {
            outputs.push(literal.text);
        }
    }
    (!outputs.is_empty()).then(|| outputs.join("\n"))
}

/// A program authoring clause, excluding output literals and page navigation.
fn program_language(prompt: &str) -> Option<String> {
    let lexicon = seed::lexicon();
    prompt.lines().find_map(|line| {
        let outside = crate::solver_handlers::text_outside_quoted_segments(line);
        let normalized = crate::engine::normalize_prompt(&outside);
        (lexicon
            .meaning("coding_request_program")
            .is_some_and(|meaning| meaning.evidenced_in(&normalized))
            && (lexicon.mentions_role(seed::ROLE_PROGRAM_REQUEST, &normalized)
                || lexicon.mentions_role(seed::ROLE_CODING_REQUEST_VERB, &normalized)))
        .then(|| crate::implementation_language::requested(&normalized))
        .flatten()
    })
}

/// Build only the operation the request explicitly specifies. Other program
/// behaviors continue through discovery/composition rather than being guessed.
#[allow(
    clippy::literal_string_with_formatting_args,
    reason = "bind named operands in source-backed Links Notation templates"
)]
pub fn answer(prompt: &str, log: &mut EventLog) -> Option<SymbolicAnswer> {
    let language = program_language(prompt)?;
    let output = explicit_stdout(prompt)?;
    let catalog = super::program_language_by_slug(&language)?;
    let extension = std::path::Path::new(catalog.save_as)
        .extension()?
        .to_str()?;
    let path = crate::agentic_coding::general_planner::typed_write_target(prompt, extension)
        .unwrap_or_else(|| catalog.save_as.to_owned());
    let root = parse_lino(CONTRACTS);
    let contract = root
        .children
        .first()?
        .children
        .iter()
        .find(|node| node.name == "language" && node.id == language)?;
    let value = string_literal(
        &output,
        contract.find_child_value("escape_characters"),
        contract.find_child_value("unicode_escape"),
    );
    let body = contract
        .find_child_value("operation")
        .replace("{value}", &value);
    let source = contract.find_child_value("entry").replace("{body}", &body);
    let mut commands: Vec<String> = catalog
        .execution
        .check_command
        .into_iter()
        .chain(std::iter::once(catalog.execution.run_command))
        .map(|command| command.replace(catalog.save_as, &path))
        .collect();
    let comment = contract.find_child_value("comment");
    let source_url = contract.find_child_value("source");
    let instructions = commands
        .iter()
        .map(|command| {
            root.children[0]
                .find_child_value("instruction")
                .replace("{comment}", comment)
                .replace("{command}", command)
        })
        .collect::<String>();
    let source = root.children[0]
        .find_child_value("documented_source")
        .replace("{instructions}", &instructions)
        .replace("{comment}", comment)
        .replace("{source}", &source);
    log.append(
        "synthesis:composition",
        "entry(print_stdout(literal))".to_owned(),
    );
    log.append("program_parameter:language", language.clone());
    log.append("program_parameter:expected_stdout", output.clone());
    log.append("knowledge_source_url", source_url.to_owned());
    let mut answer = crate::solver_handlers::finalize_simple(
        prompt,
        log,
        "write_program",
        "response:write_program:composition",
        &root.children[0]
            .find_child_value("response")
            .replace("{language}", &language)
            .replace("{source_url}", source_url)
            .replace("{source}", &source),
        1.0,
    );
    let verifier = output_verifier(&output, commands.last()?)?;
    *commands.last_mut()? = format!("sh {}", verifier.path);
    answer.execution_recipe = Some(Box::new(ExecutionRecipe {
        language,
        source,
        path,
        supporting_files: vec![verifier],
        commands,
    }));
    Some(answer)
}

#[allow(
    clippy::literal_string_with_formatting_args,
    reason = "bind operands in the shared process-verification template"
)]
fn output_verifier(expected: &str, command: &str) -> Option<ExecutionRecipeFile> {
    let root = parse_lino(include_str!("../../data/meta/process-verification.lino"));
    let contract = root.children.first()?;
    let expected = format!("'{}'", expected.replace('\'', "'\\''"));
    Some(ExecutionRecipeFile {
        path: contract.find_child_value("path").to_owned(),
        source: contract
            .find_child_value("template")
            .replace("{command}", command)
            .replace("{expected}", &expected),
    })
}

fn string_literal(value: &str, extra_escapes: &str, unicode_escape: &str) -> String {
    let mut out = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            value if value.is_control() => out.push_str(
                &unicode_escape
                    .replace("{hex4}", &format!("{:04x}", u32::from(value)))
                    .replace("{hex}", &format!("{:x}", u32::from(value)))
                    .replace("{{", "{")
                    .replace("}}", "}"),
            ),
            value => {
                if extra_escapes.contains(value) {
                    out.push('\\');
                }
                out.push(value);
            }
        }
    }
    out.push('"');
    out
}
