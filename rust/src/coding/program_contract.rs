//! Compose a process from explicit output requirements and source-backed operations.
//!
//! The output, implementation language, and artifact are independent operands.
//! A source path cannot become a string literal merely because it is quoted.

use crate::engine::{ExecutionRecipe, ExecutionRecipeFile, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::seed::{self, parser::parse_lino};

const CONTRACTS: &str = include_str!("../../embedded/data/meta/stdout-program-contracts.lino");

#[must_use]
pub fn runtime_steps(language: &str) -> Option<String> {
    let root = parse_lino(CONTRACTS);
    let contract = root
        .children
        .first()?
        .children
        .iter()
        .find(|node| node.name == "language" && node.id == language)?;
    Some(crate::version_resolution::fill_workflow_versions(
        contract.find_child_value("ci_setup"),
        &crate::version_resolution::VersionSet::for_generation(),
    ))
}

/// Read the request's explicitly quoted output operands.
///
/// The operands come from the obligation graph's clauses
/// ([`crate::intent_formalization::bound_output_literals`]): a quoted value
/// introduced by a print clause of its own obligation clause, each value
/// once, in request order (R1166-3).
#[must_use]
pub fn explicit_stdout(prompt: &str) -> Option<String> {
    let outputs = crate::intent_formalization::bound_output_literals(prompt);
    (!outputs.is_empty()).then(|| outputs.join("\n"))
}

/// A program authoring clause, excluding output literals and page navigation.
fn program_language(prompt: &str) -> Option<String> {
    let lexicon = seed::lexicon();
    prompt
        .lines()
        .find_map(|line| {
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
        .or_else(|| named_source_language(prompt))
}

/// A bare-word file path (`hello.py,` → `hello.py`), sentence marks peeled.
fn source_path(token: &str) -> &str {
    token
        .trim_start_matches(['`', '"', '\'', '('])
        .trim_end_matches(['`', '"', '\'', ',', ';', ':', '.', '!', '?', ')'])
}

/// The one relative source file with `extension` the request names, cued or
/// not: in "Write a Python program hello.py that prints …" the noun before
/// the path is no write cue, yet the path names the program's file the way
/// [`named_source_language`] reads its language from it (PR #1188 T18).
/// Two distinct such files name none.
fn named_source_file(prompt: &str, extension: &str) -> Option<String> {
    let outside = crate::solver_handlers::text_outside_quoted_segments(prompt);
    let mut paths: Vec<&str> = outside
        .split_whitespace()
        .filter(|token| source_extension(token) == Some(extension))
        .map(source_path)
        .filter(|path| !path.starts_with('/') && !path.split('/').any(|part| part == ".."))
        .collect();
    paths.dedup();
    match paths.as_slice() {
        [path] => Some((*path).to_owned()),
        _ => None,
    }
}

/// A bare-word file's extension (`hello.py` → `py`), sentence marks peeled.
fn source_extension(token: &str) -> Option<&str> {
    let path = source_path(token);
    let name = path.rsplit('/').next()?;
    let (stem, extension) = name.rsplit_once('.')?;
    (!stem.is_empty()
        && !extension.is_empty()
        && extension
            .chars()
            .all(|character| character.is_ascii_alphanumeric()))
    .then_some(extension)
}

/// A line that asks for printed output and names exactly one source file whose
/// extension is the saved-file extension of exactly one catalogued language
/// (`hello.py` → python) is a program request in that language, the way a
/// programmer reads the file name (PR #1188 dogfooding). Only a line that asks
/// to write or create one: `Change greet.py so it prints "Hi"` edits a file,
/// it does not replace it.
fn named_source_language(prompt: &str) -> Option<String> {
    let lexicon = seed::lexicon();
    prompt.lines().find_map(|line| {
        let outside = crate::solver_handlers::text_outside_quoted_segments(line);
        let normalized = crate::engine::normalize_prompt(&outside);
        if !lexicon
            .meaning("print_stdout")
            .is_some_and(|meaning| meaning.evidenced_in(&normalized))
            || !lexicon.mentions_role(seed::ROLE_CODING_REQUEST_VERB, &normalized)
        {
            return None;
        }
        let mut slugs: Vec<&str> = outside
            .split_whitespace()
            .filter_map(source_extension)
            .flat_map(|extension| {
                crate::coding::catalog::PROGRAM_LANGUAGES
                    .iter()
                    .filter(move |language| {
                        language.framework_of.is_none()
                            && source_extension(&language.save_as) == Some(extension)
                    })
                    .map(|language| language.slug)
            })
            .collect();
        slugs.sort_unstable();
        slugs.dedup();
        match slugs.as_slice() {
            [slug] => Some((*slug).to_owned()),
            _ => None,
        }
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
    let extension = std::path::Path::new(catalog.save_as.as_ref())
        .extension()?
        .to_str()?;
    let path = crate::agentic_coding::general_planner::typed_write_target(prompt, extension)
        .or_else(|| named_source_file(prompt, extension))
        .unwrap_or_else(|| catalog.save_as.to_string());
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
        .as_deref()
        .into_iter()
        .chain(std::iter::once(catalog.execution.run_command.as_ref()))
        .map(|command| command.replace(catalog.save_as.as_ref(), &path))
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
    // R1166-3/R1166-4: clauses the obligation graph cannot read, and output
    // literals it demands but the binding above does not carry, are reported
    // in the run's derivation, never dropped.
    crate::intent_formalization::record_obligation_gaps(prompt, log);
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
    let root = parse_lino(include_str!(
        "../../embedded/data/meta/process-verification.lino"
    ));
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

/// The command that calls the function `source` defines with `arguments` and
/// prints its result, from the language's `definition` and `call` contract
/// templates (PR #1188 dogfooding: "… in add.py and run it with 2 and 3").
/// `None` when the language has no call template, no definition matches, or
/// the argument count is not the parameter count.
#[must_use]
#[allow(
    clippy::literal_string_with_formatting_args,
    reason = "bind named operands in source-backed Links Notation templates"
)]
pub fn function_call_command(
    language: &str,
    path: &str,
    source: &str,
    arguments: &[String],
) -> Option<String> {
    if arguments.is_empty() {
        return None;
    }
    let root = parse_lino(CONTRACTS);
    let contract = root
        .children
        .first()?
        .children
        .iter()
        .find(|node| node.name == "language" && node.id == language)?;
    let call = contract.find_child_value("call");
    let (name, parameters) = defined_function(source, contract.find_child_value("definition"))?;
    if call.is_empty() || parameters != arguments.len() {
        return None;
    }
    let module = path
        .rsplit_once('.')
        .filter(|(_, extension)| !extension.is_empty() && !extension.contains('/'))
        .map_or(path, |(stem, _)| stem)
        .replace('/', ".");
    Some(
        call.replace("{module}", &module)
            .replace("{name}", &name)
            .replace("{arguments}", &arguments.join(", ")),
    )
}

/// The name and parameter count of the first definition `template` matches.
#[allow(
    clippy::literal_string_with_formatting_args,
    reason = "the definition template's named slots"
)]
fn defined_function(source: &str, template: &str) -> Option<(String, usize)> {
    let (prefix, after_name) = template.split_once("{name}")?;
    let (middle, suffix) = after_name.split_once("{parameters}")?;
    if prefix.is_empty() || middle.is_empty() {
        return None;
    }
    source.lines().find_map(|raw| {
        let rest = raw.trim_start().strip_prefix(prefix)?;
        let at = rest.find(middle).filter(|at| *at > 0)?;
        let name = &rest[..at];
        let tail = &rest[at + middle.len()..];
        let end = tail.rfind(suffix)?;
        let mut characters = name.chars();
        let identifier = characters
            .next()
            .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
            && characters.all(|character| character.is_ascii_alphanumeric() || character == '_');
        identifier.then(|| {
            let parameters = tail[..end]
                .split(',')
                .map(|part| part.split([':', '=']).next().unwrap_or_default().trim())
                .filter(|part| !part.is_empty())
                .count();
            (name.to_owned(), parameters)
        })
    })
}
