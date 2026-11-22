// Rediscovery from registered publisher documentation on a coding request.
// Only source references are installed. Each request acquires its own
// verified response through the runtime capture client and formalizes it.
//
// The browser twin is discoverDocumentedProgram in
// js/worker/formal_ai_worker_code_examples.js.

const DOCUMENTATION_CAPTURES: &str =
    include_str!("../embedded/data/seed/coding-documentation-captures.lino");

/// Why no program was rediscovered: the pair has no captured page (or the
/// route is retired in the policy seed), so the miss is the research miss.
pub const NO_DOCUMENTATION_CAPTURE: &str = "no_documentation_capture";

/// One documentation page captured for a `(language, task)` pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentationCapture {
    /// The language the page documents.
    pub language: String,
    /// The name the page gives that language, when the capture records one.
    pub language_name: String,
    /// The task it was captured for.
    pub task: String,
    /// The page's URL, the row's `rediscovery_source`.
    pub url: String,
    /// The mime type the page was served as.
    pub mime: String,
    /// Evidence from the acquisition, when the page has been read.
    pub source_trace: String,
    /// The verified capture was reused from the runtime cache.
    pub cached: bool,
    /// The query that finds the page again, the row's `rediscovery_query`.
    pub rediscovery_query: String,
    /// `(language, text)` of every code block on the page, in page order.
    pub blocks: Vec<(String, String)>,
}

/// What a language's catalog row asks of a program it runs.
#[derive(Debug, Clone, Copy)]
pub struct RunContract<'a> {
    /// The file the program is saved as (`Main.scala`).
    pub save_as: &'a str,
    /// The commands that check and run the saved file: the check command
    /// (when the row has one) before the run command, as a catalog row
    /// orders them.
    pub commands: &'a [&'a str],
}

impl RunContract<'_> {
    /// The names a program must declare: the stem of `save_as`, when a
    /// command names it bare (`scala Main` runs the object `Main`;
    /// `rustc main.rs -o main` names the binary `fn main` builds).
    #[must_use]
    pub fn invoked_names(&self) -> Vec<String> {
        let file = self.save_as.rsplit('/').next().unwrap_or_default();
        let stem = file.rfind('.').map_or(file, |dot| &file[..dot]);
        let named = self
            .commands
            .iter()
            .flat_map(|command| command.split_whitespace())
            .any(|word| word == stem);
        if stem.is_empty() || !named {
            return Vec::new();
        }
        vec![stem.to_owned()]
    }
}

/// Whether the policy seed keeps the documentation route active.
#[must_use]
pub fn documentation_route_active() -> bool {
    let policy = parse_lino(POLICY);
    policy
        .children
        .first()
        .and_then(|root| {
            root.children
                .iter()
                .find(|node| node.name == "documentation_route")
        })
        .is_some_and(|route| route.find_child_value("active") == "true")
}

/// The documentation captures for `language` and `task`, in seed order.
#[must_use]
pub fn documentation_captures(language: &str, task: &str) -> Vec<DocumentationCapture> {
    all_documentation_captures()
        .into_iter()
        .filter(|capture| capture.language == language && capture.task == task)
        .collect()
}

/// Every documentation capture the seed records, in seed order.
#[must_use]
pub fn all_documentation_captures() -> Vec<DocumentationCapture> {
    let seed = parse_lino(DOCUMENTATION_CAPTURES);
    let Some(root) = seed
        .children
        .iter()
        .find(|node| node.name == "coding_documentation_captures")
    else {
        return Vec::new();
    };
    root.children
        .iter()
        .filter(|node| node.name == "capture")
        .map(|node| DocumentationCapture {
            language: node.find_child_value("language").to_owned(),
            language_name: node.find_child_value("language_name").to_owned(),
            task: node.find_child_value("task").to_owned(),
            url: node.find_child_value("url").to_owned(),
            mime: node.find_child_value("mime").to_owned(),
            rediscovery_query: node.find_child_value("rediscovery_query").to_owned(),
            source_trace: String::new(),
            cached: false,
            blocks: Vec::new(),
        })
        .collect()
}

/// Retrieve and formalize verified publisher responses for a single request.
pub fn acquire_documentation_captures<T: SourceTransport>(
    language: &str,
    task: &str,
    client: &CachedSourceClient<T>,
) -> Vec<DocumentationCapture> {
    use crate::web_formalize::{PageBlock, formalize_page_with_context};
    if !documentation_route_active() {
        return Vec::new();
    }
    documentation_captures(language, task)
        .into_iter()
        .filter_map(|mut reference| {
            let capture = client.fetch(&reference.url).ok()?;
            std::str::from_utf8(capture.bytes()).ok()?;
            let (_, blocks) = formalize_page_with_context(
                capture.bytes(),
                Some(reference.mime.as_str()),
                Some(capture.source_url()),
            );
            reference.blocks = blocks
                .into_iter()
                .filter_map(|(_, block)| match block {
                    PageBlock::CodeBlock { language, text } => Some((language, text)),
                    _ => None,
                })
                .collect();
            reference.source_trace = capture.trace_payload();
            reference.cached = capture.cached();
            Some(reference)
        })
        .collect()
}

fn documentation_client(
    offline: bool,
) -> CachedSourceClient<crate::source_fetch::PublicPageTransport> {
    CachedSourceClient::new(
        crate::coding::synthesis_runtime::source_cache_root(),
        crate::source_fetch::PublicPageTransport,
    )
    .with_online(!offline)
    .with_ttl_seconds(86_400)
}

/// Whether a captured block's own language tag names `language`.
///
/// The tag names it when it is the slug itself, the formalizer's untagged
/// fallback, or one of the surfaces of the `program_language_<slug>` meaning
/// (a page tagging its example `js`, `py` or `c++`). The JavaScript twin is
/// `documentationBlockNamesLanguage`.
#[cfg(feature = "meta-language")]
fn block_names_language(tag: &str, language: &str) -> bool {
    let lower = tag.to_ascii_lowercase();
    lower == language
        || lower == "unknown"
        || crate::seed::lexicon()
            .meaning(&format!("program_language_{language}"))
            .into_iter()
            .flat_map(crate::seed::Meaning::words)
            .any(|word| word == lower)
}

/// The page example a rediscovery recomposes.
///
/// In each capture it is the first block in `language` (or untagged) that
/// decomposes into an output call printing a literal; across captures, the
/// shortest program body wins (the smallest sufficient candidate), the
/// earlier capture on a tie.
#[cfg(feature = "meta-language")]
fn documented_example<'a>(
    captures: &'a [DocumentationCapture],
    language: &str,
) -> Option<(
    &'a DocumentationCapture,
    crate::code_example_knowledge::DecomposedCodeNode,
)> {
    use crate::code_example_knowledge::{CodePartKind, decompose_code_node_from};
    let mut best: Option<(
        &DocumentationCapture,
        crate::code_example_knowledge::DecomposedCodeNode,
    )> = None;
    for capture in captures {
        for (block_language, text) in &capture.blocks {
            if !block_names_language(block_language, language) {
                continue;
            }
            let Ok(node) = decompose_code_node_from(text, language, &[], &capture.url) else {
                continue;
            };
            let prints = node
                .parts
                .iter()
                .any(|part| part.kind == CodePartKind::OutputOperation);
            if node.program_body.is_empty() || !prints {
                continue;
            }
            let shorter = best.as_ref().is_none_or(|(_, kept)| {
                node.program_body.chars().count() < kept.program_body.chars().count()
            });
            if shorter {
                best = Some((capture, node));
            }
            break;
        }
    }
    best
}

/// Rediscover a `write_program` procedure from the documentation captures.
///
/// The recipe of [`rediscover_documented_program`]: the row names the page
/// as its `rediscovery_source`.
///
/// # Errors
///
/// As [`rediscover_documented_program`].
pub fn rediscover_from_documentation(
    language: &str,
    task: &str,
    expected_output: &str,
    contract: RunContract<'_>,
) -> Result<RediscoverableRecipe, String> {
    rediscover_documented_program(language, task, expected_output, contract)
        .map(|program| program.recipe)
}

/// Rediscover a program from the documentation captures, with what
/// answering it needs.
///
/// The page example (each capture's first block that prints a literal,
/// the shortest across captures) is recomposed with
/// `expected_output` bound into its literal slot, then verified: decomposing
/// the program again must find the same output call printing
/// `expected_output`, and the program must bind the run contract
/// ([`documented_run_contract`]). The result carries the bound contract and
/// the deviation the program's verification cannot see
/// ([`documentation_deviation`]). The JavaScript twin is
/// `rediscoverDocumentedProgram`.
///
/// # Errors
///
/// [`NO_DOCUMENTATION_CAPTURE`] when the pair has no capture (or the route is
/// retired), else why the captured pages yielded no program:
/// `no_single_line_output`, `no_output_example`, `no_program_body`,
/// `verification`, or `run_contract:<names>`.
pub fn rediscover_documented_program(
    language: &str,
    task: &str,
    expected_output: &str,
    contract: RunContract<'_>,
) -> Result<DocumentedProgram, String> {
    resolve_documented_program(language, task, expected_output, contract, true)
}

/// Resolve a procedure under the current request's source policy.
pub(crate) fn resolve_documented_program(
    language: &str,
    task: &str,
    expected_output: &str,
    contract: RunContract<'_>,
    offline: bool,
) -> Result<DocumentedProgram, String> {
    let client = documentation_client(offline);
    rediscover_documented_program_with_client(language, task, expected_output, contract, &client)
}

/// Resolve a procedure from an explicitly supplied capture client.
pub fn rediscover_documented_program_with_client<T: SourceTransport>(
    language: &str,
    task: &str,
    expected_output: &str,
    contract: RunContract<'_>,
    client: &CachedSourceClient<T>,
) -> Result<DocumentedProgram, String> {
    if !documentation_route_active() || documentation_captures(language, task).is_empty() {
        return Err(NO_DOCUMENTATION_CAPTURE.to_owned());
    }
    if !cfg!(feature = "meta-language") {
        return Err("no_decomposer".to_owned());
    }
    let captures = acquire_documentation_captures(language, task, client);
    rediscover_documented_program_from(language, task, expected_output, contract, &captures)
}

#[cfg(feature = "meta-language")]
fn rediscover_documented_program_from(
    language: &str,
    task: &str,
    expected_output: &str,
    contract: RunContract<'_>,
    captures: &[DocumentationCapture],
) -> Result<DocumentedProgram, String> {
    use crate::code_example_knowledge::{
        CodePartKind, ParameterBindings, decompose_code_node_from, generalize_examples,
        recompose_for_requirement,
    };
    if captures.is_empty() {
        return Err(NO_DOCUMENTATION_CAPTURE.to_owned());
    }
    if expected_output.is_empty() || expected_output.contains('\n') {
        return Err(String::from("no_single_line_output"));
    }
    let (capture, node) =
        documented_example(captures, language).ok_or_else(|| String::from("no_output_example"))?;
    let call = node
        .parts
        .iter()
        .find(|part| part.kind == CodePartKind::OutputOperation)
        .map(|part| part.source_text.clone())
        .unwrap_or_default();
    let bindings = ParameterBindings(
        std::iter::once((String::from("output_literal"), expected_output.to_owned())).collect(),
    );
    let procedure = generalize_examples(std::slice::from_ref(&node));
    let recomposed = recompose_for_requirement(&procedure, &bindings, language)
        .map_err(|_| String::from("no_program_body"))?;
    let quote = node
        .program_body
        .find(crate::code_example_knowledge::LITERAL_SLOT)
        .and_then(|slot| node.program_body.get(..slot))
        .and_then(|prefix| prefix.chars().next_back());
    let expected_source = crate::code_example_knowledge::escape_for_quote(expected_output, quote);
    let verified = decompose_code_node_from(&recomposed.source, language, &[], &capture.url)
        .is_ok_and(|check| {
            check
                .parts
                .iter()
                .any(|part| part.kind == CodePartKind::OutputOperation && part.source_text == call)
                && check.parts.iter().any(|part| {
                    part.kind == CodePartKind::StringLiteral && part.source_text == expected_source
                })
        });
    if !verified {
        return Err(String::from("verification"));
    }
    let bound = documented_run_contract(language, contract, &recomposed.source, captures)?;
    let deviation = documentation_deviation(language, &call, &recomposed.source);
    Ok(DocumentedProgram {
        recipe: RediscoverableRecipe {
            language: language.to_owned(),
            task: task.to_owned(),
            rediscovery_query: capture.rediscovery_query.clone(),
            rediscovery_source: capture.url.clone(),
            content_id: RediscoverableRecipe::content_address(&recomposed.source),
            entry: recomposed.source,
            verified_output: expected_output.to_owned(),
        },
        contract: bound,
        deviation,
        language_name: capture.language_name.clone(),
        source_trace: capture.source_trace.clone(),
        cached: capture.cached,
    })
}

#[cfg(not(feature = "meta-language"))]
fn rediscover_documented_program_from(
    _language: &str,
    _task: &str,
    _expected_output: &str,
    _contract: RunContract<'_>,
    _captures: &[DocumentationCapture],
) -> Result<DocumentedProgram, String> {
    Err("no_decomposer".to_owned())
}

/// Record the inputs still needed to derive a source-backed procedure.
pub(crate) fn record_program_cache_miss(
    log: &mut crate::event_log::EventLog,
    language: &str,
    task: &str,
    reason: &str,
) {
    let missing = miss_research_missing().join(",");
    let mut fields = vec![
        ("outcome", "miss"),
        ("language", language),
        ("task", task),
        ("research_missing", missing.as_str()),
    ];
    if reason != NO_DOCUMENTATION_CAPTURE {
        fields.push(("documentation_rejected", reason));
    }
    log.append_fields("procedure_cache", &fields);
}

/// The commands that check and run a catalog program in argument order.
#[must_use]
pub(crate) fn catalog_run_commands(language: &crate::coding::ProgramLanguage) -> Vec<&str> {
    language
        .execution
        .check_command
        .as_deref()
        .into_iter()
        .chain(std::iter::once(language.execution.run_command.as_ref()))
        .collect()
}

/// Every `(task, language)` pair the documentation captures cover, in seed
/// order, each once; empty while the policy seed retires the route.
#[must_use]
pub fn documented_pairs() -> Vec<(String, String)> {
    if !documentation_route_active() {
        return Vec::new();
    }
    let mut pairs: Vec<(String, String)> = Vec::new();
    for capture in all_documentation_captures() {
        let pair = (capture.task, capture.language);
        if !pairs.contains(&pair) {
            pairs.push(pair);
        }
    }
    pairs
}

/// One catalog command and the line its documentation states for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentedCommand {
    /// The command the catalog row runs.
    pub catalog: String,
    /// The captured line that is the same command with the documented file
    /// name bound to the catalog's, or `None` when the pages state none.
    pub documented: Option<String>,
}

/// The check and run commands of a catalog pair, each with the line its
/// documentation captures state for it (R1165-6).
///
/// A captured line, its shell prompt removed, states a catalog command when
/// [`documented_command_matches`] binds it; a command no captured line states
/// is listed with `documented: None`. The JavaScript twin is
/// `documentedRunCommands`.
#[must_use]
pub fn documented_run_commands(language: &str, task: &str) -> Vec<DocumentedCommand> {
    documented_run_commands_from(
        language,
        &acquire_documentation_captures(language, task, &documentation_client(true)),
    )
}

/// Verify which known run commands appear in the acquired publisher pages.
#[must_use]
pub fn documented_run_commands_from(
    language: &str,
    captures: &[DocumentationCapture],
) -> Vec<DocumentedCommand> {
    let Some(row) = crate::coding::program_language_by_slug(language) else {
        return Vec::new();
    };
    let lines = documentation_command_lines(captures);
    catalog_run_commands(row)
        .into_iter()
        .map(|command| DocumentedCommand {
            catalog: command.to_owned(),
            documented: lines
                .iter()
                .find(|(line, _)| documented_command_matches(line, command, &row.save_as))
                .map(|(line, _)| line.clone()),
        })
        .collect()
}

/// Whether the enabled registry identifies a publisher for the language.
#[must_use]
pub fn language_has_documented_procedure(language: &str) -> bool {
    let needle = language.trim().to_ascii_lowercase();
    documented_pairs()
        .iter()
        .any(|(_, documented)| *documented == needle)
}

/// The `program_source` of a seed template row whose program is retired to
/// the documentation route.
pub const DOCUMENTATION_ROUTE_SOURCE: &str = "documentation_route";

/// One stored catalog program the documentation route replaces (R1165-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentedCatalogProgram {
    /// The catalog language slug.
    pub language: String,
    /// The catalog task slug.
    pub task: String,
    /// What the documentation captures yield for the task's expected output.
    pub rediscovered: Result<DocumentedProgram, String>,
}

/// The catalog pairs whose seed program is retired to the documentation route.
///
/// A row of `data/seed/hello-world-programs.lino` whose `program_source` is
/// `documentation_route` stores no program, and the Rust catalog compiles
/// none for it. This diagnostic reports what its local source cache yields.
#[must_use]
pub fn documented_catalog_programs() -> Vec<DocumentedCatalogProgram> {
    documented_catalog_programs_with_client(&documentation_client(true))
}

/// Inspect every declared pair using an explicitly supplied capture client.
#[must_use]
pub fn documented_catalog_programs_with_client<T: SourceTransport>(
    client: &CachedSourceClient<T>,
) -> Vec<DocumentedCatalogProgram> {
    let seed = parse_lino(crate::seed::HELLO_WORLD_PROGRAMS_LINO);
    seed.children
        .iter()
        .filter(|node| node.find_child_value("program_source") == DOCUMENTATION_ROUTE_SOURCE)
        .filter_map(|node| {
            let task = crate::coding::program_task_by_slug(node.find_child_value("task"))?;
            let language =
                crate::coding::program_language_by_slug(node.find_child_value("language"))?;
            Some(DocumentedCatalogProgram {
                language: language.slug.to_owned(),
                task: task.slug.to_owned(),
                rediscovered: rediscover_documented_program_with_client(
                    language.slug,
                    task.slug,
                    task.output,
                    RunContract {
                        save_as: &language.save_as,
                        commands: &catalog_run_commands(language),
                    },
                    client,
                ),
            })
        })
        .collect()
}
