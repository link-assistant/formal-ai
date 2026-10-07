// The run contract a documented program binds (issue #1165 R1165-6), and
// what else answering it needs, included into `discovery_production`.
//
// A catalog row fixes the file a program is saved as and the commands that
// check and run it. A program rediscovered from documentation keeps them
// when it declares every name they invoke; otherwise the program's own name
// binds in their place, the way a documented file name binds Kotlin's
// (`kotlinc hello.kt` states `kotlinc Main.kt`). Every command carries the
// page whose captured line states it, or `catalog`. The JavaScript twins are
// `documentedRunContract`, `documentationDeviation` and `documentationEvents`
// in js/worker/formal_ai_worker_code_examples.js.

/// The part vocabulary the decomposer reads, for its per-language rows.
const CODE_EXAMPLE_PARTS: &str = include_str!("../embedded/data/seed/code-example-parts.lino");

/// The source of a command no captured page states.
pub const COMMAND_SOURCE_CATALOG: &str = "catalog";

/// The shell prompts a documented command line may start with.
const COMMAND_PROMPTS: [&str; 3] = ["$ ", "% ", "> "];

/// One command a documented program is run with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcedCommand {
    /// `check` or `run`.
    pub role: &'static str,
    /// The command, with the program's name bound.
    pub command: String,
    /// The page whose captured line states the command, or
    /// [`COMMAND_SOURCE_CATALOG`].
    pub source: String,
}

/// The run contract a documented program binds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentedContract {
    /// The file the program is saved as.
    pub save_as: String,
    /// The check command (when the row has one) before the run command.
    pub commands: Vec<SourcedCommand>,
}

/// A program the documentation route rediscovered, with what answering it
/// needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentedProgram {
    /// The rediscoverable cache row.
    pub recipe: RediscoverableRecipe,
    /// The file and commands the program binds.
    pub contract: DocumentedContract,
    /// How the program departs from the output the catalog verifies, in a way
    /// its decomposition cannot see (`trailing_newline=absent`).
    pub deviation: Option<String>,
    /// The name the page gives its language, when the capture records one.
    pub language_name: String,
}

/// The stem of the file a program is saved as (`Main` for `src/Main.java`).
fn file_stem(save_as: &str) -> &str {
    let file = save_as.rsplit('/').next().unwrap_or_default();
    file.rfind('.').map_or(file, |dot| &file[..dot])
}

/// The values one per-language row of the part vocabulary carries
/// (`entry_container`, `bare_output_call`, `newline_token`), in seed order.
fn language_rows(language: &str, row: &str) -> Vec<String> {
    let tree = parse_lino(CODE_EXAMPLE_PARTS);
    let mut values = Vec::new();
    let mut current = String::new();
    let mut pending: Vec<&crate::seed::parser::LinoNode> = tree.children.iter().rev().collect();
    while let Some(node) = pending.pop() {
        pending.extend(node.children.iter().rev());
        if node.name == "part_language" {
            current.clone_from(&node.id);
        } else if node.name == row && current == language && !node.id.is_empty() {
            values.push(node.id.clone());
        }
    }
    values
}

/// The name a documented command binds in place of the saved file's stem
/// (R1165-6), or `None` when it is not the catalog command.
///
/// The words must agree, except that where the catalog word carries the stem,
/// the documented word may carry one other name in its place, the same name
/// everywhere (`kotlinc hello.kt -d hello.jar` binds `hello` for `kotlinc
/// Main.kt -d Main.jar`). A line equal to the catalog command binds the stem
/// itself. The JavaScript twin is `documentedCommandBinding`.
#[must_use]
pub fn documented_command_binding(
    documented: &str,
    catalog: &str,
    save_as: &str,
) -> Option<String> {
    let stem = file_stem(save_as);
    let words: Vec<&str> = documented.split_whitespace().collect();
    let expected: Vec<&str> = catalog.split_whitespace().collect();
    if words.len() != expected.len() {
        return None;
    }
    let mut bound: Option<&str> = None;
    for (said, word) in words.iter().zip(&expected) {
        if said == word {
            continue;
        }
        let at = word.find(stem).filter(|_| !stem.is_empty())?;
        let prefix = &word[..at];
        let suffix = &word[at + stem.len()..];
        if !said.starts_with(prefix)
            || !said.ends_with(suffix)
            || said.len() <= prefix.len() + suffix.len()
        {
            return None;
        }
        let name = &said[prefix.len()..said.len() - suffix.len()];
        let identifier = name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_');
        if !identifier || bound.is_some_and(|earlier| earlier != name) {
            return None;
        }
        bound = Some(name);
    }
    Some(bound.unwrap_or(stem).to_owned())
}

/// Whether a documented command is a catalog command with the documented
/// file name bound to the catalog's (R1165-6).
///
/// See [`documented_command_binding`]. The JavaScript twin is
/// `documentedCommandMatches`.
#[must_use]
pub fn documented_command_matches(documented: &str, catalog: &str, save_as: &str) -> bool {
    documented_command_binding(documented, catalog, save_as).is_some()
}

/// Every non-empty line of the captured pages' code blocks, its shell prompt
/// removed, with the page that states it.
fn documentation_command_lines(captures: &[DocumentationCapture]) -> Vec<(String, String)> {
    let mut lines = Vec::new();
    for capture in captures {
        for (_, text) in &capture.blocks {
            for raw in text.lines() {
                let trimmed = raw.trim();
                let line = COMMAND_PROMPTS
                    .iter()
                    .find_map(|prompt| trimmed.strip_prefix(prompt))
                    .unwrap_or(trimmed)
                    .trim();
                if !line.is_empty() {
                    lines.push((line.to_owned(), capture.url.clone()));
                }
            }
        }
    }
    lines
}

/// `word` with its first `stem` replaced by `name`.
fn rebind(word: &str, stem: &str, name: &str) -> String {
    word.find(stem)
        .filter(|_| !stem.is_empty() && stem != name)
        .map_or_else(
            || word.to_owned(),
            |at| format!("{}{name}{}", &word[..at], &word[at + stem.len()..]),
        )
}

/// The run contract a documented program binds (R1165-6).
///
/// The contract's file stem stays when the program declares every name the
/// commands invoke; otherwise the program's own name binds in its place, in
/// the file and in every command: the name a captured command line states
/// for a contract command, when the program declares it (Oracle's `javac
/// HelloWorldApp.java`), else the name after one of the language's
/// `entry_container` keywords (the Scala book's `object hello`). Every
/// command carries its source: the page whose captured line states it, else
/// [`COMMAND_SOURCE_CATALOG`]. The JavaScript twin is `documentedRunContract`.
///
/// # Errors
///
/// `run_contract:<names>` when the program declares neither the invoked
/// names nor a name that binds in their place.
pub fn documented_run_contract(
    language: &str,
    contract: RunContract<'_>,
    program: &str,
    captures: &[DocumentationCapture],
) -> Result<DocumentedContract, String> {
    let stem = file_stem(contract.save_as);
    let tokens: Vec<&str> = program
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .filter(|token| !token.is_empty())
        .collect();
    let missing: Vec<String> = contract
        .invoked_names()
        .into_iter()
        .filter(|name| !tokens.contains(&name.as_str()))
        .collect();
    let lines = documentation_command_lines(captures);
    let mut name = stem.to_owned();
    if !missing.is_empty() {
        let stated = contract.commands.iter().find_map(|command| {
            lines.iter().find_map(|(line, _)| {
                documented_command_binding(line, command, contract.save_as)
                    .filter(|bound| bound.as_str() != stem && tokens.contains(&bound.as_str()))
            })
        });
        let containers = language_rows(language, "entry_container");
        let declared = tokens
            .windows(2)
            .find(|pair| containers.iter().any(|keyword| keyword.as_str() == pair[0]))
            .map(|pair| pair[1].to_owned());
        name = stated
            .or(declared)
            .ok_or_else(|| format!("run_contract:{}", missing.join(",")))?;
    }
    let (directory, file) = contract
        .save_as
        .rsplit_once('/')
        .unwrap_or(("", contract.save_as));
    let bound_file = rebind(file, stem, &name);
    let save_as = if directory.is_empty() {
        bound_file
    } else {
        format!("{directory}/{bound_file}")
    };
    let last = contract.commands.len().saturating_sub(1);
    let commands = contract
        .commands
        .iter()
        .enumerate()
        .map(|(index, command)| {
            let source = lines
                .iter()
                .find(|(line, _)| documented_command_matches(line, command, contract.save_as))
                .map_or_else(|| COMMAND_SOURCE_CATALOG.to_owned(), |(_, url)| url.clone());
            let bound = if name == stem {
                (*command).to_owned()
            } else {
                command
                    .split_whitespace()
                    .map(|word| rebind(word, stem, &name))
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            SourcedCommand {
                role: if index == last { "run" } else { "check" },
                command: bound,
                source,
            }
        })
        .collect();
    Ok(DocumentedContract { save_as, commands })
}

/// How a documented program departs from the output the catalog verifies,
/// in a way its decomposition cannot see.
///
/// Its output call prints no line break (a `bare_output_call` of the part
/// vocabulary, PHP's `echo`) and its line carries none of the language's
/// `newline_token` rows, so the program prints the expected text without the
/// trailing newline: `trailing_newline=absent`. The JavaScript twin is
/// `documentationDeviation`.
#[must_use]
pub fn documentation_deviation(language: &str, call: &str, program: &str) -> Option<String> {
    if !language_rows(language, "bare_output_call")
        .iter()
        .any(|bare| bare.as_str() == call)
    {
        return None;
    }
    let line = program
        .lines()
        .find(|candidate| candidate.contains(call))
        .unwrap_or_default();
    let breaks = language_rows(language, "newline_token")
        .iter()
        .any(|token| line.contains(token.as_str()));
    (!breaks).then(|| String::from("trailing_newline=absent"))
}

/// The derivation a documented pair adds after its `procedure_cache` event,
/// as `(kind, payload)`.
///
/// One `command_source` per command (where the command shown comes from, a
/// page or the catalog) and the `documentation_deviation` the program
/// carries. The JavaScript twin is `documentationEvents`.
#[must_use]
pub fn documentation_events(
    language: &str,
    task: &str,
    program: &DocumentedProgram,
) -> Vec<(&'static str, String)> {
    let mut events: Vec<(&'static str, String)> = program
        .contract
        .commands
        .iter()
        .map(|command| {
            (
                "command_source",
                crate::event_log::render_fields(&[
                    ("language", language),
                    ("task", task),
                    ("role", command.role),
                    ("source", command.source.as_str()),
                    ("command", command.command.as_str()),
                ]),
            )
        })
        .collect();
    if let Some(deviation) = &program.deviation {
        let fields = crate::event_log::render_fields(&[("language", language), ("task", task)]);
        events.push((
            "documentation_deviation",
            [fields, deviation.clone()].join(" "),
        ));
    }
    events
}

/// The program the coding oracle answers an uncatalogued pair with, when the
/// documentation route rediscovers one (R1165-4).
///
/// The oracle answers a language the catalog has no program for (Swift); it
/// reads the documentation route before its cached snapshots. The program is
/// bound to the task's output under no run contract, as the browser oracle
/// binds it. The JavaScript twin is `codingOracleDocumentedSnippet`.
#[must_use]
pub fn documented_oracle_program(task: &str, language: &str) -> Option<DocumentedProgram> {
    let slug = language.trim().to_ascii_lowercase();
    let expected = crate::coding::program_task_by_slug(task)?.output;
    rediscover_documented_program(
        &slug,
        task,
        expected,
        RunContract {
            save_as: "",
            commands: &[],
        },
    )
    .ok()
}
